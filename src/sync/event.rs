use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};

use crate::task::scheduler;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventError {
    QueueFull,
    Timeout,
    NoCurrentTask,
}

/// Manual-reset event with a fixed-size wait queue.
pub struct Event<const N: usize> {
    inner: Mutex<RefCell<EventInner<N>>>,
}

impl<const N: usize> Event<N> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(EventInner::new())),
        }
    }

    pub fn is_set(&self) -> bool {
        free(|cs| self.inner.borrow(cs).borrow().signaled)
    }

    pub fn clear(&self) {
        free(|cs| {
            self.inner.borrow(cs).borrow_mut().signaled = false;
        });
    }

    /// Set the event and wake all waiting tasks.
    /// Returns the number of tasks unblocked.
    pub fn set(&self) -> usize {
        let mut to_wake = [0usize; N];
        let mut wake_count = 0usize;

        free(|cs| {
            let mut inner = self.inner.borrow(cs).borrow_mut();
            inner.signaled = true;
            inner.signal_seq = inner.signal_seq.wrapping_add(1);
            while let Some(pid) = inner.waiters.pop() {
                if wake_count < N {
                    to_wake[wake_count] = pid;
                    wake_count += 1;
                }
            }
        });

        for i in 0..wake_count {
            scheduler::unblock(to_wake[i]);
        }

        wake_count
    }

    /// Non-blocking wait. Returns true if already signaled.
    pub fn try_wait(&self) -> bool {
        free(|cs| self.inner.borrow(cs).borrow().signaled)
    }

    /// Block until the event is set, or timeout occurs.
    pub fn wait(&self, timeout_ms: Option<u32>) -> Result<(), EventError> {
        let pid = scheduler::current_pid().ok_or(EventError::NoCurrentTask)?;

        loop {
            let mut decision = WaitDecision::Ready;
            let mut epoch = 0u32;

            free(|cs| {
                let mut inner = self.inner.borrow(cs).borrow_mut();

                if inner.signaled {
                    decision = WaitDecision::Ready;
                    return;
                }

                if !inner.waiters.push(pid) {
                    decision = WaitDecision::Full;
                    return;
                }

                epoch = inner.signal_seq;
                decision = WaitDecision::Blocked;
                drop(inner);
                scheduler::block_current(timeout_ms);
            });

            match decision {
                WaitDecision::Ready => return Ok(()),
                WaitDecision::Full => return Err(EventError::QueueFull),
                WaitDecision::Blocked => {
                    let signaled = free(|cs| {
                        let mut inner = self.inner.borrow(cs).borrow_mut();
                        if inner.signaled || inner.signal_seq != epoch {
                            true
                        } else {
                            inner.waiters.remove(pid);
                            false
                        }
                    });

                    if signaled {
                        return Ok(());
                    }

                    if timeout_ms.is_some() {
                        return Err(EventError::Timeout);
                    }
                }
            }
        }
    }
}

struct EventInner<const N: usize> {
    signaled: bool,
    signal_seq: u32,
    waiters: WaitQueue<N>,
}

impl<const N: usize> EventInner<N> {
    const fn new() -> Self {
        Self {
            signaled: false,
            signal_seq: 0,
            waiters: WaitQueue::new(),
        }
    }
}

enum WaitDecision {
    Ready,
    Full,
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
