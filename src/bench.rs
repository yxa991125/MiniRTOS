use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicUsize, Ordering};

use cortex_m::interrupt;
use cortex_m::peripheral::{DCB, DWT};
use stm32f4xx_hal::{pac, rcc::Rcc};

use crate::device::timer::TimerMode;
use crate::ipc::mqueue::SyncMsgQueue;
use crate::sync::{
    mutex::{BlockingMutex, IrqMutex},
    semaphore::Semaphore,
};
use crate::{kernel, log, timer};

const STAGE_CONTEXT: u8 = 0;
const STAGE_PARK_HELPER: u8 = 1;
const STAGE_SEM: u8 = 2;
const STAGE_SLEEP: u8 = 3;
const STAGE_IRQ: u8 = 4;
const STAGE_QUEUE: u8 = 5;
const STAGE_MUTEX: u8 = 6;
const STAGE_TIMER_CB: u8 = 7;
const STAGE_TIMEOUT: u8 = 8;
const STAGE_SCALE: u8 = 9;
const STAGE_DONE: u8 = 10;

const BENCH_SAMPLES: u32 = 1000;
const BENCH_SAMPLES_USIZE: usize = BENCH_SAMPLES as usize;
const IRQ_BENCH_TIMER_HZ: u32 = 100;
const IRQ_TIMEOUT_MS: u32 = 50;
const WAIT_TIMEOUT_TICKS: u32 = 200;
const INVALID_PID: usize = usize::MAX;
const BUILD_PROFILE: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "release"
};
const TIMEOUT_VALIDATION_SAMPLES: u32 = 4;
const TIMEOUT_CROSS_BUCKET_DELAY_TICKS: u32 = 2;
const TIMEOUT_LONG_DELAY_TICKS: u32 = crate::task::scheduler::TIMEOUT_WHEEL_SIZE as u32 + 5;
const TIMEOUT_EARLY_UNBLOCK_TICKS: u32 = 3;
const CONTEXT_SKIP_SAMPLES: usize = 1;
const MUTEX_PI_OWNER_PRIO: u8 = 3;
const MUTEX_LOCK_SPIKE_THRESHOLD_CY: u32 = 128;
const SEM_SPIKE_THRESHOLD_CY: u32 = 1000;
const IRQ_SPIKE_THRESHOLD_CY: u32 = 900;
const QUEUE_WAKE_SPIKE_THRESHOLD_CY: u32 = 900;
const QUEUE_END_SPIKE_THRESHOLD_CY: u32 = 1100;

const SCALE_CASES: [usize; 3] = [2, 8, 32];
const SCALE_MAX_TASKS: usize = 32;
const SCALE_EXTRA_TASKS: usize = SCALE_MAX_TASKS - 1;
const SCALE_STACK_WORDS: usize = 128;

#[derive(Clone, Copy)]
#[repr(align(8))]
struct AlignedStack<const N: usize>([u32; N]);

static mut SCALE_STACKS: [AlignedStack<SCALE_STACK_WORDS>; SCALE_EXTRA_TASKS] =
    [AlignedStack([0; SCALE_STACK_WORDS]); SCALE_EXTRA_TASKS];

static STAGE: AtomicU8 = AtomicU8::new(STAGE_CONTEXT);
static CPU_HZ: AtomicU32 = AtomicU32::new(0);

static TASK_A_PID: AtomicUsize = AtomicUsize::new(INVALID_PID);
static TASK_B_PID: AtomicUsize = AtomicUsize::new(INVALID_PID);
static TASK_B_PARKED: AtomicBool = AtomicBool::new(false);

static CTX_PENDING: AtomicBool = AtomicBool::new(false);
static CTX_START: AtomicU32 = AtomicU32::new(0);
static CTX_LEFT: AtomicU32 = AtomicU32::new(BENCH_SAMPLES);

static IRQ_TIMER_TICKS: AtomicU32 = AtomicU32::new(0);
static IRQ_WAITING: AtomicBool = AtomicBool::new(false);
static IRQ_STAMP: AtomicU32 = AtomicU32::new(0);
static IRQ_UNBLOCK_DONE_STAMP: AtomicU32 = AtomicU32::new(0);
static IRQ_STAMP_SYSTICK_COUNT: AtomicU32 = AtomicU32::new(0);
static IRQ_STAMP_TIM2_COUNT: AtomicU32 = AtomicU32::new(0);
static IRQ_EVENTS: AtomicU32 = AtomicU32::new(0);

static SEM_WAITING: AtomicBool = AtomicBool::new(false);
static SEM_STAMP: AtomicU32 = AtomicU32::new(0);
static SEM_STAMP_SYSTICK_COUNT: AtomicU32 = AtomicU32::new(0);
static SEM_STAMP_TIM2_COUNT: AtomicU32 = AtomicU32::new(0);
static SEM_EVENTS: AtomicU32 = AtomicU32::new(0);

static QUEUE_WAITING: AtomicBool = AtomicBool::new(false);
static QUEUE_EVENTS: AtomicU32 = AtomicU32::new(0);
static QUEUE_WAKE_STAMP: AtomicU32 = AtomicU32::new(0);
static QUEUE_UNBLOCK_DONE_STAMP: AtomicU32 = AtomicU32::new(0);
static QUEUE_WAKE_STAMP_SYSTICK_COUNT: AtomicU32 = AtomicU32::new(0);
static QUEUE_WAKE_STAMP_TIM2_COUNT: AtomicU32 = AtomicU32::new(0);
static QUEUE_SEND_STAMP_SYSTICK_COUNT: AtomicU32 = AtomicU32::new(0);
static QUEUE_SEND_STAMP_TIM2_COUNT: AtomicU32 = AtomicU32::new(0);

static TIMER_CB_WAITING: AtomicBool = AtomicBool::new(false);
static TIMER_CB_STAMP: AtomicU32 = AtomicU32::new(0);
static TIMER_CB_EVENTS: AtomicU32 = AtomicU32::new(0);
static TIMEOUT_EARLY_EVENTS: AtomicU32 = AtomicU32::new(0);
static MUTEX_WAITER_SEQ: AtomicU32 = AtomicU32::new(0);
static MUTEX_DONE_SEQ: AtomicU32 = AtomicU32::new(0);
static MUTEX_REQUEST_STAMP: AtomicU32 = AtomicU32::new(0);
static MUTEX_UNLOCK_STAMP: AtomicU32 = AtomicU32::new(0);

static LAST_SYSTICK_EDGE_CYCLE: AtomicU32 = AtomicU32::new(0);
static LAST_SYSTICK_EDGE_TICK: AtomicU32 = AtomicU32::new(0);
static SYSTICK_ISR_COUNT: AtomicU32 = AtomicU32::new(0);
static BENCH_TIM2_ISR_COUNT: AtomicU32 = AtomicU32::new(0);

static SCALE_WORKERS_CREATED: AtomicUsize = AtomicUsize::new(0);
static SCALE_ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(2);
static SCALE_WORKER_PIDS: IrqMutex<[usize; SCALE_EXTRA_TASKS]> =
    IrqMutex::new([INVALID_PID; SCALE_EXTRA_TASKS]);

static SEM_BENCH: Semaphore<4> = Semaphore::new(0, 1);
static QUEUE_BENCH: SyncMsgQueue<8> = SyncMsgQueue::new();
static MUTEX_BENCH: IrqMutex<u32> = IrqMutex::new(0);
static PI_MUTEX_BENCH: BlockingMutex<u32, 4> = BlockingMutex::new(0);

static CTX_STATS: IrqMutex<Stats> = IrqMutex::new(Stats::new());
static CTX_SAMPLES: IrqMutex<[u32; BENCH_SAMPLES_USIZE]> = IrqMutex::new([0; BENCH_SAMPLES_USIZE]);
static SEM_STATS: IrqMutex<Stats> = IrqMutex::new(Stats::new());
static SEM_ATTR_STATS: IrqMutex<MutexSpikeAttribution> =
    IrqMutex::new(MutexSpikeAttribution::new(SEM_SPIKE_THRESHOLD_CY));
static MUTEX_WAKE_STATS: IrqMutex<Stats> = IrqMutex::new(Stats::new());

#[derive(Clone, Copy)]
struct Stats {
    min: u32,
    max: u32,
    sum: u32,
    count: u32,
}

#[derive(Clone, Copy)]
struct ValidationStats {
    pass: u32,
    fail: u32,
    min_ticks: u32,
    max_ticks: u32,
}

