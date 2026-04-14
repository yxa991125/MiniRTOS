use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use cortex_m::interrupt::free;
#[cfg(feature = "bench")]
use cortex_m::peripheral::{DCB, DWT};
use stm32f1::stm32f103::{self as pac, interrupt};

use crate::{
    ipc::ringbuf::SyncRingBuf,
    kernel,
    platform::uart::UartStats,
    sync::event::{Event, EventError},
};

pub struct BoardContext {
    reset_reason: kernel::ResetReason,
    sysclk_hz: u32,
}

impl BoardContext {
    pub fn take() -> Option<Self> {
        let _ = pac::Peripherals::take()?;
        let reset_reason = detect_reset_reason();
        clear_reset_flags();
        configure_board_peripherals();
        uart::init_hardware();
        #[cfg(not(feature = "bench"))]
        {
            uart::init_app_uart();
            controls::init_hardware();
        }
        watchdog::reset_state();

        Some(Self {
            reset_reason,
            // Keep the profile conservative and predictable in this stage.
            sysclk_hz: 8_000_000,
        })
    }

    pub fn reset_reason(&self) -> kernel::ResetReason {
        self.reset_reason
    }

    pub fn sysclk_hz(&self) -> u32 {
        self.sysclk_hz
    }

    pub fn emit_boot_banner(&self) {
        let mut tx = BootWriter;
        let msp_v = cortex_m::register::msp::read();
        let psp_v = cortex_m::register::psp::read();
        let vtor = unsafe { (*cortex_m::peripheral::SCB::PTR).vtor.read() };
        let pendsv_vec = unsafe { *((vtor as *const u32).add(14)) };
        let systick_vec = unsafe { *((vtor as *const u32).add(15)) };

        let _ = writeln!(tx, "boot ok (F103)");
        let _ = writeln!(tx, "reset={}", self.reset_reason.as_str());
        let _ = writeln!(tx, "MSP=0x{:08x} PSP=0x{:08x} VTOR=0x{:08x}", msp_v, psp_v, vtor);
        let _ = writeln!(tx, "VEC PendSV=0x{:08x} SysTick=0x{:08x}", pendsv_vec, systick_vec);
        let _ = writeln!(tx, "cpu={}Hz", self.sysclk_hz);
    }

    #[cfg(feature = "bench")]
    pub fn init_bench(&mut self, _dcb: &mut DCB, _dwt: &mut DWT) {
        panic!("bench is not supported for board-f103c8-bluepill in this stage");
    }
}

struct BootWriter;

impl Write for BootWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart::boot_write_bytes(s.as_bytes());
        Ok(())
    }
}

const RCC_BASE: usize = 0x4002_1000;
const RCC_APB2ENR: *mut u32 = (RCC_BASE + 0x18) as *mut u32;
const RCC_APB1ENR: *mut u32 = (RCC_BASE + 0x1C) as *mut u32;
const RCC_CSR: *mut u32 = (RCC_BASE + 0x24) as *mut u32;
const AFIO_MAPR: *mut u32 = 0x4001_0004 as *mut u32;
const GPIOA_CRL: *mut u32 = 0x4001_0800 as *mut u32;
const GPIOA_CRH: *mut u32 = 0x4001_0804 as *mut u32;
const GPIOB_CRH: *mut u32 = 0x4001_0C04 as *mut u32;
const GPIOC_CRH: *mut u32 = 0x4001_1004 as *mut u32;
const GPIOC_BSRR: *mut u32 = 0x4001_1010 as *mut u32;

