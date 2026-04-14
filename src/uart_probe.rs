//! Standalone UART probe mode for STM32F103-class boards.
//!
//! This module intentionally avoids RTOS/kernel modules so startup `.bss` clear stays tiny.
//! Goal: quickly verify "firmware is running + UART link + LED control" on board.

#[cfg(feature = "board-f103c8-bluepill")]
use stm32f1::stm32f103 as _;

const RCC_BASE: usize = 0x4002_1000;
const RCC_APB2ENR: *mut u32 = (RCC_BASE + 0x18) as *mut u32;
const RCC_APB1ENR: *mut u32 = (RCC_BASE + 0x1C) as *mut u32;
const AFIO_MAPR: *mut u32 = 0x4001_0004 as *mut u32;

const GPIOA_CRL: *mut u32 = 0x4001_0800 as *mut u32;
const GPIOA_CRH: *mut u32 = 0x4001_0804 as *mut u32;
const GPIOA_BSRR: *mut u32 = 0x4001_0810 as *mut u32;
const GPIOA_BRR: *mut u32 = 0x4001_0814 as *mut u32;

const GPIOB_CRH: *mut u32 = 0x4001_0C04 as *mut u32;

const GPIOC_CRH: *mut u32 = 0x4001_1004 as *mut u32;
const GPIOC_BSRR: *mut u32 = 0x4001_1010 as *mut u32;
const GPIOC_BRR: *mut u32 = 0x4001_1014 as *mut u32;

const USART1_BASE: usize = 0x4001_3800;
const USART2_BASE: usize = 0x4000_4400;
const USART3_BASE: usize = 0x4000_4800;

const USART_SR_OFFSET: usize = 0x00;
const USART_DR_OFFSET: usize = 0x04;
const USART_BRR_OFFSET: usize = 0x08;
const USART_CR1_OFFSET: usize = 0x0C;
const USART_CR2_OFFSET: usize = 0x10;
const USART_CR3_OFFSET: usize = 0x14;

const SR_RXNE: u32 = 1 << 5;
const SR_TXE: u32 = 1 << 7;
const SR_TC: u32 = 1 << 6;

const CR1_RE: u32 = 1 << 2;
const CR1_TE: u32 = 1 << 3;
const CR1_UE: u32 = 1 << 13;

const LED_PC13: u32 = 13; // active-low on many boards
const LED_PA1: u32 = 1; // active-high on some boards

// 8 MHz / 115200 => 0x45
const BRR_115200_AT_8MHZ: u32 = 0x45;

#[inline]
fn usart_sr(base: usize) -> *const u32 {
    (base + USART_SR_OFFSET) as *const u32
}

#[inline]
fn usart_dr(base: usize) -> *mut u32 {
    (base + USART_DR_OFFSET) as *mut u32
}

#[inline]
fn usart_brr(base: usize) -> *mut u32 {
    (base + USART_BRR_OFFSET) as *mut u32
}

#[inline]
fn usart_cr1(base: usize) -> *mut u32 {
    (base + USART_CR1_OFFSET) as *mut u32
}

#[inline]
fn usart_cr2(base: usize) -> *mut u32 {
    (base + USART_CR2_OFFSET) as *mut u32
}

#[inline]
fn usart_cr3(base: usize) -> *mut u32 {
    (base + USART_CR3_OFFSET) as *mut u32
}

#[inline]
fn write_uart_all(bytes: &[u8]) {
    for base in [USART1_BASE, USART2_BASE, USART3_BASE] {
        unsafe {
            for &b in bytes {
                let mut guard = 200_000u32;
                while usart_sr(base).read_volatile() & SR_TXE == 0 {
                    if guard == 0 {
                        break;
                    }
                    guard = guard.saturating_sub(1);
                }
                if guard == 0 {
                    break;
                }
                usart_dr(base).write_volatile(b as u32);
            }

            let mut guard = 200_000u32;
            while usart_sr(base).read_volatile() & SR_TC == 0 {
                if guard == 0 {
                    break;
                }
                guard = guard.saturating_sub(1);
            }
        }
    }
}

#[inline]
fn read_uart(base: usize) -> Option<u8> {
    unsafe {
        if usart_sr(base).read_volatile() & SR_RXNE == 0 {
            None
        } else {
            Some(usart_dr(base).read_volatile() as u8)
        }
    }
}

#[inline]
fn toggle_led() {
    static mut PC13_ON: bool = false;
    static mut PA1_ON: bool = false;

    unsafe {
        PC13_ON = !PC13_ON;
        if PC13_ON {
            GPIOC_BRR.write_volatile(1 << LED_PC13);
        } else {
            GPIOC_BSRR.write_volatile(1 << LED_PC13);
        }

        PA1_ON = !PA1_ON;
        if PA1_ON {
            GPIOA_BSRR.write_volatile(1 << LED_PA1);
        } else {
            GPIOA_BRR.write_volatile(1 << LED_PA1);
        }
    }
}

