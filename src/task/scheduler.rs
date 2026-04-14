use core::cell::RefCell;
use core::mem;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt::free;

use crate::task::diagnostics::{
    TaskDiagnostics, TraceCounters, TraceEvent, TraceEventKind, TraceHook,
};
use crate::task::tcb::{TaskState, Tcb};
use crate::timer::{soft_timer, systick};

#[cfg(feature = "bench")]
pub const MAX_TASKS: usize = 40;
#[cfg(not(feature = "bench"))]
pub const MAX_TASKS: usize = 8;
pub const MAX_PRIORITY: usize = 8;
pub const DEFAULT_TIME_SLICE_TICKS: u32 = 10;
pub const TIMEOUT_WHEEL_SIZE: usize = 256;
const TIMEOUT_WHEEL_MASK: usize = TIMEOUT_WHEEL_SIZE - 1;
const INVALID_PID: usize = usize::MAX;
const IDLE_PID: usize = 0;
const IDLE_STACK_WORDS: usize = 128;
const IDLE_PRIORITY: u8 = (MAX_PRIORITY - 1) as u8;
const STACK_SENTINEL: u32 = 0xA5A5_F00D;

#[repr(align(8))]
struct AlignedStack<const N: usize>([u32; N]);

static mut IDLE_STACK: AlignedStack<IDLE_STACK_WORDS> = AlignedStack([0; IDLE_STACK_WORDS]);

#[cfg(feature = "bench")]
const BENCH_TIMEOUT_TEST_STACK_WORDS: usize = 64;
#[cfg(feature = "bench")]
static mut BENCH_TIMEOUT_TEST_STACK: AlignedStack<BENCH_TIMEOUT_TEST_STACK_WORDS> =
    AlignedStack([0; BENCH_TIMEOUT_TEST_STACK_WORDS]);

static TASKS: Mutex<RefCell<[Option<Tcb>; MAX_TASKS]>> =
    Mutex::new(RefCell::new([const { None }; MAX_TASKS]));

struct ReadyQueues {
    heads: [Option<usize>; MAX_PRIORITY],
    tails: [Option<usize>; MAX_PRIORITY],
    counts: [u8; MAX_PRIORITY],
}

impl ReadyQueues {
    const fn new() -> Self {
        Self {
            heads: [const { None }; MAX_PRIORITY],
            tails: [const { None }; MAX_PRIORITY],
            counts: [0; MAX_PRIORITY],
        }
    }

    fn clear(&mut self) {
        self.heads = [const { None }; MAX_PRIORITY];
        self.tails = [const { None }; MAX_PRIORITY];
        self.counts = [0; MAX_PRIORITY];
    }
}

struct TimeoutWheel {
    heads: [Option<usize>; TIMEOUT_WHEEL_SIZE],
    tails: [Option<usize>; TIMEOUT_WHEEL_SIZE],
    last_tick: u32,
}

impl TimeoutWheel {
    const fn new() -> Self {
        Self {
            heads: [const { None }; TIMEOUT_WHEEL_SIZE],
            tails: [const { None }; TIMEOUT_WHEEL_SIZE],
            last_tick: 0,
        }
    }

    fn clear(&mut self, now: u32) {
        self.heads = [const { None }; TIMEOUT_WHEEL_SIZE];
        self.tails = [const { None }; TIMEOUT_WHEEL_SIZE];
        self.last_tick = now;
    }
}

static READY_QUEUES: Mutex<RefCell<ReadyQueues>> = Mutex::new(RefCell::new(ReadyQueues::new()));
static TIMEOUT_WHEEL: Mutex<RefCell<TimeoutWheel>> = Mutex::new(RefCell::new(TimeoutWheel::new()));
static READY_MASK: AtomicU32 = AtomicU32::new(0);

static CURRENT_PID: AtomicUsize = AtomicUsize::new(INVALID_PID);

static SCHED_STARTED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "bench")]
static TRACE_FIRST_SWITCH: AtomicBool = AtomicBool::new(true);
static TRACE_HOOK: AtomicUsize = AtomicUsize::new(0);

struct TraceCounterSet {
    task_creates: AtomicU32,
    context_switches: AtomicU32,
    task_sleeps: AtomicU32,
    task_blocks: AtomicU32,
    task_unblocks: AtomicU32,
    task_deletes: AtomicU32,
    timeout_expirations: AtomicU32,
    priority_updates: AtomicU32,
    pendsv_requests: AtomicU32,
}

impl TraceCounterSet {
    const fn new() -> Self {
        Self {
            task_creates: AtomicU32::new(0),
            context_switches: AtomicU32::new(0),
            task_sleeps: AtomicU32::new(0),
            task_blocks: AtomicU32::new(0),
            task_unblocks: AtomicU32::new(0),
            task_deletes: AtomicU32::new(0),
            timeout_expirations: AtomicU32::new(0),
            priority_updates: AtomicU32::new(0),
            pendsv_requests: AtomicU32::new(0),
        }
    }

    fn reset(&self) {
        self.task_creates.store(0, Ordering::Relaxed);
        self.context_switches.store(0, Ordering::Relaxed);
        self.task_sleeps.store(0, Ordering::Relaxed);
        self.task_blocks.store(0, Ordering::Relaxed);
        self.task_unblocks.store(0, Ordering::Relaxed);
        self.task_deletes.store(0, Ordering::Relaxed);
        self.timeout_expirations.store(0, Ordering::Relaxed);
        self.priority_updates.store(0, Ordering::Relaxed);
        self.pendsv_requests.store(0, Ordering::Relaxed);
    }

