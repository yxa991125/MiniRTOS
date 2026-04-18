use core::fmt::Write;

use cortex_m::interrupt::Mutex;
#[cfg(feature = "bench")]
use cortex_m::peripheral::{DCB, DWT};
use stm32f4xx_hal::{
    gpio::{Alternate, GpioExt, Output, PushPull, gpioa::{PA2, PA3, PA5}},
    pac,
    prelude::*,
    rcc::{Config as RccConfig, RccExt},
    serial::{Serial, config::Config as UartConfig},
    timer::PwmExt,
};
#[cfg(feature = "bench")]
use stm32f4xx_hal::rcc::Rcc;

#[cfg(not(feature = "bench"))]
use stm32f4xx_hal::timer::PwmChannel as HalPwmChannel;

use crate::{device::{gpio::GpioOutput, pwm::PwmChannel}, kernel};

pub struct BoardContext {
    reset_reason: kernel::ResetReason,
    sysclk_hz: u32,
    #[cfg(feature = "bench")]
    tim2: Option<pac::TIM2>,
    #[cfg(feature = "bench")]
    rcc: Option<Rcc>,
}

impl BoardContext {
    pub fn take() -> Option<Self> {
        let dp = pac::Peripherals::take()?;
        let reset_reason = detect_reset_reason(&dp.RCC);
        clear_reset_flags(&dp.RCC);

        let mut rcc = dp.RCC.freeze(
            RccConfig::hsi()
                .sysclk(84.MHz())
                .pclk1(42.MHz())
                .pclk2(84.MHz()),
        );
        let sysclk_hz = rcc.clocks.sysclk().raw();

        let gpioa = dp.GPIOA.split(&mut rcc);
        let tx_pin: PA2<Alternate<7>> = gpioa.pa2.into_alternate::<7>();
        let rx_pin: PA3<Alternate<7>> = gpioa.pa3.into_alternate::<7>();

        let _serial = Serial::<_, u8>::new(
            dp.USART2,
            (tx_pin, rx_pin),
            UartConfig::default().baudrate(115_200.bps()),
            &mut rcc,
        )
        .unwrap();
        uart::init_app_uart();

        #[cfg(not(feature = "bench"))]
        {
            controls::register_led(gpioa.pa5.into_push_pull_output());
            let (_, (ch1, ..)) = dp.TIM1.pwm_us(100.micros(), &mut rcc);
            let pwm = PwmChannel::new(ch1.with(gpioa.pa8));
            controls::install_pwm(pwm);
            watchdog::register(dp.IWDG);
            let _ = dp.TIM2;
        }

        Some(Self {
            reset_reason,
            sysclk_hz,
            #[cfg(feature = "bench")]
            tim2: Some(dp.TIM2),
            #[cfg(feature = "bench")]
            rcc: Some(rcc),
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

        let _ = writeln!(tx, "boot ok (F411)");
        let _ = writeln!(tx, "reset={}", self.reset_reason.as_str());
        let _ = writeln!(tx, "MSP=0x{:08x} PSP=0x{:08x} VTOR=0x{:08x}", msp_v, psp_v, vtor);
        let _ = writeln!(tx, "VEC PendSV=0x{:08x} SysTick=0x{:08x}", pendsv_vec, systick_vec);
        let _ = writeln!(tx, "cpu={}Hz", self.sysclk_hz);
    }

    #[cfg(feature = "bench")]
    pub fn init_bench(&mut self, dcb: &mut DCB, dwt: &mut DWT) {
        let tim2 = self.tim2.take().expect("F411 TIM2 already taken");
        let rcc = self.rcc.as_mut().expect("F411 RCC unavailable");
        crate::bench::init(dcb, dwt, tim2, rcc, self.sysclk_hz);
    }
}

struct BootWriter;

impl Write for BootWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::platform::uart::boot_write_bytes(s.as_bytes());
        Ok(())
    }
}

