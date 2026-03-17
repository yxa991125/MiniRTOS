use cortex_m_rt::exception;

use crate::task::scheduler;
use crate::timer::{soft_timer, systick};

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
