#[cfg(feature = "board-f411-nucleo")]
pub mod adc;
pub mod gpio;
pub mod pwm;
#[cfg(feature = "board-f411-nucleo")]
pub mod timer;
pub mod uart;
