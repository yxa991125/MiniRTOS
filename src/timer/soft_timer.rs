use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};

use super::systick;

pub const MAX_TIMERS: usize = 16;
pub const MAX_PENDING: usize = MAX_TIMERS;

pub type TimerCallback = fn(usize);

#[derive(Clone, Copy)]
struct SoftTimer {
    expire_tick: u32,
    period_ticks: u32,
    callback: TimerCallback,
    arg: usize,
    periodic: bool,
}

static TIMERS: Mutex<RefCell<[Option<SoftTimer>; MAX_TIMERS]>> =
    Mutex::new(RefCell::new([const { None }; MAX_TIMERS]));
static PENDING: Mutex<RefCell<PendingQueue>> =
    Mutex::new(RefCell::new(PendingQueue::new()));

pub fn init() {
    free(|cs| {
        let mut timers = TIMERS.borrow(cs).borrow_mut();
        for slot in timers.iter_mut() {
            *slot = None;
        }
        PENDING.borrow(cs).borrow_mut().clear();
    });
}

pub fn start_oneshot(delay_ms: u32, callback: TimerCallback, arg: usize) -> Option<usize> {
    start_internal(delay_ms, 0, false, callback, arg)
}

pub fn start_periodic(period_ms: u32, callback: TimerCallback, arg: usize) -> Option<usize> {
    if period_ms == 0 {
        return None;
    }
    start_internal(period_ms, period_ms, true, callback, arg)
}

pub fn cancel(handle: usize) -> bool {
    free(|cs| {
        let mut timers = TIMERS.borrow(cs).borrow_mut();
        if handle >= timers.len() {
            return false;
        }
        let existed = timers[handle].is_some();
        timers[handle] = None;
        existed
    })
}

pub fn on_tick(now_tick: u32) {
    free(|cs| {
        let mut timers = TIMERS.borrow(cs).borrow_mut();
        let mut pending = PENDING.borrow(cs).borrow_mut();
        for slot in timers.iter_mut() {
            let Some(mut timer) = *slot else {
                continue;
            };

            if time_after_eq(now_tick, timer.expire_tick) {
                let enqueued = pending.push((timer.callback, timer.arg));

                if enqueued {
                    if timer.periodic {
                        timer.expire_tick = timer.expire_tick.wrapping_add(timer.period_ticks);
                        *slot = Some(timer);
                    } else {
                        *slot = None;
                    }
                } else {
                    *slot = Some(timer);
                }
            }
        }
    });
}

pub fn dispatch() {
    loop {
        let next = free(|cs| PENDING.borrow(cs).borrow_mut().pop());
        let Some((callback, arg)) = next else {
            break;
        };
        callback(arg);
    }
}

fn start_internal(
    first_ms: u32,
    period_ms: u32,
    periodic: bool,
    callback: TimerCallback,
    arg: usize,
) -> Option<usize> {
    let now = systick::now();
    let first_ticks = systick::ms_to_ticks(first_ms);
    let period_ticks = systick::ms_to_ticks(period_ms);

    let timer = SoftTimer {
        expire_tick: now.wrapping_add(first_ticks),
        period_ticks,
        callback,
        arg,
        periodic,
    };

    free(|cs| {
        let mut timers = TIMERS.borrow(cs).borrow_mut();
        if let Some((index, slot)) = timers
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(timer);
            Some(index)
        } else {
            None
        }
    })
}

#[inline]
fn time_after_eq(now: u32, target: u32) -> bool {
    now.wrapping_sub(target) < 0x8000_0000
}

struct PendingQueue {
    buf: [Option<(TimerCallback, usize)>; MAX_PENDING],
    head: usize,
    tail: usize,
    len: usize,
}

impl PendingQueue {
    const fn new() -> Self {
        Self {
            buf: [const { None }; MAX_PENDING],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.len = 0;
    }

    fn push(&mut self, item: (TimerCallback, usize)) -> bool {
        if self.len >= MAX_PENDING {
            return false;
        }
        self.buf[self.tail] = Some(item);
        self.tail = (self.tail + 1) % MAX_PENDING;
        self.len += 1;
        true
    }

    fn pop(&mut self) -> Option<(TimerCallback, usize)> {
        if self.len == 0 {
            return None;
        }
        let item = self.buf[self.head].take();
        self.head = (self.head + 1) % MAX_PENDING;
        self.len -= 1;
        item
    }
}
