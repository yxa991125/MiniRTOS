use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingBufError {
    Full,
}

/// Fixed-size ring buffer for bytes.
pub struct RingBuf<const N: usize> {
    buf: [u8; N],
    head: usize,
    tail: usize,
    full: bool,
}

impl<const N: usize> RingBuf<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.full
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.full {
            N
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            N - (self.tail - self.head)
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.full = false;
    }

    pub fn push(&mut self, byte: u8) -> Result<(), RingBufError> {
        if self.full {
            return Err(RingBufError::Full);
        }

        self.buf[self.head] = byte;
        self.head = (self.head + 1) % N;
        if self.head == self.tail {
            self.full = true;
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        let byte = self.buf[self.tail];
        self.tail = (self.tail + 1) % N;
        self.full = false;
        Some(byte)
    }

    /// Push as many bytes as possible, return count pushed.
    pub fn push_slice(&mut self, data: &[u8]) -> usize {
        let mut pushed = 0;
        for &b in data {
            if self.push(b).is_err() {
                break;
            }
            pushed += 1;
        }
        pushed
    }

    /// Pop into `out`, return count popped.
    pub fn pop_slice(&mut self, out: &mut [u8]) -> usize {
        let mut popped = 0;
        for slot in out {
            if let Some(b) = self.pop() {
                *slot = b;
                popped += 1;
            } else {
                break;
            }
        }
        popped
    }
}

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

    pub fn pop(&self) -> Option<u8> {
        free(|cs| self.inner.borrow(cs).borrow_mut().pop())
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
}
