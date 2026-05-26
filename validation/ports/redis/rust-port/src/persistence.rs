use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum AofFsyncPolicy {
    Always,
    EverySec,
    NoFsync,
}

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Corrupt(String),
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::Io(e)
    }
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "io error: {}", e),
            PersistenceError::Corrupt(s) => write!(f, "corrupt: {}", s),
        }
    }
}

impl std::error::Error for PersistenceError {}

// Small placeholder: this module only exposes types used by the executor
// persistence methods implemented inside the executor module. See
// executor::RedisMiniDb::{save_snapshot,load_snapshot,append_aof,replay_aof}.