fn detect_reset_reason() -> kernel::ResetReason {
    let csr = unsafe { RCC_CSR.read_volatile() };

    // STM32F1 RCC_CSR reset flags:
    // PINRSTF[26], PORRSTF[27], SFTRSTF[28], IWDGRSTF[29], WWDGRSTF[30], LPWRRSTF[31]
    if csr & (1 << 29) != 0 {
        kernel::ResetReason::IndependentWatchdog
    } else if csr & (1 << 30) != 0 {
        kernel::ResetReason::WindowWatchdog
    } else if csr & (1 << 28) != 0 {
        kernel::ResetReason::Software
    } else if csr & (1 << 31) != 0 {
        kernel::ResetReason::LowPower
    } else if csr & (1 << 26) != 0 {
        kernel::ResetReason::PinReset
    } else if csr & (1 << 27) != 0 {
        kernel::ResetReason::PowerOn
    } else {
        kernel::ResetReason::Unknown
    }
}

fn clear_reset_flags() {
    unsafe {
        let csr = RCC_CSR.read_volatile();
        RCC_CSR.write_volatile(csr | (1 << 24));
    }
}

fn configure_board_peripherals() {
    const APB2_AFIOEN: u32 = 1 << 0;
    const APB2_IOPAEN: u32 = 1 << 2;
    const APB2_IOPBEN: u32 = 1 << 3;
    const APB2_IOPCEN: u32 = 1 << 4;
    const APB2_TIM1EN: u32 = 1 << 11;
    const APB2_USART1EN: u32 = 1 << 14;
    #[cfg(feature = "uart-probe")]
    const APB1_USART2EN: u32 = 1 << 17;
    #[cfg(feature = "uart-probe")]
    const APB1_USART3EN: u32 = 1 << 18;

    unsafe {
        let mut apb2 = RCC_APB2ENR.read_volatile();
        apb2 |= APB2_AFIOEN | APB2_IOPAEN | APB2_IOPBEN | APB2_IOPCEN | APB2_TIM1EN | APB2_USART1EN;
        RCC_APB2ENR.write_volatile(apb2);
        #[cfg(feature = "uart-probe")]
        {
            let mut apb1 = RCC_APB1ENR.read_volatile();
            apb1 |= APB1_USART2EN | APB1_USART3EN;
            RCC_APB1ENR.write_volatile(apb1);
        }

        // USART1 remap = 0 (PA9/PA10)
        let mut mapr = AFIO_MAPR.read_volatile();
        mapr &= !((1 << 2) | (1 << 3) | (0b11 << 4));
        AFIO_MAPR.write_volatile(mapr);

        #[cfg(feature = "uart-probe")]
        {
            // PA2  = alternate push-pull 50MHz (USART2_TX)
            // PA3  = floating input          (USART2_RX)
            let mut a_crl = GPIOA_CRL.read_volatile();
            a_crl &= !((0x0F << 8) | (0x0F << 12));
            a_crl |= (0x0B << 8) | (0x04 << 12);
            GPIOA_CRL.write_volatile(a_crl);
        }

        // PA8  = alternate push-pull 50MHz (TIM1 CH1)
        // PA9  = alternate push-pull 50MHz (USART1_TX)
        // PA10 = floating input          (USART1_RX)
        let mut a_crh = GPIOA_CRH.read_volatile();
        a_crh &= !((0x0F << 0) | (0x0F << 4) | (0x0F << 8));
        a_crh |= (0x0B << 0) | (0x0B << 4) | (0x04 << 8);
        GPIOA_CRH.write_volatile(a_crh);

        #[cfg(feature = "uart-probe")]
        {
            // PB10 = alternate push-pull 50MHz (USART3_TX)
            // PB11 = floating input          (USART3_RX)
            let mut b_crh = GPIOB_CRH.read_volatile();
            b_crh &= !((0x0F << 8) | (0x0F << 12));
            b_crh |= (0x0B << 8) | (0x04 << 12);
            GPIOB_CRH.write_volatile(b_crh);
        }

        // PC13 as push-pull output 2MHz; default off (high, active-low LED)
        let mut c_crh = GPIOC_CRH.read_volatile();
        c_crh &= !(0x0F << 20);
        c_crh |= 0x02 << 20;
        GPIOC_CRH.write_volatile(c_crh);
        GPIOC_BSRR.write_volatile(1 << 13);
    }
}

