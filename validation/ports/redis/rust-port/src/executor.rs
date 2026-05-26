use std::collections::HashMap;

use crate::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum RespReply {
    SimpleString(&'static str),
    BulkString(Vec<u8>),
    NullBulkString,
    Integer(i64),
    Error(String),
}

impl RespReply {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::SimpleString(value) => encode_prefixed_string(b'+', value.as_bytes()),
            Self::BulkString(value) => encode_bulk_string(value),
            Self::NullBulkString => b"$-1\r\n".to_vec(),
            Self::Integer(value) => format!(":{value}\r\n").into_bytes(),
            Self::Error(message) => encode_error(message),
        }
    }
}

#[derive(Debug, Default)]
pub struct RedisMiniDb {
    strings: HashMap<Vec<u8>, Vec<u8>>,
}

impl RedisMiniDb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(&mut self, command: Command) -> RespReply {
        let mut args = command.args;
        if args.is_empty() {
            return RespReply::Error("ERR unknown command ''".to_string());
        }

        let command_name = args.remove(0);

        if command_name.eq_ignore_ascii_case(b"PING") {
            self.execute_ping(args)
        } else if command_name.eq_ignore_ascii_case(b"ECHO") {
            execute_echo(args)
        } else if command_name.eq_ignore_ascii_case(b"SET") {
            self.execute_set(args)
        } else if command_name.eq_ignore_ascii_case(b"GET") {
            self.execute_get(args)
        } else if command_name.eq_ignore_ascii_case(b"DEL") {
            self.execute_del(args)
        } else if command_name.eq_ignore_ascii_case(b"EXISTS") {
            self.execute_exists(args)
        } else if command_name.eq_ignore_ascii_case(b"INCR") {
            self.execute_incr_by(args, 1, "incr")
        } else if command_name.eq_ignore_ascii_case(b"DECR") {
            self.execute_incr_by(args, -1, "decr")
        } else if command_name.eq_ignore_ascii_case(b"INCRBY") {
            self.execute_incrby(args)
        } else {
            RespReply::Error(format!(
                "ERR unknown command '{}'",
                String::from_utf8_lossy(&command_name)
            ))
        }
    }

    fn execute_ping(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        match args.len() {
            0 => RespReply::SimpleString("PONG"),
            1 => {
                let mut args = args;
                RespReply::BulkString(args.remove(0))
            }
            _ => wrong_arity("ping"),
        }
    }

    fn execute_set(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("set");
        }

        let mut args = args;
        let key = args.remove(0);
        let value = args.remove(0);
        self.strings.insert(key, value);
        RespReply::SimpleString("OK")
    }

    fn execute_get(&self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("get");
        }

        match self.strings.get(&args[0]) {
            Some(value) => RespReply::BulkString(value.to_vec()),
            None => RespReply::NullBulkString,
        }
    }

    fn execute_del(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.is_empty() {
            return wrong_arity("del");
        }

        let mut deleted = 0i64;
        for key in args {
            if self.strings.remove(&key).is_some() {
                deleted += 1;
            }
        }
        RespReply::Integer(deleted)
    }

    fn execute_exists(&self, args: Vec<Vec<u8>>) -> RespReply {
        if args.is_empty() {
            return wrong_arity("exists");
        }

        let mut count = 0i64;
        for key in args {
            if self.strings.contains_key(&key) {
                count += 1;
            }
        }
        RespReply::Integer(count)
    }

    fn execute_incrby(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("incrby");
        }

        let mut args = args;
        let key = args.remove(0);
        let delta = match parse_integer(&args[0]) {
            Some(delta) => delta,
            None => return integer_error(),
        };

        self.increment_key(key, delta)
    }

    fn execute_incr_by(&mut self, args: Vec<Vec<u8>>, delta: i64, command_name: &str) -> RespReply {
        if args.len() != 1 {
            return wrong_arity(command_name);
        }

        let mut args = args;
        let key = args.remove(0);
        self.increment_key(key, delta)
    }

    fn increment_key(&mut self, key: Vec<u8>, delta: i64) -> RespReply {
        let current = match self.strings.get(&key) {
            Some(value) => match parse_integer(value) {
                Some(value) => value,
                None => return integer_error(),
            },
            None => 0,
        };

        match current.checked_add(delta) {
            Some(next) => {
                self.strings.insert(key, next.to_string().into_bytes());
                RespReply::Integer(next)
            }
            None => RespReply::Error("ERR increment or decrement would overflow".to_string()),
        }
    }
}

fn execute_echo(args: Vec<Vec<u8>>) -> RespReply {
    if args.len() != 1 {
        return wrong_arity("echo");
    }

    let mut args = args;
    RespReply::BulkString(args.remove(0))
}

fn wrong_arity(command_name: &str) -> RespReply {
    RespReply::Error(format!(
        "ERR wrong number of arguments for '{}' command",
        command_name
    ))
}

fn parse_integer(value: &[u8]) -> Option<i64> {
    std::str::from_utf8(value).ok()?.parse::<i64>().ok()
}

fn integer_error() -> RespReply {
    RespReply::Error("ERR value is not an integer or out of range".to_string())
}

fn encode_prefixed_string(prefix: u8, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(value.len() + 3);
    encoded.push(prefix);
    encoded.extend_from_slice(value);
    encoded.extend_from_slice(b"\r\n");
    encoded
}

fn encode_bulk_string(value: &[u8]) -> Vec<u8> {
    let mut encoded = format!("${}\r\n", value.len()).into_bytes();
    encoded.extend_from_slice(value);
    encoded.extend_from_slice(b"\r\n");
    encoded
}

fn encode_error(message: &str) -> Vec<u8> {
    encode_prefixed_string(b'-', message.as_bytes())
}
