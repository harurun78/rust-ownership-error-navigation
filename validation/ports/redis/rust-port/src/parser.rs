use crate::{Command, RespError};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    Complete(Command),
    Incomplete,
}

#[derive(Debug, Default)]
pub struct RespCommandParser {
    buffer: Vec<u8>,
}

impl RespCommandParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn append(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn parse_available(&mut self) -> Result<ParseOutcome, RespError> {
        let (command, consumed) = match parse_multibulk(&self.buffer)? {
            ParsedFrame::Complete { command, consumed } => (command, consumed),
            ParsedFrame::Incomplete => return Ok(ParseOutcome::Incomplete),
        };

        self.buffer.drain(..consumed);
        Ok(ParseOutcome::Complete(command))
    }
}

enum ParsedFrame {
    Complete { command: Command, consumed: usize },
    Incomplete,
}

fn parse_multibulk(buffer: &[u8]) -> Result<ParsedFrame, RespError> {
    if buffer.is_empty() {
        return Ok(ParsedFrame::Incomplete);
    }

    if buffer[0] != b'*' {
        return Err(RespError::ExpectedArray);
    }

    let header_end = match find_crlf(buffer, 1) {
        Some(index) => index,
        None => return Ok(ParsedFrame::Incomplete),
    };
    let arg_count =
        parse_positive_usize(&buffer[1..header_end]).map_err(|_| RespError::InvalidArrayLength)?;
    if arg_count == 0 {
        return Err(RespError::InvalidArrayLength);
    }

    let mut cursor = header_end + 2;
    let mut args = Vec::with_capacity(arg_count);

    for _ in 0..arg_count {
        if cursor >= buffer.len() {
            return Ok(ParsedFrame::Incomplete);
        }
        if buffer[cursor] != b'$' {
            return Err(RespError::ExpectedBulkString);
        }

        let length_end = match find_crlf(buffer, cursor + 1) {
            Some(index) => index,
            None => return Ok(ParsedFrame::Incomplete),
        };
        let bulk_len = parse_usize(&buffer[cursor + 1..length_end])
            .map_err(|_| RespError::InvalidBulkLength)?;
        let data_start = length_end + 2;
        let data_end = data_start + bulk_len;
        let frame_end = data_end + 2;

        if frame_end > buffer.len() {
            return Ok(ParsedFrame::Incomplete);
        }
        if &buffer[data_end..frame_end] != b"\r\n" {
            return Err(RespError::InvalidBulkTerminator);
        }

        args.push(buffer[data_start..data_end].to_vec());
        cursor = frame_end;
    }

    Ok(ParsedFrame::Complete {
        command: Command::new(args),
        consumed: cursor,
    })
}

fn find_crlf(buffer: &[u8], start: usize) -> Option<usize> {
    buffer
        .get(start..)?
        .windows(2)
        .position(|window| window == b"\r\n")
        .map(|offset| start + offset)
}

fn parse_positive_usize(bytes: &[u8]) -> Result<usize, ()> {
    let value = parse_usize(bytes)?;
    if value == 0 {
        return Err(());
    }
    Ok(value)
}

fn parse_usize(bytes: &[u8]) -> Result<usize, ()> {
    if bytes.is_empty() {
        return Err(());
    }

    let mut value = 0usize;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return Err(());
        }
        let digit = usize::from(byte - b'0');
        value = value.checked_mul(10).ok_or(())?;
        value = value.checked_add(digit).ok_or(())?;
    }
    Ok(value)
}