    fn snapshot(&self) -> TraceCounters {
        TraceCounters {
            task_creates: self.task_creates.load(Ordering::Relaxed),
            context_switches: self.context_switches.load(Ordering::Relaxed),
            task_sleeps: self.task_sleeps.load(Ordering::Relaxed),
            task_blocks: self.task_blocks.load(Ordering::Relaxed),
            task_unblocks: self.task_unblocks.load(Ordering::Relaxed),
            task_deletes: self.task_deletes.load(Ordering::Relaxed),
            timeout_expirations: self.timeout_expirations.load(Ordering::Relaxed),
            priority_updates: self.priority_updates.load(Ordering::Relaxed),
            pendsv_requests: self.pendsv_requests.load(Ordering::Relaxed),
        }
    }
}

static TRACE_COUNTERS: TraceCounterSet = TraceCounterSet::new();

unsafe extern "C" {
    fn __start_first_task(sp: *mut u32) -> !;
}

/// Initialize task storage, ready set, and reset current pid.
pub fn init() {
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();
        let mut timeout_wheel = TIMEOUT_WHEEL.borrow(cs).borrow_mut();

        tasks.iter_mut().for_each(|slot| *slot = None);
        ready_queues.clear();
        timeout_wheel.clear(systick::now());
        READY_MASK.store(0, Ordering::Relaxed);

        CURRENT_PID.store(INVALID_PID, Ordering::Relaxed);

        init_idle_task(&mut tasks, &mut ready_queues);
    });

    TRACE_COUNTERS.reset();
    TRACE_HOOK.store(0, Ordering::Relaxed);
}

pub fn current_pid() -> Option<usize> {
    let pid = CURRENT_PID.load(Ordering::Relaxed);
    if pid == INVALID_PID { None } else { Some(pid) }
}

pub fn list_tasks(buffer: &mut [usize]) -> usize {
    free(|cs| {
        let tasks = TASKS.borrow(cs).borrow();
        let mut written = 0;

        for (pid, slot) in tasks.iter().enumerate() {
            if slot.is_some() && written < buffer.len() {
                buffer[written] = pid;
                written += 1;
            }
        }

        written
    })
}

pub fn task_diagnostics(pid: usize) -> Option<TaskDiagnostics> {
    free(|cs| {
        let tasks = TASKS.borrow(cs).borrow();
        let task = tasks.get(pid).and_then(|slot| slot.as_ref())?;
        let stack_size_words = stack_size_words(task);
        let stack_free_low_water_words = stack_free_low_water_words(task);
        let now = systick::now();
        let heartbeat_age_ticks = now.wrapping_sub(task.heartbeat_last_seen_tick);
        let heartbeat_stale =
            task.heartbeat_registered && heartbeat_age_ticks > task.heartbeat_timeout_ticks;

        Some(TaskDiagnostics {
            pid: task.pid,
            state: task.state,
            base_priority: task.base_priority,
            priority: task.priority,
            remaining_slice: task.remaining_slice,
            wake_tick: task.wake_tick,
            has_timeout: task.has_timeout,
            runtime_ticks: task.runtime_ticks,
            stack_size_words,
            stack_free_low_water_words,
            stack_used_high_water_words: stack_size_words
                .saturating_sub(stack_free_low_water_words),
            heartbeat_registered: task.heartbeat_registered,
            heartbeat_timeout_ticks: task.heartbeat_timeout_ticks,
            heartbeat_age_ticks,
            heartbeat_stale,
        })
    })
}

pub fn register_task_heartbeat(pid: usize, timeout_ms: u32) -> bool {
    let timeout_ticks = timeout_delay_ticks(timeout_ms);
    let now = systick::now();

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) else {
            return false;
        };

        task.heartbeat_registered = true;
        task.heartbeat_timeout_ticks = timeout_ticks;
        task.heartbeat_last_seen_tick = now;
        true
    })
}

pub fn register_current_heartbeat(timeout_ms: u32) -> bool {
    current_pid()
        .map(|pid| register_task_heartbeat(pid, timeout_ms))
        .unwrap_or(false)
}

pub fn task_heartbeat() -> bool {
    let Some(pid) = current_pid() else {
        return false;
    };
    let now = systick::now();

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) else {
            return false;
        };
        if !task.heartbeat_registered {
            return false;
        }

        task.heartbeat_last_seen_tick = now;
        true
    })
}

pub fn trace_counters() -> TraceCounters {
    TRACE_COUNTERS.snapshot()
}

pub fn clear_trace_counters() {
    TRACE_COUNTERS.reset();
}

pub fn set_trace_hook(hook: Option<TraceHook>) {
    let raw = hook.map_or(0, |hook| hook as usize);
    TRACE_HOOK.store(raw, Ordering::Relaxed);
}

pub fn task_priority(pid: usize) -> Option<u8> {
    free(|cs| {
        TASKS
            .borrow(cs)
            .borrow()
            .get(pid)
            .and_then(|slot| slot.as_ref())
            .map(|task| task.priority)
    })
}

pub fn current_priority() -> Option<u8> {
    current_pid().and_then(task_priority)
}

#[inline]
fn trace_counter(kind: TraceEventKind) -> &'static AtomicU32 {
    match kind {
        TraceEventKind::TaskCreate => &TRACE_COUNTERS.task_creates,
        TraceEventKind::ContextSwitch => &TRACE_COUNTERS.context_switches,
        TraceEventKind::TaskSleep => &TRACE_COUNTERS.task_sleeps,
        TraceEventKind::TaskBlock => &TRACE_COUNTERS.task_blocks,
        TraceEventKind::TaskUnblock => &TRACE_COUNTERS.task_unblocks,
        TraceEventKind::TaskDelete => &TRACE_COUNTERS.task_deletes,
        TraceEventKind::TimeoutExpire => &TRACE_COUNTERS.timeout_expirations,
        TraceEventKind::PriorityUpdate => &TRACE_COUNTERS.priority_updates,
        TraceEventKind::PendSvRequest => &TRACE_COUNTERS.pendsv_requests,
    }
}

