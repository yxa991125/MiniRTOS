use core::cell::RefCell;

use cortex_m::interrupt::{Mutex, free};

use crate::task::scheduler;

/// IRQ-safe mutex wrapper using `cortex_m::interrupt::Mutex`.
pub struct IrqMutex<T> {
    inner: Mutex<RefCell<T>>,
}

impl<T> IrqMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(value)),
        }
    }

    pub fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        free(|cs| {
            let mut guard = self.inner.borrow(cs).borrow_mut();
            f(&mut *guard)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutexError {
    QueueFull,
    Timeout,
    NoCurrentTask,
    NotOwner,
    WouldDeadlock,
}

/// Blocking mutex with a fixed-size wait queue and basic priority inheritance.
///
/// This mutex is non-recursive: locking it again from the owning task returns
/// `WouldDeadlock`.
pub struct BlockingMutex<T, const N: usize> {
    inner: Mutex<RefCell<BlockingMutexInner<T, N>>>,
}

impl<T, const N: usize> BlockingMutex<T, N> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(BlockingMutexInner::new(value))),
        }
    }

    pub fn acquire(&self, timeout_ms: Option<u32>) -> Result<(), MutexError> {
        let pid = scheduler::current_pid().ok_or(MutexError::NoCurrentTask)?;

        loop {
            let waiter_prio = scheduler::task_priority(pid).ok_or(MutexError::NoCurrentTask)?;
            let mut decision = AcquireDecision::Acquired;
            let mut boost_owner = None;

            free(|cs| {
                let mut inner = self.inner.borrow(cs).borrow_mut();
                match inner.owner {
                    None => {
                        inner.owner = Some(pid);
                        decision = AcquireDecision::Acquired;
                    }
                    Some(owner) if owner == pid => {
                        decision = AcquireDecision::WouldDeadlock;
                    }
                    Some(owner) => {
                        if !inner.waiters.contains(pid) {
                            if !inner.waiters.push(pid) {
                                decision = AcquireDecision::QueueFull;
                                return;
                            }
                        }

                        boost_owner = Some(owner);
                        decision = AcquireDecision::Blocked;
                    }
                }
            });

            if let Some(owner) = boost_owner {
                let _ = scheduler::add_priority_boost(owner, waiter_prio);
                scheduler::block_current(timeout_ms);
            }

            match decision {
                AcquireDecision::Acquired => return Ok(()),
                AcquireDecision::QueueFull => return Err(MutexError::QueueFull),
                AcquireDecision::WouldDeadlock => return Err(MutexError::WouldDeadlock),
                AcquireDecision::Blocked => {
                    let mut acquired = false;
                    let mut removed_from_waiters = false;
                    let mut owner_to_deboost = None;

                    free(|cs| {
                        let mut inner = self.inner.borrow(cs).borrow_mut();
                        if inner.owner == Some(pid) {
                            acquired = true;
                            return;
                        }

                        if inner.waiters.remove(pid) {
                            removed_from_waiters = true;
                            owner_to_deboost = inner.owner;
                        }
                    });

                    if let Some(owner) = owner_to_deboost {
                        let _ = scheduler::remove_priority_boost(owner, waiter_prio);
                    }

                    if acquired {
                        return Ok(());
                    }

                    if removed_from_waiters && timeout_ms.is_some() {
                        return Err(MutexError::Timeout);
                    }
                }
            }
        }
    }

    pub fn release(&self) -> Result<(), MutexError> {
        let pid = scheduler::current_pid().ok_or(MutexError::NoCurrentTask)?;
        let mut error = None;
        let mut old_owner = None;
        let mut old_waiters = [usize::MAX; N];
        let mut old_waiter_count = 0usize;
        let mut new_owner = None;
        let mut remaining_waiters = [usize::MAX; N];
        let mut remaining_waiter_count = 0usize;

        free(|cs| {
            let mut inner = self.inner.borrow(cs).borrow_mut();
            if inner.owner != Some(pid) {
                error = Some(MutexError::NotOwner);
                return;
            }

            old_owner = Some(pid);
            old_waiter_count = inner.waiters.snapshot(&mut old_waiters);

            if let Some(next_pid) = inner.waiters.pop_highest() {
                inner.owner = Some(next_pid);
                new_owner = Some(next_pid);
                remaining_waiter_count = inner.waiters.snapshot(&mut remaining_waiters);
            } else {
                inner.owner = None;
            }
        });

        if let Some(err) = error {
            return Err(err);
        }

        if let Some(owner) = old_owner {
            for waiter in old_waiters.iter().copied().take(old_waiter_count) {
                if let Some(prio) = scheduler::task_priority(waiter) {
                    let _ = scheduler::remove_priority_boost(owner, prio);
                }
            }
        }

        if let Some(owner) = new_owner {
            for waiter in remaining_waiters
                .iter()
                .copied()
                .take(remaining_waiter_count)
            {
                if let Some(prio) = scheduler::task_priority(waiter) {
                    let _ = scheduler::add_priority_boost(owner, prio);
                }
            }
            scheduler::unblock(owner);
        }

        Ok(())
    }

    pub fn with_owner<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, MutexError> {
        let pid = scheduler::current_pid().ok_or(MutexError::NoCurrentTask)?;

        free(|cs| {
            let mut inner = self.inner.borrow(cs).borrow_mut();
            if inner.owner != Some(pid) {
                return Err(MutexError::NotOwner);
            }

            Ok(f(&mut inner.value))
        })
    }
}

struct BlockingMutexInner<T, const N: usize> {
    owner: Option<usize>,
    value: T,
    waiters: WaitQueue<N>,
}

impl<T, const N: usize> BlockingMutexInner<T, N> {
    const fn new(value: T) -> Self {
        Self {
            owner: None,
            value,
            waiters: WaitQueue::new(),
        }
    }
}

enum AcquireDecision {
    Acquired,
    QueueFull,
    Blocked,
    WouldDeadlock,
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

    fn contains(&self, pid: usize) -> bool {
        self.buf.iter().any(|slot| slot.as_ref() == Some(&pid))
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

    fn pop_highest(&mut self) -> Option<usize> {
        if self.len == 0 {
            return None;
        }

        let mut best_idx = None;
        let mut best_prio = u8::MAX;
        for (idx, slot) in self.buf.iter().enumerate() {
            if let Some(pid) = slot {
                let prio = scheduler::task_priority(*pid).unwrap_or(u8::MAX);
                if best_idx.is_none() || prio < best_prio {
                    best_idx = Some(idx);
                    best_prio = prio;
                }
            }
        }

        let idx = best_idx?;
        let pid = self.buf[idx].take()?;
        self.len -= 1;
        Some(pid)
    }

    fn snapshot(&self, out: &mut [usize; N]) -> usize {
        let mut count = 0usize;
        for slot in self.buf.iter() {
            if let Some(pid) = slot {
                out[count] = *pid;
                count += 1;
                if count == N {
                    break;
                }
            }
        }
        count
    }
}
