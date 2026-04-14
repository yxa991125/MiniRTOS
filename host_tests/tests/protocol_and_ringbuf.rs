#![allow(dead_code)]

#[path = "../../src/app_protocol.rs"]
mod app_protocol;

#[path = "../../src/ipc/ringbuf_core.rs"]
mod ringbuf_core;

use app_protocol::{parse_command, LedCommand, LineAssembler, LineAssemblerEvent, ParsedCommand};
use ringbuf_core::{RingBuf, RingBufError};

#[test]
fn parse_basic_commands() {
    assert_eq!(parse_command("PING"), Ok(ParsedCommand::Ping));
    assert_eq!(parse_command("ECHO soak"), Ok(ParsedCommand::Echo("soak")));
    assert_eq!(
        parse_command("LED toggle"),
        Ok(ParsedCommand::Led(LedCommand::Toggle))
    );
    assert_eq!(parse_command("PWM 100"), Ok(ParsedCommand::Pwm(100)));
    assert_eq!(parse_command("STAT"), Ok(ParsedCommand::Stat));
}

#[test]
fn reject_invalid_commands() {
    assert!(parse_command("").is_err());
    assert!(parse_command("LED blink").is_err());
    assert!(parse_command("PWM 101").is_err());
    assert!(parse_command("PWM nope").is_err());
    assert!(parse_command("UNKNOWN").is_err());
}

#[test]
fn line_assembler_handles_complete_partial_and_sticky_lines() {
    let mut assembler = LineAssembler::<8>::new();

    for byte in b"PING\r" {
        assert!(matches!(assembler.push_byte(*byte), LineAssemblerEvent::None));
    }
    assert!(matches!(assembler.push_byte(b'\n'), LineAssemblerEvent::Line(_)));

    let mut outputs = [0u8; 2];
    let mut count = 0usize;
    for byte in b"ECHO\r\nSTAT\r\n" {
        if let LineAssemblerEvent::Line(line) = assembler.push_byte(*byte) {
            outputs[count] = line.as_bytes().len() as u8;
            count += 1;
        }
    }

    assert_eq!(count, 2);
    assert_eq!(outputs, [4, 4]);
}

#[test]
fn line_assembler_recovers_after_overflow() {
    let mut assembler = LineAssembler::<4>::new();

    for byte in b"HELLO" {
        if *byte == b'O' {
            assert_eq!(assembler.push_byte(*byte), LineAssemblerEvent::Dropped);
        } else {
            assert_eq!(assembler.push_byte(*byte), LineAssemblerEvent::None);
        }
    }

    assert_eq!(assembler.push_byte(b'\n'), LineAssemblerEvent::None);

    for byte in b"OK\r\n" {
        if let LineAssemblerEvent::Line(line) = assembler.push_byte(*byte) {
            assert_eq!(line.as_bytes(), b"OK");
            return;
        }
    }

    panic!("expected recovered line");
}

#[test]
fn ringbuf_push_pop_wrap_and_full() {
    let mut buf = RingBuf::<4>::new();
    assert_eq!(buf.capacity(), 4);
    assert!(buf.is_empty());

    assert_eq!(buf.push_slice(b"abc"), 3);
    assert_eq!(buf.len(), 3);
    assert_eq!(buf.pop(), Some(b'a'));
    assert_eq!(buf.push_slice(b"de"), 2);
    assert!(buf.is_full());
    assert_eq!(buf.push(b'f'), Err(RingBufError::Full));

    let mut out = [0u8; 4];
    let popped = buf.pop_slice(&mut out);
    assert_eq!(popped, 4);
    assert_eq!(&out[..popped], b"bcde");
    assert!(buf.is_empty());
}
