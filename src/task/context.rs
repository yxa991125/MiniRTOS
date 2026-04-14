use cortex_m::register::{msp, psp};
use cortex_m_rt::{ExceptionFrame, exception};

use crate::task::scheduler;
use crate::timer::{soft_timer, systick};

fn fault_log_line(msg: &str) {
    crate::log::emergency_log_line(msg);
}

fn fault_log_fmt(args: core::fmt::Arguments<'_>) {
    crate::log::emergency_write_fmt(args);
    crate::log::emergency_write_str("\r\n");
}

fn read_debug_reg(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_volatile() }
}

fn dump_fault_context(tag: &str, ef: Option<&ExceptionFrame>) {
    fault_log_line(tag);

    let msp_v = msp::read();
    let psp_v = psp::read();
    let cfsr = read_debug_reg(0xE000_ED28);
    let hfsr = read_debug_reg(0xE000_ED2C);
    let mmfar = read_debug_reg(0xE000_ED34);
    let bfar = read_debug_reg(0xE000_ED38);

    fault_log_fmt(format_args!(
        "fault: MSP=0x{msp_v:08x} PSP=0x{psp_v:08x} CFSR=0x{cfsr:08x} HFSR=0x{hfsr:08x} MMFAR=0x{mmfar:08x} BFAR=0x{bfar:08x}"
    ));

    if let Some(ef) = ef {
        fault_log_fmt(format_args!(
            "fault: R0=0x{:08x} R1=0x{:08x} R2=0x{:08x} R3=0x{:08x}",
            ef.r0(),
            ef.r1(),
            ef.r2(),
            ef.r3()
        ));
        fault_log_fmt(format_args!(
            "fault: R12=0x{:08x} LR=0x{:08x} PC=0x{:08x} xPSR=0x{:08x}",
            ef.r12(),
            ef.lr(),
            ef.pc(),
            ef.xpsr()
        ));
    }
}

/// SysTick 中断：系统节拍
#[exception]
fn SysTick() {
    let now = systick::on_tick();
    #[cfg(feature = "bench")]
    crate::bench::on_systick_edge(now);
    soft_timer::on_tick(now);
    scheduler::tick_at(now);
}

// 这个函数将来给 PendSV 的汇编入口调用
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cortexos_switch_context(sp: *mut u32) -> *mut u32 {
    scheduler::context_switch(sp)
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    dump_fault_context("fault: HardFault", Some(_ef));
    loop {
        cortex_m::asm::bkpt();
    }
}

#[exception]
fn MemoryManagement() -> ! {
    dump_fault_context("fault: MemManage", None);
    loop {
        cortex_m::asm::bkpt();
    }
}

#[exception]
fn BusFault() -> ! {
    dump_fault_context("fault: BusFault", None);
    loop {
        cortex_m::asm::bkpt();
    }
}

#[exception]
fn UsageFault() -> ! {
    dump_fault_context("fault: UsageFault", None);
    loop {
        cortex_m::asm::bkpt();
    }
}