#[derive(Clone, Copy)]
struct PercentileStats {
    stats: Stats,
    skipped: u32,
    p50: u32,
    p95: u32,
}

#[derive(Clone, Copy)]
struct MutexBenchStats {
    lock_unlock: Stats,
    lock_unlock_attr: MutexSpikeAttribution,
    waiter_wake: Stats,
    pi_enter: Stats,
    pi_exit: Stats,
}

#[derive(Clone, Copy)]
struct MetricBenchStats {
    stats: Stats,
    attr: MutexSpikeAttribution,
}

#[derive(Clone, Copy)]
struct IrqBenchStats {
    stats: Stats,
    attr: MutexSpikeAttribution,
    clean_breakdown: TwoPhaseCleanBreakdown,
}

#[derive(Clone, Copy)]
struct QueueBenchStats {
    wake: Stats,
    wake_attr: MutexSpikeAttribution,
    wake_clean_breakdown: TwoPhaseCleanBreakdown,
    end_to_end: Stats,
    end_attr: MutexSpikeAttribution,
    end_clean_breakdown: FourPhaseCleanBreakdown,
}

#[derive(Clone, Copy)]
struct MutexSpikeAttribution {
    threshold_cy: u32,
    overlap_samples: u32,
    spikes: u32,
    irq_spikes: u32,
    clean_spikes: u32,
    systick_spikes: u32,
    tim2_spikes: u32,
    max_irq_spike_cy: u32,
    max_clean_spike_cy: u32,
}

#[derive(Clone, Copy)]
struct TwoPhaseCleanBreakdown {
    clean_spikes: u32,
    first_dominant: u32,
    second_dominant: u32,
    max_first_cy: u32,
    max_second_cy: u32,
}

#[derive(Clone, Copy)]
struct FourPhaseCleanBreakdown {
    clean_spikes: u32,
    first_dominant: u32,
    second_dominant: u32,
    third_dominant: u32,
    fourth_dominant: u32,
    max_first_cy: u32,
    max_second_cy: u32,
    max_third_cy: u32,
    max_fourth_cy: u32,
}

impl ValidationStats {
    const fn new() -> Self {
        Self {
            pass: 0,
            fail: 0,
            min_ticks: u32::MAX,
            max_ticks: 0,
        }
    }

    fn observe(&mut self, ticks: u32, passed: bool) {
        if ticks < self.min_ticks {
            self.min_ticks = ticks;
        }
        if ticks > self.max_ticks {
            self.max_ticks = ticks;
        }
        if passed {
            self.pass = self.pass.saturating_add(1);
        } else {
            self.fail = self.fail.saturating_add(1);
        }
    }

    fn normalized(self) -> Self {
        if self.pass == 0 && self.fail == 0 {
            Self {
                pass: 0,
                fail: 0,
                min_ticks: 0,
                max_ticks: 0,
            }
        } else {
            Self {
                min_ticks: if self.min_ticks == u32::MAX {
                    0
                } else {
                    self.min_ticks
                },
                ..self
            }
        }
    }
}

impl Stats {
    const fn new() -> Self {
        Self {
            min: u32::MAX,
            max: 0,
            sum: 0,
            count: 0,
        }
    }

    fn update(&mut self, sample: u32) {
        if sample < self.min {
            self.min = sample;
        }
        if sample > self.max {
            self.max = sample;
        }
        self.sum = self.sum.saturating_add(sample);
        self.count = self.count.saturating_add(1);
    }

    fn avg(&self) -> u32 {
        if self.count == 0 {
            0
        } else {
            self.sum / self.count
        }
    }

    fn normalized(self) -> Self {
        if self.count == 0 {
            Self {
                min: 0,
                max: 0,
                sum: 0,
                count: 0,
            }
        } else {
            self
        }
    }
}

impl PercentileStats {
    const fn empty() -> Self {
        Self {
            stats: Stats {
                min: 0,
                max: 0,
                sum: 0,
                count: 0,
            },
            skipped: 0,
            p50: 0,
            p95: 0,
        }
    }
}

impl MutexSpikeAttribution {
    const fn new(threshold_cy: u32) -> Self {
        Self {
            threshold_cy,
            overlap_samples: 0,
            spikes: 0,
            irq_spikes: 0,
            clean_spikes: 0,
            systick_spikes: 0,
            tim2_spikes: 0,
            max_irq_spike_cy: 0,
            max_clean_spike_cy: 0,
        }
    }

    fn observe(&mut self, delta: u32, systick_delta: u32, tim2_delta: u32) {
        let irq_overlap = systick_delta != 0 || tim2_delta != 0;
        if irq_overlap {
            self.overlap_samples = self.overlap_samples.saturating_add(1);
        }

        if delta < self.threshold_cy {
            return;
        }

        self.spikes = self.spikes.saturating_add(1);
        if irq_overlap {
            self.irq_spikes = self.irq_spikes.saturating_add(1);
            self.systick_spikes = self
                .systick_spikes
                .saturating_add((systick_delta != 0) as u32);
            self.tim2_spikes = self.tim2_spikes.saturating_add((tim2_delta != 0) as u32);
            self.max_irq_spike_cy = self.max_irq_spike_cy.max(delta);
        } else {
            self.clean_spikes = self.clean_spikes.saturating_add(1);
            self.max_clean_spike_cy = self.max_clean_spike_cy.max(delta);
        }
    }
}

impl TwoPhaseCleanBreakdown {
    const fn new() -> Self {
        Self {
            clean_spikes: 0,
            first_dominant: 0,
            second_dominant: 0,
            max_first_cy: 0,
            max_second_cy: 0,
        }
    }

    fn observe(&mut self, first_phase: u32, second_phase: u32) {
        self.clean_spikes = self.clean_spikes.saturating_add(1);
        if first_phase >= second_phase {
            self.first_dominant = self.first_dominant.saturating_add(1);
        } else {
            self.second_dominant = self.second_dominant.saturating_add(1);
        }
        self.max_first_cy = self.max_first_cy.max(first_phase);
        self.max_second_cy = self.max_second_cy.max(second_phase);
    }
}

impl FourPhaseCleanBreakdown {
    const fn new() -> Self {
        Self {
            clean_spikes: 0,
            first_dominant: 0,
            second_dominant: 0,
            third_dominant: 0,
            fourth_dominant: 0,
            max_first_cy: 0,
            max_second_cy: 0,
            max_third_cy: 0,
            max_fourth_cy: 0,
        }
    }

    fn observe(
        &mut self,
        first_phase: u32,
        second_phase: u32,
        third_phase: u32,
        fourth_phase: u32,
    ) {
        self.clean_spikes = self.clean_spikes.saturating_add(1);
        let phases = [first_phase, second_phase, third_phase, fourth_phase];
        let mut dominant = 0usize;
        for idx in 1..phases.len() {
            if phases[idx] > phases[dominant] {
                dominant = idx;
            }
        }

        match dominant {
            0 => self.first_dominant = self.first_dominant.saturating_add(1),
            1 => self.second_dominant = self.second_dominant.saturating_add(1),
            2 => self.third_dominant = self.third_dominant.saturating_add(1),
            _ => self.fourth_dominant = self.fourth_dominant.saturating_add(1),
        }

        self.max_first_cy = self.max_first_cy.max(first_phase);
        self.max_second_cy = self.max_second_cy.max(second_phase);
        self.max_third_cy = self.max_third_cy.max(third_phase);
        self.max_fourth_cy = self.max_fourth_cy.max(fourth_phase);
    }
}

