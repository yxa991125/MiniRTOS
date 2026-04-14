use crate::bsp::current;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UartRole {
    BootConsole,
    AppUart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UartStats {
    pub rx_bytes: u32,
    pub tx_bytes: u32,
    pub rx_overflows: u32,
    pub tx_overflows: u32,
    pub rx_errors: u32,
    pub rx_pending: usize,
    pub tx_pending: usize,
}

pub fn boot_write_bytes(bytes: &[u8]) {
    current::uart::boot_write_bytes(bytes);
}

pub fn app_is_ready() -> bool {
    current::uart::app_is_ready()
}

pub fn app_wait_for_rx(timeout_ms: Option<u32>) -> Result<(), crate::sync::event::EventError> {
    current::uart::app_wait_for_rx(timeout_ms)
}

pub fn app_clear_rx_event() {
    current::uart::app_clear_rx_event();
}

pub fn app_read_byte() -> Option<u8> {
    current::uart::app_read_byte()
}

pub fn app_wait_for_tx(timeout_ms: Option<u32>) -> Result<(), crate::sync::event::EventError> {
    current::uart::app_wait_for_tx(timeout_ms)
}

pub fn app_clear_tx_event() {
    current::uart::app_clear_tx_event();
}

pub fn app_enqueue_tx_bytes(bytes: &[u8]) -> usize {
    current::uart::app_enqueue_tx_bytes(bytes)
}

pub fn app_drain_tx() -> usize {
    current::uart::app_drain_tx()
}

pub fn app_stats() -> UartStats {
    current::uart::app_stats()
}

pub fn log_bytes(bytes: &[u8]) -> usize {
    if app_is_ready() {
        app_enqueue_tx_bytes(bytes)
    } else {
        boot_write_bytes(bytes);
        bytes.len()
    }
}