pub mod controls {
    use super::*;

    const GPIOA_CRL: *mut u32 = 0x4001_0800 as *mut u32;
    const GPIOA_ODR: *const u32 = 0x4001_080C as *const u32;
    const GPIOA_BSRR: *mut u32 = 0x4001_0810 as *mut u32;
    const GPIOA_BRR: *mut u32 = 0x4001_0814 as *mut u32;
    const GPIOC_ODR: *const u32 = 0x4001_100C as *const u32;
    const GPIOC_BSRR: *mut u32 = 0x4001_1010 as *mut u32;
    const GPIOC_BRR: *mut u32 = 0x4001_1014 as *mut u32;

    const TIM1_PSC: *mut u32 = 0x4001_2C28 as *mut u32;
    const TIM1_ARR: *mut u32 = 0x4001_2C2C as *mut u32;
    const TIM1_CCMR1: *mut u32 = 0x4001_2C18 as *mut u32;
    const TIM1_CCER: *mut u32 = 0x4001_2C20 as *mut u32;
    const TIM1_BDTR: *mut u32 = 0x4001_2C44 as *mut u32;
    const TIM1_EGR: *mut u32 = 0x4001_2C14 as *mut u32;
    const TIM1_CR1: *mut u32 = 0x4001_2C00 as *mut u32;
    const TIM1_CCR1: *mut u32 = 0x4001_2C34 as *mut u32;

    const LED_PIN: u32 = 13;
    // Some STM32F103RCT6 boards route user LED to PA1 (active-high).
    const LED_ALT_PIN: u32 = 1;
    const PWM_ARR: u32 = 999;
    const PWM_PSC: u32 = 7;

    static LED_READY: AtomicBool = AtomicBool::new(false);
    static PWM_READY: AtomicBool = AtomicBool::new(false);

    pub fn init_hardware() {
        free(|_| unsafe {
            // PA1 optional user LED (active-high on some boards)
            let mut a_crl = GPIOA_CRL.read_volatile();
            a_crl &= !(0x0F << (LED_ALT_PIN * 4));
            a_crl |= 0x02 << (LED_ALT_PIN * 4); // push-pull output 2MHz
            GPIOA_CRL.write_volatile(a_crl);
            GPIOA_BRR.write_volatile(1 << LED_ALT_PIN); // default off

            // TIM1 CH1 PWM mode 1, 1kHz base at 8MHz clock.
            TIM1_PSC.write_volatile(PWM_PSC);
            TIM1_ARR.write_volatile(PWM_ARR);
            TIM1_CCR1.write_volatile(0);
            TIM1_CCMR1.write_volatile((6 << 4) | (1 << 3)); // OC1M=110, OC1PE=1
            TIM1_CCER.write_volatile(1 << 0); // CC1E
            TIM1_BDTR.write_volatile(1 << 15); // MOE
            TIM1_EGR.write_volatile(1); // UG
            TIM1_CR1.write_volatile((1 << 7) | (1 << 0)); // ARPE + CEN

            GPIOC_BSRR.write_volatile(1 << LED_PIN); // active-low LED: off
        });

        LED_READY.store(true, Ordering::Release);
        PWM_READY.store(true, Ordering::Release);
    }

    pub fn led_available() -> bool {
        LED_READY.load(Ordering::Acquire)
    }

    pub fn set_led(on: bool) -> bool {
        if !led_available() {
            return false;
        }

        free(|_| unsafe {
            if on {
                // active-low LED on PC13
                GPIOC_BRR.write_volatile(1 << LED_PIN);
                // active-high LED on PA1 (board dependent)
                GPIOA_BSRR.write_volatile(1 << LED_ALT_PIN);
            } else {
                GPIOC_BSRR.write_volatile(1 << LED_PIN);
                GPIOA_BRR.write_volatile(1 << LED_ALT_PIN);
            }
        });
        true
    }

