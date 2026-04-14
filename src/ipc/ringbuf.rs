use core::cell::RefCell;

use cortex_m::interrupt::{CriticalSection, Mutex, free};

pub use crate::ipc::ringbuf_core::{RingBuf, RingBufError};

/// IRQ-safe wrapper for `RingBuf`.
pub struct SyncRingBuf<const N: usize> {
    inner: Mutex<RefCell<RingBuf<N>>>,
}

impl<const N: usize> SyncRingBuf<N> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(RingBuf::new())),
        }
    }

    pub fn push(&self, byte: u8) -> Result<(), RingBufError> {
        free(|cs| self.inner.borrow(cs).borrow_mut().push(byte))
    }

    pub fn push_from_isr(&self, byte: u8) -> Result<(), RingBufError> {
        let cs: &CriticalSection = unsafe { &CriticalSection::new() };
        self.inner.borrow(cs).borrow_mut().push(byte)
    }

    pub fn pop(&self) -> Option<u8> {
        free(|cs| self.inner.borrow(cs).borrow_mut().pop())
    }

    pub fn pop_from_isr(&self) -> Option<u8> {
        let cs: &CriticalSection = unsafe { &CriticalSection::new() };
        self.inner.borrow(cs).borrow_mut().pop()
    }

    pub fn push_slice(&self, data: &[u8]) -> usize {
        free(|cs| self.inner.borrow(cs).borrow_mut().push_slice(data))
    }

    pub fn pop_slice(&self, out: &mut [u8]) -> usize {
        free(|cs| self.inner.borrow(cs).borrow_mut().pop_slice(out))
    }

    pub fn len(&self) -> usize {
        free(|cs| self.inner.borrow(cs).borrow().len())
    }

    pub fn is_empty(&self) -> bool {
        free(|cs| self.inner.borrow(cs).borrow().is_empty())
    }

    pub fn is_full(&self) -> bool {
        free(|cs| self.inner.borrow(cs).borrow().is_full())
    }

    pub fn clear(&self) {
        free(|cs| self.inner.borrow(cs).borrow_mut().clear())
    }
}
