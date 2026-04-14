use crate::platform::uart;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IoHealthSnapshot {
    pub uart_rx_bytes: u32,
    pub uart_tx_bytes: u32,
    pub uart_rx_overflows: u32,
    pub uart_tx_overflows: u32,
    pub uart_rx_errors: u32,
}

pub fn io_health() -> IoHealthSnapshot {
    let stats = uart::app_stats();
    IoHealthSnapshot {
        uart_rx_bytes: stats.rx_bytes,
        uart_tx_bytes: stats.tx_bytes,
        uart_rx_overflows: stats.rx_overflows,
        uart_tx_overflows: stats.tx_overflows,
        uart_rx_errors: stats.rx_errors,
    }
}