#[inline]
fn emit_trace(kind: TraceEventKind, pid: usize, aux: usize) {
    trace_counter(kind).fetch_add(1, Ordering::Relaxed);

    let hook = TRACE_HOOK.load(Ordering::Relaxed);
    if hook == 0 {
        return;
    }

    let event = TraceEvent {
        tick: systick::now(),
        kind,
        pid,
        aux,
    };

    let hook: TraceHook = unsafe { mem::transmute(hook) };
    hook(event);
}

#[inline]
pub fn request_context_switch() {
    emit_trace(
        TraceEventKind::PendSvRequest,
        CURRENT_PID.load(Ordering::Relaxed),
        0,
    );
    cortex_m::peripheral::SCB::set_pendsv();
}

/// Create a task, place it into the task table, and mark it ready.
pub fn create_task(
    entry: fn(usize) -> !,
    arg: usize,
    stack: &'static mut [u32],
    priority: u8,
) -> Option<usize> {
    if priority as usize >= MAX_PRIORITY {
        return None;
    }

    let mut created_pid = None;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();

        if let Some(pid) = tasks.iter().position(|task| task.is_none()) {
            fill_stack_pattern(stack);
            // Stack grows downwards: end points to the current top (empty stack).
            let stack_start = stack.as_mut_ptr();
            let stack_end = unsafe { stack_start.add(stack.len()) };
            let sp = init_stack_frame(stack_start, stack_end, entry, arg);

            tasks[pid] = Some(Tcb::init(
                pid,
                sp,
                priority,
                DEFAULT_TIME_SLICE_TICKS,
                stack_start,
                stack_end,
                entry,
                arg,
            ));
            let _ = ready_push_back(&mut ready_queues, &mut tasks, pid);
            created_pid = Some(pid);
        }
    });

    if let Some(pid) = created_pid {
        emit_trace(TraceEventKind::TaskCreate, pid, priority as usize);
    }

    created_pid
}

pub fn tick() {
    let now = systick::now();
    tick_at(now);
}

pub fn tick_at(now: u32) {
    if !SCHED_STARTED.load(Ordering::Relaxed) {
        return;
    }

    let mut pend_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();
        let mut timeout_wheel = TIMEOUT_WHEEL.borrow(cs).borrow_mut();

        account_runtime_tick(&mut tasks);

        let mut processed_tick = timeout_wheel.last_tick;
        while processed_tick != now {
            processed_tick = processed_tick.wrapping_add(1);
            process_timeout_slot(
                &mut timeout_wheel,
                &mut ready_queues,
                &mut tasks,
                processed_tick,
                &mut pend_switch,
            );
        }
        timeout_wheel.last_tick = now;

        let current_pid = CURRENT_PID.load(Ordering::Relaxed);
        let mut requeue_current = false;

        if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            if current.state == TaskState::Running {
                if current_pid == IDLE_PID {
                    if !pend_switch && READY_MASK.load(Ordering::Relaxed) != 0 {
                        pend_switch = true;
                    }
                } else {
                    if current.remaining_slice > 0 {
                        current.remaining_slice -= 1;
                    }

                    if current.remaining_slice == 0 {
                        current.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
                        current.state = TaskState::Ready;
                        requeue_current = true;
                        pend_switch = true;
                    } else if !pend_switch {
                        if let Some(best_prio) =
                            highest_ready_priority(READY_MASK.load(Ordering::Relaxed))
                        {
                            if best_prio < current.priority {
                                current.state = TaskState::Ready;
                                requeue_current = true;
                                pend_switch = true;
                            }
                        }
                    }
                }
            } else {
                pend_switch = true;
            }
        } else {
            pend_switch = true;
        }

        if requeue_current {
            let _ = ready_push_back(&mut ready_queues, &mut tasks, current_pid);
        }
    });

    if pend_switch {
        request_context_switch();
    }
}

/// 启动第一个任务：选择一个 Ready 任务，切到 PSP，并进入 Thread mode 执行该任务。
pub fn start_first_task() -> ! {
    // 从任务表里选出的首任务 sp（指向软件帧起点 R4..R11）
    let mut first_sp: *mut u32 = core::ptr::null_mut();

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();

        let pid = ready_pop_highest(&mut ready_queues, &mut tasks).unwrap_or(IDLE_PID);
        let sp = tasks[pid]
            .as_ref()
            .map(|task| task.sp)
            .expect("ready task missing");

        // 标记为 Running
        if let Some(t) = tasks[pid].as_mut() {
            t.state = TaskState::Running;
            if t.remaining_slice == 0 {
                t.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
            }
        }

        // 设置当前 pid（此处在临界区里，Relaxed 足够）
        CURRENT_PID.store(pid, Ordering::Relaxed);

        first_sp = sp;
    });

    // ---- 临界区外：做少量 sanity check，避免直接 HardFault ----
    if first_sp.is_null() {
        loop {
            cortex_m::asm::bkpt();
        }
    }

    // 8-byte 对齐检查：AAPCS + Cortex-M 异常入栈要求
    if (first_sp as usize & 0x7) != 0 {
        loop {
            cortex_m::asm::bkpt();
        }
    }

    unsafe {
        // first_sp 指向软件帧起点（R4..R11 共 8 words）
        // 硬件帧起点在 sw 之上 8 words
        let hw = first_sp.add(8);

        let pc = hw.add(6).read(); // 异常返回后将跳转的 PC
        let xpsr = hw.add(7).read(); // xPSR，要求 T-bit = 1

        // 关键不变量：Thumb 位 + xPSR 的 T-bit
        let pc_thumb = (pc & 1) == 1;
        let xpsr_tbit = (xpsr & 0x0100_0000) != 0;

        if !(pc_thumb && xpsr_tbit) {
            // 在这里停住，用调试器看 pc/xpsr/栈内容
            loop {
                cortex_m::asm::bkpt();
            }
        }

        // 标记调度器已启动：允许 SysTick 触发 PendSV
        SCHED_STARTED.store(true, Ordering::Relaxed);

        // 跳入首任务：该函数应通过 SVC/异常返回方式切 PSP 并进入 Thread mode
        __start_first_task(first_sp)
    }
}

