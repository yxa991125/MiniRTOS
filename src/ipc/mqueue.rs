use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgQueueError {
    Full,
}

/// Fixed-size message queue for `usize` messages.
pub struct MsgQueue<const N: usize> {
    buf: [usize; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<const N: usize> MsgQueue<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.len = 0;
    }

    pub fn send(&mut self, msg: usize) -> Result<(), MsgQueueError> {
        if self.is_full() {
            return Err(MsgQueueError::Full);
        }

        self.buf[self.head] = msg;
        self.head = (self.head + 1) % N;
        self.len += 1;
        Ok(())
    }

    pub fn recv(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        let msg = self.buf[self.tail];
        self.tail = (self.tail + 1) % N;
        self.len -= 1;
        Some(msg)
    }
}

/// IRQ-safe wrapper for `MsgQueue`.
pub struct SyncMsgQueue<const N: usize> {
    inner: Mutex<RefCell<MsgQueue<N>>>,
}

impl<const N: usize> SyncMsgQueue<N> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(MsgQueue::new())),
        }
    }

    pub fn send(&self, msg: usize) -> Result<(), MsgQueueError> {
        free(|cs| self.inner.borrow(cs).borrow_mut().send(msg))
    }

    pub fn recv(&self) -> Option<usize> {
        free(|cs| self.inner.borrow(cs).borrow_mut().recv())
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
}
