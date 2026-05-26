#[derive(Debug, PartialEq, Eq)]
pub enum RespError {
    ExpectedArray,
    InvalidMultibulkLength,
    ExpectedBulkString,
    InvalidBulkLength,
    InvalidBulkTerminator,
    LineTooLong,
}