    pub fn toggle_led() -> bool {
        if !led_available() {
            return false;
        }

        free(|_| unsafe {
            let a_odr = GPIOA_ODR.read_volatile();
            let odr = GPIOC_ODR.read_volatile();
            if (odr & (1 << LED_PIN)) != 0 {
                GPIOC_BRR.write_volatile(1 << LED_PIN);
            } else {
                GPIOC_BSRR.write_volatile(1 << LED_PIN);
            }
            if (a_odr & (1 << LED_ALT_PIN)) != 0 {
                GPIOA_BRR.write_volatile(1 << LED_ALT_PIN);
            } else {
                GPIOA_BSRR.write_volatile(1 << LED_ALT_PIN);
            }
        });
        true
    }

    pub fn pwm_available() -> bool {
        PWM_READY.load(Ordering::Acquire)
    }

    pub fn set_pwm_percent(percent: u8) -> bool {
        if !pwm_available() {
            return false;
        }

        let clamped = percent.min(100) as u32;
        let duty = ((PWM_ARR + 1) * clamped) / 100;
        free(|_| unsafe {
            TIM1_CCR1.write_volatile(duty.min(PWM_ARR));
        });
        true
    }
}

pub mod uart {
    use super::*;

    const USART1_BASE: usize = 0x4001_3800;
    #[cfg(feature = "uart-probe")]
    const USART2_BASE: usize = 0x4000_4400;
    #[cfg(feature = "uart-probe")]
    const USART3_BASE: usize = 0x4000_4800;
    const USART_SR_OFFSET: usize = 0x00;
    const USART_DR_OFFSET: usize = 0x04;
    const USART_BRR_OFFSET: usize = 0x08;
    const USART_CR1_OFFSET: usize = 0x0C;
    const USART_CR2_OFFSET: usize = 0x10;
    const USART_CR3_OFFSET: usize = 0x14;

    const SR_PE: u32 = 1 << 0;
    const SR_FE: u32 = 1 << 1;
    const SR_NE: u32 = 1 << 2;
    const SR_ORE: u32 = 1 << 3;
    const SR_RXNE: u32 = 1 << 5;
    const SR_TXE: u32 = 1 << 7;
    const SR_TC: u32 = 1 << 6;

    const CR1_RE: u32 = 1 << 2;
    const CR1_TE: u32 = 1 << 3;
    const CR1_RXNEIE: u32 = 1 << 5;
    const CR1_UE: u32 = 1 << 13;

    // 8 MHz / 115200 -> BRR = 0x45
    const USART1_BRR_115200_AT_8MHZ: u32 = 0x45;

    #[cfg(feature = "uart-probe")]
    const ACTIVE_UART_BASES: &[usize] = &[USART1_BASE, USART2_BASE, USART3_BASE];
    #[cfg(not(feature = "uart-probe"))]
    const ACTIVE_UART_BASES: &[usize] = &[USART1_BASE];

    pub const RX_BUF_SIZE: usize = 256;
    pub const TX_BUF_SIZE: usize = 1024;

    static APP_UART_READY: AtomicBool = AtomicBool::new(false);
    static RX_RING: SyncRingBuf<RX_BUF_SIZE> = SyncRingBuf::new();
    static TX_RING: SyncRingBuf<TX_BUF_SIZE> = SyncRingBuf::new();
    static RX_EVENT: Event<1> = Event::new();
    static TX_EVENT: Event<1> = Event::new();

    static RX_BYTES: AtomicU32 = AtomicU32::new(0);
    static TX_BYTES: AtomicU32 = AtomicU32::new(0);
    static RX_OVERFLOWS: AtomicU32 = AtomicU32::new(0);
    static TX_OVERFLOWS: AtomicU32 = AtomicU32::new(0);
    static RX_ERRORS: AtomicU32 = AtomicU32::new(0);

    #[inline]
    fn sr(base: usize) -> *const u32 {
        (base + USART_SR_OFFSET) as *const u32
    }

    #[inline]
    fn dr(base: usize) -> *mut u32 {
        (base + USART_DR_OFFSET) as *mut u32
    }