/// Save current task context, pick the next ready task, and return its stack pointer.
pub fn context_switch(save_sp: *mut u32) -> *mut u32 {
    let mut next_sp: *mut u32 = ptr::null_mut();
    let mut switched_from = INVALID_PID;
    let mut switched_to = INVALID_PID;
    #[cfg(feature = "bench")]
    let mut trace_switch: Option<(usize, usize, *mut u32, *mut u32, u32, u32)> = None;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();
        let current_pid = CURRENT_PID.load(Ordering::Relaxed);
        switched_from = current_pid;
        let mut requeue_current = false;

        // Save current task context and requeue the running task if it is still runnable.
        if current_pid != IDLE_PID {
            if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
                current.sp = save_sp;
                if current.state == TaskState::Running {
                    current.state = TaskState::Ready;
                    requeue_current = true;
                }
            }
        } else if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            current.sp = save_sp;
        }

        if requeue_current {
            let _ = ready_push_back(&mut ready_queues, &mut tasks, current_pid);
        }

        let mut pid = ready_pop_highest(&mut ready_queues, &mut tasks).unwrap_or(IDLE_PID);
        let mut sp = tasks
            .get(pid)
            .and_then(|task| task.as_ref())
            .map(|task| task.sp)
            .unwrap_or(ptr::null_mut());

        let selected_sane = tasks
            .get(pid)
            .and_then(|task| task.as_ref())
            .map(|task| stack_pointer_is_sane(task, sp))
            .unwrap_or(false);

        if !selected_sane {
            pid = IDLE_PID;
            sp = tasks
                .get(IDLE_PID)
                .and_then(|task| task.as_ref())
                .map(|task| task.sp)
                .unwrap_or(save_sp);
        }

        if let Some(task) = tasks.get_mut(pid).and_then(|t| t.as_mut()) {
            task.state = TaskState::Running;
            if task.remaining_slice == 0 {
                task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
            }
        }
        CURRENT_PID.store(pid, Ordering::Relaxed);
        switched_to = pid;

        next_sp = sp;

        #[cfg(feature = "bench")]
        if TRACE_FIRST_SWITCH.swap(false, Ordering::Relaxed) {
            let (pc, xpsr) = stack_frame_signature(sp);
            trace_switch = Some((current_pid, pid, save_sp, sp, pc, xpsr));
        }
    });

    #[cfg(feature = "bench")]
    if let Some((from_pid, to_pid, saved_sp, selected_sp, pc, xpsr)) = trace_switch {
        crate::log::emergency_write_fmt(format_args!(
            "bench: ctxsw first from={} to={} save_sp=0x{:08x} next_sp=0x{:08x} pc=0x{:08x} xpsr=0x{:08x}\r\n",
            from_pid, to_pid, saved_sp as u32, selected_sp as u32, pc, xpsr
        ));
    }

    emit_trace(TraceEventKind::ContextSwitch, switched_to, switched_from);

    next_sp
}

pub fn sleep_ms(ms: u32) {
    let current_pid = CURRENT_PID.load(Ordering::Relaxed);
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut timeout_wheel = TIMEOUT_WHEEL.borrow(cs).borrow_mut();
        let mut queue_timeout = false;
        let now = systick::now();
        if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            let now = systick::now();
            let wake = now.wrapping_add(timeout_delay_ticks(ms));
            current.state = TaskState::Sleeping;
            current.wake_tick = wake;
            current.has_timeout = false;
            queue_timeout = true;
        }

        if queue_timeout {
            let _ = timeout_push(&mut timeout_wheel, &mut tasks, current_pid, now);
        }
    });

    emit_trace(TraceEventKind::TaskSleep, current_pid, ms as usize);
    request_context_switch();
}

pub fn block_current(timeout_ms: Option<u32>) {
    let current_pid = CURRENT_PID.load(Ordering::Relaxed);
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut timeout_wheel = TIMEOUT_WHEEL.borrow(cs).borrow_mut();
        let now = systick::now();
        let mut queue_timeout = false;
        if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            let (has_timeout, wake_tick) = if let Some(ms) = timeout_ms {
                (true, now.wrapping_add(timeout_delay_ticks(ms)))
            } else {
                (false, 0)
            };

            current.state = TaskState::Blocked;
            current.has_timeout = has_timeout;
            current.wake_tick = wake_tick;
            queue_timeout = has_timeout;
        }

        if queue_timeout {
            let _ = timeout_push(&mut timeout_wheel, &mut tasks, current_pid, now);
        }
    });

    emit_trace(
        TraceEventKind::TaskBlock,
        current_pid,
        timeout_ms.unwrap_or(0) as usize,
    );
    request_context_switch();
}

