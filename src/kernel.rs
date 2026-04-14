//! Kernel facade for scheduler/timer APIs.

use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::log;
use crate::platform;
#[allow(unused_imports)]
pub use crate::task::diagnostics::{
    ResetReason, SystemHealth, TaskDiagnostics, TraceCounters, TraceEvent, TraceEventKind,
    TraceHook,
};
use crate::task::scheduler;
use crate::timer::{soft_timer, systick};
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

pub type TaskEntry = fn(usize) -> !;
pub type TaskId = usize;
pub type TimerHandle = usize;
pub const MAX_TASKS: usize = scheduler::MAX_TASKS;
const STACK_WARNING_WORDS: usize = 64;

static RESET_REASON: Mutex<RefCell<ResetReason>> = Mutex::new(RefCell::new(ResetReason::Unknown));
static WATCHDOG_ENABLED: AtomicBool = AtomicBool::new(false);
static WATCHDOG_FEEDS: AtomicU32 = AtomicU32::new(0);

#[inline]
pub fn init() {
    scheduler::init();
    soft_timer::init();
    WATCHDOG_ENABLED.store(false, Ordering::Relaxed);
    WATCHDOG_FEEDS.store(0, Ordering::Relaxed);
    cortex_m::interrupt::free(|cs| {
        *RESET_REASON.borrow(cs).borrow_mut() = ResetReason::Unknown;
    });
}

#[inline]
pub fn create_task(
    entry: TaskEntry,
    arg: usize,
    stack: &'static mut [u32],
    priority: u8,
) -> Option<TaskId> {
    scheduler::create_task(entry, arg, stack, priority)
}

#[inline]
pub fn start() -> ! {
    scheduler::start_first_task()
}

#[inline]
pub fn yield_now() {
    scheduler::request_context_switch();
}

#[inline]
pub fn sleep_ms(ms: u32) {
    scheduler::sleep_ms(ms);
}

#[inline]
pub fn block_current(timeout_ms: Option<u32>) {
    scheduler::block_current(timeout_ms);
}

#[inline]
pub fn unblock(pid: TaskId) -> bool {
    scheduler::unblock(pid)
}

#[inline]
pub fn delete_task(pid: TaskId) -> bool {
    scheduler::delete_task(pid)
}

#[inline]
pub fn exit_current() -> ! {
    scheduler::exit_current()
}

#[inline]
pub fn set_priority(pid: TaskId, new_prio: u8) -> bool {
    scheduler::set_priority(pid, new_prio)
}

#[inline]
pub fn current_pid() -> Option<TaskId> {
    scheduler::current_pid()
}

#[inline]
pub fn task_priority(pid: TaskId) -> Option<u8> {
    scheduler::task_priority(pid)
}

#[inline]
pub fn current_priority() -> Option<u8> {
    scheduler::current_priority()
}

#[inline]
pub fn list_tasks(buffer: &mut [TaskId]) -> usize {
    scheduler::list_tasks(buffer)
}

#[inline]
pub fn task_diagnostics(pid: TaskId) -> Option<TaskDiagnostics> {
    scheduler::task_diagnostics(pid)
}

#[inline]
pub fn trace_counters() -> TraceCounters {
    scheduler::trace_counters()
}

#[inline]
pub fn clear_trace_counters() {
    scheduler::clear_trace_counters();
}

#[inline]
pub fn set_trace_hook(hook: Option<TraceHook>) {
    scheduler::set_trace_hook(hook);
}

pub fn set_reset_reason(reason: ResetReason) {
    cortex_m::interrupt::free(|cs| {
        *RESET_REASON.borrow(cs).borrow_mut() = reason;
    });
}

pub fn reset_reason() -> ResetReason {
    cortex_m::interrupt::free(|cs| *RESET_REASON.borrow(cs).borrow())
}

pub fn enable_watchdog(timeout_ms: u32) -> bool {
    if !platform::watchdog::start(timeout_ms) {
        return false;
    }

    WATCHDOG_ENABLED.store(true, Ordering::Relaxed);
    WATCHDOG_FEEDS.store(0, Ordering::Relaxed);
    true
}

pub fn register_task_heartbeat(pid: TaskId, timeout_ms: u32) -> bool {
    scheduler::register_task_heartbeat(pid, timeout_ms)
}

pub fn register_current_heartbeat(timeout_ms: u32) -> bool {
    scheduler::register_current_heartbeat(timeout_ms)
}

pub fn task_heartbeat() -> bool {
    scheduler::task_heartbeat()
}