    #[inline]
    fn brr(base: usize) -> *mut u32 {
        (base + USART_BRR_OFFSET) as *mut u32
    }

    #[inline]
    fn cr1(base: usize) -> *mut u32 {
        (base + USART_CR1_OFFSET) as *mut u32
    }

    #[inline]
    fn cr2(base: usize) -> *mut u32 {
        (base + USART_CR2_OFFSET) as *mut u32
    }

    #[inline]
    fn cr3(base: usize) -> *mut u32 {
        (base + USART_CR3_OFFSET) as *mut u32
    }

    pub fn init_hardware() {
        unsafe {
            for &base in ACTIVE_UART_BASES {
                cr2(base).write_volatile(0);
                cr3(base).write_volatile(0);
                brr(base).write_volatile(USART1_BRR_115200_AT_8MHZ);
                cr1(base).write_volatile(CR1_UE | CR1_TE | CR1_RE);
            }
        }
    }

    pub fn init_app_uart() {
        free(|_| {
            RX_RING.clear();
            TX_RING.clear();
            RX_EVENT.clear();
            TX_EVENT.clear();
        });

        RX_BYTES.store(0, Ordering::Relaxed);
        TX_BYTES.store(0, Ordering::Relaxed);
        RX_OVERFLOWS.store(0, Ordering::Relaxed);
        TX_OVERFLOWS.store(0, Ordering::Relaxed);
        RX_ERRORS.store(0, Ordering::Relaxed);

        unsafe {
            for &base in ACTIVE_UART_BASES {
                let bits = cr1(base).read_volatile();
                cr1(base).write_volatile(bits | CR1_RXNEIE);
            }
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);
            #[cfg(feature = "uart-probe")]
            {
                cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART2);
                cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART3);
            }
        }

        APP_UART_READY.store(true, Ordering::Release);
    }

    pub fn boot_write_bytes(bytes: &[u8]) {
        for &base in ACTIVE_UART_BASES {
            let mut base_ready = true;
            unsafe {
                for &byte in bytes {
                    let mut guard = 200_000u32;
                    while sr(base).read_volatile() & SR_TXE == 0 {
                        if guard == 0 {
                            base_ready = false;
                            break;
                        }
                        guard = guard.saturating_sub(1);
                    }
                    if !base_ready {
                        break;
                    }
                    dr(base).write_volatile(byte as u32);
                }
                if !base_ready {
                    continue;
                }

                let mut guard = 200_000u32;
                while sr(base).read_volatile() & SR_TC == 0 {
                    if guard == 0 {
                        break;
                    }
                    guard = guard.saturating_sub(1);
                }
            }
        }
    }

    pub fn app_is_ready() -> bool {
        APP_UART_READY.load(Ordering::Acquire)
    }

    pub fn app_wait_for_rx(timeout_ms: Option<u32>) -> Result<(), EventError> {
        RX_EVENT.wait(timeout_ms)
    }

    pub fn app_clear_rx_event() {
        RX_EVENT.clear();
    }

    pub fn app_read_byte() -> Option<u8> {
        RX_RING.pop()
    }

    pub fn app_wait_for_tx(timeout_ms: Option<u32>) -> Result<(), EventError> {
        TX_EVENT.wait(timeout_ms)
    }

    pub fn app_clear_tx_event() {
        TX_EVENT.clear();
    }

    pub fn app_enqueue_tx_bytes(bytes: &[u8]) -> usize {
        let written = TX_RING.push_slice(bytes);
        let dropped = bytes.len().saturating_sub(written);
        if dropped > 0 {
            TX_OVERFLOWS.fetch_add(dropped as u32, Ordering::Relaxed);
        }
        if written > 0 {
            let _ = TX_EVENT.set();
        }
        written
    }

    pub fn app_drain_tx() -> usize {
        let mut buf = [0u8; 32];
        let mut total = 0usize;

        loop {
            let count = TX_RING.pop_slice(&mut buf);
            if count == 0 {
                return total;
            }

            boot_write_bytes(&buf[..count]);
            total += count;
            TX_BYTES.fetch_add(count as u32, Ordering::Relaxed);
        }
    }

    pub fn app_stats() -> UartStats {
        UartStats {
            rx_bytes: RX_BYTES.load(Ordering::Relaxed),
            tx_bytes: TX_BYTES.load(Ordering::Relaxed),
            rx_overflows: RX_OVERFLOWS.load(Ordering::Relaxed),
            tx_overflows: TX_OVERFLOWS.load(Ordering::Relaxed),
            rx_errors: RX_ERRORS.load(Ordering::Relaxed),
            rx_pending: RX_RING.len(),
            tx_pending: TX_RING.len(),
        }
    }

    fn handle_uart_irq(base: usize) {
        let mut received = false;

        loop {
            let status = unsafe { sr(base).read_volatile() };
            let has_error = status & (SR_PE | SR_FE | SR_NE | SR_ORE) != 0;
            if status & SR_RXNE == 0 {
                // Clear sticky error flags even if no pending payload remains.
                if has_error {
                    let _ = unsafe { dr(base).read_volatile() };
                    RX_ERRORS.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                break;
            }

            let byte = unsafe { dr(base).read_volatile() as u8 };
            if has_error {
                RX_ERRORS.fetch_add(1, Ordering::Relaxed);
                continue;
            }

            if RX_RING.push_from_isr(byte).is_ok() {
                RX_BYTES.fetch_add(1, Ordering::Relaxed);
                received = true;
            } else {
                RX_OVERFLOWS.fetch_add(1, Ordering::Relaxed);
            }
        }

        if received {
            let _ = RX_EVENT.set();
        }
    }

    #[interrupt]
    fn USART1() {
        handle_uart_irq(USART1_BASE);
    }

    #[interrupt]
    fn USART2() {
        #[cfg(feature = "uart-probe")]
        handle_uart_irq(USART2_BASE);
    }

    #[interrupt]
    fn USART3() {
        #[cfg(feature = "uart-probe")]
        handle_uart_irq(USART3_BASE);
    }
}

