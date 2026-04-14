use crate::platform::uart;
pub use crate::platform::uart::UartStats;
use crate::sync::event::EventError;

pub fn is_ready() -> bool {
    uart::app_is_ready()
}

pub fn wait_for_rx(timeout_ms: Option<u32>) -> Result<(), EventError> {
    uart::app_wait_for_rx(timeout_ms)
}

pub fn clear_rx_event() {
    uart::app_clear_rx_event();
}

pub fn read_byte() -> Option<u8> {
    uart::app_read_byte()
}

pub fn wait_for_tx(timeout_ms: Option<u32>) -> Result<(), EventError> {
    uart::app_wait_for_tx(timeout_ms)
}

pub fn clear_tx_event() {
    uart::app_clear_tx_event();
}

pub fn enqueue_tx_bytes(bytes: &[u8]) -> usize {
    uart::app_enqueue_tx_bytes(bytes)
}

pub fn drain_tx() -> usize {
    uart::app_drain_tx()
}

pub fn stats() -> UartStats {
    uart::app_stats()
}

pub fn log_bytes(bytes: &[u8]) -> usize {
    uart::log_bytes(bytes)
}

pub fn raw_write_bytes(bytes: &[u8]) {
    uart::boot_write_bytes(bytes)
}