pub fn init(dcb: &mut DCB, dwt: &mut DWT, tim2: pac::TIM2, rcc: &mut Rcc, cpu_hz: u32) {
    dcb.enable_trace();
    dwt.enable_cycle_counter();
    dwt.set_cycle_count(0);

    CPU_HZ.store(cpu_hz, Ordering::Relaxed);
    STAGE.store(STAGE_CONTEXT, Ordering::Relaxed);
    TASK_A_PID.store(INVALID_PID, Ordering::Relaxed);
    TASK_B_PID.store(INVALID_PID, Ordering::Relaxed);
    TASK_B_PARKED.store(false, Ordering::Relaxed);

    CTX_PENDING.store(false, Ordering::Relaxed);
    CTX_START.store(0, Ordering::Relaxed);
    CTX_LEFT.store(BENCH_SAMPLES, Ordering::Relaxed);

    IRQ_TIMER_TICKS.store(0, Ordering::Relaxed);
    IRQ_WAITING.store(false, Ordering::Relaxed);
    IRQ_STAMP.store(0, Ordering::Relaxed);
    IRQ_UNBLOCK_DONE_STAMP.store(0, Ordering::Relaxed);
    IRQ_STAMP_SYSTICK_COUNT.store(0, Ordering::Relaxed);
    IRQ_STAMP_TIM2_COUNT.store(0, Ordering::Relaxed);
    IRQ_EVENTS.store(0, Ordering::Relaxed);

    SEM_WAITING.store(false, Ordering::Relaxed);
    SEM_STAMP.store(0, Ordering::Relaxed);
    SEM_STAMP_SYSTICK_COUNT.store(0, Ordering::Relaxed);
    SEM_STAMP_TIM2_COUNT.store(0, Ordering::Relaxed);
    SEM_EVENTS.store(0, Ordering::Relaxed);

    QUEUE_WAITING.store(false, Ordering::Relaxed);
    QUEUE_EVENTS.store(0, Ordering::Relaxed);
    QUEUE_WAKE_STAMP.store(0, Ordering::Relaxed);
    QUEUE_UNBLOCK_DONE_STAMP.store(0, Ordering::Relaxed);
    QUEUE_WAKE_STAMP_SYSTICK_COUNT.store(0, Ordering::Relaxed);
    QUEUE_WAKE_STAMP_TIM2_COUNT.store(0, Ordering::Relaxed);
    QUEUE_SEND_STAMP_SYSTICK_COUNT.store(0, Ordering::Relaxed);
    QUEUE_SEND_STAMP_TIM2_COUNT.store(0, Ordering::Relaxed);

    TIMER_CB_WAITING.store(false, Ordering::Relaxed);
    TIMER_CB_STAMP.store(0, Ordering::Relaxed);
    TIMER_CB_EVENTS.store(0, Ordering::Relaxed);
    TIMEOUT_EARLY_EVENTS.store(0, Ordering::Relaxed);
    MUTEX_WAITER_SEQ.store(0, Ordering::Relaxed);
    MUTEX_DONE_SEQ.store(0, Ordering::Relaxed);
    MUTEX_REQUEST_STAMP.store(0, Ordering::Relaxed);
    MUTEX_UNLOCK_STAMP.store(0, Ordering::Relaxed);

    LAST_SYSTICK_EDGE_CYCLE.store(0, Ordering::Relaxed);
    LAST_SYSTICK_EDGE_TICK.store(0, Ordering::Relaxed);
    SYSTICK_ISR_COUNT.store(0, Ordering::Relaxed);
    BENCH_TIM2_ISR_COUNT.store(0, Ordering::Relaxed);

    SCALE_WORKERS_CREATED.store(0, Ordering::Relaxed);
    SCALE_ACTIVE_TASKS.store(2, Ordering::Relaxed);
    SCALE_WORKER_PIDS.lock(|pids| *pids = [INVALID_PID; SCALE_EXTRA_TASKS]);

    while SEM_BENCH.try_acquire() {}
    while QUEUE_BENCH.recv().is_some() {}

    CTX_STATS.lock(|stats| *stats = Stats::new());
    CTX_SAMPLES.lock(|samples| *samples = [0; BENCH_SAMPLES_USIZE]);
    SEM_STATS.lock(|stats| *stats = Stats::new());
    SEM_ATTR_STATS.lock(|stats| *stats = MutexSpikeAttribution::new(SEM_SPIKE_THRESHOLD_CY));
    MUTEX_WAKE_STATS.lock(|stats| *stats = Stats::new());

    timer::hw_timer::init_tim2(
        tim2,
        rcc,
        IRQ_BENCH_TIMER_HZ,
        TimerMode::Periodic,
        on_bench_timer_tick,
    );

    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench init: profile={} samples={} cpu={}Hz timer={}Hz",
            BUILD_PROFILE, BENCH_SAMPLES, cpu_hz, IRQ_BENCH_TIMER_HZ
        );
    });
}

pub fn register_boot_tasks(task_a_pid: usize, task_b_pid: usize) {
    TASK_A_PID.store(task_a_pid, Ordering::Relaxed);
    TASK_B_PID.store(task_b_pid, Ordering::Relaxed);
}

pub fn task_a(_arg: usize) -> ! {
    if let Some(pid) = kernel::current_pid() {
        TASK_A_PID.store(pid, Ordering::Relaxed);
        log::emergency_log_line("bench: task_a entered");
    }

    log::emergency_log_line("bench: context-switch start");
    run_context_bench();
    print_percentile_stats("context_switch_a_to_b", compute_context_stats());

    STAGE.store(STAGE_PARK_HELPER, Ordering::Relaxed);
    if !wait_for_helper_park() {
        log::log_line("bench: helper park timeout");
    } else {
        log::log_line("bench: helper parked");
    }

    STAGE.store(STAGE_SEM, Ordering::Relaxed);
    TASK_B_PARKED.store(false, Ordering::Relaxed);
    if let Some(pid) = task_b_pid() {
        let _ = kernel::unblock(pid);
    }

    log::log_line("bench: semaphore start");
    let sem_stats = run_semaphore_bench();
    print_stats("semaphore_give_to_taskb_wake", sem_stats.stats);
    print_attribution("semaphore_give_to_taskb_wake", sem_stats.attr);

    repark_helper();

    STAGE.store(STAGE_SLEEP, Ordering::Relaxed);
    log::log_line("bench: sleep-wakeup start");
    let sleep_stats = run_sleep_bench();
    print_stats("sleep_1tick_extra", sleep_stats);

    STAGE.store(STAGE_IRQ, Ordering::Relaxed);
    log::log_line("bench: irq-to-task start");
    let irq_stats = run_irq_bench();
    print_stats("tim2_irq_to_task", irq_stats.stats);
    print_attribution("tim2_irq_to_task", irq_stats.attr);
    print_two_phase_clean_breakdown(
        "tim2_irq_to_task",
        "unblock",
        "resume",
        irq_stats.clean_breakdown,
    );

    STAGE.store(STAGE_QUEUE, Ordering::Relaxed);
    log::log_line("bench: queue-latency start");
    let queue_stats = run_queue_bench();
    print_stats("queue_wake_latency", queue_stats.wake);
    print_attribution("queue_wake_latency", queue_stats.wake_attr);
    print_two_phase_clean_breakdown(
        "queue_wake_latency",
        "unblock",
        "resume",
        queue_stats.wake_clean_breakdown,
    );
    print_stats("queue_end_to_end_latency", queue_stats.end_to_end);
    print_attribution("queue_end_to_end_latency", queue_stats.end_attr);
    print_four_phase_clean_breakdown(
        "queue_end_to_end_latency",
        "send",
        "unblock",
        "resume",
        "recv",
        queue_stats.end_clean_breakdown,
    );

    STAGE.store(STAGE_MUTEX, Ordering::Relaxed);
    log::log_line("bench: mutex-latency start");
    let mutex_stats = run_mutex_bench();
    print_stats("mutex_lock_unlock", mutex_stats.lock_unlock);
    print_attribution("mutex_lock_unlock", mutex_stats.lock_unlock_attr);
    print_stats("mutex_waiter_wake_latency", mutex_stats.waiter_wake);
    print_stats("priority_inheritance_enter_latency", mutex_stats.pi_enter);
    print_stats("priority_inheritance_exit_latency", mutex_stats.pi_exit);
    log::log_line("bench:mutex_priority_inheritance supported=1 mode=blocking_mutex");

    STAGE.store(STAGE_TIMER_CB, Ordering::Relaxed);
    log::log_line("bench: soft-timer-callback start");
    let timer_cb_stats = run_timer_callback_bench();
    print_stats("soft_timer_callback_to_task", timer_cb_stats);

    STAGE.store(STAGE_TIMEOUT, Ordering::Relaxed);
    log::log_line("bench: timeout-wheel start");
    run_timeout_validation_bench();

    STAGE.store(STAGE_SCALE, Ordering::Relaxed);
    log::log_line("bench: scheduler-scale start");
    run_scaling_bench();

    STAGE.store(STAGE_DONE, Ordering::Relaxed);
    log::log_line("bench complete");

    loop {
        kernel::sleep_ms(1000);
    }
}

