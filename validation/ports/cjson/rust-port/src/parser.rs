use crate::error::ParseError;
use crate::value::JsonValue;

const MAX_RECURSION_DEPTH: usize = 128;

pub fn parse_scalar(input: &str) -> Result<JsonValue, ParseError> {
    let mut parser = Parser::new(input);
    parser.skip_entry_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();

    if parser.is_eof() {
        Ok(value)
    } else {
        Err(ParseError::TrailingCharacters { pos: parser.pos })
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, ParseError> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(ParseError::RecursionLimit { pos: self.pos });
        }

        match self.peek_char() {
            Some('n') => self.parse_literal("null", JsonValue::Null),
            Some('t') => self.parse_literal("true", JsonValue::Bool(true)),
            Some('f') => self.parse_literal("false", JsonValue::Bool(false)),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('-') | Some('0'..='9') => self.parse_number().map(JsonValue::Number),
            Some('[') => self.parse_array(depth),
            Some('{') => self.parse_object(depth),
            Some(ch) => Err(ParseError::UnexpectedChar { ch, pos: self.pos }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, ParseError> {
        self.consume_char();
        self.skip_whitespace();

        let mut values = Vec::new();
        if self.peek_char() == Some(']') {
            self.consume_char();
            return Ok(JsonValue::Array(values));
        }

        loop {
            values.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume_char();
                    return Ok(JsonValue::Array(values));
                }
                Some(ch) => return Err(ParseError::UnexpectedChar { ch, pos: self.pos }),
                None => return Err(ParseError::UnexpectedEof),
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, ParseError> {
        self.consume_char();
        self.skip_whitespace();

        let mut entries = Vec::new();
        if self.peek_char() == Some('}') {
            self.consume_char();
            return Ok(JsonValue::Object(entries));
        }

        loop {
            let key = match self.peek_char() {
                Some('"') => self.parse_string()?,
                Some(ch) => return Err(ParseError::UnexpectedChar { ch, pos: self.pos }),
                None => return Err(ParseError::UnexpectedEof),
            };

            self.skip_whitespace();
            match self.peek_char() {
                Some(':') => {
                    self.consume_char();
                    self.skip_whitespace();
                }
                Some(ch) => return Err(ParseError::UnexpectedChar { ch, pos: self.pos }),
                None => return Err(ParseError::UnexpectedEof),
            }

            let value = self.parse_value(depth + 1)?;
            entries.push((key, value));
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume_char();
                    return Ok(JsonValue::Object(entries));
                }
                Some(ch) => return Err(ParseError::UnexpectedChar { ch, pos: self.pos }),
                None => return Err(ParseError::UnexpectedEof),
            }
        }
    }

    fn parse_literal(
        &mut self,
        literal: &'static str,
        value: JsonValue,
    ) -> Result<JsonValue, ParseError> {
        if self.input[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(value)
        } else {
            Err(ParseError::InvalidLiteral { pos: self.pos })
        }
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.pos;

        if self.peek_char() == Some('-') {
            self.consume_char();
        }

        match self.peek_char() {
            Some('0') => {
                self.consume_char();
                if matches!(self.peek_char(), Some('0'..='9')) {
                    return Err(ParseError::InvalidNumber { pos: self.pos });
                }
            }
            Some('1'..='9') => {
                self.consume_digits();
            }
            _ => return Err(ParseError::InvalidNumber { pos: start }),
        }

        if self.peek_char() == Some('.') {
            self.consume_char();
            if !matches!(self.peek_char(), Some('0'..='9')) {
                return Err(ParseError::InvalidNumber { pos: self.pos });
            }
            self.consume_digits();
        }

        if matches!(self.peek_char(), Some('e') | Some('E')) {
            self.consume_char();
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                self.consume_char();
            }
            if !matches!(self.peek_char(), Some('0'..='9')) {
                return Err(ParseError::InvalidNumber { pos: self.pos });
            }
            self.consume_digits();
        }

        self.input[start..self.pos]
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber { pos: start })
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        if self.consume_char() != Some('"') {
            return Err(ParseError::InvalidString { pos: start });
        }

        let mut output = String::new();
        while let Some(ch) = self.consume_char() {
            match ch {
                '"' => return Ok(output),
                '\\' => output.push(self.parse_escape()?),
                ch if ch <= '\u{001f}' => {
                    return Err(ParseError::InvalidString { pos: self.pos });
                }
                ch => output.push(ch),
            }
        }

        Err(ParseError::UnexpectedEof)
    }

    fn parse_escape(&mut self) -> Result<char, ParseError> {
        let escape_pos = self.pos;
        match self.consume_char() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\u{0008}'),
            Some('f') => Ok('\u{000c}'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => self.parse_unicode_escape(escape_pos),
            Some(_) => Err(ParseError::InvalidEscape { pos: escape_pos }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn parse_unicode_escape(&mut self, escape_pos: usize) -> Result<char, ParseError> {
        let first = self.parse_hex4(escape_pos)?;

        if (0xd800..=0xdbff).contains(&first) {
            if self.consume_char() != Some('\\') || self.consume_char() != Some('u') {
                return Err(ParseError::InvalidUnicodeEscape { pos: escape_pos });
            }
            let second = self.parse_hex4(escape_pos)?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(ParseError::InvalidUnicodeEscape { pos: escape_pos });
            }
            let codepoint =
                0x10000 + (((first - 0xd800) as u32) << 10) + ((second - 0xdc00) as u32);
            char::from_u32(codepoint).ok_or(ParseError::InvalidUnicodeEscape { pos: escape_pos })
        } else if (0xdc00..=0xdfff).contains(&first) {
            Err(ParseError::InvalidUnicodeEscape { pos: escape_pos })
        } else {
            char::from_u32(first as u32).ok_or(ParseError::InvalidUnicodeEscape { pos: escape_pos })
        }
    }

    fn parse_hex4(&mut self, escape_pos: usize) -> Result<u16, ParseError> {
        let mut value = 0u16;
        for _ in 0..4 {
            let ch = self
                .consume_char()
                .ok_or(ParseError::InvalidUnicodeEscape { pos: escape_pos })?;
            let digit = ch
                .to_digit(16)
                .ok_or(ParseError::InvalidUnicodeEscape { pos: escape_pos })?;
            value = (value << 4) | digit as u16;
        }
        Ok(value)
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek_char(), Some('0'..='9')) {
            self.consume_char();
        }
    }

    fn skip_entry_whitespace(&mut self) {
        if self.input.starts_with('\u{feff}') {
            self.pos = '\u{feff}'.len_utf8();
        }
        self.skip_whitespace();
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(' ' | '\n' | '\r' | '\t')) {
            self.consume_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn consume_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}
