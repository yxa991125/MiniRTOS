use crate::task::tcb::TaskState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskDiagnostics {
    pub pid: usize,
    pub state: TaskState,
    pub base_priority: u8,
    pub priority: u8,
    pub remaining_slice: u32,
    pub wake_tick: u32,
    pub has_timeout: bool,
    pub runtime_ticks: u32,
    pub stack_size_words: usize,
    pub stack_free_low_water_words: usize,
    pub stack_used_high_water_words: usize,
    pub heartbeat_registered: bool,
    pub heartbeat_timeout_ticks: u32,
    pub heartbeat_age_ticks: u32,
    pub heartbeat_stale: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResetReason {
    #[default]
    Unknown,
    PowerOn,
    PinReset,
    Software,
    IndependentWatchdog,
    WindowWatchdog,
    Brownout,
    LowPower,
}

impl ResetReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::PowerOn => "power_on",
            Self::PinReset => "pin_reset",
            Self::Software => "software",
            Self::IndependentWatchdog => "iwdg",
            Self::WindowWatchdog => "wwdg",
            Self::Brownout => "brownout",
            Self::LowPower => "low_power",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SystemHealth {
    pub uptime_ticks: u32,
    pub live_tasks: u32,
    pub registered_heartbeats: u32,
    pub stale_tasks: u32,
    pub stack_warning_tasks: u32,
    pub reset_reason: ResetReason,
    pub watchdog_enabled: bool,
    pub watchdog_feeds: u32,
    pub uart_rx_bytes: u32,
    pub uart_tx_bytes: u32,
    pub uart_rx_overflows: u32,
    pub uart_tx_overflows: u32,
    pub uart_rx_errors: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TraceCounters {
    pub task_creates: u32,
    pub context_switches: u32,
    pub task_sleeps: u32,
    pub task_blocks: u32,
    pub task_unblocks: u32,
    pub task_deletes: u32,
    pub timeout_expirations: u32,
    pub priority_updates: u32,
    pub pendsv_requests: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceEventKind {
    TaskCreate,
    ContextSwitch,
    TaskSleep,
    TaskBlock,
    TaskUnblock,
    TaskDelete,
    TimeoutExpire,
    PriorityUpdate,
    PendSvRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceEvent {
    pub tick: u32,
    pub kind: TraceEventKind,
    pub pid: usize,
    pub aux: usize,
}

pub type TraceHook = fn(TraceEvent);
