use cortex_m::interrupt::{free, CriticalSection};
use cortex_m::peripheral::NVIC;

use stm32f4xx_hal::pac;

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

#[inline]
pub fn enable_irq(irq: pac::Interrupt) {
    unsafe { NVIC::unmask(irq) };
}

#[inline]
pub fn disable_irq(irq: pac::Interrupt) {
    NVIC::mask(irq);
}

#[inline]
pub fn set_priority(irq: pac::Interrupt, prio: u8) {
    unsafe { NVIC::set_priority(irq, prio) };
}

#[inline]
pub fn clear_pending(irq: pac::Interrupt) {
    NVIC::unpend(irq);
}

#[inline]
pub fn pend(irq: pac::Interrupt) {
    NVIC::pend(irq);
}
