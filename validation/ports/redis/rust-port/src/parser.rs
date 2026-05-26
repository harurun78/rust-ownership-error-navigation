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

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn parse_available(&mut self) -> Result<ParseOutcome, RespError> {
        let command = match parse_request(&self.buffer, self.max_line_length)? {
            ParsedFrame::Complete { command, consumed } => {
                self.buffer.drain(..consumed);
                command
            }
            ParsedFrame::CompleteMultibulk {
                bulk_ranges,
                consumed,
            } => Command::new(extract_owned_bulk_args(
                &mut self.buffer,
                bulk_ranges,
                consumed,
            )),
            ParsedFrame::Incomplete => return Ok(ParseOutcome::Incomplete),
        };

        Ok(ParseOutcome::Complete(command))
    }
}

#[derive(Debug)]
struct BulkRange {
    start: usize,
    end: usize,
}

enum ParsedFrame {
    Complete {
        command: Command,
        consumed: usize,
    },
    CompleteMultibulk {
        bulk_ranges: Vec<BulkRange>,
        consumed: usize,
    },
    Incomplete,
}

fn extract_owned_bulk_args(
    buffer: &mut Vec<u8>,
    bulk_ranges: Vec<BulkRange>,
    consumed: usize,
) -> Vec<Vec<u8>> {
    let remaining = buffer.split_off(consumed);
    let mut frame = std::mem::replace(buffer, remaining);
    let mut args = Vec::with_capacity(bulk_ranges.len());

    for range in bulk_ranges.into_iter().rev() {
        let _suffix = frame.split_off(range.end);
        args.push(frame.split_off(range.start));
    }

    args.reverse();
    args
}

fn parse_request(buffer: &[u8], max_line_length: usize) -> Result<ParsedFrame, RespError> {
    if buffer.first() == Some(&b'*') {
        parse_multibulk(buffer, max_line_length)
    } else {
        parse_inline(buffer, max_line_length)
    }
}

fn parse_inline(buffer: &[u8], max_line_length: usize) -> Result<ParsedFrame, RespError> {
    if buffer.is_empty() {
        return Ok(ParsedFrame::Incomplete);
    }

    let line_end = match find_crlf_or_line_too_long(buffer, 0, max_line_length)? {
        Some(index) => index,
        None => return Ok(ParsedFrame::Incomplete),
    };

    let args = split_inline_args(&buffer[..line_end])?;
    Ok(ParsedFrame::Complete {
        command: Command::new(args),
        consumed: line_end + 2,
    })
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
    let mut bulk_ranges = Vec::with_capacity(arg_count);

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

        bulk_ranges.push(BulkRange {
            start: data_start,
            end: data_end,
        });
        cursor = frame_end;
    }

    Ok(ParsedFrame::CompleteMultibulk {
        bulk_ranges,
        consumed: cursor,
    })
}

fn split_inline_args(line: &[u8]) -> Result<Vec<Vec<u8>>, RespError> {
    let mut args = Vec::new();
    let mut current = Vec::new();
    let mut index = 0;
    let mut quote = None;
    let mut token_started = false;

    while index < line.len() {
        let byte = line[index];

        if let Some(quote_byte) = quote {
            if byte == quote_byte {
                quote = None;
            } else if quote_byte == b'"' && byte == b'\\' {
                index += 1;
                if index >= line.len() {
                    return Err(RespError::UnbalancedQuote);
                }
                current.push(unescape_inline_byte(line[index]));
            } else {
                current.push(byte);
            }
        } else if byte == b' ' || byte == b'\t' {
            if token_started {
                args.push(current);
                current = Vec::new();
                token_started = false;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
            token_started = true;
        } else {
            current.push(byte);
            token_started = true;
        }

        index += 1;
    }

    if quote.is_some() {
        return Err(RespError::UnbalancedQuote);
    }
    if token_started {
        args.push(current);
    }

    Ok(args)
}

fn unescape_inline_byte(byte: u8) -> u8 {
    match byte {
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        b'b' => 8,
        b'a' => 7,
        other => other,
    }
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
