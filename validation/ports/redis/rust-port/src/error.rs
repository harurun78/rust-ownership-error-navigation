#[derive(Debug, PartialEq, Eq)]
pub enum RespError {
    ExpectedArray,
    InvalidArrayLength,
    ExpectedBulkString,
    InvalidBulkLength,
    InvalidBulkTerminator,
}
