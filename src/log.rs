use core::fmt::Write;

const LOGGER_BUF_SIZE: usize = 256;

pub(crate) struct Logger {
    len: usize,
    buf: [u8; LOGGER_BUF_SIZE],
}

impl Logger {
    fn new() -> Self {
        Self {
            len: 0,
            buf: [0; LOGGER_BUF_SIZE],
        }
    }

    fn flush(&mut self) -> core::fmt::Result {
        if self.len == 0 {
            return Ok(());
        }

        if crate::device::uart::log_bytes(&self.buf[..self.len]) == self.len {
            self.len = 0;
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            if self.len == self.buf.len() {
                self.flush()?;
            }

            self.buf[self.len] = byte;
            self.len += 1;

            if byte == b'\n' {
                self.flush()?;
            }
        }

        Ok(())
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

pub fn init() {}

pub fn with_logger<R>(f: impl FnOnce(&mut Logger) -> R) -> Option<R> {
    let mut logger = Logger::new();
    let result = f(&mut logger);
    let _ = logger.flush();
    Some(result)
}

pub fn log_line(s: &str) {
    with_logger(|tx| {
        let _ = writeln!(tx, "{s}");
    });
}

pub fn emergency_write_str(s: &str) {
    crate::platform::uart::boot_write_bytes(s.as_bytes());
}

struct EmergencyWriter;

impl Write for EmergencyWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        emergency_write_str(s);
        Ok(())
    }
}

pub fn emergency_write_fmt(args: core::fmt::Arguments<'_>) {
    let mut writer = EmergencyWriter;
    let _ = writer.write_fmt(args);
}

pub fn emergency_log_line(s: &str) {
    emergency_write_str(s);
    emergency_write_str("\r\n");
}
