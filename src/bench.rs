use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};

use cortex_m::interrupt;
use cortex_m::peripheral::{DCB, DWT};
use stm32f4xx_hal::{pac, rcc::Rcc};

use crate::device::timer::TimerMode;
use crate::ipc::mqueue::SyncMsgQueue;
use crate::sync::{mutex::IrqMutex, semaphore::Semaphore};
use crate::{kernel, log, timer};

const STAGE_CONTEXT: u8 = 0;
const STAGE_PARK_HELPER: u8 = 1;
const STAGE_SEM: u8 = 2;
const STAGE_SLEEP: u8 = 3;
const STAGE_IRQ: u8 = 4;
const STAGE_QUEUE: u8 = 5;
const STAGE_MUTEX: u8 = 6;
const STAGE_TIMER_CB: u8 = 7;
const STAGE_SCALE: u8 = 8;
const STAGE_DONE: u8 = 9;

const BENCH_SAMPLES: u32 = 1000;
const IRQ_BENCH_TIMER_HZ: u32 = 100;
const IRQ_TIMEOUT_MS: u32 = 50;
const WAIT_TIMEOUT_TICKS: u32 = 200;
const INVALID_PID: usize = usize::MAX;
const BUILD_PROFILE: &str = if cfg!(debug_assertions) { "debug" } else { "release" };

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
static IRQ_EVENTS: AtomicU32 = AtomicU32::new(0);

static SEM_WAITING: AtomicBool = AtomicBool::new(false);
static SEM_STAMP: AtomicU32 = AtomicU32::new(0);
static SEM_EVENTS: AtomicU32 = AtomicU32::new(0);

static QUEUE_WAITING: AtomicBool = AtomicBool::new(false);
static QUEUE_EVENTS: AtomicU32 = AtomicU32::new(0);
static QUEUE_WAKE_STAMP: AtomicU32 = AtomicU32::new(0);

static TIMER_CB_WAITING: AtomicBool = AtomicBool::new(false);
static TIMER_CB_STAMP: AtomicU32 = AtomicU32::new(0);
static TIMER_CB_EVENTS: AtomicU32 = AtomicU32::new(0);

static LAST_SYSTICK_EDGE_CYCLE: AtomicU32 = AtomicU32::new(0);
static LAST_SYSTICK_EDGE_TICK: AtomicU32 = AtomicU32::new(0);

static SCALE_WORKERS_CREATED: AtomicUsize = AtomicUsize::new(0);
static SCALE_ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(2);
static SCALE_WORKER_PIDS: IrqMutex<[usize; SCALE_EXTRA_TASKS]> =
    IrqMutex::new([INVALID_PID; SCALE_EXTRA_TASKS]);

static SEM_BENCH: Semaphore<4> = Semaphore::new(0, 1);
static QUEUE_BENCH: SyncMsgQueue<8> = SyncMsgQueue::new();
static MUTEX_BENCH: IrqMutex<u32> = IrqMutex::new(0);

static CTX_STATS: IrqMutex<Stats> = IrqMutex::new(Stats::new());
static SEM_STATS: IrqMutex<Stats> = IrqMutex::new(Stats::new());

#[derive(Clone, Copy)]
struct Stats {
    min: u32,
    max: u32,
    sum: u32,
    count: u32,
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
    IRQ_EVENTS.store(0, Ordering::Relaxed);

    SEM_WAITING.store(false, Ordering::Relaxed);
    SEM_STAMP.store(0, Ordering::Relaxed);
    SEM_EVENTS.store(0, Ordering::Relaxed);

    QUEUE_WAITING.store(false, Ordering::Relaxed);
    QUEUE_EVENTS.store(0, Ordering::Relaxed);
    QUEUE_WAKE_STAMP.store(0, Ordering::Relaxed);

    TIMER_CB_WAITING.store(false, Ordering::Relaxed);
    TIMER_CB_STAMP.store(0, Ordering::Relaxed);
    TIMER_CB_EVENTS.store(0, Ordering::Relaxed);

    LAST_SYSTICK_EDGE_CYCLE.store(0, Ordering::Relaxed);
    LAST_SYSTICK_EDGE_TICK.store(0, Ordering::Relaxed);

