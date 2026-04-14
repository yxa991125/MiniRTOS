use crate::task::scheduler::MAX_PRIORITY;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Sleeping,
}

#[repr(C)]
pub struct Tcb {
    pub pid: usize,
    pub sp: *mut u32, // Stack pointer
    pub base_priority: u8,
    pub priority: u8,
    pub priority_boosts: [u8; MAX_PRIORITY],
    pub state: TaskState,
    pub remaining_slice: u32,
    pub wake_tick: u32,
    pub has_timeout: bool,
    pub ready_prev: Option<usize>,
    pub ready_next: Option<usize>,
    pub in_ready_queue: bool,
    pub timeout_prev: Option<usize>,
    pub timeout_next: Option<usize>,
    pub in_timeout_queue: bool,
    pub timeout_rounds: u32,
    pub stack_start: *mut u32, // Start of the stack
    pub stack_end: *mut u32,   // End of the stack
    pub entry: fn(usize) -> !,
    pub arg: usize,
    pub runtime_ticks: u32,
    pub heartbeat_registered: bool,
    pub heartbeat_timeout_ticks: u32,
    pub heartbeat_last_seen_tick: u32,
}

impl Tcb {
    /// Initialize a task control block in the ready state.
    pub fn init(
        pid: usize,
        sp: *mut u32,
        priority: u8,
        remaining_slice: u32,
        stack_start: *mut u32,
        stack_end: *mut u32,
        entry: fn(usize) -> !,
        arg: usize,
    ) -> Self {
        Self {
            pid,
            sp,
            base_priority: priority,
            priority,
            priority_boosts: [0; MAX_PRIORITY],
            state: TaskState::Ready,
            remaining_slice,
            wake_tick: 0,
            has_timeout: false,
            ready_prev: None,
            ready_next: None,
            in_ready_queue: false,
            timeout_prev: None,
            timeout_next: None,
            in_timeout_queue: false,
            timeout_rounds: 0,
            stack_start,
            stack_end,
            entry,
            arg,
            runtime_ticks: 0,
            heartbeat_registered: false,
            heartbeat_timeout_ticks: 0,
            heartbeat_last_seen_tick: 0,
        }
    }
}

unsafe impl Send for Tcb {}