pub fn task_b(_arg: usize) -> ! {
    if let Some(pid) = kernel::current_pid() {
        TASK_B_PID.store(pid, Ordering::Relaxed);
        log::emergency_log_line("bench: task_b entered");
    }

    loop {
        match STAGE.load(Ordering::Relaxed) {
            STAGE_CONTEXT => {
                if CTX_PENDING.swap(false, Ordering::Relaxed) {
                    let left = CTX_LEFT.load(Ordering::Relaxed);
                    if left > 0 {
                        let delta =
                            DWT::cycle_count().wrapping_sub(CTX_START.load(Ordering::Relaxed));
                        let index = BENCH_SAMPLES.saturating_sub(left) as usize;
                        CTX_SAMPLES.lock(|samples| {
                            if index < samples.len() {
                                samples[index] = delta;
                            }
                        });
                        CTX_STATS.lock(|stats| stats.update(delta));
                        CTX_LEFT.store(left - 1, Ordering::Relaxed);
                    }
                }
                kernel::yield_now();
            }
            STAGE_PARK_HELPER => {
                // Atomically publish "parked" and block, so task_a cannot observe
                // parked=true before this task is actually moved to Blocked.
                interrupt::free(|_| {
                    if STAGE.load(Ordering::Relaxed) == STAGE_PARK_HELPER {
                        TASK_B_PARKED.store(true, Ordering::Relaxed);
                        kernel::block_current(None);
                    }
                });
            }
            STAGE_SEM => {
                TASK_B_PARKED.store(false, Ordering::Relaxed);
                SEM_WAITING.store(true, Ordering::Relaxed);
                if SEM_BENCH.acquire(Some(IRQ_TIMEOUT_MS)).is_ok() {
                    let (delta, systick_after, tim2_after) = interrupt::free(|_| {
                        (
                            DWT::cycle_count().wrapping_sub(SEM_STAMP.load(Ordering::Relaxed)),
                            SYSTICK_ISR_COUNT.load(Ordering::Relaxed),
                            BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
                        )
                    });
                    let systick_delta =
                        systick_after.wrapping_sub(SEM_STAMP_SYSTICK_COUNT.load(Ordering::Relaxed));
                    let tim2_delta =
                        tim2_after.wrapping_sub(SEM_STAMP_TIM2_COUNT.load(Ordering::Relaxed));
                    SEM_STATS.lock(|stats| stats.update(delta));
                    SEM_ATTR_STATS.lock(|stats| stats.observe(delta, systick_delta, tim2_delta));
                    SEM_EVENTS.fetch_add(1, Ordering::Relaxed);
                }
                SEM_WAITING.store(false, Ordering::Relaxed);
                kernel::yield_now();
            }
            STAGE_MUTEX => {
                let target = MUTEX_WAITER_SEQ.load(Ordering::Relaxed);
                let done = MUTEX_DONE_SEQ.load(Ordering::Relaxed);
                if target > done {
                    MUTEX_REQUEST_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
                    if PI_MUTEX_BENCH.acquire(Some(IRQ_TIMEOUT_MS)).is_ok() {
                        let unlock_stamp = MUTEX_UNLOCK_STAMP.load(Ordering::Relaxed);
                        if unlock_stamp != 0 {
                            let delta = DWT::cycle_count().wrapping_sub(unlock_stamp);
                            MUTEX_WAKE_STATS.lock(|stats| stats.update(delta));
                        }
                        let _ = PI_MUTEX_BENCH.with_owner(|value| {
                            *value = value.wrapping_add(1);
                        });
                        let _ = PI_MUTEX_BENCH.release();
                    } else {
                        log_sample_timeout("mutex_waiter", target.saturating_sub(1));
                    }

                    MUTEX_DONE_SEQ.store(target, Ordering::Relaxed);
                    kernel::block_current(None);
                } else {
                    kernel::block_current(None);
                }
            }
            STAGE_SCALE => kernel::block_current(None),
            _ => kernel::sleep_ms(5),
        }
    }
}

fn scale_worker(arg: usize) -> ! {
    let slot = arg;
    loop {
        if STAGE.load(Ordering::Relaxed) == STAGE_SCALE
            && slot < SCALE_ACTIVE_TASKS.load(Ordering::Relaxed)
        {
            kernel::yield_now();
        } else {
            kernel::block_current(None);
        }
    }
}

fn run_context_bench() {
    STAGE.store(STAGE_CONTEXT, Ordering::Relaxed);

    while CTX_LEFT.load(Ordering::Relaxed) > 0 {
        if !CTX_PENDING.load(Ordering::Relaxed) {
            CTX_START.store(DWT::cycle_count(), Ordering::Relaxed);
            CTX_PENDING.store(true, Ordering::Relaxed);
        }
        kernel::yield_now();
    }

    CTX_PENDING.store(false, Ordering::Relaxed);
}

fn compute_context_stats() -> PercentileStats {
    let total = CTX_STATS
        .lock(|stats| stats.count as usize)
        .min(BENCH_SAMPLES_USIZE);
    let skipped = total.min(CONTEXT_SKIP_SAMPLES);
    let kept = total.saturating_sub(skipped);

    if kept == 0 {
        return PercentileStats::empty();
    }

    let mut sorted = [0u32; BENCH_SAMPLES_USIZE];
    CTX_SAMPLES.lock(|samples| {
        for (dst, src) in sorted.iter_mut().zip(samples.iter()).take(total) {
            *dst = *src;
        }
    });

    let mut stats = Stats::new();
    for sample in sorted.iter().take(total).skip(skipped) {
        stats.update(*sample);
    }

    let window = &mut sorted[skipped..total];
    window.sort_unstable();

    PercentileStats {
        stats: stats.normalized(),
        skipped: skipped as u32,
        p50: percentile_nearest_rank(window, 50),
        p95: percentile_nearest_rank(window, 95),
    }
}

