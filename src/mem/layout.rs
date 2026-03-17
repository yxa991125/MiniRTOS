#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemLayout {
    pub flash_start: usize,
    pub flash_size: usize,
    pub ram_start: usize,
    pub ram_size: usize,
    pub stack_start: usize,
}

impl MemLayout {
    pub const fn new(
        flash_start: usize,
        flash_size: usize,
        ram_start: usize,
        ram_size: usize,
        stack_start: usize,
    ) -> Self {
        Self {
            flash_start,
            flash_size,
            ram_start,
            ram_size,
            stack_start,
        }
    }

    #[inline]
    pub const fn flash_end(&self) -> usize {
        self.flash_start + self.flash_size
    }

    #[inline]
    pub const fn ram_end(&self) -> usize {
        self.ram_start + self.ram_size
    }
}

unsafe extern "C" {
    static __STACK_START: u8;
}

pub const FLASH_START: usize = 0x0800_0000;
pub const FLASH_SIZE: usize = 512 * 1024;
pub const RAM_START: usize = 0x2000_0000;
pub const RAM_SIZE: usize = 128 * 1024;

pub fn layout() -> MemLayout {
    let stack_start = unsafe { &__STACK_START as *const u8 as usize };
    MemLayout::new(FLASH_START, FLASH_SIZE, RAM_START, RAM_SIZE, stack_start)
}