fn detect_reset_reason(rcc: &pac::RCC) -> kernel::ResetReason {
    let csr = rcc.csr().read();

    if csr.wdgrstf().is_reset() {
        kernel::ResetReason::IndependentWatchdog
    } else if csr.wwdgrstf().is_reset() {
        kernel::ResetReason::WindowWatchdog
    } else if csr.sftrstf().is_reset() {
        kernel::ResetReason::Software
    } else if csr.lpwrrstf().is_reset() {
        kernel::ResetReason::LowPower
    } else if csr.padrstf().is_reset() {
        kernel::ResetReason::PinReset
    } else if csr.borrstf().is_reset() && !csr.porrstf().is_reset() {
        kernel::ResetReason::Brownout
    } else if csr.porrstf().is_reset() || csr.borrstf().is_reset() {
        kernel::ResetReason::PowerOn
    } else {
        kernel::ResetReason::Unknown
    }
}

fn clear_reset_flags(rcc: &pac::RCC) {
    rcc.csr().modify(|_, w| w.rmvf().clear());
}

pub mod controls {
    use super::*;
    use crate::sync::mutex::IrqMutex;

    #[cfg(not(feature = "bench"))]
    type LedPin = PA5<Output<PushPull>>;
    #[cfg(not(feature = "bench"))]
    type PwmPin = HalPwmChannel<pac::TIM1, 0>;

    #[cfg(not(feature = "bench"))]
    static LED: IrqMutex<Option<GpioOutput<LedPin>>> = IrqMutex::new(None);
    #[cfg(not(feature = "bench"))]
    static PWM: IrqMutex<Option<PwmChannel<PwmPin>>> = IrqMutex::new(None);

    #[cfg(not(feature = "bench"))]
    pub fn register_led(pin: LedPin) {
        let mut led = GpioOutput::new(pin);
        let _ = led.set_low();
        LED.lock(|slot| *slot = Some(led));
    }

    #[cfg(not(feature = "bench"))]
    pub fn install_pwm(mut pwm: PwmChannel<PwmPin>) {
        let _ = pwm.set_duty_percent(0);
        pwm.inner_mut().enable();
        PWM.lock(|slot| *slot = Some(pwm));
    }

    #[cfg(not(feature = "bench"))]
    pub fn led_available() -> bool {
        LED.lock(|slot| slot.is_some())
    }

    #[cfg(feature = "bench")]
    pub fn led_available() -> bool {
        false
    }

    #[cfg(not(feature = "bench"))]
    pub fn set_led(on: bool) -> bool {
        LED.lock(|slot| {
            let Some(led) = slot.as_mut() else {
                return false;
            };
            let _ = if on { led.set_high() } else { led.set_low() };
            true
        })
    }

    #[cfg(feature = "bench")]
    pub fn set_led(_on: bool) -> bool {
        false
    }

    #[cfg(not(feature = "bench"))]
    pub fn toggle_led() -> bool {
        LED.lock(|slot| {
            let Some(led) = slot.as_mut() else {
                return false;
            };
            let _ = led.toggle();
            true
        })
    }

    #[cfg(feature = "bench")]
    pub fn toggle_led() -> bool {
        false
    }

    #[cfg(not(feature = "bench"))]
    pub fn pwm_available() -> bool {
        PWM.lock(|slot| slot.is_some())
    }

    #[cfg(feature = "bench")]
    pub fn pwm_available() -> bool {
        false
    }

    #[cfg(not(feature = "bench"))]
    pub fn set_pwm_percent(percent: u8) -> bool {
        PWM.lock(|slot| {
            let Some(pwm) = slot.as_mut() else {
                return false;
            };
            let _ = pwm.set_duty_percent(percent);
            true
        })
    }

    #[cfg(feature = "bench")]
    pub fn set_pwm_percent(_percent: u8) -> bool {
        false
    }
}

pub mod uart {
    use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    use cortex_m::interrupt::free;
    use stm32f4xx_hal::pac::interrupt;

    use crate::ipc::ringbuf::SyncRingBuf;
    use crate::platform::uart::UartStats;
    use crate::sync::event::{Event, EventError};

    const USART2_BASE: usize = 0x4000_4400;
    const USART_SR_OFFSET: usize = 0x00;
    const USART_DR_OFFSET: usize = 0x04;
    const USART_CR1_OFFSET: usize = 0x0C;