fn run_semaphore_bench() -> MetricBenchStats {
    while SEM_BENCH.try_acquire() {}
    SEM_WAITING.store(false, Ordering::Relaxed);
    SEM_EVENTS.store(0, Ordering::Relaxed);
    SEM_STATS.lock(|stats| *stats = Stats::new());
    SEM_ATTR_STATS.lock(|stats| *stats = MutexSpikeAttribution::new(SEM_SPIKE_THRESHOLD_CY));

    for sample in 0..BENCH_SAMPLES {
        if !wait_for_flag(&SEM_WAITING, true) {
            log_sample_timeout("semaphore_waiter", sample);
            break;
        }

        SEM_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
        SEM_STAMP_SYSTICK_COUNT.store(SYSTICK_ISR_COUNT.load(Ordering::Relaxed), Ordering::Relaxed);
        SEM_STAMP_TIM2_COUNT.store(
            BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        let _ = SEM_BENCH.release();

        if !wait_for_counter(&SEM_EVENTS, sample + 1) {
            log_sample_timeout("semaphore_wake", sample);
            break;
        }
    }

    MetricBenchStats {
        stats: SEM_STATS.lock(|stats| stats.normalized()),
        attr: SEM_ATTR_STATS.lock(|stats| *stats),
    }
}

fn run_sleep_bench() -> Stats {
    let mut stats = Stats::new();

    for sample in 0..BENCH_SAMPLES {
        if !wait_for_systick_edge() {
            log_sample_timeout("sleep", sample);
            break;
        }

        kernel::sleep_ms(1);
        // Measure wakeup latency from the most recent SysTick edge.
        // Snapshot inside a critical section to avoid edge/update races.
        let latency = interrupt::free(|_| {
            let edge = LAST_SYSTICK_EDGE_CYCLE.load(Ordering::Relaxed);
            DWT::cycle_count().wrapping_sub(edge)
        });
        stats.update(latency);
    }

    stats.normalized()
}

fn run_irq_bench() -> IrqBenchStats {
    let mut stats = Stats::new();
    let mut attr = MutexSpikeAttribution::new(IRQ_SPIKE_THRESHOLD_CY);
    let mut clean_breakdown = TwoPhaseCleanBreakdown::new();

    for sample in 0..BENCH_SAMPLES {
        if !wait_for_timer_edge() {
            log_sample_timeout("irq_wait_edge", sample);
            break;
        }

        IRQ_UNBLOCK_DONE_STAMP.store(0, Ordering::Relaxed);
        let events_before = IRQ_EVENTS.load(Ordering::Relaxed);
        interrupt::free(|_| {
            IRQ_WAITING.store(true, Ordering::Relaxed);
            kernel::block_current(Some(IRQ_TIMEOUT_MS));
        });

        let events_after = IRQ_EVENTS.load(Ordering::Relaxed);
        if events_after != events_before {
            let (delta, irq_stamp, unblock_done, now, systick_after, tim2_after) =
                interrupt::free(|_| {
                    let now = DWT::cycle_count();
                    let irq_stamp = IRQ_STAMP.load(Ordering::Relaxed);
                    (
                        now.wrapping_sub(irq_stamp),
                        irq_stamp,
                        IRQ_UNBLOCK_DONE_STAMP.load(Ordering::Relaxed),
                        now,
                        SYSTICK_ISR_COUNT.load(Ordering::Relaxed),
                        BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
                    )
                });
            let systick_delta =
                systick_after.wrapping_sub(IRQ_STAMP_SYSTICK_COUNT.load(Ordering::Relaxed));
            let tim2_delta = tim2_after.wrapping_sub(IRQ_STAMP_TIM2_COUNT.load(Ordering::Relaxed));
            attr.observe(delta, systick_delta, tim2_delta);
            if delta >= IRQ_SPIKE_THRESHOLD_CY && systick_delta == 0 && tim2_delta == 0 {
                let unblock_phase = unblock_done.wrapping_sub(irq_stamp);
                let resume_phase = now.wrapping_sub(unblock_done);
                clean_breakdown.observe(unblock_phase, resume_phase);
            }
            stats.update(delta);
        } else {
            log_sample_timeout("irq_block", sample);
        }
    }

    IrqBenchStats {
        stats: stats.normalized(),
        attr,
        clean_breakdown,
    }
}

fn run_queue_bench() -> QueueBenchStats {
    while QUEUE_BENCH.recv().is_some() {}
    QUEUE_WAITING.store(false, Ordering::Relaxed);
    QUEUE_EVENTS.store(0, Ordering::Relaxed);
    QUEUE_WAKE_STAMP.store(0, Ordering::Relaxed);

    let mut wake_stats = Stats::new();
    let mut end_to_end_stats = Stats::new();
    let mut wake_attr = MutexSpikeAttribution::new(QUEUE_WAKE_SPIKE_THRESHOLD_CY);
    let mut end_attr = MutexSpikeAttribution::new(QUEUE_END_SPIKE_THRESHOLD_CY);
    let mut wake_clean_breakdown = TwoPhaseCleanBreakdown::new();
    let mut end_clean_breakdown = FourPhaseCleanBreakdown::new();

    for sample in 0..BENCH_SAMPLES {
        if !wait_for_timer_edge() {
            log_sample_timeout("queue_wait_edge", sample);
            break;
        }

        QUEUE_UNBLOCK_DONE_STAMP.store(0, Ordering::Relaxed);
        let events_before = QUEUE_EVENTS.load(Ordering::Relaxed);
        interrupt::free(|_| {
            QUEUE_WAITING.store(true, Ordering::Relaxed);
            kernel::block_current(Some(IRQ_TIMEOUT_MS));
        });

        let events_after = QUEUE_EVENTS.load(Ordering::Relaxed);
        if events_after == events_before {
            log_sample_timeout("queue_block", sample);
            continue;
        }

        let (wake_delta, wake_stamp, unblock_done, wake_now, systick_after, tim2_after) =
            interrupt::free(|_| {
                let wake_now = DWT::cycle_count();
                let wake_stamp = QUEUE_WAKE_STAMP.load(Ordering::Relaxed);
                (
                    wake_now.wrapping_sub(wake_stamp),
                    wake_stamp,
                    QUEUE_UNBLOCK_DONE_STAMP.load(Ordering::Relaxed),
                    wake_now,
                    SYSTICK_ISR_COUNT.load(Ordering::Relaxed),
                    BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
                )
            });
        let wake_systick_delta =
            systick_after.wrapping_sub(QUEUE_WAKE_STAMP_SYSTICK_COUNT.load(Ordering::Relaxed));
        let wake_tim2_delta =
            tim2_after.wrapping_sub(QUEUE_WAKE_STAMP_TIM2_COUNT.load(Ordering::Relaxed));
        wake_attr.observe(wake_delta, wake_systick_delta, wake_tim2_delta);
        if wake_delta >= QUEUE_WAKE_SPIKE_THRESHOLD_CY
            && wake_systick_delta == 0
            && wake_tim2_delta == 0
        {
            let unblock_phase = unblock_done.wrapping_sub(wake_stamp);
            let resume_phase = wake_now.wrapping_sub(unblock_done);
            wake_clean_breakdown.observe(unblock_phase, resume_phase);
        }
        wake_stats.update(wake_delta);

        let recv_start = DWT::cycle_count();
        if let Some(stamp) = QUEUE_BENCH.recv() {
            let recv_end = DWT::cycle_count();
            let (delta, wake_stamp, unblock_done, systick_after, tim2_after) =
                interrupt::free(|_| {
                    (
                        recv_end.wrapping_sub(stamp as u32),
                        QUEUE_WAKE_STAMP.load(Ordering::Relaxed),
                        QUEUE_UNBLOCK_DONE_STAMP.load(Ordering::Relaxed),
                        SYSTICK_ISR_COUNT.load(Ordering::Relaxed),
                        BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
                    )
                });
            let end_systick_delta =
                systick_after.wrapping_sub(QUEUE_SEND_STAMP_SYSTICK_COUNT.load(Ordering::Relaxed));
            let end_tim2_delta =
                tim2_after.wrapping_sub(QUEUE_SEND_STAMP_TIM2_COUNT.load(Ordering::Relaxed));
            end_attr.observe(delta, end_systick_delta, end_tim2_delta);
            if delta >= QUEUE_END_SPIKE_THRESHOLD_CY
                && end_systick_delta == 0
                && end_tim2_delta == 0
            {
                let send_phase = wake_stamp.wrapping_sub(stamp as u32);
                let unblock_phase = unblock_done.wrapping_sub(wake_stamp);
                let resume_phase = recv_start.wrapping_sub(unblock_done);
                let recv_phase = recv_end.wrapping_sub(recv_start);
                end_clean_breakdown.observe(send_phase, unblock_phase, resume_phase, recv_phase);
            }
            end_to_end_stats.update(delta);
        } else {
            log_sample_timeout("queue_recv", sample);
        }
    }

    QueueBenchStats {
        wake: wake_stats.normalized(),
        wake_attr,
        wake_clean_breakdown,
        end_to_end: end_to_end_stats.normalized(),
        end_attr,
        end_clean_breakdown,
    }
}

fn run_mutex_bench() -> MutexBenchStats {
    let mut lock_unlock = Stats::new();
    let mut lock_unlock_attr = MutexSpikeAttribution::new(MUTEX_LOCK_SPIKE_THRESHOLD_CY);

    for _ in 0..BENCH_SAMPLES {
        let systick_before = SYSTICK_ISR_COUNT.load(Ordering::Relaxed);
        let tim2_before = BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed);
        let start = DWT::cycle_count();
        MUTEX_BENCH.lock(|value| {
            *value = value.wrapping_add(1);
        });
        let (delta, systick_after, tim2_after) = interrupt::free(|_| {
            (
                DWT::cycle_count().wrapping_sub(start),
                SYSTICK_ISR_COUNT.load(Ordering::Relaxed),
                BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
            )
        });
        lock_unlock_attr.observe(
            delta,
            systick_after.wrapping_sub(systick_before),
            tim2_after.wrapping_sub(tim2_before),
        );
        lock_unlock.update(delta);
    }

    let mut waiter_wake = Stats::new();
    let mut pi_enter = Stats::new();
    let mut pi_exit = Stats::new();

    MUTEX_WAITER_SEQ.store(0, Ordering::Relaxed);
    MUTEX_DONE_SEQ.store(0, Ordering::Relaxed);
    MUTEX_REQUEST_STAMP.store(0, Ordering::Relaxed);
    MUTEX_UNLOCK_STAMP.store(0, Ordering::Relaxed);
    MUTEX_WAKE_STATS.lock(|stats| *stats = Stats::new());

    let Some(task_a_pid) = kernel::current_pid() else {
        return MutexBenchStats {
            lock_unlock: lock_unlock.normalized(),
            lock_unlock_attr,
            waiter_wake: waiter_wake.normalized(),
            pi_enter: pi_enter.normalized(),
            pi_exit: pi_exit.normalized(),
        };
    };
    let Some(task_b_pid) = task_b_pid() else {
        return MutexBenchStats {
            lock_unlock: lock_unlock.normalized(),
            lock_unlock_attr,
            waiter_wake: waiter_wake.normalized(),
            pi_enter: pi_enter.normalized(),
            pi_exit: pi_exit.normalized(),
        };
    };

    let original_a_prio = kernel::current_priority().unwrap_or(1);
    let original_b_prio = kernel::task_priority(task_b_pid).unwrap_or(1);

    let _ = kernel::unblock(task_b_pid);
    kernel::yield_now();

    let _ = kernel::set_priority(task_a_pid, MUTEX_PI_OWNER_PRIO);
    let _ = kernel::set_priority(task_b_pid, original_b_prio.min(original_a_prio));

    for sample in 0..BENCH_SAMPLES {
        if PI_MUTEX_BENCH.acquire(Some(IRQ_TIMEOUT_MS)).is_err() {
            log_sample_timeout("mutex_owner_lock", sample);
            break;
        }
        let _ = PI_MUTEX_BENCH.with_owner(|value| {
            *value = value.wrapping_add(1);
        });

        let seq = sample + 1;
        MUTEX_REQUEST_STAMP.store(0, Ordering::Relaxed);
        MUTEX_UNLOCK_STAMP.store(0, Ordering::Relaxed);
        MUTEX_WAITER_SEQ.store(seq, Ordering::Relaxed);

        let _ = kernel::unblock(task_b_pid);
        kernel::yield_now();

        if !wait_for_nonzero(&MUTEX_REQUEST_STAMP) {
            let _ = PI_MUTEX_BENCH.release();
            log_sample_timeout("mutex_pi_enter", sample);
            break;
        }

        let request_stamp = MUTEX_REQUEST_STAMP.load(Ordering::Relaxed);
        if request_stamp == 0 || !wait_for_current_priority(original_b_prio.min(original_a_prio)) {
            let _ = PI_MUTEX_BENCH.release();
            log_sample_timeout("mutex_pi_boost", sample);
            break;
        }

        pi_enter.update(DWT::cycle_count().wrapping_sub(request_stamp));

        let unlock_stamp = DWT::cycle_count();
        MUTEX_UNLOCK_STAMP.store(unlock_stamp, Ordering::Relaxed);
        if PI_MUTEX_BENCH.release().is_err() {
            log_sample_timeout("mutex_owner_unlock", sample);
            break;
        }

        if !wait_for_counter(&MUTEX_DONE_SEQ, seq) {
            log_sample_timeout("mutex_waiter_done", sample);
            break;
        }

        if kernel::current_priority() == Some(MUTEX_PI_OWNER_PRIO) {
            pi_exit.update(DWT::cycle_count().wrapping_sub(unlock_stamp));
        } else {
            log_sample_timeout("mutex_pi_restore", sample);
            break;
        }
    }

    let _ = kernel::set_priority(task_a_pid, original_a_prio);
    let _ = kernel::set_priority(task_b_pid, original_b_prio);
    let _ = kernel::unblock(task_b_pid);

    waiter_wake = MUTEX_WAKE_STATS.lock(|stats| stats.normalized());

    MutexBenchStats {
        lock_unlock: lock_unlock.normalized(),
        lock_unlock_attr,
        waiter_wake,
        pi_enter: pi_enter.normalized(),
        pi_exit: pi_exit.normalized(),
    }
}

