use stm32f4xx_hal::hal::digital::{InputPin, OutputPin, StatefulOutputPin};

pub struct GpioOutput<P> {
    pin: P,
}

impl<P: OutputPin> GpioOutput<P> {
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    pub fn set_high(&mut self) -> Result<(), P::Error> {
        self.pin.set_high()
    }

    pub fn set_low(&mut self) -> Result<(), P::Error> {
        self.pin.set_low()
    }

    pub fn release(self) -> P {
        self.pin
    }
}

impl<P: OutputPin + StatefulOutputPin> GpioOutput<P> {
    pub fn toggle(&mut self) -> Result<(), P::Error> {
        if self.pin.is_set_high()? {
            self.pin.set_low()
        } else {
            self.pin.set_high()
        }
    }
}

pub struct GpioInput<P> {
    pin: P,
}

impl<P: InputPin> GpioInput<P> {
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    pub fn is_high(&mut self) -> Result<bool, P::Error> {
        self.pin.is_high()
    }

    pub fn is_low(&mut self) -> Result<bool, P::Error> {
        self.pin.is_low()
    }

    pub fn release(self) -> P {
        self.pin
    }
}
