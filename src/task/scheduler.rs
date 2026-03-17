use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
use cortex_m::interrupt::free;

use crate::task::tcb::{Tcb, TaskState};

pub const MAX_TASKS: usize = 8;
pub const MAX_PRIORITY: usize = 8;

static TASKS: Mutex<RefCell<[Option<Tcb>; MAX_TASKS]>> =
    Mutex::new(RefCell::new([None; MAX_TASKS]));

// Ready set represented as a bitmap of priorities.
static READY_SET: Mutex<RefCell<[bool; MAX_PRIORITY]>> =
    Mutex::new(RefCell::new([false; MAX_PRIORITY]));

static CURRENT_PID: Mutex<RefCell<usize>> =
    Mutex::new(RefCell::new(0));

/// Initialize task storage, ready set, and reset current pid.
pub fn init_scheduler() {
    free(|cs| {
        TASKS.borrow(cs)
            .borrow_mut()
            .iter_mut()
            .for_each(|slot| *slot = None);

        READY_SET
            .borrow(cs)
            .borrow_mut()
            .iter_mut()
            .for_each(|flag| *flag = false);

        *CURRENT_PID.borrow(cs).borrow_mut() = 0;
    });
}


pub fn create_task() {
    
}

pub fn tick() {

}

pub fn start_first_task() {

}

pub fn switch_context() {

}

