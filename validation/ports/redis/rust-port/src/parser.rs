use crate::{Command, RespError};

const DEFAULT_MAX_LINE_LENGTH: usize = 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    Complete(Command),
    Incomplete,
}

#[derive(Debug)]
pub struct RespCommandParser {
    buffer: Vec<u8>,
    max_line_length: usize,
}

impl Default for RespCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RespCommandParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            max_line_length: DEFAULT_MAX_LINE_LENGTH,
        }
    }

    pub fn with_max_line_length(max_line_length: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_line_length,
        }
    }

    pub fn append(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn parse_available(&mut self) -> Result<ParseOutcome, RespError> {
        let (command, consumed) = match parse_multibulk(&self.buffer, self.max_line_length)? {
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

fn parse_multibulk(buffer: &[u8], max_line_length: usize) -> Result<ParsedFrame, RespError> {
    if buffer.is_empty() {
        return Ok(ParsedFrame::Incomplete);
    }

    let header_end = match find_crlf_or_line_too_long(buffer, 0, max_line_length)? {
        Some(index) => index,
        None => return Ok(ParsedFrame::Incomplete),
    };

    if buffer[0] != b'*' {
        return Err(RespError::ExpectedArray);
    }

    let arg_count = parse_positive_usize(&buffer[1..header_end])
        .map_err(|_| RespError::InvalidMultibulkLength)?;
    if arg_count == 0 {
        return Err(RespError::InvalidMultibulkLength);
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

        let length_end = match find_crlf_or_line_too_long(buffer, cursor, max_line_length)? {
            Some(index) => index,
            None => return Ok(ParsedFrame::Incomplete),
        };
        let bulk_len = parse_usize(&buffer[cursor + 1..length_end])
            .map_err(|_| RespError::InvalidBulkLength)?;
        let data_start = length_end + 2;
        let data_end = data_start
            .checked_add(bulk_len)
            .ok_or(RespError::InvalidBulkLength)?;
        let frame_end = data_end
            .checked_add(2)
            .ok_or(RespError::InvalidBulkLength)?;

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

fn find_crlf_or_line_too_long(
    buffer: &[u8],
    start: usize,
    max_line_length: usize,
) -> Result<Option<usize>, RespError> {
    let line_end = find_crlf(buffer, start);
    let available_line_len = match line_end {
        Some(index) => index.saturating_sub(start),
        None => buffer.len().saturating_sub(start),
    };

    if available_line_len > max_line_length {
        return Err(RespError::LineTooLong);
    }

    Ok(line_end)
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