pub fn unblock(pid: usize) -> bool {
    let mut unblocked = false;
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();
        let mut timeout_wheel = TIMEOUT_WHEEL.borrow(cs).borrow_mut();
        let mut remove_timeout = false;
        let mut make_ready = false;
        if let Some(task) = tasks.get(pid).and_then(|t| t.as_ref()) {
            remove_timeout = task.in_timeout_queue;
            if task.state != TaskState::Ready || !task.in_ready_queue {
                make_ready = true;
                unblocked = true;
            }
        }

        if remove_timeout {
            let _ = timeout_remove(&mut timeout_wheel, &mut tasks, pid);
        }

        if make_ready {
            if let Some(task) = tasks.get_mut(pid).and_then(|t| t.as_mut()) {
                task.state = TaskState::Ready;
                task.has_timeout = false;
                task.wake_tick = 0;
                task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
            }
            let _ = ready_push_back(&mut ready_queues, &mut tasks, pid);
        }
    });

    if unblocked && SCHED_STARTED.load(Ordering::Relaxed) {
        emit_trace(TraceEventKind::TaskUnblock, pid, 0);
        request_context_switch();
    }

    unblocked
}

pub fn delete_task(pid: usize) -> bool {
    if pid == IDLE_PID {
        return false;
    }

    let mut removed = false;
    let mut need_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();
        let mut timeout_wheel = TIMEOUT_WHEEL.borrow(cs).borrow_mut();

        let Some(task) = tasks.get(pid).and_then(|t| t.as_ref()) else {
            return;
        };

        let was_ready = task.state == TaskState::Ready && task.in_ready_queue;
        let was_running = task.state == TaskState::Running;
        let was_timed = task.in_timeout_queue;

        if was_ready {
            let _ = ready_remove(&mut ready_queues, &mut tasks, pid);
        }
        if was_timed {
            let _ = timeout_remove(&mut timeout_wheel, &mut tasks, pid);
        }

        if was_running {
            need_switch = true;
        }

        tasks[pid] = None;
        removed = true;

        if CURRENT_PID.load(Ordering::Relaxed) == pid {
            CURRENT_PID.store(INVALID_PID, Ordering::Relaxed);
            need_switch = true;
        }
    });

    if need_switch && SCHED_STARTED.load(Ordering::Relaxed) {
        request_context_switch();
    }

    if removed {
        emit_trace(TraceEventKind::TaskDelete, pid, 0);
    }

    removed
}

pub fn exit_current() -> ! {
    let pid = CURRENT_PID.load(Ordering::Relaxed);
    let _ = delete_task(pid);
    request_context_switch();
    loop {
        cortex_m::asm::wfi();
    }
}

pub fn set_priority(pid: usize, new_prio: u8) -> bool {
    if new_prio as usize >= MAX_PRIORITY || pid == IDLE_PID {
        return false;
    }

    let mut updated = false;
    let mut need_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();

        let Some(task) = tasks.get(pid).and_then(|t| t.as_ref()) else {
            return;
        };

        let old_base = task.base_priority;
        if old_base == new_prio {
            updated = true;
            return;
        }

        if let Some(task) = tasks.get_mut(pid).and_then(|t| t.as_mut()) {
            task.base_priority = new_prio;
        }

        updated = true;
        need_switch = update_effective_priority_locked(&mut ready_queues, &mut tasks, pid);
    });

    if updated && need_switch && SCHED_STARTED.load(Ordering::Relaxed) {
        request_context_switch();
    }

    if updated {
        emit_trace(TraceEventKind::PriorityUpdate, pid, new_prio as usize);
    }

    updated
}

pub fn add_priority_boost(pid: usize, boost_prio: u8) -> bool {
    if pid == IDLE_PID || boost_prio as usize >= MAX_PRIORITY {
        return false;
    }

    let mut changed = false;
    let mut need_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();

        let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) else {
            return;
        };

        let slot = &mut task.priority_boosts[boost_prio as usize];
        *slot = slot.saturating_add(1);
        changed = true;
        need_switch = update_effective_priority_locked(&mut ready_queues, &mut tasks, pid);
    });

    if changed && need_switch && SCHED_STARTED.load(Ordering::Relaxed) {
        request_context_switch();
    }

    if changed {
        emit_trace(TraceEventKind::PriorityUpdate, pid, boost_prio as usize);
    }

    changed
}

pub fn remove_priority_boost(pid: usize, boost_prio: u8) -> bool {
    if pid == IDLE_PID || boost_prio as usize >= MAX_PRIORITY {
        return false;
    }

    let mut changed = false;
    let mut need_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_queues = READY_QUEUES.borrow(cs).borrow_mut();

        let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) else {
            return;
        };

        let slot = &mut task.priority_boosts[boost_prio as usize];
        if *slot == 0 {
            return;
        }

        *slot -= 1;
        changed = true;
        need_switch = update_effective_priority_locked(&mut ready_queues, &mut tasks, pid);
    });

    if changed && need_switch && SCHED_STARTED.load(Ordering::Relaxed) {
        request_context_switch();
    }

    if changed {
        emit_trace(TraceEventKind::PriorityUpdate, pid, boost_prio as usize);
    }

    changed
}

#[inline]
fn time_after_eq(now: u32, target: u32) -> bool {
    now.wrapping_sub(target) < 0x8000_0000
}

#[inline]
fn effective_priority(task: &Tcb) -> u8 {
    let mut effective = task.base_priority;
    for (prio, count) in task.priority_boosts.iter().enumerate() {
        if *count > 0 {
            effective = effective.min(prio as u8);
            break;
        }
    }
    effective
}

