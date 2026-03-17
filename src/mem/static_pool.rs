use core::cell::UnsafeCell;
use core::mem::{align_of, size_of};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolError {
    OutOfMemory,
    InvalidLayout,
}

/// Simple fixed-block allocator.
pub struct StaticPool<const BLOCK_SIZE: usize, const BLOCK_COUNT: usize> {
    storage: UnsafeCell<[AlignedBlock<BLOCK_SIZE>; BLOCK_COUNT]>,
    free: UnsafeCell<[bool; BLOCK_COUNT]>,
}

unsafe impl<const BLOCK_SIZE: usize, const BLOCK_COUNT: usize> Sync
    for StaticPool<BLOCK_SIZE, BLOCK_COUNT>
{
}

impl<const BLOCK_SIZE: usize, const BLOCK_COUNT: usize> StaticPool<BLOCK_SIZE, BLOCK_COUNT> {
    pub const fn new() -> Self {
        Self {
            storage: UnsafeCell::new([AlignedBlock::new(); BLOCK_COUNT]),
            free: UnsafeCell::new([true; BLOCK_COUNT]),
        }
    }

    #[inline]
    const fn block_stride() -> usize {
        size_of::<AlignedBlock<BLOCK_SIZE>>()
    }

    #[inline]
    pub const fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    #[inline]
    pub const fn block_count(&self) -> usize {
        BLOCK_COUNT
    }

    #[inline]
    pub fn available(&self) -> usize {
        // Safe because we only read the bitmap.
        let free = unsafe { &*self.free.get() };
        free.iter().filter(|v| **v).count()
    }

    pub fn alloc(&self, size: usize, align: usize) -> Result<*mut u8, PoolError> {
        if size == 0 || size > BLOCK_SIZE {
            return Err(PoolError::OutOfMemory);
        }
        if align == 0 || align > POOL_ALIGN {
            return Err(PoolError::InvalidLayout);
        }

        let free = unsafe { &mut *self.free.get() };
        let storage = unsafe { &mut *self.storage.get() };

        for (index, slot) in free.iter_mut().enumerate() {
            if *slot {
                let ptr = storage[index].as_mut_ptr();
                *slot = false;
                return Ok(ptr);
            }
        }

        Err(PoolError::OutOfMemory)
    }

    pub fn alloc_for<T>(&self) -> Result<*mut T, PoolError> {
        if size_of::<T>() > BLOCK_SIZE {
            return Err(PoolError::OutOfMemory);
        }
        if align_of::<T>() > POOL_ALIGN {
            return Err(PoolError::InvalidLayout);
        }

        self.alloc(size_of::<T>(), align_of::<T>())
            .map(|ptr| ptr as *mut T)
    }

    pub fn free_ptr(&self, ptr: *mut u8) -> Result<(), PoolError> {
        if ptr.is_null() {
            return Err(PoolError::InvalidLayout);
        }

        let base_ptr = self.storage.get() as usize;
        let stride = Self::block_stride();
        let end_ptr = base_ptr + stride * BLOCK_COUNT;
        let ptr_val = ptr as usize;

        if ptr_val < base_ptr || ptr_val >= end_ptr {
            return Err(PoolError::InvalidLayout);
        }

        let offset = ptr_val - base_ptr;
        if offset % stride != 0 {
            return Err(PoolError::InvalidLayout);
        }

        let index = offset / stride;
        let free = unsafe { &mut *self.free.get() };
        free[index] = true;
        Ok(())
    }
}

const POOL_ALIGN: usize = 8;

#[derive(Clone, Copy)]
#[repr(align(8))]
struct AlignedBlock<const N: usize>([u8; N]);

impl<const N: usize> AlignedBlock<N> {
    const fn new() -> Self {
        Self([0; N])
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }
}
