use crate::device::gpio::GpioOutput;
use crate::kernel;
use crate::log;
use crate::sync::mutex::IrqMutex;
use stm32f4xx_hal::gpio::{gpioa::PA5, Output, PushPull};

pub type LedPin = PA5<Output<PushPull>>;

static LED: IrqMutex<Option<GpioOutput<LedPin>>> = IrqMutex::new(None);

pub fn init_led(pin: LedPin) {
    let mut led = GpioOutput::new(pin);
    let _ = led.set_low();
    LED.lock(|slot| *slot = Some(led));
}

pub fn task1(_arg: usize) -> ! {
    loop {
        log::log_line("task1: uart hello");
        kernel::sleep_ms(1000);
    }
}

pub fn task2(_arg: usize) -> ! {
    loop {
        LED.lock(|slot| {
            if let Some(led) = slot.as_mut() {
                let _ = led.toggle();
            }
        });
        kernel::sleep_ms(300);
    }
}