fn update_effective_priority_locked(
    ready_queues: &mut ReadyQueues,
    tasks: &mut [Option<Tcb>; MAX_TASKS],
    pid: usize,
) -> bool {
    let Some(task) = tasks.get(pid).and_then(|slot| slot.as_ref()) else {
        return false;
    };

    let old_prio = task.priority;
    let new_prio = effective_priority(task);
    if old_prio == new_prio {
        return false;
    }

    let was_ready = task.state == TaskState::Ready && task.in_ready_queue;
    let was_running = task.state == TaskState::Running;
    let current_pid = CURRENT_PID.load(Ordering::Relaxed);
    let mut need_switch = false;

    if was_ready {
        let _ = ready_remove(ready_queues, tasks, pid);
    }

    if let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) {
        task.priority = new_prio;
    }

    if was_ready {
        let _ = ready_push_back(ready_queues, tasks, pid);
        if current_pid != pid {
            if let Some(current) = tasks.get(current_pid).and_then(|slot| slot.as_ref()) {
                if current.state == TaskState::Running && new_prio < current.priority {
                    need_switch = true;
                }
            }
        }
    }

    if was_running {
        if let Some(best_prio) = highest_ready_priority(READY_MASK.load(Ordering::Relaxed)) {
            if best_prio < new_prio {
                need_switch = true;
            }
        }
    }

    need_switch
}

#[inline]
fn ready_push_back(
    ready_queues: &mut ReadyQueues,
    tasks: &mut [Option<Tcb>; MAX_TASKS],
    pid: usize,
) -> bool {
    let Some(task) = tasks.get(pid).and_then(|slot| slot.as_ref()) else {
        return false;
    };

    let prio = task.priority as usize;
    if prio >= MAX_PRIORITY || task.in_ready_queue {
        return false;
    }

    let prev_tail = ready_queues.tails[prio];

    if let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) {
        task.ready_prev = prev_tail;
        task.ready_next = None;
        task.in_ready_queue = true;
    }

    if let Some(tail_pid) = prev_tail {
        if let Some(tail) = tasks.get_mut(tail_pid).and_then(|slot| slot.as_mut()) {
            tail.ready_next = Some(pid);
        }
    } else {
        ready_queues.heads[prio] = Some(pid);
    }

    ready_queues.tails[prio] = Some(pid);
    ready_queues.counts[prio] = ready_queues.counts[prio].saturating_add(1);
    if ready_queues.counts[prio] == 1 {
        READY_MASK.fetch_or(1u32 << prio, Ordering::Relaxed);
    }

    true
}

#[inline]
fn ready_remove(
    ready_queues: &mut ReadyQueues,
    tasks: &mut [Option<Tcb>; MAX_TASKS],
    pid: usize,
) -> bool {
    let Some(task) = tasks.get(pid).and_then(|slot| slot.as_ref()) else {
        return false;
    };

    let prio = task.priority as usize;
    if prio >= MAX_PRIORITY || !task.in_ready_queue {
        return false;
    }

    let prev = task.ready_prev;
    let next = task.ready_next;

    if let Some(prev_pid) = prev {
        if let Some(prev_task) = tasks.get_mut(prev_pid).and_then(|slot| slot.as_mut()) {
            prev_task.ready_next = next;
        }
    } else {
        ready_queues.heads[prio] = next;
    }

    if let Some(next_pid) = next {
        if let Some(next_task) = tasks.get_mut(next_pid).and_then(|slot| slot.as_mut()) {
            next_task.ready_prev = prev;
        }
    } else {
        ready_queues.tails[prio] = prev;
    }

    if let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) {
        task.ready_prev = None;
        task.ready_next = None;
        task.in_ready_queue = false;
    }

    if ready_queues.counts[prio] > 0 {
        ready_queues.counts[prio] -= 1;
        if ready_queues.counts[prio] == 0 {
            READY_MASK.fetch_and(!(1u32 << prio), Ordering::Relaxed);
        }
    }

    true
}

#[inline]
fn ready_pop_highest(
    ready_queues: &mut ReadyQueues,
    tasks: &mut [Option<Tcb>; MAX_TASKS],
) -> Option<usize> {
    let prio = highest_ready_priority(READY_MASK.load(Ordering::Relaxed))? as usize;
    let pid = ready_queues.heads[prio]?;
    let removed = ready_remove(ready_queues, tasks, pid);
    if removed { Some(pid) } else { None }
}

#[inline]
fn highest_ready_priority(mask: u32) -> Option<u8> {
    if mask == 0 {
        None
    } else {
        Some(mask.trailing_zeros() as u8)
    }
}

#[inline]
fn timeout_delay_ticks(ms: u32) -> u32 {
    let ticks = systick::ms_to_ticks(ms);
    if ticks == 0 { 1 } else { ticks }
}

#[inline]
fn timeout_bucket_index(tick: u32) -> usize {
    (tick as usize) & TIMEOUT_WHEEL_MASK
}

#[inline]
fn timeout_push(
    timeout_wheel: &mut TimeoutWheel,
    tasks: &mut [Option<Tcb>; MAX_TASKS],
    pid: usize,
    now: u32,
) -> bool {
    let Some(task) = tasks.get(pid).and_then(|slot| slot.as_ref()) else {
        return false;
    };

    if task.in_timeout_queue {
        return false;
    }

    let wake_tick = task.wake_tick;
    let delta = wake_tick.wrapping_sub(now);
    if delta == 0 {
        return false;
    }

    let bucket = timeout_bucket_index(wake_tick);
    let prev_tail = timeout_wheel.tails[bucket];
    let rounds = (delta - 1) / (TIMEOUT_WHEEL_SIZE as u32);

    if let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) {
        task.timeout_prev = prev_tail;
        task.timeout_next = None;
        task.in_timeout_queue = true;
        task.timeout_rounds = rounds;
    }

    if let Some(tail_pid) = prev_tail {
        if let Some(tail) = tasks.get_mut(tail_pid).and_then(|slot| slot.as_mut()) {
            tail.timeout_next = Some(pid);
        }
    } else {
        timeout_wheel.heads[bucket] = Some(pid);
    }

    timeout_wheel.tails[bucket] = Some(pid);
    true
}

