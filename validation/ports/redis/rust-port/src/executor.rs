use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::{Duration, Instant};

use crate::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum RespReply {
    SimpleString(&'static str),
    BulkString(Vec<u8>),
    NullBulkString,
    Integer(i64),
    Array(Vec<RespReply>),
    Error(String),
}

impl RespReply {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::SimpleString(value) => encode_prefixed_string(b'+', value.as_bytes()),
            Self::BulkString(value) => encode_bulk_string(value),
            Self::NullBulkString => b"$-1\r\n".to_vec(),
            Self::Integer(value) => format!(":{value}\r\n").into_bytes(),
            Self::Array(values) => encode_array(values),
            Self::Error(message) => encode_error(message),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RedisValue {
    String(Vec<u8>),
    List(Vec<Vec<u8>>),
    Hash(BTreeMap<Vec<u8>, Vec<u8>>),
    Set(BTreeSet<Vec<u8>>),
}

#[derive(Debug, Default)]
pub struct RedisMiniDb {
    values: HashMap<Vec<u8>, RedisValue>,
    expires_at: HashMap<Vec<u8>, Instant>,
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
        } else if command_name.eq_ignore_ascii_case(b"EXPIRE") {
            self.execute_expire(args)
        } else if command_name.eq_ignore_ascii_case(b"TTL") {
            self.execute_ttl(args)
        } else if command_name.eq_ignore_ascii_case(b"PERSIST") {
            self.execute_persist(args)
        } else if command_name.eq_ignore_ascii_case(b"INCR") {
            self.execute_incr_by(args, 1, "incr")
        } else if command_name.eq_ignore_ascii_case(b"DECR") {
            self.execute_incr_by(args, -1, "decr")
        } else if command_name.eq_ignore_ascii_case(b"INCRBY") {
            self.execute_incrby(args)
        } else if command_name.eq_ignore_ascii_case(b"LPUSH") {
            self.execute_push(args, ListSide::Left)
        } else if command_name.eq_ignore_ascii_case(b"RPUSH") {
            self.execute_push(args, ListSide::Right)
        } else if command_name.eq_ignore_ascii_case(b"LPOP") {
            self.execute_pop(args, ListSide::Left)
        } else if command_name.eq_ignore_ascii_case(b"RPOP") {
            self.execute_pop(args, ListSide::Right)
        } else if command_name.eq_ignore_ascii_case(b"LRANGE") {
            self.execute_lrange(args)
        } else if command_name.eq_ignore_ascii_case(b"HSET") {
            self.execute_hset(args)
        } else if command_name.eq_ignore_ascii_case(b"HGET") {
            self.execute_hget(args)
        } else if command_name.eq_ignore_ascii_case(b"HDEL") {
            self.execute_hdel(args)
        } else if command_name.eq_ignore_ascii_case(b"HGETALL") {
            self.execute_hgetall(args)
        } else if command_name.eq_ignore_ascii_case(b"SADD") {
            self.execute_sadd(args)
        } else if command_name.eq_ignore_ascii_case(b"SREM") {
            self.execute_srem(args)
        } else if command_name.eq_ignore_ascii_case(b"SISMEMBER") {
            self.execute_sismember(args)
        } else if command_name.eq_ignore_ascii_case(b"SMEMBERS") {
            self.execute_smembers(args)
        } else if command_name.eq_ignore_ascii_case(b"SUNIONSTORE") {
            self.execute_set_store(args, SetStoreOp::Union)
        } else if command_name.eq_ignore_ascii_case(b"SINTERSTORE") {
            self.execute_set_store(args, SetStoreOp::Intersection)
        } else if command_name.eq_ignore_ascii_case(b"SDIFFSTORE") {
            self.execute_set_store(args, SetStoreOp::Difference)
        } else if command_name.eq_ignore_ascii_case(b"TYPE") {
            self.execute_type(args)
        } else if command_name.eq_ignore_ascii_case(b"RENAME") {
            self.execute_rename(args)
        } else if command_name.eq_ignore_ascii_case(b"RENAMENX") {
            self.execute_renamenx(args)
        } else if command_name.eq_ignore_ascii_case(b"KEYS") {
            self.execute_keys(args)
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
        self.remove_if_expired(&key);
        if matches!(
            self.values.get(&key),
            Some(RedisValue::List(_)) | Some(RedisValue::Hash(_)) | Some(RedisValue::Set(_))
        ) {
            return wrong_type();
        }

        self.expires_at.remove(&key);
        self.values.insert(key, RedisValue::String(value));
        RespReply::SimpleString("OK")
    }

    fn execute_get(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("get");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(value)) => RespReply::BulkString(value.to_vec()),
            Some(RedisValue::List(_)) | Some(RedisValue::Hash(_)) | Some(RedisValue::Set(_)) => {
                wrong_type()
            }
            None => RespReply::NullBulkString,
        }
    }

    fn execute_del(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.is_empty() {
            return wrong_arity("del");
        }

        let mut deleted = 0i64;
        for key in args {
            self.remove_if_expired(&key);
            if self.values.remove(&key).is_some() {
                deleted += 1;
            }
            self.expires_at.remove(&key);
        }
        RespReply::Integer(deleted)
    }

    fn execute_exists(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.is_empty() {
            return wrong_arity("exists");
        }

        let mut count = 0i64;
        for key in args {
            self.remove_if_expired(&key);
            if self.values.contains_key(&key) {
                count += 1;
            }
        }
        RespReply::Integer(count)
    }

    fn execute_expire(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("expire");
        }

        let mut args = args;
        let key = args.remove(0);
        self.remove_if_expired(&key);
        if !self.values.contains_key(&key) {
            return RespReply::Integer(0);
        }

        let seconds = match parse_integer(&args[0]) {
            Some(seconds) => seconds,
            None => return integer_error(),
        };
        if seconds <= 0 {
            self.values.remove(&key);
            self.expires_at.remove(&key);
            return RespReply::Integer(1);
        }

        match Instant::now().checked_add(Duration::from_secs(seconds as u64)) {
            Some(deadline) => {
                self.expires_at.insert(key, deadline);
                RespReply::Integer(1)
            }
            None => RespReply::Error("ERR invalid expire time".to_string()),
        }
    }

    fn execute_ttl(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("ttl");
        }

        self.remove_if_expired(&args[0]);
        if !self.values.contains_key(&args[0]) {
            return RespReply::Integer(-2);
        }

        match self.expires_at.get(&args[0]) {
            Some(deadline) => {
                let ttl = deadline.saturating_duration_since(Instant::now()).as_secs() as i64;
                RespReply::Integer(ttl)
            }
            None => RespReply::Integer(-1),
        }
    }

    fn execute_persist(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("persist");
        }

        self.remove_if_expired(&args[0]);
        if !self.values.contains_key(&args[0]) {
            return RespReply::Integer(0);
        }

        if self.expires_at.remove(&args[0]).is_some() {
            RespReply::Integer(1)
        } else {
            RespReply::Integer(0)
        }
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
        self.remove_if_expired(&key);
        let current = match self.values.get(&key) {
            Some(RedisValue::String(value)) => match parse_integer(value) {
                Some(value) => value,
                None => return integer_error(),
            },
            Some(RedisValue::List(_)) | Some(RedisValue::Hash(_)) | Some(RedisValue::Set(_)) => {
                return wrong_type();
            }
            None => 0,
        };

        match current.checked_add(delta) {
            Some(next) => {
                self.expires_at.remove(&key);
                self.values
                    .insert(key, RedisValue::String(next.to_string().into_bytes()));
                RespReply::Integer(next)
            }
            None => RespReply::Error("ERR increment or decrement would overflow".to_string()),
        }
    }

    fn execute_push(&mut self, args: Vec<Vec<u8>>, side: ListSide) -> RespReply {
        if args.len() < 2 {
            return wrong_arity(side.push_command_name());
        }

        let mut args = args;
        let key = args.remove(0);
        self.remove_if_expired(&key);
        match self.values.get_mut(&key) {
            Some(RedisValue::String(_)) | Some(RedisValue::Hash(_)) | Some(RedisValue::Set(_)) => {
                wrong_type()
            }
            Some(RedisValue::List(list)) => {
                for value in args {
                    match side {
                        ListSide::Left => list.insert(0, value),
                        ListSide::Right => list.push(value),
                    }
                }
                self.expires_at.remove(&key);
                RespReply::Integer(list.len() as i64)
            }
            None => {
                let mut list = Vec::new();
                for value in args {
                    match side {
                        ListSide::Left => list.insert(0, value),
                        ListSide::Right => list.push(value),
                    }
                }
                let len = list.len() as i64;
                self.values.insert(key, RedisValue::List(list));
                RespReply::Integer(len)
            }
        }
    }

    fn execute_pop(&mut self, args: Vec<Vec<u8>>, side: ListSide) -> RespReply {
        if args.len() != 1 {
            return wrong_arity(side.pop_command_name());
        }

        self.remove_if_expired(&args[0]);
        match self.values.get_mut(&args[0]) {
            Some(RedisValue::String(_)) | Some(RedisValue::Hash(_)) | Some(RedisValue::Set(_)) => {
                wrong_type()
            }
            Some(RedisValue::List(list)) => match side {
                ListSide::Left => {
                    if list.is_empty() {
                        RespReply::NullBulkString
                    } else {
                        RespReply::BulkString(list.remove(0))
                    }
                }
                ListSide::Right => match list.pop() {
                    Some(value) => RespReply::BulkString(value),
                    None => RespReply::NullBulkString,
                },
            },
            None => RespReply::NullBulkString,
        }
    }

    fn execute_lrange(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("lrange");
        }

        let start = match parse_integer(&args[1]) {
            Some(value) => value,
            None => return integer_error(),
        };
        let stop = match parse_integer(&args[2]) {
            Some(value) => value,
            None => return integer_error(),
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_)) | Some(RedisValue::Hash(_)) | Some(RedisValue::Set(_)) => {
                wrong_type()
            }
            Some(RedisValue::List(list)) => match normalize_range(list.len(), start, stop) {
                Some((start, stop)) => RespReply::Array(
                    list[start..=stop]
                        .iter()
                        .map(|value| RespReply::BulkString(value.to_vec()))
                        .collect(),
                ),
                None => RespReply::Array(Vec::new()),
            },
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_hset(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 3 || args.len() % 2 == 0 {
            return wrong_arity("hset");
        }

        let mut args = args;
        let key = args.remove(0);
        self.remove_if_expired(&key);
        match self.values.get_mut(&key) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Set(_)) => {
                wrong_type()
            }
            Some(RedisValue::Hash(hash)) => {
                let mut added = 0i64;
                while !args.is_empty() {
                    let field = args.remove(0);
                    let value = args.remove(0);
                    if !hash.contains_key(&field) {
                        added += 1;
                    }
                    hash.insert(field, value);
                }
                self.expires_at.remove(&key);
                RespReply::Integer(added)
            }
            None => {
                let mut hash = BTreeMap::new();
                let mut added = 0i64;
                while !args.is_empty() {
                    let field = args.remove(0);
                    let value = args.remove(0);
                    if !hash.contains_key(&field) {
                        added += 1;
                    }
                    hash.insert(field, value);
                }
                self.values.insert(key, RedisValue::Hash(hash));
                RespReply::Integer(added)
            }
        }
    }

    fn execute_hget(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("hget");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Set(_)) => {
                wrong_type()
            }
            Some(RedisValue::Hash(hash)) => match hash.get(&args[1]) {
                Some(value) => RespReply::BulkString(value.to_vec()),
                None => RespReply::NullBulkString,
            },
            None => RespReply::NullBulkString,
        }
    }

    fn execute_hdel(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("hdel");
        }

        let key = &args[0];
        self.remove_if_expired(key);
        let mut remove_key = false;
        let removed = match self.values.get_mut(key) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Set(_)) => {
                return wrong_type();
            }
            Some(RedisValue::Hash(hash)) => {
                let mut removed = 0i64;
                for field in &args[1..] {
                    if hash.remove(field).is_some() {
                        removed += 1;
                    }
                }
                remove_key = hash.is_empty();
                removed
            }
            None => 0,
        };

        if remove_key {
            self.values.remove(key);
            self.expires_at.remove(key);
        }

        RespReply::Integer(removed)
    }

    fn execute_hgetall(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("hgetall");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Set(_)) => {
                wrong_type()
            }
            Some(RedisValue::Hash(hash)) => {
                let mut values = Vec::with_capacity(hash.len() * 2);
                for (field, value) in hash {
                    values.push(RespReply::BulkString(field.to_vec()));
                    values.push(RespReply::BulkString(value.to_vec()));
                }
                RespReply::Array(values)
            }
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_sadd(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("sadd");
        }

        let mut args = args;
        let key = args.remove(0);
        self.remove_if_expired(&key);
        match self.values.get_mut(&key) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Hash(_)) => {
                wrong_type()
            }
            Some(RedisValue::Set(set)) => {
                let mut added = 0i64;
                for member in args {
                    if set.insert(member) {
                        added += 1;
                    }
                }
                self.expires_at.remove(&key);
                RespReply::Integer(added)
            }
            None => {
                let mut set = BTreeSet::new();
                let mut added = 0i64;
                for member in args {
                    if set.insert(member) {
                        added += 1;
                    }
                }
                self.values.insert(key, RedisValue::Set(set));
                RespReply::Integer(added)
            }
        }
    }

    fn execute_srem(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("srem");
        }

        let key = &args[0];
        self.remove_if_expired(key);
        let mut remove_key = false;
        let removed = match self.values.get_mut(key) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Hash(_)) => {
                return wrong_type();
            }
            Some(RedisValue::Set(set)) => {
                let mut removed = 0i64;
                for member in &args[1..] {
                    if set.remove(member) {
                        removed += 1;
                    }
                }
                remove_key = set.is_empty();
                removed
            }
            None => 0,
        };

        if remove_key {
            self.values.remove(key);
            self.expires_at.remove(key);
        } else if removed > 0 {
            self.expires_at.remove(key);
        }

        RespReply::Integer(removed)
    }

    fn execute_sismember(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("sismember");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Hash(_)) => {
                wrong_type()
            }
            Some(RedisValue::Set(set)) => RespReply::Integer(i64::from(set.contains(&args[1]))),
            None => RespReply::Integer(0),
        }
    }

    fn execute_smembers(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("smembers");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_)) | Some(RedisValue::List(_)) | Some(RedisValue::Hash(_)) => {
                wrong_type()
            }
            Some(RedisValue::Set(set)) => RespReply::Array(
                set.iter()
                    .map(|member| RespReply::BulkString(member.to_vec()))
                    .collect(),
            ),
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_set_store(&mut self, args: Vec<Vec<u8>>, operation: SetStoreOp) -> RespReply {
        if args.len() < 2 {
            return wrong_arity(operation.command_name());
        }

        let mut args = args;
        let destination = args.remove(0);
        let source_keys = args;

        for key in &source_keys {
            self.remove_if_expired(key);
        }
        for key in &source_keys {
            match self.values.get(key) {
                Some(RedisValue::Set(_)) | None => {}
                Some(RedisValue::String(_))
                | Some(RedisValue::List(_))
                | Some(RedisValue::Hash(_)) => return wrong_type(),
            }
        }

        let result = match operation {
            SetStoreOp::Union => self.set_union(&source_keys),
            SetStoreOp::Intersection => self.set_intersection(&source_keys),
            SetStoreOp::Difference => self.set_difference(&source_keys),
        };
        let len = result.len() as i64;

        self.expires_at.remove(&destination);
        if result.is_empty() {
            self.values.remove(&destination);
        } else {
            self.values.insert(destination, RedisValue::Set(result));
        }

        RespReply::Integer(len)
    }

    fn set_union(&self, source_keys: &[Vec<u8>]) -> BTreeSet<Vec<u8>> {
        let mut result = BTreeSet::new();
        for key in source_keys {
            if let Some(RedisValue::Set(set)) = self.values.get(key) {
                for member in set {
                    result.insert(member.to_vec());
                }
            }
        }
        result
    }

    fn set_intersection(&self, source_keys: &[Vec<u8>]) -> BTreeSet<Vec<u8>> {
        let mut keys = source_keys.iter();
        let first_key = match keys.next() {
            Some(key) => key,
            None => return BTreeSet::new(),
        };
        let first_set = match self.values.get(first_key) {
            Some(RedisValue::Set(set)) => set,
            _ => return BTreeSet::new(),
        };

        let mut result = BTreeSet::new();
        'member: for member in first_set {
            for key in keys.clone() {
                match self.values.get(key) {
                    Some(RedisValue::Set(set)) if set.contains(member) => {}
                    _ => continue 'member,
                }
            }
            result.insert(member.to_vec());
        }
        result
    }

    fn set_difference(&self, source_keys: &[Vec<u8>]) -> BTreeSet<Vec<u8>> {
        let mut keys = source_keys.iter();
        let first_key = match keys.next() {
            Some(key) => key,
            None => return BTreeSet::new(),
        };
        let first_set = match self.values.get(first_key) {
            Some(RedisValue::Set(set)) => set,
            _ => return BTreeSet::new(),
        };

        let mut result = BTreeSet::new();
        'member: for member in first_set {
            for key in keys.clone() {
                if let Some(RedisValue::Set(set)) = self.values.get(key) {
                    if set.contains(member) {
                        continue 'member;
                    }
                }
            }
            result.insert(member.to_vec());
        }
        result
    }

    fn execute_type(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("type");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(value) => RespReply::SimpleString(value.type_name()),
            None => RespReply::SimpleString("none"),
        }
    }

    fn execute_rename(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("rename");
        }

        let mut args = args;
        let source = args.remove(0);
        let destination = args.remove(0);
        self.rename_key(source, destination, false)
    }

    fn execute_renamenx(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("renamenx");
        }

        let mut args = args;
        let source = args.remove(0);
        let destination = args.remove(0);
        self.rename_key(source, destination, true)
    }

    fn rename_key(
        &mut self,
        source: Vec<u8>,
        destination: Vec<u8>,
        only_if_absent: bool,
    ) -> RespReply {
        self.remove_if_expired(&source);
        self.remove_if_expired(&destination);

        if !self.values.contains_key(&source) {
            return RespReply::Error("ERR no such key".to_string());
        }
        if only_if_absent && self.values.contains_key(&destination) {
            return RespReply::Integer(0);
        }
        if source == destination {
            return if only_if_absent {
                RespReply::Integer(0)
            } else {
                RespReply::SimpleString("OK")
            };
        }

        let value = self.values.remove(&source).expect("source exists");
        let deadline = self.expires_at.remove(&source);
        self.expires_at.remove(&destination);
        if let Some(deadline) = deadline {
            self.expires_at.insert(destination.to_vec(), deadline);
        }
        self.values.insert(destination, value);

        if only_if_absent {
            RespReply::Integer(1)
        } else {
            RespReply::SimpleString("OK")
        }
    }

    fn execute_keys(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("keys");
        }
        if args[0] != b"*" {
            return RespReply::Error("ERR only KEYS * is supported".to_string());
        }

        self.remove_expired_keys();
        let mut keys: Vec<&Vec<u8>> = self.values.keys().collect();
        keys.sort();
        RespReply::Array(
            keys.into_iter()
                .map(|key| RespReply::BulkString(key.to_vec()))
                .collect(),
        )
    }

    fn remove_expired_keys(&mut self) {
        let now = Instant::now();
        let expired: Vec<Vec<u8>> = self
            .expires_at
            .iter()
            .filter_map(|(key, deadline)| {
                if *deadline <= now {
                    Some(key.to_vec())
                } else {
                    None
                }
            })
            .collect();

        for key in expired {
            self.values.remove(&key);
            self.expires_at.remove(&key);
        }
    }

    fn remove_if_expired(&mut self, key: &[u8]) {
        if self
            .expires_at
            .get(key)
            .is_some_and(|deadline| *deadline <= Instant::now())
        {
            self.values.remove(key);
            self.expires_at.remove(key);
        }
    }
}

