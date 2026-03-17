use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};
use stm32f4xx_hal::pac;
use stm32f4xx_hal::pac::interrupt;

use crate::device::timer::{HalTimerHz, HardwareTimer, TimerMode};

pub type HwTimerCallback = fn();

struct Tim2State {
    timer: HalTimerHz<pac::TIM2>,
    callback: HwTimerCallback,
}

static TIM2_STATE: Mutex<RefCell<Option<Tim2State>>> = Mutex::new(RefCell::new(None));
static TIM3_STATE: Mutex<RefCell<Option<Tim3State>>> = Mutex::new(RefCell::new(None));

struct Tim3State {
    timer: HalTimerHz<pac::TIM3>,
    callback: HwTimerCallback,
}

pub fn init_tim2(
    tim2: pac::TIM2,
    rcc: &mut stm32f4xx_hal::rcc::Rcc,
    freq_hz: u32,
    mode: TimerMode,
    callback: HwTimerCallback,
) {
    let mut timer = HalTimerHz::new(tim2, rcc);
    timer.configure(freq_hz, mode);
    timer.listen_update();
    timer.start();

    free(|cs| {
        *TIM2_STATE.borrow(cs).borrow_mut() = Some(Tim2State { timer, callback });
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
    }
}

pub fn init_tim3(
    tim3: pac::TIM3,
    rcc: &mut stm32f4xx_hal::rcc::Rcc,
    freq_hz: u32,
    mode: TimerMode,
    callback: HwTimerCallback,
) {
    let mut timer = HalTimerHz::new(tim3, rcc);
    timer.configure(freq_hz, mode);
    timer.listen_update();
    timer.start();

    free(|cs| {
        *TIM3_STATE.borrow(cs).borrow_mut() = Some(Tim3State { timer, callback });
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM3);
    }
}

#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(state) = TIM2_STATE.borrow(cs).borrow_mut().as_mut() {
            if state.timer.wait().is_ok() {
                (state.callback)();
            }
        }
    });
}

#[interrupt]
fn TIM3() {
    free(|cs| {
        if let Some(state) = TIM3_STATE.borrow(cs).borrow_mut().as_mut() {
            if state.timer.wait().is_ok() {
                (state.callback)();
            }
        }
    });
}
