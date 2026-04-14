#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedCommand {
    On,
    Off,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParsedCommand<'a> {
    Ping,
    Echo(&'a str),
    Led(LedCommand),
    Pwm(u8),
    Stat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    Unknown,
    InvalidLed,
    InvalidPwm,
}

pub const MAX_LINE_LEN: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompleteLine<const N: usize> {
    len: usize,
    bytes: [u8; N],
}

impl<const N: usize> CompleteLine<N> {
    pub const fn new(bytes: [u8; N], len: usize) -> Self {
        Self { len, bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineAssemblerEvent<const N: usize> {
    None,
    Line(CompleteLine<N>),
    Dropped,
}

pub struct LineAssembler<const N: usize> {
    bytes: [u8; N],
    len: usize,
    dropping: bool,
}

impl<const N: usize> LineAssembler<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
            dropping: false,
        }
    }

    pub fn push_byte(&mut self, byte: u8) -> LineAssemblerEvent<N> {
        match byte {
            b'\r' => LineAssemblerEvent::None,
            b'\n' => {
                if self.dropping {
                    self.dropping = false;
                    self.len = 0;
                    return LineAssemblerEvent::None;
                }

                if self.len == 0 {
                    return LineAssemblerEvent::None;
                }

                let line = CompleteLine::new(self.bytes, self.len);
                self.len = 0;
                LineAssemblerEvent::Line(line)
            }
            _ => {
                if self.dropping {
                    return LineAssemblerEvent::None;
                }

                if self.len < N {
                    self.bytes[self.len] = byte;
                    self.len += 1;
                    LineAssemblerEvent::None
                } else {
                    self.len = 0;
                    self.dropping = true;
                    LineAssemblerEvent::Dropped
                }
            }
        }
    }
}

pub fn parse_command(text: &str) -> Result<ParsedCommand<'_>, ParseError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut parts = text.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();

    if cmd.eq_ignore_ascii_case("PING") {
        return Ok(ParsedCommand::Ping);
    }

    if cmd.eq_ignore_ascii_case("ECHO") {
        return Ok(ParsedCommand::Echo(arg));
    }

    if cmd.eq_ignore_ascii_case("LED") {
        if arg.eq_ignore_ascii_case("ON") {
            return Ok(ParsedCommand::Led(LedCommand::On));
        }
        if arg.eq_ignore_ascii_case("OFF") {
            return Ok(ParsedCommand::Led(LedCommand::Off));
        }
        if arg.eq_ignore_ascii_case("TOGGLE") {
            return Ok(ParsedCommand::Led(LedCommand::Toggle));
        }
        return Err(ParseError::InvalidLed);
    }

    if cmd.eq_ignore_ascii_case("PWM") {
        let Ok(percent) = arg.parse::<u8>() else {
            return Err(ParseError::InvalidPwm);
        };
        if percent > 100 {
            return Err(ParseError::InvalidPwm);
        }
        return Ok(ParsedCommand::Pwm(percent));
    }

    if cmd.eq_ignore_ascii_case("STAT") {
        return Ok(ParsedCommand::Stat);
    }

    Err(ParseError::Unknown)
}
