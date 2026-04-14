use crate::device::gpio::GpioOutput;
use crate::device::pwm::PwmChannel;
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotorDir {
    Forward,
    Reverse,
}

pub struct Motor<PWM, DIR> {
    pwm: PwmChannel<PWM>,
    dir: GpioOutput<DIR>,
    max_duty: u16,
    enabled: bool,
    dir_state: MotorDir,
}

impl<PWM, DIR> Motor<PWM, DIR>
where
    PWM: SetDutyCycle,
    DIR: OutputPin,
{
    pub fn new(pwm: PWM, dir: DIR) -> Self {
        let pwm = PwmChannel::new(pwm);
        let dir = GpioOutput::new(dir);
        let max_duty = pwm.max_duty();
        Self {
            pwm,
            dir,
            max_duty,
            enabled: false,
            dir_state: MotorDir::Forward,
        }
    }

    pub fn enable(&mut self) -> Result<(), PWM::Error> {
        self.enabled = true;
        self.pwm.set_duty(0)
    }

    pub fn disable(&mut self) -> Result<(), PWM::Error> {
        self.enabled = false;
        self.pwm.set_duty(0)
    }

    pub fn set_direction(&mut self, dir: MotorDir) -> Result<(), DIR::Error> {
        self.dir_state = dir;
        match dir {
            MotorDir::Forward => self.dir.set_low(),
            MotorDir::Reverse => self.dir.set_high(),
        }
    }

    pub fn set_speed_percent(&mut self, percent: u8) -> Result<(), PWM::Error> {
        if !self.enabled {
            return self.pwm.set_duty(0);
        }
        let pct = percent.min(100) as u32;
        let duty = (self.max_duty as u32 * pct / 100) as u16;
        self.pwm.set_duty(duty)
    }

    pub fn release(self) -> (PWM, DIR) {
        (self.pwm.release(), self.dir.release())
    }
}
