use cortex_m::interrupt::{free, CriticalSection};

#[inline]
pub fn enable() {
    cortex_m::interrupt::enable();
}

#[inline]
pub fn disable() {
    cortex_m::interrupt::disable();
}

#[inline]
pub fn with_critical_section<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    free(f)
}
