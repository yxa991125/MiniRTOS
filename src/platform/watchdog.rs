use crate::bsp::current;

pub fn start(timeout_ms: u32) -> bool {
    current::watchdog::start(timeout_ms)
}

pub fn feed() -> bool {
    current::watchdog::feed()
}