pub fn system_health() -> SystemHealth {
    let uptime_ticks = now_ticks();
    let mut task_ids = [usize::MAX; scheduler::MAX_TASKS];
    let count = list_tasks(&mut task_ids);
    let mut live_tasks = 0u32;
    let mut registered_heartbeats = 0u32;
    let mut stale_tasks = 0u32;
    let mut stack_warning_tasks = 0u32;

    for &pid in task_ids.iter().take(count) {
        if let Some(task) = task_diagnostics(pid) {
            live_tasks += 1;
            if task.heartbeat_registered {
                registered_heartbeats += 1;
                if task.heartbeat_stale {
                    stale_tasks += 1;
                }
            }
            if task.stack_free_low_water_words <= STACK_WARNING_WORDS {
                stack_warning_tasks += 1;
            }
        }
    }

    let io = platform::diagnostics::io_health();

    SystemHealth {
        uptime_ticks,
        live_tasks,
        registered_heartbeats,
        stale_tasks,
        stack_warning_tasks,
        reset_reason: reset_reason(),
        watchdog_enabled: WATCHDOG_ENABLED.load(Ordering::Relaxed),
        watchdog_feeds: WATCHDOG_FEEDS.load(Ordering::Relaxed),
        uart_rx_bytes: io.uart_rx_bytes,
        uart_tx_bytes: io.uart_tx_bytes,
        uart_rx_overflows: io.uart_rx_overflows,
        uart_tx_overflows: io.uart_tx_overflows,
        uart_rx_errors: io.uart_rx_errors,
    }
}

pub fn feed_watchdog_if_healthy() -> bool {
    if !WATCHDOG_ENABLED.load(Ordering::Relaxed) {
        return false;
    }

    let health = system_health();
    if health.registered_heartbeats == 0 || health.stale_tasks != 0 {
        return false;
    }

    let fed = platform::watchdog::feed();

    if fed {
        WATCHDOG_FEEDS.fetch_add(1, Ordering::Relaxed);
    }

    fed
}

pub fn log_diagnostics() {
    let health = system_health();
    let counters = trace_counters();
    let mut task_ids = [usize::MAX; scheduler::MAX_TASKS];
    let count = list_tasks(&mut task_ids);

    log::with_logger(|tx| {
        let _ = writeln!(
            tx,
            "diag: health uptime={} reset={} wd={} feeds={} live={} hb={} stale={} stack_warn={} uart_rx={} uart_tx={} rxov={} txov={} rxerr={}",
            health.uptime_ticks,
            health.reset_reason.as_str(),
            health.watchdog_enabled,
            health.watchdog_feeds,
            health.live_tasks,
            health.registered_heartbeats,
            health.stale_tasks,
            health.stack_warning_tasks,
            health.uart_rx_bytes,
            health.uart_tx_bytes,
            health.uart_rx_overflows,
            health.uart_tx_overflows,
            health.uart_rx_errors,
        );
        let _ = writeln!(
            tx,
            "diag: trace create={} ctxsw={} sleep={} block={} unblock={} delete={} timeout={} prio={} pendsv={}",
            counters.task_creates,
            counters.context_switches,
            counters.task_sleeps,
            counters.task_blocks,
            counters.task_unblocks,
            counters.task_deletes,
            counters.timeout_expirations,
            counters.priority_updates,
            counters.pendsv_requests,
        );

        for &pid in task_ids.iter().take(count) {
            if let Some(task) = task_diagnostics(pid) {
                let _ = writeln!(
                    tx,
                    "diag: task pid={} state={:?} prio={}/{} slice={} runtime={}ticks stack_used={}/{}w stack_free_low={}w wake_tick={} timeout={} hb={} age={}ticks/{}ticks stale={}",
                    task.pid,
                    task.state,
                    task.priority,
                    task.base_priority,
                    task.remaining_slice,
                    task.runtime_ticks,
                    task.stack_used_high_water_words,
                    task.stack_size_words,
                    task.stack_free_low_water_words,
                    task.wake_tick,
                    task.has_timeout,
                    task.heartbeat_registered,
                    task.heartbeat_age_ticks,
                    task.heartbeat_timeout_ticks,
                    task.heartbeat_stale,
                );
            }
        }
    });
}

#[inline]
pub fn now_ticks() -> u32 {
    systick::now()
}

#[inline]
pub fn now_ms() -> u32 {
    systick::ticks_to_ms(systick::now())
}

#[inline]
pub fn delay_ms(ms: u32) {
    systick::delay_ms(ms);
}

#[inline]
pub fn start_timer_oneshot(
    delay_ms: u32,
    callback: soft_timer::TimerCallback,
    arg: usize,
) -> Option<TimerHandle> {
    soft_timer::start_oneshot(delay_ms, callback, arg)
}

#[inline]
pub fn start_timer_periodic(
    period_ms: u32,
    callback: soft_timer::TimerCallback,
    arg: usize,
) -> Option<TimerHandle> {
    soft_timer::start_periodic(period_ms, callback, arg)
}

#[inline]
pub fn cancel_timer(handle: TimerHandle) -> bool {
    soft_timer::cancel(handle)
}

#[inline]
pub fn dispatch_timers() {
    soft_timer::dispatch();
}

pub const TICK_HZ: u32 = systick::TICK_HZ;
