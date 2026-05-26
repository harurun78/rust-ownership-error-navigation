use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::{Duration, Instant};

use crate::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum RespReply {
    SimpleString(&'static str),
    BulkString(Vec<u8>),
    NullBulkString,
    NullArray,
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
            Self::NullArray => b"*-1\r\n".to_vec(),
            Self::Integer(value) => format!(":{value}\r\n").into_bytes(),
            Self::Array(values) => encode_array(values),
            Self::Error(message) => encode_error(message),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandCategory {
    Connection,
    String,
    List,
    Hash,
    Set,
    SortedSet,
    Stream,
    Keyspace,
    Transaction,
    Server,
}

impl CommandCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Connection => "connection",
            Self::String => "string",
            Self::List => "list",
            Self::Hash => "hash",
            Self::Set => "set",
            Self::SortedSet => "sorted-set",
            Self::Stream => "stream",
            Self::Keyspace => "keyspace",
            Self::Transaction => "transaction",
            Self::Server => "server",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CommandMetadata {
    pub name: &'static str,
    pub category: CommandCategory,
}

#[derive(Debug, PartialEq, Eq)]
enum RedisValue {
    String(Vec<u8>),
    List(Vec<Vec<u8>>),
    Hash(BTreeMap<Vec<u8>, Vec<u8>>),
    Set(BTreeSet<Vec<u8>>),
    ZSet(BTreeMap<Vec<u8>, i64>),
    Stream(BTreeMap<(u64, u64), StreamEntry>),
}

#[derive(Debug, PartialEq, Eq)]
struct StreamEntry {
    id: Vec<u8>,
    fields: Vec<(Vec<u8>, Vec<u8>)>,
}

#[derive(Debug, Default)]
pub struct RedisMiniDb {
    values: HashMap<Vec<u8>, RedisValue>,
    expires_at: HashMap<Vec<u8>, Instant>,
    key_versions: HashMap<Vec<u8>, u64>,
    watched_keys: HashMap<Vec<u8>, u64>,
    transaction_queue: Option<Vec<Command>>,
}

impl RedisMiniDb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(&mut self, command: Command) -> RespReply {
        if command.args.is_empty() {
            return RespReply::Error("ERR unknown command ''".to_string());
        }

        let command_kind = find_command_spec(&command.args[0]).map(|spec| spec.kind);
        if let Some(kind) = command_kind {
            if kind.is_transaction_control() {
                return self.execute_transaction_control(kind, command.args);
            }
        }

        if let Some(queue) = self.transaction_queue.as_mut() {
            queue.push(command);
            return RespReply::SimpleString("QUEUED");
        }

        let mut args = command.args;
        let command_name = args.remove(0);
        self.execute_immediate(command_kind, command_name, args)
    }

    fn execute_transaction_control(
        &mut self,
        command_kind: CommandKind,
        args: Vec<Vec<u8>>,
    ) -> RespReply {
        match command_kind {
            CommandKind::Multi => self.execute_multi(args),
            CommandKind::Exec => self.execute_exec(args),
            CommandKind::Discard => self.execute_discard(args),
            CommandKind::Watch => self.execute_watch(args),
            CommandKind::Unwatch => self.execute_unwatch(args),
            _ => unreachable!("only transaction-control commands are routed here"),
        }
    }

    fn execute_immediate(
        &mut self,
        command_kind: Option<CommandKind>,
        command_name: Vec<u8>,
        args: Vec<Vec<u8>>,
    ) -> RespReply {
        let Some(command_kind) = command_kind else {
            return RespReply::Error(format!(
                "ERR unknown command '{}'",
                String::from_utf8_lossy(&command_name)
            ));
        };

        match command_kind {
            CommandKind::Ping => self.execute_ping(args),
            CommandKind::Echo => execute_echo(args),
            CommandKind::Set => self.execute_set(args),
            CommandKind::Get => self.execute_get(args),
            CommandKind::Del => self.execute_del(args),
            CommandKind::Exists => self.execute_exists(args),
            CommandKind::Expire => self.execute_expire(args),
            CommandKind::Ttl => self.execute_ttl(args),
            CommandKind::Persist => self.execute_persist(args),
            CommandKind::Incr => self.execute_incr_by(args, 1, "incr"),
            CommandKind::Decr => self.execute_incr_by(args, -1, "decr"),
            CommandKind::IncrBy => self.execute_incrby(args),
            CommandKind::LPush => self.execute_push(args, ListSide::Left),
            CommandKind::RPush => self.execute_push(args, ListSide::Right),
            CommandKind::LPop => self.execute_pop(args, ListSide::Left),
            CommandKind::RPop => self.execute_pop(args, ListSide::Right),
            CommandKind::LRange => self.execute_lrange(args),
            CommandKind::HSet => self.execute_hset(args),
            CommandKind::HGet => self.execute_hget(args),
            CommandKind::HDel => self.execute_hdel(args),
            CommandKind::HGetAll => self.execute_hgetall(args),
            CommandKind::SAdd => self.execute_sadd(args),
            CommandKind::SRem => self.execute_srem(args),
            CommandKind::SIsMember => self.execute_sismember(args),
            CommandKind::SMembers => self.execute_smembers(args),
            CommandKind::SUnionStore => self.execute_set_store(args, SetStoreOp::Union),
            CommandKind::SInterStore => self.execute_set_store(args, SetStoreOp::Intersection),
            CommandKind::SDiffStore => self.execute_set_store(args, SetStoreOp::Difference),
            CommandKind::ZAdd => self.execute_zadd(args),
            CommandKind::ZRem => self.execute_zrem(args),
            CommandKind::ZScore => self.execute_zscore(args),
            CommandKind::ZRange => self.execute_zrange(args),
            CommandKind::XAdd => self.execute_xadd(args),
            CommandKind::XLen => self.execute_xlen(args),
            CommandKind::XRange => self.execute_xrange(args),
            CommandKind::Type => self.execute_type(args),
            CommandKind::Rename => self.execute_rename(args),
            CommandKind::RenameNx => self.execute_renamenx(args),
            CommandKind::Keys => self.execute_keys(args),
            CommandKind::Scan => self.execute_scan(args),
            CommandKind::Multi
            | CommandKind::Exec
            | CommandKind::Discard
            | CommandKind::Watch
            | CommandKind::Unwatch => self.execute_transaction_control(command_kind, args),
        }
    }

    fn execute_multi(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("multi");
        }
        if self.transaction_queue.is_some() {
            return RespReply::Error("ERR MULTI calls can not be nested".to_string());
        }

        self.transaction_queue = Some(Vec::new());
        RespReply::SimpleString("OK")
    }

    fn execute_exec(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("exec");
        }

        let Some(queue) = self.transaction_queue.take() else {
            return RespReply::Error("ERR EXEC without MULTI".to_string());
        };

        if self.watched_key_changed() {
            self.watched_keys.clear();
            return RespReply::NullArray;
        }

        let reply = RespReply::Array(
            queue
                .into_iter()
                .map(|command| self.execute(command))
                .collect(),
        );
        self.watched_keys.clear();
        reply
    }

    fn execute_discard(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("discard");
        }
        if self.transaction_queue.take().is_none() {
            return RespReply::Error("ERR DISCARD without MULTI".to_string());
        }

        self.watched_keys.clear();
        RespReply::SimpleString("OK")
    }

    fn execute_watch(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("watch");
        }
        if self.transaction_queue.is_some() {
            return RespReply::Error("ERR WATCH inside MULTI is not allowed".to_string());
        }

        for key in &args[1..] {
            self.remove_if_expired(key);
        }
        for key in args.into_iter().skip(1) {
            let version = self.current_key_version(&key);
            self.watched_keys.insert(key, version);
        }

        RespReply::SimpleString("OK")
    }

    fn execute_unwatch(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("unwatch");
        }

        self.watched_keys.clear();
        RespReply::SimpleString("OK")
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
            Some(RedisValue::List(_))
                | Some(RedisValue::Hash(_))
                | Some(RedisValue::Set(_))
                | Some(RedisValue::ZSet(_))
                | Some(RedisValue::Stream(_))
        ) {
            return wrong_type();
        }

        self.expires_at.remove(&key);
        self.bump_key_version(&key);
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
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
                self.bump_key_version(&key);
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
            self.bump_key_version(&key);
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
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            None => 0,
        };

        match current.checked_add(delta) {
            Some(next) => {
                self.expires_at.remove(&key);
                self.bump_key_version(&key);
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
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::List(list)) => {
                for value in args {
                    match side {
                        ListSide::Left => list.insert(0, value),
                        ListSide::Right => list.push(value),
                    }
                }
                self.expires_at.remove(&key);
                let len = list.len() as i64;
                self.bump_key_version(&key);
                RespReply::Integer(len)
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
                self.bump_key_version(&key);
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
        let key = &args[0];
        let reply = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
        };
        if matches!(reply, RespReply::BulkString(_)) {
            self.bump_key_version(key);
        }
        reply
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
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
                self.bump_key_version(&key);
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
                self.bump_key_version(&key);
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
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
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
        if removed > 0 {
            self.bump_key_version(key);
        }

        RespReply::Integer(removed)
    }

    fn execute_hgetall(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("hgetall");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Set(set)) => {
                let mut added = 0i64;
                for member in args {
                    if set.insert(member) {
                        added += 1;
                    }
                }
                self.expires_at.remove(&key);
                if added > 0 {
                    self.bump_key_version(&key);
                }
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
                if added > 0 {
                    self.bump_key_version(&key);
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
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
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
        if removed > 0 {
            self.bump_key_version(key);
        }

        RespReply::Integer(removed)
    }

    fn execute_sismember(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("sismember");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
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
                | Some(RedisValue::Hash(_))
                | Some(RedisValue::ZSet(_))
                | Some(RedisValue::Stream(_)) => return wrong_type(),
            }
        }

        let result = match operation {
            SetStoreOp::Union => self.set_union(&source_keys),
            SetStoreOp::Intersection => self.set_intersection(&source_keys),
            SetStoreOp::Difference => self.set_difference(&source_keys),
        };
        let len = result.len() as i64;

        self.expires_at.remove(&destination);
        self.bump_key_version(&destination);
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

    fn execute_zadd(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 3 || args.len() % 2 == 0 {
            return wrong_arity("zadd");
        }

        let mut args = args;
        let key = args.remove(0);
        let mut pairs = Vec::new();
        while !args.is_empty() {
            let score = match parse_integer(&args[0]) {
                Some(score) => score,
                None => return integer_error(),
            };
            args.remove(0);
            let member = args.remove(0);
            pairs.push((member, score));
        }

        self.remove_if_expired(&key);
        match self.values.get_mut(&key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::ZSet(zset)) => {
                let mut added = 0i64;
                for (member, score) in pairs {
                    if !zset.contains_key(&member) {
                        added += 1;
                    }
                    zset.insert(member, score);
                }
                self.expires_at.remove(&key);
                self.bump_key_version(&key);
                RespReply::Integer(added)
            }
            None => {
                let mut zset = BTreeMap::new();
                let mut added = 0i64;
                for (member, score) in pairs {
                    if zset.insert(member, score).is_none() {
                        added += 1;
                    }
                }
                self.bump_key_version(&key);
                self.values.insert(key, RedisValue::ZSet(zset));
                RespReply::Integer(added)
            }
        }
    }

    fn execute_zrem(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("zrem");
        }

        let key = &args[0];
        self.remove_if_expired(key);
        let mut remove_key = false;
        let removed = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::ZSet(zset)) => {
                let mut removed = 0i64;
                for member in &args[1..] {
                    if zset.remove(member).is_some() {
                        removed += 1;
                    }
                }
                remove_key = zset.is_empty();
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
        if removed > 0 {
            self.bump_key_version(key);
        }

        RespReply::Integer(removed)
    }

    fn execute_zscore(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("zscore");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::ZSet(zset)) => match zset.get(&args[1]) {
                Some(score) => RespReply::BulkString(score.to_string().into_bytes()),
                None => RespReply::NullBulkString,
            },
            None => RespReply::NullBulkString,
        }
    }

    fn execute_zrange(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("zrange");
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
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::ZSet(zset)) => match normalize_range(zset.len(), start, stop) {
                Some((start, stop)) => {
                    let mut entries: Vec<(&Vec<u8>, &i64)> = zset.iter().collect();
                    entries.sort_by(|(left_member, left_score), (right_member, right_score)| {
                        left_score
                            .cmp(right_score)
                            .then_with(|| left_member.cmp(right_member))
                    });
                    RespReply::Array(
                        entries[start..=stop]
                            .iter()
                            .map(|(member, _score)| RespReply::BulkString(member.to_vec()))
                            .collect(),
                    )
                }
                None => RespReply::Array(Vec::new()),
            },
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_xadd(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 4 || args.len() % 2 != 0 {
            return wrong_arity("xadd");
        }

        let mut args = args;
        let key = args.remove(0);
        let id = args.remove(0);
        let parsed_id = match parse_stream_id(&id) {
            Some(id) => id,
            None => return invalid_stream_id(),
        };
        let mut fields = Vec::new();
        while !args.is_empty() {
            let field = args.remove(0);
            let value = args.remove(0);
            fields.push((field, value));
        }

        self.remove_if_expired(&key);
        match self.values.get_mut(&key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => wrong_type(),
            Some(RedisValue::Stream(stream)) => {
                stream.insert(
                    parsed_id,
                    StreamEntry {
                        id: id.to_vec(),
                        fields,
                    },
                );
                self.expires_at.remove(&key);
                self.bump_key_version(&key);
                RespReply::BulkString(id)
            }
            None => {
                let mut stream = BTreeMap::new();
                stream.insert(
                    parsed_id,
                    StreamEntry {
                        id: id.to_vec(),
                        fields,
                    },
                );
                self.bump_key_version(&key);
                self.values.insert(key, RedisValue::Stream(stream));
                RespReply::BulkString(id)
            }
        }
    }

    fn execute_xlen(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("xlen");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => wrong_type(),
            Some(RedisValue::Stream(stream)) => RespReply::Integer(stream.len() as i64),
            None => RespReply::Integer(0),
        }
    }

    fn execute_xrange(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("xrange");
        }

        let start = match parse_stream_bound(&args[1], StreamBoundKind::Minimum) {
            Some(id) => id,
            None => return invalid_stream_id(),
        };
        let end = match parse_stream_bound(&args[2], StreamBoundKind::Maximum) {
            Some(id) => id,
            None => return invalid_stream_id(),
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => wrong_type(),
            Some(RedisValue::Stream(stream)) => RespReply::Array(
                stream
                    .range(start..=end)
                    .map(|(_id, entry)| {
                        let mut field_values = Vec::with_capacity(entry.fields.len() * 2);
                        for (field, value) in &entry.fields {
                            field_values.push(RespReply::BulkString(field.to_vec()));
                            field_values.push(RespReply::BulkString(value.to_vec()));
                        }
                        RespReply::Array(vec![
                            RespReply::BulkString(entry.id.to_vec()),
                            RespReply::Array(field_values),
                        ])
                    })
                    .collect(),
            ),
            None => RespReply::Array(Vec::new()),
        }
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
        self.bump_key_version(&source);
        self.bump_key_version(&destination);
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

    fn execute_scan(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 && args.len() != 3 {
            return wrong_arity("scan");
        }

        let cursor = match parse_scan_index(&args[0]) {
            Some(cursor) => cursor,
            None => return RespReply::Error("ERR invalid cursor".to_string()),
        };
        let count = if args.len() == 3 {
            if !args[1].eq_ignore_ascii_case(b"COUNT") {
                return RespReply::Error("ERR unsupported SCAN option".to_string());
            }
            match parse_scan_index(&args[2]) {
                Some(0) | None => return RespReply::Error("ERR invalid COUNT".to_string()),
                Some(count) => Some(count),
            }
        } else {
            None
        };

        self.remove_expired_keys();
        let mut keys: Vec<&Vec<u8>> = self.values.keys().collect();
        keys.sort();
        if cursor > keys.len() {
            return RespReply::Error("ERR invalid cursor".to_string());
        }

        let end = match count {
            Some(count) => cursor.saturating_add(count).min(keys.len()),
            None => keys.len(),
        };
        let next_cursor = if end < keys.len() { end } else { 0 };
        let key_replies = keys[cursor..end]
            .iter()
            .map(|key| RespReply::BulkString(key.to_vec()))
            .collect();

        RespReply::Array(vec![
            RespReply::BulkString(next_cursor.to_string().into_bytes()),
            RespReply::Array(key_replies),
        ])
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
            self.bump_key_version(&key);
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
            self.bump_key_version(key);
        }
    }

    fn current_key_version(&self, key: &[u8]) -> u64 {
        self.key_versions.get(key).copied().unwrap_or(0)
    }

    fn bump_key_version(&mut self, key: &[u8]) {
        let version = self.key_versions.entry(key.to_vec()).or_insert(0);
        *version = version.saturating_add(1);
    }

    fn watched_key_changed(&self) -> bool {
        self.watched_keys
            .iter()
            .any(|(key, watched_version)| self.current_key_version(key) != *watched_version)
    }
}

