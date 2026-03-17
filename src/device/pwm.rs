use stm32f4xx_hal::hal::pwm::SetDutyCycle;

pub struct PwmChannel<P> {
    channel: P,
}

impl<P: SetDutyCycle> PwmChannel<P> {
    pub fn new(channel: P) -> Self {
        Self { channel }
    }

    pub fn max_duty(&self) -> u16 {
        self.channel.max_duty_cycle()
    }

    pub fn set_duty(&mut self, duty: u16) -> Result<(), P::Error> {
        self.channel.set_duty_cycle(duty)
    }

    pub fn set_duty_percent(&mut self, percent: u8) -> Result<(), P::Error> {
        let max = self.channel.max_duty_cycle() as u32;
        let duty = (max * percent as u32 / 100) as u16;
        self.channel.set_duty_cycle(duty)
    }

    pub fn set_duty_fraction(&mut self, num: u16, denom: u16) -> Result<(), P::Error> {
        self.channel.set_duty_cycle_fraction(num, denom)
    }

    pub fn release(self) -> P {
        self.channel
    }
}