fn run_timer_callback_bench() -> Stats {
    TIMER_CB_WAITING.store(false, Ordering::Relaxed);
    TIMER_CB_STAMP.store(0, Ordering::Relaxed);
    TIMER_CB_EVENTS.store(0, Ordering::Relaxed);

    let mut stats = Stats::new();

    for sample in 0..BENCH_SAMPLES {
        let events_before = TIMER_CB_EVENTS.load(Ordering::Relaxed);

        let armed = interrupt::free(|_| {
            TIMER_CB_WAITING.store(true, Ordering::Relaxed);
            if kernel::start_timer_oneshot(1, on_soft_timer_callback, 0).is_some() {
                kernel::block_current(Some(IRQ_TIMEOUT_MS));
                true
            } else {
                TIMER_CB_WAITING.store(false, Ordering::Relaxed);
                false
            }
        });

        if !armed {
            log_sample_timeout("soft_timer_arm", sample);
            break;
        }

        let events_after = TIMER_CB_EVENTS.load(Ordering::Relaxed);
        if events_after == events_before {
            log_sample_timeout("soft_timer_wait", sample);
            continue;
        }

        let delta = DWT::cycle_count().wrapping_sub(TIMER_CB_STAMP.load(Ordering::Relaxed));
        stats.update(delta);
    }

    stats.normalized()
}

fn run_timeout_validation_bench() {
    let sleep_1tick = validate_sleep_delay(1, TIMEOUT_VALIDATION_SAMPLES);
    print_validation("timeout_wheel_sleep_1tick", 1, 2, sleep_1tick);

    let cross_bucket =
        validate_cross_bucket_delay(TIMEOUT_CROSS_BUCKET_DELAY_TICKS, TIMEOUT_VALIDATION_SAMPLES);
    print_validation(
        "timeout_wheel_cross_bucket",
        TIMEOUT_CROSS_BUCKET_DELAY_TICKS,
        TIMEOUT_CROSS_BUCKET_DELAY_TICKS + 1,
        cross_bucket,
    );

    let long_delay = validate_sleep_delay(TIMEOUT_LONG_DELAY_TICKS, TIMEOUT_VALIDATION_SAMPLES);
    print_validation(
        "timeout_wheel_long_delay",
        TIMEOUT_LONG_DELAY_TICKS,
        TIMEOUT_LONG_DELAY_TICKS + 1,
        long_delay,
    );

    let early_unblock = validate_early_unblock(
        TIMEOUT_LONG_DELAY_TICKS,
        TIMEOUT_EARLY_UNBLOCK_TICKS,
        TIMEOUT_VALIDATION_SAMPLES,
    );
    print_validation(
        "timeout_wheel_early_unblock",
        TIMEOUT_EARLY_UNBLOCK_TICKS,
        TIMEOUT_EARLY_UNBLOCK_TICKS + 1,
        early_unblock,
    );

    let wrap_ok = crate::task::scheduler::bench_validate_timeout_wraparound();
    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:timeout_wheel_wraparound pass={} fail={}",
            if wrap_ok { 1 } else { 0 },
            if wrap_ok { 0 } else { 1 }
        );
    });
}

fn validate_sleep_delay(delay_ticks: u32, samples: u32) -> ValidationStats {
    let mut stats = ValidationStats::new();

    for sample in 0..samples {
        if !wait_for_systick_edge() {
            log_sample_timeout("timeout_sleep_edge", sample);
            stats.observe(0, false);
            break;
        }

        let start = kernel::now_ticks();
        kernel::sleep_ms(delay_ticks);
        let elapsed = kernel::now_ticks().wrapping_sub(start);
        let passed = elapsed >= delay_ticks && elapsed <= delay_ticks.saturating_add(1);
        stats.observe(elapsed, passed);
    }

    stats.normalized()
}

fn validate_cross_bucket_delay(delay_ticks: u32, samples: u32) -> ValidationStats {
    let mut stats = ValidationStats::new();
    let target_mod = crate::task::scheduler::TIMEOUT_WHEEL_SIZE as u32 - 1;

    for sample in 0..samples {
        if !wait_for_systick_edge_mod(target_mod) {
            log_sample_timeout("timeout_cross_bucket_edge", sample);
            stats.observe(0, false);
            break;
        }

        let start = kernel::now_ticks();
        kernel::sleep_ms(delay_ticks);
        let elapsed = kernel::now_ticks().wrapping_sub(start);
        let passed = elapsed >= delay_ticks && elapsed <= delay_ticks.saturating_add(1);
        stats.observe(elapsed, passed);
    }

    stats.normalized()
}

fn validate_early_unblock(timeout_ticks: u32, unblock_ticks: u32, samples: u32) -> ValidationStats {
    let mut stats = ValidationStats::new();
    TIMEOUT_EARLY_EVENTS.store(0, Ordering::Relaxed);

    for sample in 0..samples {
        if !wait_for_systick_edge() {
            log_sample_timeout("timeout_early_unblock_edge", sample);
            stats.observe(0, false);
            break;
        }

        let before = TIMEOUT_EARLY_EVENTS.load(Ordering::Relaxed);
        let start = kernel::now_ticks();
        let armed = interrupt::free(|_| {
            if kernel::start_timer_oneshot(unblock_ticks, on_timeout_early_unblock, 0).is_some() {
                kernel::block_current(Some(timeout_ticks));
                true
            } else {
                false
            }
        });

        if !armed {
            log_sample_timeout("timeout_early_unblock_arm", sample);
            stats.observe(0, false);
            break;
        }

        let elapsed = kernel::now_ticks().wrapping_sub(start);
        let after = TIMEOUT_EARLY_EVENTS.load(Ordering::Relaxed);
        let fired = after == before.saturating_add(1);
        let passed = fired
            && elapsed >= unblock_ticks
            && elapsed <= unblock_ticks.saturating_add(1)
            && elapsed < timeout_ticks;
        stats.observe(elapsed, passed);
    }

    stats.normalized()
}

