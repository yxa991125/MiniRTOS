//! Kernel facade for scheduler/timer APIs.

use crate::task::scheduler;
use crate::timer::{soft_timer, systick};

pub type TaskEntry = fn(usize) -> !;
pub type TaskId = usize;
pub type TimerHandle = usize;

#[inline]
pub fn init() {
    scheduler::init();
    soft_timer::init();
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
    cortex_m::peripheral::SCB::set_pendsv();
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