#[inline]
fn timeout_remove(
    timeout_wheel: &mut TimeoutWheel,
    tasks: &mut [Option<Tcb>; MAX_TASKS],
    pid: usize,
) -> bool {
    let Some(task) = tasks.get(pid).and_then(|slot| slot.as_ref()) else {
        return false;
    };

    if !task.in_timeout_queue {
        return false;
    }

    let bucket = timeout_bucket_index(task.wake_tick);
    let prev = task.timeout_prev;
    let next = task.timeout_next;

    if let Some(prev_pid) = prev {
        if let Some(prev_task) = tasks.get_mut(prev_pid).and_then(|slot| slot.as_mut()) {
            prev_task.timeout_next = next;
        }
    } else {
        timeout_wheel.heads[bucket] = next;
    }

    if let Some(next_pid) = next {
        if let Some(next_task) = tasks.get_mut(next_pid).and_then(|slot| slot.as_mut()) {
            next_task.timeout_prev = prev;
        }
    } else {
        timeout_wheel.tails[bucket] = prev;
    }

    if let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) {
        task.timeout_prev = None;
        task.timeout_next = None;
        task.in_timeout_queue = false;
        task.timeout_rounds = 0;
    }

    true
}

fn process_timeout_slot(
    timeout_wheel: &mut TimeoutWheel,
    ready_queues: &mut ReadyQueues,
    tasks: &mut [Option<Tcb>; MAX_TASKS],
    tick: u32,
    pend_switch: &mut bool,
) {
    let bucket = timeout_bucket_index(tick);
    let mut cursor = timeout_wheel.heads[bucket];

    while let Some(pid) = cursor {
        let next = tasks
            .get(pid)
            .and_then(|slot| slot.as_ref())
            .and_then(|task| task.timeout_next);

        let mut due = false;
        if let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) {
            if task.timeout_rounds > 0 {
                task.timeout_rounds -= 1;
            } else if time_after_eq(tick, task.wake_tick) {
                due = true;
            }
        }

        if due {
            let _ = timeout_remove(timeout_wheel, tasks, pid);

            let mut became_ready = false;
            if let Some(task) = tasks.get_mut(pid).and_then(|slot| slot.as_mut()) {
                match task.state {
                    TaskState::Sleeping => {
                        task.state = TaskState::Ready;
                        task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
                        task.wake_tick = 0;
                        became_ready = true;
                    }
                    TaskState::Blocked if task.has_timeout => {
                        task.state = TaskState::Ready;
                        task.has_timeout = false;
                        task.wake_tick = 0;
                        task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
                        became_ready = true;
                    }
                    _ => {
                        task.has_timeout = false;
                        task.wake_tick = 0;
                    }
                }
            }

            if became_ready {
                let _ = ready_push_back(ready_queues, tasks, pid);
                emit_trace(TraceEventKind::TimeoutExpire, pid, tick as usize);
                *pend_switch = true;
            }
        }

        cursor = next;
    }
}

#[cfg(feature = "bench")]
fn bench_timeout_test_entry(_arg: usize) -> ! {
    loop {
        cortex_m::asm::nop();
    }
}

#[cfg(feature = "bench")]
pub fn bench_validate_timeout_wraparound() -> bool {
    free(|_| {
        // Keep the validation isolated from the live scheduler state. The bench
        // stage runs while tasks are active, so it must not reset global task
        // tables or publish a temporary READY_MASK outside this critical section.
        let saved_ready_mask = READY_MASK.swap(0, Ordering::Relaxed);

        let result = (|| {
            let mut tasks: [Option<Tcb>; MAX_TASKS] = [const { None }; MAX_TASKS];
            let mut ready_queues = ReadyQueues::new();
            let mut timeout_wheel = TimeoutWheel::new();

            let stack = unsafe {
                let stack_ptr = core::ptr::addr_of_mut!(BENCH_TIMEOUT_TEST_STACK.0) as *mut u32;
                core::slice::from_raw_parts_mut(stack_ptr, BENCH_TIMEOUT_TEST_STACK_WORDS)
            };

            fill_stack_pattern(stack);
            let stack_start = stack.as_mut_ptr();
            let stack_end = unsafe { stack_start.add(stack.len()) };
            let sp = init_stack_frame(stack_start, stack_end, bench_timeout_test_entry, 0);

            let now = u32::MAX - 2;
            let wake_tick = now.wrapping_add(5);
            let pid = 1usize;

            timeout_wheel.clear(now);

            tasks[pid] = Some(Tcb::init(
                pid,
                sp,
                1,
                DEFAULT_TIME_SLICE_TICKS,
                stack_start,
                stack_end,
                bench_timeout_test_entry,
                0,
            ));

            if let Some(task) = tasks[pid].as_mut() {
                task.state = TaskState::Sleeping;
                task.wake_tick = wake_tick;
                task.has_timeout = false;
            }

            if !timeout_push(&mut timeout_wheel, &mut tasks, pid, now) {
                return false;
            }

            let mut pend_switch = false;
            let mut tick = now;
            while tick != wake_tick {
                tick = tick.wrapping_add(1);
                process_timeout_slot(
                    &mut timeout_wheel,
                    &mut ready_queues,
                    &mut tasks,
                    tick,
                    &mut pend_switch,
                );
            }

            let Some(task) = tasks[pid].as_ref() else {
                return false;
            };

            task.state == TaskState::Ready
                && task.in_ready_queue
                && !task.in_timeout_queue
                && task.wake_tick == 0
        })();

        READY_MASK.store(saved_ready_mask, Ordering::Relaxed);
        result
    })
}

