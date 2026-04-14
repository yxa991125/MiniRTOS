use embedded_hal::digital::InputPin;

pub struct DigitalSensor<P> {
    pin: P,
}

impl<P: InputPin> DigitalSensor<P> {
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    pub fn is_active(&mut self) -> Result<bool, P::Error> {
        self.pin.is_high()
    }

    pub fn release(self) -> P {
        self.pin
    }
}