pub mod watchdog {
    use super::*;

    const IWDG_KR: *mut u32 = 0x4000_3000 as *mut u32;
    const IWDG_PR: *mut u32 = 0x4000_3004 as *mut u32;
    const IWDG_RLR: *mut u32 = 0x4000_3008 as *mut u32;
    const IWDG_SR: *const u32 = 0x4000_300C as *const u32;

    static STARTED: AtomicBool = AtomicBool::new(false);

    pub fn reset_state() {
        STARTED.store(false, Ordering::Relaxed);
    }

    pub fn start(timeout_ms: u32) -> bool {
        let reload = timeout_ms_to_reload(timeout_ms);

        free(|_| unsafe {
            IWDG_KR.write_volatile(0x5555);
            IWDG_PR.write_volatile(4); // /64
            IWDG_RLR.write_volatile(reload);

            let mut guard = 10_000u32;
            while (IWDG_SR.read_volatile() & 0x03) != 0 && guard > 0 {
                guard = guard.saturating_sub(1);
            }

            IWDG_KR.write_volatile(0xAAAA);
            IWDG_KR.write_volatile(0xCCCC);
        });

        STARTED.store(true, Ordering::Release);
        true
    }

    pub fn feed() -> bool {
        free(|_| unsafe {
            IWDG_KR.write_volatile(0xAAAA);
        });
        STARTED.load(Ordering::Acquire)
    }

    fn timeout_ms_to_reload(timeout_ms: u32) -> u32 {
        // LSI ~40kHz, prescaler /64 => 625Hz tick (~1.6ms)
        // reload = timeout * 625 / 1000 - 1
        let ticks = timeout_ms.saturating_mul(625).saturating_div(1000);
        ticks.saturating_sub(1).min(0x0FFF)
    }
}
