use core::cell::RefCell;

use cortex_m::interrupt::{Mutex, free};

use crate::task::scheduler;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemaphoreError {
    QueueFull,
    Timeout,
    NoCurrentTask,
}

/// Counting semaphore with a fixed-size wait queue.
pub struct Semaphore<const N: usize> {
    inner: Mutex<RefCell<SemaphoreInner<N>>>,
}

impl<const N: usize> Semaphore<N> {
    pub const fn new(initial: u32, max: u32) -> Self {
        let init = if initial > max { max } else { initial };
        Self {
            inner: Mutex::new(RefCell::new(SemaphoreInner::new(init, max))),
        }
    }

    pub fn available(&self) -> u32 {
        free(|cs| self.inner.borrow(cs).borrow().count)
    }

    pub fn max(&self) -> u32 {
        free(|cs| self.inner.borrow(cs).borrow().max)
    }

    /// Non-blocking acquire.
    pub fn try_acquire(&self) -> bool {
        free(|cs| {
            let mut inner = self.inner.borrow(cs).borrow_mut();
            if inner.count > 0 {
                inner.count -= 1;
                true
            } else {
                false
            }
        })
    }

    /// Release a token. Returns true if released or a waiter is unblocked.
    pub fn release(&self) -> bool {
        let mut to_unblock = None;
        let mut released = false;

        free(|cs| {
            let mut inner = self.inner.borrow(cs).borrow_mut();
            if let Some(pid) = inner.waiters.pop() {
                if inner.granted.push(pid) {
                    to_unblock = Some(pid);
                    released = true;
                } else {
                    if inner.count < inner.max {
                        inner.count += 1;
                        released = true;
                    }
                    to_unblock = Some(pid);
                }
            } else if inner.count < inner.max {
                inner.count += 1;
                released = true;
            }
        });

        if let Some(pid) = to_unblock {
            scheduler::unblock(pid);
        }

        released
    }

    /// Acquire with optional timeout.
    pub fn acquire(&self, timeout_ms: Option<u32>) -> Result<(), SemaphoreError> {
        let pid = scheduler::current_pid().ok_or(SemaphoreError::NoCurrentTask)?;

        loop {
            let mut decision = AcquireDecision::Acquired;

            free(|cs| {
                let mut inner = self.inner.borrow(cs).borrow_mut();
                if inner.count > 0 {
                    inner.count -= 1;
                    decision = AcquireDecision::Acquired;
                    return;
                }

                if inner.granted.remove(pid) {
                    decision = AcquireDecision::Acquired;
                    return;
                }

                if !inner.waiters.push(pid) {
                    decision = AcquireDecision::QueueFull;
                    return;
                }

                decision = AcquireDecision::Blocked;
                drop(inner);
                scheduler::block_current(timeout_ms);
            });

            match decision {
                AcquireDecision::Acquired => return Ok(()),
                AcquireDecision::QueueFull => return Err(SemaphoreError::QueueFull),
                AcquireDecision::Blocked => {
                    let acquired = free(|cs| {
                        let mut inner = self.inner.borrow(cs).borrow_mut();
                        if inner.granted.remove(pid) {
                            true
                        } else if inner.count > 0 {
                            inner.count -= 1;
                            true
                        } else {
                            inner.waiters.remove(pid);
                            false
                        }
                    });

                    if acquired {
                        return Ok(());
                    }

                    if timeout_ms.is_some() {
                        return Err(SemaphoreError::Timeout);
                    }
                }
            }
        }
    }
}

struct SemaphoreInner<const N: usize> {
    count: u32,
    max: u32,
    waiters: WaitQueue<N>,
    granted: WaitQueue<N>,
}

impl<const N: usize> SemaphoreInner<N> {
    const fn new(initial: u32, max: u32) -> Self {
        Self {
            count: initial,
            max,
            waiters: WaitQueue::new(),
            granted: WaitQueue::new(),
        }
    }
}

enum AcquireDecision {
    Acquired,
    QueueFull,
    Blocked,
}

struct WaitQueue<const N: usize> {
    buf: [Option<usize>; N],
    len: usize,
}

impl<const N: usize> WaitQueue<N> {
    const fn new() -> Self {
        Self {
            buf: [const { None }; N],
            len: 0,
        }
    }

    fn push(&mut self, pid: usize) -> bool {
        if self.len >= N {
            return false;
        }
        for slot in self.buf.iter_mut() {
            if slot.is_none() {
                *slot = Some(pid);
                self.len += 1;
                return true;
            }
        }
        false
    }

    fn pop(&mut self) -> Option<usize> {
        if self.len == 0 {
            return None;
        }
        for slot in self.buf.iter_mut() {
            if let Some(pid) = slot.take() {
                self.len -= 1;
                return Some(pid);
            }
        }
        None
    }

    fn remove(&mut self, pid: usize) -> bool {
        if self.len == 0 {
            return false;
        }
        for slot in self.buf.iter_mut() {
            if slot.as_ref() == Some(&pid) {
                *slot = None;
                self.len -= 1;
                return true;
            }
        }
        false
    }
}
