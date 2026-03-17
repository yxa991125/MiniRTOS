use core::sync::atomic::{AtomicI32, Ordering};

#[derive(Default)]
pub struct Encoder {
    count: AtomicI32,
}

impl Encoder {
    pub const fn new() -> Self {
        Self {
            count: AtomicI32::new(0),
        }
    }

    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
    }

    pub fn add(&self, delta: i32) {
        self.count.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn get(&self) -> i32 {
        self.count.load(Ordering::Relaxed)
    }
}
