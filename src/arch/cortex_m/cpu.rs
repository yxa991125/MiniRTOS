use cortex_m::asm;
use cortex_m::register::{msp, psp};

#[inline]
pub fn nop() {
    asm::nop();
}

#[inline]
pub fn wfi() {
    asm::wfi();
}

#[inline]
pub fn wfe() {
    asm::wfe();
}

#[inline]
pub fn sev() {
    asm::sev();
}

#[inline]
pub fn isb() {
    asm::isb();
}

#[inline]
pub fn dsb() {
    asm::dsb();
}

#[inline]
pub fn dmb() {
    asm::dmb();
}

#[inline]
pub fn bkpt() {
    asm::bkpt();
}

#[inline]
pub fn delay(cycles: u32) {
    asm::delay(cycles);
}

#[inline]
pub fn read_msp() -> u32 {
    msp::read()
}

#[inline]
pub fn write_msp(value: u32) {
    unsafe { msp::write(value) };
}

#[inline]
pub fn read_psp() -> u32 {
    psp::read()
}

#[inline]
pub fn write_psp(value: u32) {
    unsafe { psp::write(value) };
}
