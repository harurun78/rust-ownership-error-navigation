//! Minimal RESP2 command parser used for Redis porting validation.
//!
//! This crate intentionally covers only request-frame parsing. It does not
//! implement Redis networking, command execution, replies, or server state.

mod command;
mod error;
mod parser;

pub use command::Command;
pub use error::RespError;
pub use parser::{ParseOutcome, RespCommandParser};