    const SR_PE: u32 = 1 << 0;
    const SR_FE: u32 = 1 << 1;
    const SR_NE: u32 = 1 << 2;
    const SR_ORE: u32 = 1 << 3;
    const SR_RXNE: u32 = 1 << 5;
    const SR_TXE: u32 = 1 << 7;
    const SR_TC: u32 = 1 << 6;
    const CR1_RXNEIE: u32 = 1 << 5;

    pub const RX_BUF_SIZE: usize = 256;
    pub const TX_BUF_SIZE: usize = 1024;
    const TX_DRAIN_BURST_BYTES: usize = 64;

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
    fn sr() -> *const u32 {
        (USART2_BASE + USART_SR_OFFSET) as *const u32
    }

    #[inline]
    fn dr() -> *mut u32 {
        (USART2_BASE + USART_DR_OFFSET) as *mut u32
    }

    #[inline]
    fn cr1() -> *mut u32 {
        (USART2_BASE + USART_CR1_OFFSET) as *mut u32
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
            let bits = cr1().read_volatile();
            cr1().write_volatile(bits | CR1_RXNEIE);
            cortex_m::peripheral::NVIC::unmask(stm32f4xx_hal::pac::Interrupt::USART2);
        }

        APP_UART_READY.store(true, Ordering::Release);
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
            if total >= TX_DRAIN_BURST_BYTES {
                break;
            }

            let remaining = TX_DRAIN_BURST_BYTES - total;
            let chunk = remaining.min(buf.len());
            let count = TX_RING.pop_slice(&mut buf[..chunk]);
            if count == 0 {
                break;
            }

            boot_write_bytes(&buf[..count]);
            total += count;
            TX_BYTES.fetch_add(count as u32, Ordering::Relaxed);
        }

        if TX_RING.len() > 0 {
            let _ = TX_EVENT.set();
        }

        total
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

    pub fn boot_write_bytes(bytes: &[u8]) {
        unsafe {
            for &byte in bytes {
                while sr().read_volatile() & SR_TXE == 0 {}
                dr().write_volatile(byte as u32);
            }

            while sr().read_volatile() & SR_TC == 0 {}
        }
    }

    #[interrupt]
    fn USART2() {
        let mut received = false;

        loop {
            let status = unsafe { sr().read_volatile() };
            let has_error = status & (SR_PE | SR_FE | SR_NE | SR_ORE) != 0;
            if status & SR_RXNE == 0 {
                // Clear sticky error flags even when no payload is pending.
                if has_error {
                    let _ = unsafe { dr().read_volatile() };
                    RX_ERRORS.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                break;
            }

            let byte = unsafe { dr().read_volatile() as u8 };
            if has_error {
                RX_ERRORS.fetch_add(1, Ordering::Relaxed);
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
}

pub mod watchdog {
    use super::*;

    use core::cell::RefCell;
    use cortex_m::interrupt::free;
    use fugit::MillisDurationU32 as MilliSeconds;
    use stm32f4xx_hal::watchdog::IndependentWatchdog;

    static WATCHDOG: Mutex<RefCell<Option<IndependentWatchdog>>> = Mutex::new(RefCell::new(None));

    pub fn register(iwdg: pac::IWDG) {
        let watchdog = IndependentWatchdog::new(iwdg);
        free(|cs| {
            *WATCHDOG.borrow(cs).borrow_mut() = Some(watchdog);
        });
    }

    pub fn start(timeout_ms: u32) -> bool {
        free(|cs| {
            let mut watchdog = WATCHDOG.borrow(cs).borrow_mut();
            let Some(watchdog) = watchdog.as_mut() else {
                return false;
            };
            watchdog.start(MilliSeconds::from_ticks(timeout_ms));
            true
        })
    }

    pub fn feed() -> bool {
        free(|cs| {
            let mut watchdog = WATCHDOG.borrow(cs).borrow_mut();
            let Some(watchdog) = watchdog.as_mut() else {
                return false;
            };
            watchdog.feed();
            true
        })
    }
}