    SCALE_WORKERS_CREATED.store(0, Ordering::Relaxed);
    SCALE_ACTIVE_TASKS.store(2, Ordering::Relaxed);
    SCALE_WORKER_PIDS.lock(|pids| *pids = [INVALID_PID; SCALE_EXTRA_TASKS]);

    while SEM_BENCH.try_acquire() {}
    while QUEUE_BENCH.recv().is_some() {}

    CTX_STATS.lock(|stats| *stats = Stats::new());
    SEM_STATS.lock(|stats| *stats = Stats::new());

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
            BUILD_PROFILE,
            BENCH_SAMPLES,
            cpu_hz,
            IRQ_BENCH_TIMER_HZ
        );
    });
}

pub fn task_a(_arg: usize) -> ! {
    if let Some(pid) = kernel::current_pid() {
        TASK_A_PID.store(pid, Ordering::Relaxed);
    }

    while TASK_B_PID.load(Ordering::Relaxed) == INVALID_PID {
        kernel::yield_now();
    }

    log::log_line("bench: context-switch start");
    run_context_bench();
    print_stats("context_switch_a_to_b", CTX_STATS.lock(|stats| stats.normalized()));

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
    print_stats("semaphore_give_to_taskb_wake", sem_stats);

    repark_helper();

    STAGE.store(STAGE_SLEEP, Ordering::Relaxed);
    log::log_line("bench: sleep-wakeup start");
    let sleep_stats = run_sleep_bench();
    print_stats("sleep_1tick_extra", sleep_stats);

    STAGE.store(STAGE_IRQ, Ordering::Relaxed);
    log::log_line("bench: irq-to-task start");
    let irq_stats = run_irq_bench();
    print_stats("tim2_irq_to_task", irq_stats);

    STAGE.store(STAGE_QUEUE, Ordering::Relaxed);
    log::log_line("bench: queue-latency start");
    let (queue_wake_stats, queue_end_to_end_stats) = run_queue_bench();
    print_stats("queue_wake_latency", queue_wake_stats);
    print_stats("queue_end_to_end_latency", queue_end_to_end_stats);

    STAGE.store(STAGE_MUTEX, Ordering::Relaxed);
    log::log_line("bench: mutex-latency start");
    let mutex_stats = run_mutex_bench();
    print_stats("mutex_lock_unlock", mutex_stats);
    log::log_line(
        "bench:mutex_priority_inheritance supported=0 reason=irq_mutex_non_blocking",
    );

    STAGE.store(STAGE_TIMER_CB, Ordering::Relaxed);
    log::log_line("bench: soft-timer-callback start");
    let timer_cb_stats = run_timer_callback_bench();
    print_stats("soft_timer_callback_to_task", timer_cb_stats);

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
    }

    loop {
        match STAGE.load(Ordering::Relaxed) {
            STAGE_CONTEXT => {
                if CTX_PENDING.swap(false, Ordering::Relaxed) {
                    let left = CTX_LEFT.load(Ordering::Relaxed);
                    if left > 0 {
                        let delta = DWT::cycle_count().wrapping_sub(CTX_START.load(Ordering::Relaxed));
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
                    let delta = DWT::cycle_count().wrapping_sub(SEM_STAMP.load(Ordering::Relaxed));
                    SEM_STATS.lock(|stats| stats.update(delta));
                    SEM_EVENTS.fetch_add(1, Ordering::Relaxed);
                }
                SEM_WAITING.store(false, Ordering::Relaxed);
                kernel::yield_now();
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

fn run_semaphore_bench() -> Stats {
    while SEM_BENCH.try_acquire() {}
    SEM_WAITING.store(false, Ordering::Relaxed);
    SEM_EVENTS.store(0, Ordering::Relaxed);
    SEM_STATS.lock(|stats| *stats = Stats::new());

    for sample in 0..BENCH_SAMPLES {
        if !wait_for_flag(&SEM_WAITING, true) {
            log_sample_timeout("semaphore_waiter", sample);
            break;
        }

        SEM_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
        let _ = SEM_BENCH.release();

        if !wait_for_counter(&SEM_EVENTS, sample + 1) {
            log_sample_timeout("semaphore_wake", sample);
            break;
        }
    }

    SEM_STATS.lock(|stats| stats.normalized())
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

fn run_irq_bench() -> Stats {
    let mut stats = Stats::new();

    for sample in 0..BENCH_SAMPLES {
        if !wait_for_timer_edge() {
            log_sample_timeout("irq_wait_edge", sample);
            break;
        }

        let events_before = IRQ_EVENTS.load(Ordering::Relaxed);
        interrupt::free(|_| {
            IRQ_WAITING.store(true, Ordering::Relaxed);
            kernel::block_current(Some(IRQ_TIMEOUT_MS));
        });

        let events_after = IRQ_EVENTS.load(Ordering::Relaxed);
        if events_after != events_before {
            let delta = DWT::cycle_count().wrapping_sub(IRQ_STAMP.load(Ordering::Relaxed));
            stats.update(delta);
        } else {
            log_sample_timeout("irq_block", sample);
        }
    }

    stats.normalized()
}

fn run_queue_bench() -> (Stats, Stats) {
    while QUEUE_BENCH.recv().is_some() {}
    QUEUE_WAITING.store(false, Ordering::Relaxed);
    QUEUE_EVENTS.store(0, Ordering::Relaxed);
    QUEUE_WAKE_STAMP.store(0, Ordering::Relaxed);

    let mut wake_stats = Stats::new();
    let mut end_to_end_stats = Stats::new();

    for sample in 0..BENCH_SAMPLES {
        if !wait_for_timer_edge() {
            log_sample_timeout("queue_wait_edge", sample);
            break;
        }

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

        let wake_delta = DWT::cycle_count().wrapping_sub(QUEUE_WAKE_STAMP.load(Ordering::Relaxed));
        wake_stats.update(wake_delta);

        if let Some(stamp) = QUEUE_BENCH.recv() {
            let delta = DWT::cycle_count().wrapping_sub(stamp as u32);
            end_to_end_stats.update(delta);
        } else {
            log_sample_timeout("queue_recv", sample);
        }
    }

    (wake_stats.normalized(), end_to_end_stats.normalized())
}

fn run_mutex_bench() -> Stats {
    let mut stats = Stats::new();

    for _ in 0..BENCH_SAMPLES {
        let start = DWT::cycle_count();
        MUTEX_BENCH.lock(|value| {
            *value = value.wrapping_add(1);
        });
        let delta = DWT::cycle_count().wrapping_sub(start);
        stats.update(delta);
    }

    stats.normalized()
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
                    tasks,
                    created_workers,
                    needed_workers
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
                    tasks,
                    available,
                    needed_workers
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
        let mut min = u32::MAX;
        let mut max = 0;

        for value in per_switch_avg {
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
                    "bench:scheduler_o1_check per_switch_avg_2_8_32={}/{}/{}cy ratio={}permille verdict={}",
                    per_switch_avg[0],
                    per_switch_avg[1],
                    per_switch_avg[2],
                    ratio_permille,
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
    if pid == INVALID_PID {
        None
    } else {
        Some(pid)
    }
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

    match STAGE.load(Ordering::Relaxed) {
        STAGE_IRQ => {
            if IRQ_WAITING.swap(false, Ordering::Relaxed) {
                IRQ_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);
                IRQ_EVENTS.fetch_add(1, Ordering::Relaxed);

                let pid = TASK_A_PID.load(Ordering::Relaxed);
                if pid != INVALID_PID {
                    let _ = kernel::unblock(pid);
                }
            }
        }
        STAGE_QUEUE => {
            if QUEUE_WAITING.swap(false, Ordering::Relaxed) {
                let send_stamp = DWT::cycle_count();
                if QUEUE_BENCH.send(send_stamp as usize).is_ok() {
                    QUEUE_EVENTS.fetch_add(1, Ordering::Relaxed);
                }
                QUEUE_WAKE_STAMP.store(DWT::cycle_count(), Ordering::Relaxed);

                let pid = TASK_A_PID.load(Ordering::Relaxed);
                if pid != INVALID_PID {
                    let _ = kernel::unblock(pid);
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
}
