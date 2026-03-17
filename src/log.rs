// src/log.rs
use core::fmt::Write;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use stm32f4xx_hal::pac::USART2;
use stm32f4xx_hal::serial::Tx;

// 全局串口发送端，用互斥 + RefCell 封装
static LOGGER: Mutex<RefCell<Option<Tx<USART2>>>> =
    Mutex::new(RefCell::new(None));

// 在 main 里调用，把 tx 注册进来
pub fn init(tx: Tx<USART2>) {
    cortex_m::interrupt::free(|cs| {
        *LOGGER.borrow(cs).borrow_mut() = Some(tx);
    });
}

pub fn with_logger<R>(f: impl FnOnce(&mut Tx<USART2>) -> R) -> Option<R> {
    cortex_m::interrupt::free(|cs| {
        let mut logger = LOGGER.borrow(cs).borrow_mut();
        logger.as_mut().map(f)
    })
}

// 简单打印一行字符串
pub fn log_line(s: &str) {
    with_logger(|tx| {
            let _ = writeln!(tx, "{s}");
    });
}
