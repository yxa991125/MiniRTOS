use core::hint::spin_loop;
use core::sync::atomic::{AtomicU32, Ordering};

pub const TICK_HZ: u32 = 1_000;

static TICKS: AtomicU32 = AtomicU32::new(0);

#[inline]
pub fn now() -> u32 {
    TICKS.load(Ordering::Relaxed)
}

#[inline]
pub fn on_tick() -> u32 {
    TICKS.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
}

#[inline]
pub fn ms_to_ticks(ms: u32) -> u32 {
    let ticks = (ms as u64) * (TICK_HZ as u64) / 1000;
    if ticks > u32::MAX as u64 {
        u32::MAX
    } else {
        ticks as u32
    }
}

#[inline]
pub fn ticks_to_ms(ticks: u32) -> u32 {
    let ms = (ticks as u64) * 1000 / (TICK_HZ as u64);
    if ms > u32::MAX as u64 {
        u32::MAX
    } else {
        ms as u32
    }
}

pub fn delay_ms(ms: u32) {
    let start = now();
    let wait = ms_to_ticks(ms);
    while now().wrapping_sub(start) < wait {
        spin_loop();
    }
}
