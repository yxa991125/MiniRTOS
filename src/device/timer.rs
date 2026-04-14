use crate::timer::systick;

use stm32f4xx_hal::rcc::Rcc;
use stm32f4xx_hal::time::Hertz;
use stm32f4xx_hal::timer::{CounterHz, Error as HalTimerError, Event, Instance, TimerExt};
use stm32f4xx_hal::{Listen, nb};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerMode {
    OneShot,
    Periodic,
}

pub trait TimerDevice {
    fn now_ticks(&self) -> u32;
    fn ticks_to_ms(&self, ticks: u32) -> u32;
    fn ms_to_ticks(&self, ms: u32) -> u32;
    fn delay_ms(&self, ms: u32);
}

pub trait HardwareTimer {
    /// `period_ticks` is interpreted by the implementation.
    /// For `HalTimerHz`, it represents a frequency in Hz.
    fn configure(&mut self, period_ticks: u32, mode: TimerMode);
    fn start(&mut self);
    fn stop(&mut self);
    fn is_running(&self) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerError {
    Disabled,
    WrongAutoReload,
    InvalidPeriod,
}

impl From<HalTimerError> for TimerError {
    fn from(err: HalTimerError) -> Self {
        match err {
            HalTimerError::Disabled => TimerError::Disabled,
            HalTimerError::WrongAutoReload => TimerError::WrongAutoReload,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemTimer;

impl TimerDevice for SystemTimer {
    #[inline]
    fn now_ticks(&self) -> u32 {
        systick::now()
    }

    #[inline]
    fn ticks_to_ms(&self, ticks: u32) -> u32 {
        systick::ticks_to_ms(ticks)
    }

    #[inline]
    fn ms_to_ticks(&self, ms: u32) -> u32 {
        systick::ms_to_ticks(ms)
    }

    #[inline]
    fn delay_ms(&self, ms: u32) {
        systick::delay_ms(ms)
    }
}

pub const SYSTEM_TIMER: SystemTimer = SystemTimer;

pub struct HalTimerHz<TIM: Instance> {
    counter: CounterHz<TIM>,
    period_hz: u32,
    mode: TimerMode,
    running: bool,
}

impl<TIM: Instance> HalTimerHz<TIM> {
    pub fn new(tim: TIM, rcc: &mut Rcc) -> Self {
        Self {
            counter: tim.counter_hz(rcc),
            period_hz: 1,
            mode: TimerMode::Periodic,
            running: false,
        }
    }

    pub fn start_result(&mut self) -> Result<(), TimerError> {
        if self.period_hz == 0 {
            return Err(TimerError::InvalidPeriod);
        }
        self.counter
            .start(Hertz::from_raw(self.period_hz))
            .map_err(TimerError::from)?;
        self.running = true;
        Ok(())
    }

    pub fn wait(&mut self) -> nb::Result<(), TimerError> {
        match self.counter.wait() {
            Ok(()) => {
                if self.mode == TimerMode::OneShot {
                    let _ = self.counter.cancel();
                    self.running = false;
                }
                Ok(())
            }
            Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(err)) => Err(nb::Error::Other(err.into())),
        }
    }

    pub fn listen_update(&mut self) {
        self.counter.listen(Event::Update);
    }

    pub fn unlisten_update(&mut self) {
        self.counter.unlisten(Event::Update);
    }

    pub fn release(self) -> TIM {
        self.counter.release().release()
    }
}

impl<TIM: Instance> HardwareTimer for HalTimerHz<TIM> {
    fn configure(&mut self, period_ticks: u32, mode: TimerMode) {
        self.period_hz = period_ticks.max(1);
        self.mode = mode;
    }

    fn start(&mut self) {
        let _ = self.start_result();
    }

    fn stop(&mut self) {
        let _ = self.counter.cancel();
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}
