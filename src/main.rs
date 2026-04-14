#![no_std]
#![no_main]

#[cfg(not(feature = "uart-probe"))]
use core::ptr;
#[cfg(not(feature = "uart-probe"))]
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use panic_halt as _;

#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
mod app;
#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
mod app_protocol;
#[cfg(all(feature = "bench", not(feature = "uart-probe")))]
mod bench;
#[cfg(not(feature = "uart-probe"))]
mod bsp;
#[cfg(not(feature = "uart-probe"))]
mod device;
#[cfg(not(feature = "uart-probe"))]
mod driver;
#[cfg(not(feature = "uart-probe"))]
mod ipc;
#[cfg(not(feature = "uart-probe"))]
mod kernel;
#[cfg(not(feature = "uart-probe"))]
mod log;
#[cfg(not(feature = "uart-probe"))]
mod mem;
#[cfg(not(feature = "uart-probe"))]
mod platform;
#[cfg(not(feature = "uart-probe"))]
mod sync;
#[cfg(not(feature = "uart-probe"))]
mod task;
#[cfg(not(feature = "uart-probe"))]
mod timer;
#[cfg(feature = "uart-probe")]
mod uart_probe;

#[cfg(all(feature = "bench", feature = "uart-probe"))]
compile_error!("features `bench` and `uart-probe` cannot be enabled together");

#[repr(align(8))]
struct AlignedStack<const N: usize>([u32; N]);

#[cfg(feature = "bench")]
const STACK_TASK1_WORDS: usize = 4096;
#[cfg(feature = "bench")]
const STACK_TASK2_WORDS: usize = 1024;

#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
const STACK_UART_RX_WORDS: usize = 512;
#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
const STACK_APP_CMD_WORDS: usize = 768;
#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
const STACK_UART_TX_WORDS: usize = 384;
#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
const STACK_HEALTH_WORDS: usize = 384;

#[cfg(feature = "bench")]
static mut STACK_TASK1: AlignedStack<STACK_TASK1_WORDS> = AlignedStack([0; STACK_TASK1_WORDS]);
#[cfg(feature = "bench")]
static mut STACK_TASK2: AlignedStack<STACK_TASK2_WORDS> = AlignedStack([0; STACK_TASK2_WORDS]);

#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
static mut STACK_UART_RX: AlignedStack<STACK_UART_RX_WORDS> =
    AlignedStack([0; STACK_UART_RX_WORDS]);
#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
static mut STACK_APP_CMD: AlignedStack<STACK_APP_CMD_WORDS> =
    AlignedStack([0; STACK_APP_CMD_WORDS]);
#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
static mut STACK_UART_TX: AlignedStack<STACK_UART_TX_WORDS> =
    AlignedStack([0; STACK_UART_TX_WORDS]);
#[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
static mut STACK_HEALTH: AlignedStack<STACK_HEALTH_WORDS> = AlignedStack([0; STACK_HEALTH_WORDS]);

#[entry]
fn main() -> ! {
    #[cfg(feature = "uart-probe")]
    {
        return uart_probe::run();
    }

    #[cfg(not(feature = "uart-probe"))]
    {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let board = bsp::current::BoardContext::take().expect("failed to initialize board");
    board.emit_boot_banner();

    let sysclk_hz = board.sysclk_hz();
    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(sysclk_hz / 1000 - 1);
    cp.SYST.clear_current();

    unsafe {
        let scb = &mut *(cortex_m::peripheral::SCB::PTR as *mut cortex_m::peripheral::scb::RegisterBlock);
        scb.shpr[10].write(0xFF);
        scb.shpr[11].write(0xFE);
        let shcsr = 0xE000_ED24 as *mut u32;
        ptr::write_volatile(shcsr, ptr::read_volatile(shcsr) | (1 << 16) | (1 << 17) | (1 << 18));
    }

    log::init();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    kernel::init();
    kernel::set_reset_reason(board.reset_reason());

    #[cfg(feature = "bench")]
    {
        let mut board = board;
        board.init_bench(&mut cp.DCB, &mut cp.DWT);
    }

    #[cfg(feature = "bench")]
    {
        let (stack1, stack2): (&'static mut [u32], &'static mut [u32]) = unsafe {
            let stack1_ptr = core::ptr::addr_of_mut!(STACK_TASK1.0) as *mut u32;
            let stack2_ptr = core::ptr::addr_of_mut!(STACK_TASK2.0) as *mut u32;
            (
                core::slice::from_raw_parts_mut(stack1_ptr, STACK_TASK1_WORDS),
                core::slice::from_raw_parts_mut(stack2_ptr, STACK_TASK2_WORDS),
            )
        };

        let task_a_pid = kernel::create_task(bench::task_a, 0, stack1, 1)
            .expect("failed to create bench::task_a");
        let task_b_pid = kernel::create_task(bench::task_b, 0, stack2, 1)
            .expect("failed to create bench::task_b");
        bench::register_boot_tasks(task_a_pid, task_b_pid);
        log::with_logger(|tx| {
            use core::fmt::Write;
            let _ = writeln!(
                tx,
                "bench tasks created: task_a={} task_b={}, start_first_task()",
                task_a_pid, task_b_pid
            );
        });
    }

    #[cfg(all(not(feature = "bench"), not(feature = "uart-probe")))]
    {
        let (rx_stack, cmd_stack, tx_stack, health_stack) = unsafe {
            let rx_ptr = core::ptr::addr_of_mut!(STACK_UART_RX.0) as *mut u32;
            let cmd_ptr = core::ptr::addr_of_mut!(STACK_APP_CMD.0) as *mut u32;
            let tx_ptr = core::ptr::addr_of_mut!(STACK_UART_TX.0) as *mut u32;
            let health_ptr = core::ptr::addr_of_mut!(STACK_HEALTH.0) as *mut u32;

            (
                core::slice::from_raw_parts_mut(rx_ptr, STACK_UART_RX_WORDS),
                core::slice::from_raw_parts_mut(cmd_ptr, STACK_APP_CMD_WORDS),
                core::slice::from_raw_parts_mut(tx_ptr, STACK_UART_TX_WORDS),
                core::slice::from_raw_parts_mut(health_ptr, STACK_HEALTH_WORDS),
            )
        };

        let [rx_pid, cmd_pid, tx_pid, health_pid] =
            app::create_default_tasks(rx_stack, cmd_stack, tx_stack, health_stack)
                .expect("failed to create default app tasks");

        log::with_logger(|tx| {
            use core::fmt::Write;
            let _ = writeln!(
                tx,
                "app tasks created: rx={} cmd={} tx={} health={}, start_first_task()",
                rx_pid, cmd_pid, tx_pid, health_pid
            );
        });

        #[cfg(all(not(debug_assertions), feature = "board-f411-nucleo"))]
        {
            // Start watchdog after app tasks are registered.
            let _ = kernel::enable_watchdog(1_500);
        }
    }

    kernel::start();
    }
}
