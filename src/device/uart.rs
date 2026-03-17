use core::fmt;

use stm32f4xx_hal::gpio::PushPull;
use stm32f4xx_hal::rcc::Rcc;
use stm32f4xx_hal::serial::{self, config::Config, config::InvalidConfig, Serial, Tx, Rx};

pub type UartSerial<UART> = Serial<UART>;
pub type UartTx<UART> = Tx<UART>;
pub type UartRx<UART> = Rx<UART>;

pub struct UartPort<UART: serial::CommonPins + serial::Instance> {
    serial: Serial<UART>,
}

impl<UART: serial::CommonPins + serial::Instance> UartPort<UART> {
    pub fn new(
        uart: UART,
        pins: (
            impl Into<UART::Tx<PushPull>>,
            impl Into<UART::Rx<PushPull>>,
        ),
        config: impl Into<Config>,
        rcc: &mut Rcc,
    ) -> Result<Self, InvalidConfig> {
        let serial = Serial::new(uart, pins, config, rcc)?;
        Ok(Self { serial })
    }

    pub fn split(self) -> (Tx<UART>, Rx<UART>) {
        self.serial.split()
    }

    pub fn release(self) -> (UART, (Option<UART::Tx<PushPull>>, Option<UART::Rx<PushPull>>)) {
        self.serial.release()
    }

    pub fn into_inner(self) -> Serial<UART> {
        self.serial
    }
}

impl<UART: serial::CommonPins + serial::Instance> fmt::Write for UartPort<UART> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial.write_str(s)
    }
}