#[inline]
fn fill_stack_pattern(stack: &mut [u32]) {
    stack.fill(STACK_SENTINEL);
}

#[inline]
fn account_runtime_tick(tasks: &mut [Option<Tcb>; MAX_TASKS]) {
    let current_pid = CURRENT_PID.load(Ordering::Relaxed);
    if let Some(task) = tasks.get_mut(current_pid).and_then(|slot| slot.as_mut()) {
        if task.state == TaskState::Running {
            task.runtime_ticks = task.runtime_ticks.saturating_add(1);
        }
    }
}

#[inline]
fn stack_size_words(task: &Tcb) -> usize {
    (task.stack_end as usize - task.stack_start as usize) / core::mem::size_of::<u32>()
}

#[inline]
fn stack_free_low_water_words(task: &Tcb) -> usize {
    let mut cursor = task.stack_start;
    while cursor < task.stack_end {
        let word = unsafe { cursor.read() };
        if word != STACK_SENTINEL {
            break;
        }
        cursor = unsafe { cursor.add(1) };
    }

    (cursor as usize - task.stack_start as usize) / core::mem::size_of::<u32>()
}

fn init_idle_task(tasks: &mut [Option<Tcb>; MAX_TASKS], _ready_queues: &mut ReadyQueues) {
    let stack = unsafe {
        let stack_ptr = core::ptr::addr_of_mut!(IDLE_STACK.0) as *mut u32;
        core::slice::from_raw_parts_mut(stack_ptr, IDLE_STACK_WORDS)
    };

    fill_stack_pattern(stack);
    let stack_start = stack.as_mut_ptr();
    let stack_end = unsafe { stack_start.add(stack.len()) };
    let sp = init_stack_frame(stack_start, stack_end, idle_task, 0);

    tasks[IDLE_PID] = Some(Tcb::init(
        IDLE_PID,
        sp,
        IDLE_PRIORITY,
        DEFAULT_TIME_SLICE_TICKS,
        stack_start,
        stack_end,
        idle_task,
        0,
    ));
}

#[allow(dead_code)]
#[inline]
fn ready_count(ready_queues: &ReadyQueues, prio: u8) -> u8 {
    let idx = prio as usize;
    if idx >= MAX_PRIORITY {
        0
    } else {
        ready_queues.counts[idx]
    }
}

fn idle_task(_arg: usize) -> ! {
    loop {
        soft_timer::dispatch();
        #[cfg(feature = "bench")]
        {
            if crate::bench::idle_allows_wfi() {
                cortex_m::asm::wfi();
            } else {
                cortex_m::asm::nop();
            }
        }
        #[cfg(not(feature = "bench"))]
        {
            cortex_m::asm::wfi();
        }
    }
}

const INITIAL_XPSR: u32 = 0x0100_0000;

#[inline]
fn stack_pointer_is_sane(task: &Tcb, sp: *mut u32) -> bool {
    let sp_v = sp as usize;
    let start = task.stack_start as usize;
    let end = task.stack_end as usize;

    !sp.is_null()
        && (sp_v & 0x7) == 0
        && sp_v >= start
        && sp_v + (16 * core::mem::size_of::<u32>()) <= end
}

#[cfg(feature = "bench")]
#[inline]
fn stack_frame_signature(sp: *mut u32) -> (u32, u32) {
    if sp.is_null() || (sp as usize & 0x7) != 0 {
        return (0, 0);
    }

    unsafe {
        let hw = sp.add(8);
        (hw.add(6).read(), hw.add(7).read())
    }
}

#[inline(never)]
fn task_exit_trap() -> ! {
    exit_current()
}

fn init_stack_frame(
    stack_start: *mut u32,
    stack_end: *mut u32,
    entry: fn(usize) -> !,
    arg: usize,
) -> *mut u32 {
    let mut sp = stack_end as usize;
    sp &= !0x7;
    let sp = sp as *mut u32;

    let pc = (entry as usize as u32) | 1;
    let lr = (task_exit_trap as usize as u32) | 1;

    unsafe {
        // 1) 先为“硬件帧”预留 8 words（R0..xPSR），放在靠近 stack_end 的位置
        let hw = sp.sub(8);
        let sw = hw.sub(8);

        if sw < stack_start {
            loop {
                cortex_m::asm::bkpt();
            }
        }

        hw.add(0).write(arg as u32); // R0
        hw.add(1).write(0); // R1
        hw.add(2).write(0); // R2
        hw.add(3).write(0); // R3
        hw.add(4).write(0); // R12
        hw.add(5).write(lr); // LR
        hw.add(6).write(pc); // PC  (必须是 0x0800_xxxx)
        hw.add(7).write(INITIAL_XPSR); // xPSR (必须是 0x0100_0000)

        // 2) 再为“软件帧”预留 8 words（R4..R11），放在更低地址处
        let sw = hw.sub(8);
        for i in 0..8 {
            sw.add(i).write(0);
        }

        // 返回 sp(sw)：boot.S 会先 pop R4..R11，然后 PSP 正好指向 hw(R0)
        sw
    }
}