fn run_scaling_bench() {
    let created_workers = ensure_scale_workers();
    let mut per_switch_avg: [u32; SCALE_CASES.len()] = [0; SCALE_CASES.len()];
    let mut valid_cases = 0usize;

    for (case_index, tasks) in SCALE_CASES.iter().enumerate() {
        if *tasks < 2 {
            continue;
        }

        let needed_workers = tasks - 1;
        if created_workers < needed_workers {
            log::with_logger(|tx| {
                let _ = writeln!(
                    tx,
                    "bench:scheduler_scale tasks={} skipped workers={} needed={}",
                    tasks, created_workers, needed_workers
                );
            });
            continue;
        }

        let available = unblock_scale_workers(needed_workers);
        if available < needed_workers {
            log::with_logger(|tx| {
                let _ = writeln!(
                    tx,
                    "bench:scheduler_scale tasks={} skipped available={} needed={}",
                    tasks, available, needed_workers
                );
            });
            continue;
        }

        SCALE_ACTIVE_TASKS.store(*tasks, Ordering::Relaxed);

        for _ in 0..(tasks * 3) {
            kernel::yield_now();
        }

        let mut stats = Stats::new();
        let mut prev = DWT::cycle_count();

        for _ in 0..BENCH_SAMPLES {
            kernel::yield_now();
            let now = DWT::cycle_count();
            stats.update(now.wrapping_sub(prev));
            prev = now;
        }

        let stats = stats.normalized();
        print_scaling_stats(*tasks, stats);

        let task_count = *tasks as u32;
        if task_count > 0 {
            per_switch_avg[case_index] = stats.avg() / task_count;
            valid_cases += 1;
        }
    }

    if valid_cases == SCALE_CASES.len() {
        let steady_values = [per_switch_avg[1], per_switch_avg[2]];
        let mut min = u32::MAX;
        let mut max = 0;

        for value in steady_values {
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
        }

        if min > 0 {
            let ratio_permille = max.saturating_mul(1000) / min;
            let verdict = if ratio_permille <= 1200 {
                "likely_o1"
            } else {
                "not_o1"
            };

            log::with_logger(|tx| {
                let _ = writeln!(
                    tx,
                    "bench:scheduler_o1_check steady_per_switch_avg_8_32={}/{}cy ratio={}permille baseline_2task={}cy verdict={}",
                    per_switch_avg[1],
                    per_switch_avg[2],
                    ratio_permille,
                    per_switch_avg[0],
                    verdict
                );
            });
        }
    }

    SCALE_ACTIVE_TASKS.store(2, Ordering::Relaxed);
}

fn ensure_scale_workers() -> usize {
    let mut created = SCALE_WORKERS_CREATED.load(Ordering::Relaxed);

    while created < SCALE_EXTRA_TASKS {
        let stack = unsafe {
            let ptr = core::ptr::addr_of_mut!(SCALE_STACKS[created].0) as *mut u32;
            core::slice::from_raw_parts_mut(ptr, SCALE_STACK_WORDS)
        };

        if let Some(pid) = kernel::create_task(scale_worker, created + 1, stack, 1) {
            SCALE_WORKER_PIDS.lock(|pids| {
                if created < pids.len() {
                    pids[created] = pid;
                }
            });
            let _ = kernel::unblock(pid);
            created += 1;
            SCALE_WORKERS_CREATED.store(created, Ordering::Relaxed);
        } else {
            break;
        }
    }

    created
}

fn unblock_scale_workers(count: usize) -> usize {
    let mut available = 0usize;
    SCALE_WORKER_PIDS.lock(|pids| {
        let limit = count.min(pids.len());
        for pid in pids.iter().take(limit) {
            if *pid != INVALID_PID {
                available += 1;
                let _ = kernel::unblock(*pid);
            }
        }
    });
    available
}

fn print_scaling_stats(tasks: usize, stats: Stats) {
    let task_count = tasks as u32;
    if task_count == 0 {
        return;
    }

    let per_switch_min = stats.min / task_count;
    let per_switch_avg = stats.avg() / task_count;
    let per_switch_max = stats.max / task_count;

    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:scheduler_scale tasks={} rounds={} round_min={}cy/{}us round_avg={}cy/{}us round_max={}cy/{}us per_switch_min={}cy/{}us per_switch_avg={}cy/{}us per_switch_max={}cy/{}us",
            tasks,
            stats.count,
            stats.min,
            cycles_to_us(stats.min),
            stats.avg(),
            cycles_to_us(stats.avg()),
            stats.max,
            cycles_to_us(stats.max),
            per_switch_min,
            cycles_to_us(per_switch_min),
            per_switch_avg,
            cycles_to_us(per_switch_avg),
            per_switch_max,
            cycles_to_us(per_switch_max),
        );
    });
}

fn percentile_nearest_rank(sorted: &[u32], percent: u32) -> u32 {
    if sorted.is_empty() {
        return 0;
    }

    let rank = ((sorted.len() as u32)
        .saturating_mul(percent)
        .saturating_add(99)
        / 100)
        .saturating_sub(1) as usize;

    sorted[rank.min(sorted.len() - 1)]
}

fn repark_helper() {
    TASK_B_PARKED.store(false, Ordering::Relaxed);
    STAGE.store(STAGE_PARK_HELPER, Ordering::Relaxed);
    if let Some(pid) = task_b_pid() {
        let _ = kernel::unblock(pid);
    }

    if !wait_for_helper_park() {
        log::log_line("bench: helper re-park timeout");
    }
}

fn task_b_pid() -> Option<usize> {
    let pid = TASK_B_PID.load(Ordering::Relaxed);
    if pid == INVALID_PID { None } else { Some(pid) }
}

fn wait_for_systick_edge() -> bool {
    let now = kernel::now_ticks();
    let start = DWT::cycle_count();
    let timeout_cycles = cycles_per_tick().saturating_mul(WAIT_TIMEOUT_TICKS);
    while kernel::now_ticks() == now {
        if DWT::cycle_count().wrapping_sub(start) >= timeout_cycles {
            return false;
        }
        cortex_m::asm::nop();
    }
    true
}

fn wait_for_systick_edge_mod(target_mod: u32) -> bool {
    for _ in 0..=(crate::task::scheduler::TIMEOUT_WHEEL_SIZE as u32) {
        if !wait_for_systick_edge() {
            return false;
        }
        if kernel::now_ticks() % (crate::task::scheduler::TIMEOUT_WHEEL_SIZE as u32) == target_mod {
            return true;
        }
    }
    false
}

fn wait_for_timer_edge() -> bool {
    let tick = IRQ_TIMER_TICKS.load(Ordering::Relaxed);
    let start = DWT::cycle_count();
    let timeout_cycles = cycles_per_tick().saturating_mul(WAIT_TIMEOUT_TICKS);
    while IRQ_TIMER_TICKS.load(Ordering::Relaxed) == tick {
        if DWT::cycle_count().wrapping_sub(start) >= timeout_cycles {
            return false;
        }
        cortex_m::asm::nop();
    }
    true
}

fn wait_for_helper_park() -> bool {
    let start = DWT::cycle_count();
    let timeout_cycles = cycles_per_tick().saturating_mul(WAIT_TIMEOUT_TICKS);
    while !TASK_B_PARKED.load(Ordering::Relaxed) {
        if DWT::cycle_count().wrapping_sub(start) >= timeout_cycles {
            return false;
        }
        kernel::yield_now();
    }
    true
}

fn wait_for_flag(flag: &AtomicBool, expected: bool) -> bool {
    let start = DWT::cycle_count();
    let timeout_cycles = cycles_per_tick().saturating_mul(WAIT_TIMEOUT_TICKS);

    while flag.load(Ordering::Relaxed) != expected {
        if DWT::cycle_count().wrapping_sub(start) >= timeout_cycles {
            return false;
        }
        kernel::yield_now();
    }

    true
}

fn wait_for_nonzero(value: &AtomicU32) -> bool {
    let start = DWT::cycle_count();
    let timeout_cycles = cycles_per_tick().saturating_mul(WAIT_TIMEOUT_TICKS);

    loop {
        let current = value.load(Ordering::Relaxed);
        if current != 0 {
            return true;
        }

        if DWT::cycle_count().wrapping_sub(start) >= timeout_cycles {
            return false;
        }
        kernel::yield_now();
    }
}

