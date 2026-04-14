use crate::bsp::current;

pub fn led_available() -> bool {
    current::controls::led_available()
}

pub fn set_led(on: bool) -> bool {
    current::controls::set_led(on)
}

pub fn toggle_led() -> bool {
    current::controls::toggle_led()
}

pub fn pwm_available() -> bool {
    current::controls::pwm_available()
}

pub fn set_pwm_percent(percent: u8) -> bool {
    current::controls::set_pwm_percent(percent)
}