fn init_board() {
    const APB2_AFIOEN: u32 = 1 << 0;
    const APB2_IOPAEN: u32 = 1 << 2;
    const APB2_IOPBEN: u32 = 1 << 3;
    const APB2_IOPCEN: u32 = 1 << 4;
    const APB2_USART1EN: u32 = 1 << 14;
    const APB1_USART2EN: u32 = 1 << 17;
    const APB1_USART3EN: u32 = 1 << 18;

    unsafe {
        let mut apb2 = RCC_APB2ENR.read_volatile();
        apb2 |= APB2_AFIOEN | APB2_IOPAEN | APB2_IOPBEN | APB2_IOPCEN | APB2_USART1EN;
        RCC_APB2ENR.write_volatile(apb2);

        let mut apb1 = RCC_APB1ENR.read_volatile();
        apb1 |= APB1_USART2EN | APB1_USART3EN;
        RCC_APB1ENR.write_volatile(apb1);

        // Keep default remap:
        // USART1 -> PA9/PA10
        // USART2 -> PA2/PA3
        // USART3 -> PB10/PB11
        let mut mapr = AFIO_MAPR.read_volatile();
        mapr &= !((1 << 2) | (1 << 3) | (0b11 << 4));
        AFIO_MAPR.write_volatile(mapr);

        // PA2 TX(AF PP), PA3 RX(input floating), PA1 LED(output PP)
        let mut a_crl = GPIOA_CRL.read_volatile();
        a_crl &= !((0x0F << 8) | (0x0F << 12) | (0x0F << 4));
        a_crl |= (0x0B << 8) | (0x04 << 12) | (0x02 << 4);
        GPIOA_CRL.write_volatile(a_crl);

        // PA9 TX(AF PP), PA10 RX(input floating)
        let mut a_crh = GPIOA_CRH.read_volatile();
        a_crh &= !((0x0F << 4) | (0x0F << 8));
        a_crh |= (0x0B << 4) | (0x04 << 8);
        GPIOA_CRH.write_volatile(a_crh);

        // PB10 TX(AF PP), PB11 RX(input floating)
        let mut b_crh = GPIOB_CRH.read_volatile();
        b_crh &= !((0x0F << 8) | (0x0F << 12));
        b_crh |= (0x0B << 8) | (0x04 << 12);
        GPIOB_CRH.write_volatile(b_crh);

        // PC13 LED output PP 2MHz
        let mut c_crh = GPIOC_CRH.read_volatile();
        c_crh &= !(0x0F << 20);
        c_crh |= 0x02 << 20;
        GPIOC_CRH.write_volatile(c_crh);

        // default LEDs off
        GPIOC_BSRR.write_volatile(1 << LED_PC13);
        GPIOA_BRR.write_volatile(1 << LED_PA1);
    }
}

fn init_uart(base: usize) {
    unsafe {
        usart_cr2(base).write_volatile(0);
        usart_cr3(base).write_volatile(0);
        usart_brr(base).write_volatile(BRR_115200_AT_8MHZ);
        usart_cr1(base).write_volatile(CR1_UE | CR1_TE | CR1_RE);
    }
}

pub fn run() -> ! {
    init_board();
    init_uart(USART1_BASE);
    init_uart(USART2_BASE);
    init_uart(USART3_BASE);

    write_uart_all(b"boot ok (F103)\r\n");
    write_uart_all(b"uart probe mode ready\r\n");
    write_uart_all(b"routes: USART1(PA9/PA10), USART2(PA2/PA3), USART3(PB10/PB11)\r\n");
    write_uart_all(b"send any line, board will echo with prefix 'rx:'\r\n");

    let mut line_buf = [0u8; 128];
    let mut line_len = 0usize;
    // Keep probe feedback frequent so host can attach serial after boot and still
    // observe liveness quickly.
    let mut hb = 0u32;
    let mut blink = 0u32;

    loop {
        for base in [USART1_BASE, USART2_BASE, USART3_BASE] {
            while let Some(byte) = read_uart(base) {
                match byte {
                    b'\r' | b'\n' => {
                        if line_len != 0 {
                            write_uart_all(b"rx: ");
                            write_uart_all(&line_buf[..line_len]);
                            write_uart_all(b"\r\n");
                            line_len = 0;
                        }
                    }
                    _ => {
                        if line_len < line_buf.len() {
                            line_buf[line_len] = byte;
                            line_len += 1;
                        } else {
                            write_uart_all(b"rx: [line overflow]\r\n");
                            line_len = 0;
                        }
                    }
                }
            }
        }

        hb = hb.wrapping_add(1);
        if hb >= 200_000 {
            hb = 0;
            write_uart_all(b"uart probe heartbeat\r\n");
        }

        blink = blink.wrapping_add(1);
        if blink >= 100_000 {
            blink = 0;
            toggle_led();
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn __cortexos_switch_context(saved_sp: *mut u32) -> *mut u32 {
    // Probe mode never enters scheduler/PendSV; provide a link-time stub for shared asm objects.
    saved_sp
}