fn wait_for_current_priority(target: u8) -> bool {
    let start = DWT::cycle_count();
    let timeout_cycles = cycles_per_tick().saturating_mul(WAIT_TIMEOUT_TICKS);

    while kernel::current_priority() != Some(target) {
        if DWT::cycle_count().wrapping_sub(start) >= timeout_cycles {
            return false;
        }
        kernel::yield_now();
    }

    true
}

fn wait_for_counter(counter: &AtomicU32, target: u32) -> bool {
    let start = DWT::cycle_count();
    let timeout_cycles = cycles_per_tick().saturating_mul(WAIT_TIMEOUT_TICKS);

    while counter.load(Ordering::Relaxed) < target {
        if DWT::cycle_count().wrapping_sub(start) >= timeout_cycles {
            return false;
        }
        kernel::yield_now();
    }

    true
}

fn on_bench_timer_tick() {
    IRQ_TIMER_TICKS.fetch_add(1, Ordering::Relaxed);
    BENCH_TIM2_ISR_COUNT.fetch_add(1, Ordering::Relaxed);

    match STAGE.load(Ordering::Relaxed) {
        STAGE_IRQ => {
            if IRQ_WAITING.swap(false, Ordering::Relaxed) {
                IRQ_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
                IRQ_STAMP_SYSTICK_COUNT
                    .store(SYSTICK_ISR_COUNT.load(Ordering::Relaxed), Ordering::Relaxed);
                IRQ_STAMP_TIM2_COUNT.store(
                    BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
                IRQ_EVENTS.fetch_add(1, Ordering::Relaxed);

                let pid = TASK_A_PID.load(Ordering::Relaxed);
                if pid != INVALID_PID {
                    let _ = kernel::unblock(pid);
                    IRQ_UNBLOCK_DONE_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
                }
            }
        }
        STAGE_QUEUE => {
            if QUEUE_WAITING.swap(false, Ordering::Relaxed) {
                let send_stamp = DWT::cycle_count();
                if QUEUE_BENCH.send_from_isr(send_stamp as usize).is_ok() {
                    QUEUE_SEND_STAMP_SYSTICK_COUNT
                        .store(SYSTICK_ISR_COUNT.load(Ordering::Relaxed), Ordering::Relaxed);
                    QUEUE_SEND_STAMP_TIM2_COUNT.store(
                        BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    QUEUE_EVENTS.fetch_add(1, Ordering::Relaxed);
                }
                QUEUE_WAKE_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
                QUEUE_WAKE_STAMP_SYSTICK_COUNT
                    .store(SYSTICK_ISR_COUNT.load(Ordering::Relaxed), Ordering::Relaxed);
                QUEUE_WAKE_STAMP_TIM2_COUNT.store(
                    BENCH_TIM2_ISR_COUNT.load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );

                let pid = TASK_A_PID.load(Ordering::Relaxed);
                if pid != INVALID_PID {
                    let _ = kernel::unblock(pid);
                    QUEUE_UNBLOCK_DONE_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
                }
            }
        }
        _ => {}
    }
}

fn on_soft_timer_callback(_arg: usize) {
    TIMER_CB_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
    TIMER_CB_EVENTS.fetch_add(1, Ordering::Relaxed);

    if TIMER_CB_WAITING.swap(false, Ordering::Relaxed) {
        let pid = TASK_A_PID.load(Ordering::Relaxed);
        if pid != INVALID_PID {
            let _ = kernel::unblock(pid);
        }
    }
}

fn on_timeout_early_unblock(_arg: usize) {
    TIMEOUT_EARLY_EVENTS.fetch_add(1, Ordering::Relaxed);

    let pid = TASK_A_PID.load(Ordering::Relaxed);
    if pid != INVALID_PID {
        let _ = kernel::unblock(pid);
    }
}

fn cycles_per_tick() -> u32 {
    let cpu_hz = CPU_HZ.load(Ordering::Relaxed) as u64;
    let tick_hz = crate::timer::systick::TICK_HZ as u64;
    if tick_hz == 0 {
        0
    } else {
        (cpu_hz / tick_hz) as u32
    }
}

fn cycles_to_us(cycles: u32) -> u32 {
    let cpu_hz = CPU_HZ.load(Ordering::Relaxed) as u64;
    if cpu_hz == 0 {
        0
    } else {
        ((cycles as u64) * 1_000_000 / cpu_hz) as u32
    }
}

fn print_stats(name: &str, stats: Stats) {
    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:{} count={} min={}cy/{}us avg={}cy/{}us max={}cy/{}us",
            name,
            stats.count,
            stats.min,
            cycles_to_us(stats.min),
            stats.avg(),
            cycles_to_us(stats.avg()),
            stats.max,
            cycles_to_us(stats.max),
        );
    });
}

fn print_attribution(name: &str, attr: MutexSpikeAttribution) {
    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:{}_attribution threshold={}cy overlap_samples={} spikes={} irq_spikes={} clean_spikes={} systick_spikes={} tim2_spikes={} max_irq_spike={}cy max_clean_spike={}cy",
            name,
            attr.threshold_cy,
            attr.overlap_samples,
            attr.spikes,
            attr.irq_spikes,
            attr.clean_spikes,
            attr.systick_spikes,
            attr.tim2_spikes,
            attr.max_irq_spike_cy,
            attr.max_clean_spike_cy,
        );
    });
}

fn print_two_phase_clean_breakdown(
    name: &str,
    first_label: &str,
    second_label: &str,
    stats: TwoPhaseCleanBreakdown,
) {
    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:{}_clean_breakdown clean_spikes={} {}_dominant={} {}_dominant={} max_{}={}cy max_{}={}cy",
            name,
            stats.clean_spikes,
            first_label,
            stats.first_dominant,
            second_label,
            stats.second_dominant,
            first_label,
            stats.max_first_cy,
            second_label,
            stats.max_second_cy,
        );
    });
}

fn print_four_phase_clean_breakdown(
    name: &str,
    first_label: &str,
    second_label: &str,
    third_label: &str,
    fourth_label: &str,
    stats: FourPhaseCleanBreakdown,
) {
    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:{}_clean_breakdown clean_spikes={} {}_dominant={} {}_dominant={} {}_dominant={} {}_dominant={} max_{}={}cy max_{}={}cy max_{}={}cy max_{}={}cy",
            name,
            stats.clean_spikes,
            first_label,
            stats.first_dominant,
            second_label,
            stats.second_dominant,
            third_label,
            stats.third_dominant,
            fourth_label,
            stats.fourth_dominant,
            first_label,
            stats.max_first_cy,
            second_label,
            stats.max_second_cy,
            third_label,
            stats.max_third_cy,
            fourth_label,
            stats.max_fourth_cy,
        );
    });
}

fn print_percentile_stats(name: &str, stats: PercentileStats) {
    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:{} count={} skipped={} min={}cy/{}us avg={}cy/{}us p50={}cy/{}us p95={}cy/{}us max={}cy/{}us",
            name,
            stats.stats.count,
            stats.skipped,
            stats.stats.min,
            cycles_to_us(stats.stats.min),
            stats.stats.avg(),
            cycles_to_us(stats.stats.avg()),
            stats.p50,
            cycles_to_us(stats.p50),
            stats.p95,
            cycles_to_us(stats.p95),
            stats.stats.max,
            cycles_to_us(stats.stats.max),
        );
    });
}

fn print_validation(name: &str, expected_min: u32, expected_max: u32, stats: ValidationStats) {
    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "bench:{} pass={} fail={} expected={}..{}ticks observed_min={} observed_max={}",
            name,
            stats.pass,
            stats.fail,
            expected_min,
            expected_max,
            stats.min_ticks,
            stats.max_ticks,
        );
    });
}

fn log_sample_timeout(stage: &str, sample: u32) {
    log::with_logger(|tx| {
        let _ = writeln!(tx, "bench:{} timeout sample={}", stage, sample);
    });
}

#[cfg(feature = "bench")]
pub fn idle_allows_wfi() -> bool {
    STAGE.load(Ordering::Relaxed) != STAGE_SLEEP
}

#[cfg(feature = "bench")]
pub fn on_systick_edge(now_tick: u32) {
    LAST_SYSTICK_EDGE_CYCLE.store(DWT::cycle_count(), Ordering::Relaxed);
    LAST_SYSTICK_EDGE_TICK.store(now_tick, Ordering::Relaxed);
    SYSTICK_ISR_COUNT.fetch_add(1, Ordering::Relaxed);
}