impl RedisValue {
    fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::List(_) => "list",
            Self::Hash(_) => "hash",
            Self::Set(_) => "set",
            Self::ZSet(_) => "zset",
            Self::Stream(_) => "stream",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CommandKind {
    Ping,
    Echo,
    Set,
    Get,
    Del,
    Exists,
    Expire,
    Ttl,
    Persist,
    Incr,
    Decr,
    IncrBy,
    LPush,
    RPush,
    LPop,
    RPop,
    LRange,
    HSet,
    HGet,
    HDel,
    HGetAll,
    SAdd,
    SRem,
    SIsMember,
    SMembers,
    SUnionStore,
    SInterStore,
    SDiffStore,
    ZAdd,
    ZRem,
    ZScore,
    ZRange,
    XAdd,
    XLen,
    XRange,
    Type,
    Rename,
    RenameNx,
    Keys,
    Scan,
    Multi,
    Exec,
    Discard,
    Watch,
    Unwatch,
}

impl CommandKind {
    fn is_transaction_control(self) -> bool {
        matches!(
            self,
            Self::Multi | Self::Exec | Self::Discard | Self::Watch | Self::Unwatch
        )
    }
}

#[derive(Debug, Copy, Clone)]
struct CommandSpec {
    metadata: CommandMetadata,
    kind: CommandKind,
}

static COMMAND_SPECS: &[CommandSpec] = &[
    command_spec("PING", CommandCategory::Connection, CommandKind::Ping),
    command_spec("ECHO", CommandCategory::Connection, CommandKind::Echo),
    command_spec("SET", CommandCategory::String, CommandKind::Set),
    command_spec("GET", CommandCategory::String, CommandKind::Get),
    command_spec("DEL", CommandCategory::Keyspace, CommandKind::Del),
    command_spec("EXISTS", CommandCategory::Keyspace, CommandKind::Exists),
    command_spec("EXPIRE", CommandCategory::Keyspace, CommandKind::Expire),
    command_spec("TTL", CommandCategory::Keyspace, CommandKind::Ttl),
    command_spec("PERSIST", CommandCategory::Keyspace, CommandKind::Persist),
    command_spec("INCR", CommandCategory::String, CommandKind::Incr),
    command_spec("DECR", CommandCategory::String, CommandKind::Decr),
    command_spec("INCRBY", CommandCategory::String, CommandKind::IncrBy),
    command_spec("LPUSH", CommandCategory::List, CommandKind::LPush),
    command_spec("RPUSH", CommandCategory::List, CommandKind::RPush),
    command_spec("LPOP", CommandCategory::List, CommandKind::LPop),
    command_spec("RPOP", CommandCategory::List, CommandKind::RPop),
    command_spec("LRANGE", CommandCategory::List, CommandKind::LRange),
    command_spec("HSET", CommandCategory::Hash, CommandKind::HSet),
    command_spec("HGET", CommandCategory::Hash, CommandKind::HGet),
    command_spec("HDEL", CommandCategory::Hash, CommandKind::HDel),
    command_spec("HGETALL", CommandCategory::Hash, CommandKind::HGetAll),
    command_spec("SADD", CommandCategory::Set, CommandKind::SAdd),
    command_spec("SREM", CommandCategory::Set, CommandKind::SRem),
    command_spec("SISMEMBER", CommandCategory::Set, CommandKind::SIsMember),
    command_spec("SMEMBERS", CommandCategory::Set, CommandKind::SMembers),
    command_spec(
        "SUNIONSTORE",
        CommandCategory::Set,
        CommandKind::SUnionStore,
    ),
    command_spec(
        "SINTERSTORE",
        CommandCategory::Set,
        CommandKind::SInterStore,
    ),
    command_spec("SDIFFSTORE", CommandCategory::Set, CommandKind::SDiffStore),
    command_spec("ZADD", CommandCategory::SortedSet, CommandKind::ZAdd),
    command_spec("ZREM", CommandCategory::SortedSet, CommandKind::ZRem),
    command_spec("ZSCORE", CommandCategory::SortedSet, CommandKind::ZScore),
    command_spec("ZRANGE", CommandCategory::SortedSet, CommandKind::ZRange),
    command_spec("XADD", CommandCategory::Stream, CommandKind::XAdd),
    command_spec("XLEN", CommandCategory::Stream, CommandKind::XLen),
    command_spec("XRANGE", CommandCategory::Stream, CommandKind::XRange),
    command_spec("TYPE", CommandCategory::Keyspace, CommandKind::Type),
    command_spec("RENAME", CommandCategory::Keyspace, CommandKind::Rename),
    command_spec("RENAMENX", CommandCategory::Keyspace, CommandKind::RenameNx),
    command_spec("KEYS", CommandCategory::Keyspace, CommandKind::Keys),
    command_spec("SCAN", CommandCategory::Keyspace, CommandKind::Scan),
    command_spec("MULTI", CommandCategory::Transaction, CommandKind::Multi),
    command_spec("EXEC", CommandCategory::Transaction, CommandKind::Exec),
    command_spec(
        "DISCARD",
        CommandCategory::Transaction,
        CommandKind::Discard,
    ),
    command_spec("WATCH", CommandCategory::Transaction, CommandKind::Watch),
    command_spec(
        "UNWATCH",
        CommandCategory::Transaction,
        CommandKind::Unwatch,
    ),
];

const fn command_spec(
    name: &'static str,
    category: CommandCategory,
    kind: CommandKind,
) -> CommandSpec {
    CommandSpec {
        metadata: CommandMetadata { name, category },
        kind,
    }
}

pub fn normalize_command_name(command_name: &[u8]) -> Option<&'static str> {
    find_command_spec(command_name).map(|spec| spec.metadata.name)
}

