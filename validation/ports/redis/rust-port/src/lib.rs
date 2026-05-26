//! Minimal Redis-compatible components used for Redis porting validation.
//!
//! This crate started as a RESP2 request parser and now also includes an
//! in-memory command executor, reply encoding, command metadata, transactions,
//! expiration, scans, streams, and other incremental slices used to measure
//! ownership diagnostics during a Rust Redis port.

mod command;
mod error;
mod executor;
mod parser;

pub use command::Command;
pub use error::RespError;
pub use executor::{
    CommandCategory, CommandMetadata, RedisMiniDb, RedisMiniSession, RespProtocolVersion,
    RespReply, command_metadata, normalize_command_name,
};
pub use parser::{ParseOutcome, RespCommandParser};