impl RedisValue {
    fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::List(_) => "list",
            Self::Hash(_) => "hash",
            Self::Set(_) => "set",
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ListSide {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
enum SetStoreOp {
    Union,
    Intersection,
    Difference,
}

impl SetStoreOp {
    fn command_name(self) -> &'static str {
        match self {
            Self::Union => "sunionstore",
            Self::Intersection => "sinterstore",
            Self::Difference => "sdiffstore",
        }
    }
}

impl ListSide {
    fn push_command_name(self) -> &'static str {
        match self {
            Self::Left => "lpush",
            Self::Right => "rpush",
        }
    }

    fn pop_command_name(self) -> &'static str {
        match self {
            Self::Left => "lpop",
            Self::Right => "rpop",
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

fn wrong_type() -> RespReply {
    RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    )
}

fn normalize_range(len: usize, start: i64, stop: i64) -> Option<(usize, usize)> {
    if len == 0 {
        return None;
    }

    let len = len as i64;
    let mut start = if start < 0 { len + start } else { start };
    let mut stop = if stop < 0 { len + stop } else { stop };

    if start < 0 {
        start = 0;
    }
    if stop < 0 || start >= len || start > stop {
        return None;
    }
    if stop >= len {
        stop = len - 1;
    }

    Some((start as usize, stop as usize))
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

fn encode_array(values: &[RespReply]) -> Vec<u8> {
    let mut encoded = format!("*{}\r\n", values.len()).into_bytes();
    for value in values {
        encoded.extend_from_slice(&value.encode());
    }
    encoded
}

fn encode_error(message: &str) -> Vec<u8> {
    encode_prefixed_string(b'-', message.as_bytes())
}