pub fn command_metadata(command_name: &[u8]) -> Option<CommandMetadata> {
    find_command_spec(command_name).map(|spec| spec.metadata)
}

fn find_command_spec(command_name: &[u8]) -> Option<&'static CommandSpec> {
    COMMAND_SPECS
        .iter()
        .find(|spec| command_name.eq_ignore_ascii_case(spec.metadata.name.as_bytes()))
}

#[derive(Debug, Copy, Clone)]
enum StreamBoundKind {
    Minimum,
    Maximum,
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

fn parse_scan_index(value: &[u8]) -> Option<usize> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(value).ok()?.parse::<usize>().ok()
}

fn integer_error() -> RespReply {
    RespReply::Error("ERR value is not an integer or out of range".to_string())
}

fn parse_stream_id(value: &[u8]) -> Option<(u64, u64)> {
    let separator = value.iter().position(|byte| *byte == b'-')?;
    if value[separator + 1..].contains(&b'-') {
        return None;
    }
    let milliseconds = parse_unsigned_part(&value[..separator])?;
    let sequence = parse_unsigned_part(&value[separator + 1..])?;
    Some((milliseconds, sequence))
}

fn parse_unsigned_part(value: &[u8]) -> Option<u64> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(value).ok()?.parse::<u64>().ok()
}

fn parse_stream_bound(value: &[u8], kind: StreamBoundKind) -> Option<(u64, u64)> {
    match (value, kind) {
        (b"-", StreamBoundKind::Minimum) => Some((0, 0)),
        (b"+", StreamBoundKind::Maximum) => Some((u64::MAX, u64::MAX)),
        _ => parse_stream_id(value),
    }
}

fn invalid_stream_id() -> RespReply {
    RespReply::Error("ERR Invalid stream ID specified as stream command argument".to_string())
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
