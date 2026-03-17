use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};

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
