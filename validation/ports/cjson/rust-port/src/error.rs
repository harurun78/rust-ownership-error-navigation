#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedChar { ch: char, pos: usize },
    InvalidLiteral { pos: usize },
    InvalidNumber { pos: usize },
    InvalidString { pos: usize },
    InvalidEscape { pos: usize },
    InvalidUnicodeEscape { pos: usize },
    TrailingCharacters { pos: usize },
    Unsupported { feature: &'static str, pos: usize },
}
