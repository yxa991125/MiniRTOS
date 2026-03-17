#![no_std]
#![no_main]

use panic_halt as _;

use core::fmt::Write;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
#[cfg(not(feature = "bench"))]
use core::sync::atomic::{AtomicU32, Ordering};

use stm32f4xx_hal::{
    gpio::GpioExt,
    pac,
    prelude::*,
    rcc::Config as RccConfig,
    rcc::RccExt,
    serial::{config::Config as UartConfig, Serial},
};

mod task;
mod kernel;
mod log;
#[cfg(not(feature = "bench"))]
mod app;
#[cfg(feature = "bench")]
mod bench;
mod timer;
mod device;
mod ipc;
mod mem;
mod driver;
mod sync;

#[cfg(not(feature = "bench"))]
static TIM2_TICKS: AtomicU32 = AtomicU32::new(0);
#[cfg(not(feature = "bench"))]
static TIM3_TICKS: AtomicU32 = AtomicU32::new(0);

#[cfg(not(feature = "bench"))]
fn tim2_tick() {
    TIM2_TICKS.fetch_add(1, Ordering::Relaxed);
}

#[cfg(not(feature = "bench"))]
fn tim3_tick() {
    TIM3_TICKS.fetch_add(1, Ordering::Relaxed);
}

/// 强制 8-byte 对齐的任务栈包装（Cortex-M 异常/PSP 更稳妥）
#[repr(align(8))]
struct AlignedStack<const N: usize>([u32; N]);

#[cfg(feature = "bench")]
const STACK_TASK1_WORDS: usize = 1024;
#[cfg(feature = "bench")]
const STACK_TASK2_WORDS: usize = 512;
#[cfg(not(feature = "bench"))]
const STACK_TASK1_WORDS: usize = 256;
#[cfg(not(feature = "bench"))]
const STACK_TASK2_WORDS: usize = 256;

// benchmark ????? `core::fmt` ???????????????????
static mut STACK_TASK1: AlignedStack<STACK_TASK1_WORDS> = AlignedStack([0; STACK_TASK1_WORDS]);
static mut STACK_TASK2: AlignedStack<STACK_TASK2_WORDS> = AlignedStack([0; STACK_TASK2_WORDS]);

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    // 1) 时钟
    let mut rcc = dp.RCC.freeze(
        RccConfig::hsi()
            .sysclk(84.MHz())
            .pclk1(42.MHz())
            .pclk2(84.MHz()),
    );

    // 2) USART2 (PA2/PA3) 串口（用于看到“启动是否正常”）
    let gpioa = dp.GPIOA.split(&mut rcc);

    let tx_pin = gpioa.pa2.into_alternate::<7>();
    let rx_pin = gpioa.pa3.into_alternate::<7>();

    let serial = Serial::new(
        dp.USART2,
        (tx_pin, rx_pin),
        UartConfig::default().baudrate(115_200.bps()),
        &mut rcc,
    )
    .unwrap();

    let (mut tx, _rx) = serial.split();
    writeln!(tx, "boot ok (F411)").ok();

    // 3) SysTick 1ms
    let sysclk_hz = rcc.clocks.sysclk().raw();
    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(sysclk_hz / 1000 - 1);
    cp.SYST.clear_current();

    unsafe {
        let scb: &mut cortex_m::peripheral::scb::RegisterBlock =
            &mut *(cortex_m::peripheral::SCB::PTR as *mut cortex_m::peripheral::scb::RegisterBlock);
        scb.shpr[10].write(0xFF);
        scb.shpr[11].write(0xFE);
    }

    use cortex_m::register::{msp, psp};
    use cortex_m::peripheral::SCB;

    let msp_v = msp::read();
    let psp_v = psp::read();
    let vtor = unsafe { (*SCB::PTR).vtor.read() };

    writeln!(tx, "MSP=0x{:08x} PSP=0x{:08x} VTOR=0x{:08x}", msp_v, psp_v, vtor).ok();

    // 同时把 SysTick/PendSV 的向量表项也读出来（index 14/15）
    let pendsv_vec = unsafe { *((vtor as *const u32).add(14)) };
    let systick_vec = unsafe { *((vtor as *const u32).add(15)) };
    writeln!(tx, "VEC PendSV=0x{:08x} SysTick=0x{:08x}", pendsv_vec, systick_vec).ok();

    log::init(tx);

    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    // 4) 初始化调度器
    kernel::init();

    #[cfg(feature = "bench")]
    bench::init(&mut cp.DCB, &mut cp.DWT, dp.TIM2, &mut rcc, sysclk_hz);

    #[cfg(not(feature = "bench"))]
    // TIM2 硬件定时器示例：1kHz 周期中断
    timer::hw_timer::init_tim2(
        dp.TIM2,
        &mut rcc,
        1_000,
        device::timer::TimerMode::Periodic,
        tim2_tick,
    );

    #[cfg(not(feature = "bench"))]
    // TIM3 硬件定时器示例：500Hz 周期中断
    timer::hw_timer::init_tim3(
        dp.TIM3,
        &mut rcc,
        500,
        device::timer::TimerMode::Periodic,
        tim3_tick,
    );

    #[cfg(not(feature = "bench"))]
    // 5) LED（PA5）与应用初始化
    {
        let led = gpioa.pa5.into_push_pull_output();
        app::init_led(led);
    }

    // 6) 用 raw pointer -> slice 的方式拿到栈（避免 &mut static）
    let (stack1, stack2): (&'static mut [u32], &'static mut [u32]) = unsafe {
        let stack1_ptr = core::ptr::addr_of_mut!(STACK_TASK1.0) as *mut u32;
        let stack2_ptr = core::ptr::addr_of_mut!(STACK_TASK2.0) as *mut u32;

        let stack1 = core::slice::from_raw_parts_mut(stack1_ptr, STACK_TASK1_WORDS);
        let stack2 = core::slice::from_raw_parts_mut(stack2_ptr, STACK_TASK2_WORDS);

        (stack1, stack2)
    };
    
    #[cfg(feature = "bench")]
    {
        kernel::create_task(bench::task_a, 0, stack1, 1);
        kernel::create_task(bench::task_b, 0, stack2, 1);
        log::log_line("bench tasks created, start_first_task()");
    }

    #[cfg(not(feature = "bench"))]
    {
        kernel::create_task(app::task1, 0, stack1, 1);
        kernel::create_task(app::task2, 0, stack2, 2);
        log::log_line("tasks created, start_first_task()");
    }

    // 8) 启动第一个任务（不返回）
    kernel::start();
}
