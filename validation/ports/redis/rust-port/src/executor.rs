use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::{Duration, Instant};

use crate::Command;

const DATABASE_COUNT: usize = 16;
const MAX_STRING_SIZE: usize = 512 * 1024 * 1024;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SetCondition {
    Always,
    OnlyIfAbsent,
    OnlyIfPresent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct SetOptions {
    condition: SetCondition,
    get: bool,
    expiration: Option<Duration>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum RespProtocolVersion {
    #[default]
    Resp2,
    Resp3,
}

impl RespProtocolVersion {
    fn number(self) -> i64 {
        match self {
            Self::Resp2 => 2,
            Self::Resp3 => 3,
        }
    }
}

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
        self.encode_with_protocol(RespProtocolVersion::Resp2)
    }

    pub fn encode_with_protocol(&self, protocol_version: RespProtocolVersion) -> Vec<u8> {
        match self {
            Self::SimpleString(value) => encode_prefixed_string(b'+', value.as_bytes()),
            Self::BulkString(value) => encode_bulk_string(value),
            Self::NullBulkString | Self::NullArray
                if protocol_version == RespProtocolVersion::Resp3 =>
            {
                b"_\r\n".to_vec()
            }
            Self::NullBulkString => b"$-1\r\n".to_vec(),
            Self::NullArray => b"*-1\r\n".to_vec(),
            Self::Integer(value) => format!(":{value}\r\n").into_bytes(),
            Self::Array(values) => encode_array(values, protocol_version),
            Self::Error(message) => encode_error(message),
        }
    }
}

#[derive(Debug, Default)]
pub struct RedisMiniSession {
    db: RedisMiniDb,
    protocol_version: RespProtocolVersion,
}

impl RedisMiniSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn protocol_version(&self) -> RespProtocolVersion {
        self.protocol_version
    }

    pub fn execute(&mut self, command: Command) -> RespReply {
        if is_hello_command(&command.args) {
            return self.execute_hello(command.args);
        }

        self.db.execute(command)
    }

    pub fn execute_encoded(&mut self, command: Command) -> Vec<u8> {
        let reply = self.execute(command);
        reply.encode_with_protocol(self.protocol_version)
    }

    fn execute_hello(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("hello");
        }

        let Some(protocol_version) = parse_protocol_version(&args[1]) else {
            return unsupported_protocol_version();
        };

        self.protocol_version = protocol_version;
        hello_reply(protocol_version)
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
    Stream(StreamData),
}

#[derive(Debug, PartialEq, Eq)]
struct StreamEntry {
    id: Vec<u8>,
    fields: Vec<(Vec<u8>, Vec<u8>)>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct StreamData {
    entries: BTreeMap<(u64, u64), StreamEntry>,
    groups: BTreeMap<Vec<u8>, StreamConsumerGroup>,
}

#[derive(Debug, PartialEq, Eq)]
struct StreamConsumerGroup {
    last_delivered_id: (u64, u64),
    consumers: BTreeSet<Vec<u8>>,
    pending: BTreeMap<(u64, u64), StreamPendingEntry>,
}

#[derive(Debug, PartialEq, Eq)]
struct StreamPendingEntry {
    consumer: Vec<u8>,
}

#[derive(Debug, Default)]
struct DatabaseState {
    values: HashMap<Vec<u8>, RedisValue>,
    expires_at: HashMap<Vec<u8>, Instant>,
    key_versions: HashMap<Vec<u8>, u64>,
}

#[derive(Debug)]
pub struct RedisMiniDb {
    values: HashMap<Vec<u8>, RedisValue>,
    expires_at: HashMap<Vec<u8>, Instant>,
    key_versions: HashMap<Vec<u8>, u64>,
    watched_keys: HashMap<Vec<u8>, u64>,
    transaction_queue: Option<Vec<Command>>,
    selected_db: usize,
    databases: Vec<DatabaseState>,
}

impl Default for RedisMiniDb {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
            expires_at: HashMap::new(),
            key_versions: HashMap::new(),
            watched_keys: HashMap::new(),
            transaction_queue: None,
            selected_db: 0,
            databases: (0..DATABASE_COUNT)
                .map(|_| DatabaseState::default())
                .collect(),
        }
    }
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

        if matches!(command_kind, Some(CommandKind::Select)) && self.transaction_queue.is_some() {
            return RespReply::Error("ERR SELECT inside MULTI is not allowed".to_string());
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
            CommandKind::MGet => self.execute_mget(args),
            CommandKind::MSet => self.execute_mset(args),
            CommandKind::Append => self.execute_append(args),
            CommandKind::StrLen => self.execute_strlen(args),
            CommandKind::GetRange => self.execute_getrange(args),
            CommandKind::SetRange => self.execute_setrange(args),
            CommandKind::GetSet => self.execute_getset(args),
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
            CommandKind::LLen => self.execute_llen(args),
            CommandKind::LIndex => self.execute_lindex(args),
            CommandKind::LSet => self.execute_lset(args),
            CommandKind::LTrim => self.execute_ltrim(args),
            CommandKind::LRem => self.execute_lrem(args),
            CommandKind::RPopLPush => self.execute_rpoplpush(args),
            CommandKind::LMove => self.execute_lmove(args),
            CommandKind::BLPop => self.execute_blocking_pop(args, ListSide::Left),
            CommandKind::BRPop => self.execute_blocking_pop(args, ListSide::Right),
            CommandKind::BLMove => self.execute_blmove(args),
            CommandKind::HSet => self.execute_hset(args),
            CommandKind::HGet => self.execute_hget(args),
            CommandKind::HMGet => self.execute_hmget(args),
            CommandKind::HDel => self.execute_hdel(args),
            CommandKind::HGetAll => self.execute_hgetall(args),
            CommandKind::HKeys => self.execute_hkeys(args),
            CommandKind::HVals => self.execute_hvals(args),
            CommandKind::HLen => self.execute_hlen(args),
            CommandKind::HIncrBy => self.execute_hincrby(args),
            CommandKind::HScan => self.execute_hscan(args),
            CommandKind::SAdd => self.execute_sadd(args),
            CommandKind::SRem => self.execute_srem(args),
            CommandKind::SIsMember => self.execute_sismember(args),
            CommandKind::SMembers => self.execute_smembers(args),
            CommandKind::SCard => self.execute_scard(args),
            CommandKind::SPop => self.execute_spop(args),
            CommandKind::SRandMember => self.execute_srandmember(args),
            CommandKind::SMove => self.execute_smove(args),
            CommandKind::SDiff => self.execute_set_read(args, SetReadOp::Difference),
            CommandKind::SInter => self.execute_set_read(args, SetReadOp::Intersection),
            CommandKind::SUnion => self.execute_set_read(args, SetReadOp::Union),
            CommandKind::SScan => self.execute_sscan(args),
            CommandKind::SUnionStore => self.execute_set_store(args, SetStoreOp::Union),
            CommandKind::SInterStore => self.execute_set_store(args, SetStoreOp::Intersection),
            CommandKind::SDiffStore => self.execute_set_store(args, SetStoreOp::Difference),
            CommandKind::ZAdd => self.execute_zadd(args),
            CommandKind::ZRem => self.execute_zrem(args),
            CommandKind::ZScore => self.execute_zscore(args),
            CommandKind::ZRange => self.execute_zrange(args),
            CommandKind::ZCard => self.execute_zcard(args),
            CommandKind::ZCount => self.execute_zcount(args),
            CommandKind::ZRank => self.execute_zrank(args, false),
            CommandKind::ZRevRank => self.execute_zrank(args, true),
            CommandKind::ZRevRange => self.execute_zrevrange(args),
            CommandKind::ZRangeByScore => self.execute_zrangebyscore(args),
            CommandKind::ZRemRangeByRank => self.execute_zremrangebyrank(args),
            CommandKind::ZRemRangeByScore => self.execute_zremrangebyscore(args),
            CommandKind::ZRangeByLex => self.execute_zrangebylex(args),
            CommandKind::ZLexCount => self.execute_zlexcount(args),
            CommandKind::ZRemRangeByLex => self.execute_zremrangebylex(args),
            CommandKind::ZScan => self.execute_zscan(args),
            CommandKind::XAdd => self.execute_xadd(args),
            CommandKind::XLen => self.execute_xlen(args),
            CommandKind::XRange => self.execute_xrange(args),
            CommandKind::XRead => self.execute_xread(args),
            CommandKind::XDel => self.execute_xdel(args),
            CommandKind::XTrim => self.execute_xtrim(args),
            CommandKind::XGroup => self.execute_xgroup(args),
            CommandKind::XReadGroup => self.execute_xreadgroup(args),
            CommandKind::XAck => self.execute_xack(args),
            CommandKind::XPending => self.execute_xpending(args),
            CommandKind::XClaim => self.execute_xclaim(args),
            CommandKind::Type => self.execute_type(args),
            CommandKind::Rename => self.execute_rename(args),
            CommandKind::RenameNx => self.execute_renamenx(args),
            CommandKind::Keys => self.execute_keys(args),
            CommandKind::Scan => self.execute_scan(args),
            CommandKind::Hello => execute_hello(args),
            CommandKind::Select => self.execute_select(args),
            CommandKind::DbSize => self.execute_dbsize(args),
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

    fn execute_select(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("select");
        }

        let Some(index) = parse_database_index(&args[0]) else {
            return RespReply::Error("ERR invalid DB index".to_string());
        };
        if index >= DATABASE_COUNT {
            return RespReply::Error("ERR invalid DB index".to_string());
        }

        self.swap_selected_database(index);
        self.watched_keys.clear();
        RespReply::SimpleString("OK")
    }

    fn execute_dbsize(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if !args.is_empty() {
            return wrong_arity("dbsize");
        }

        self.remove_expired_keys();
        RespReply::Integer(self.values.len() as i64)
    }

    fn swap_selected_database(&mut self, index: usize) {
        if index == self.selected_db {
            return;
        }

        self.databases[self.selected_db] = DatabaseState {
            values: std::mem::take(&mut self.values),
            expires_at: std::mem::take(&mut self.expires_at),
            key_versions: std::mem::take(&mut self.key_versions),
        };
        let next = std::mem::take(&mut self.databases[index]);
        self.values = next.values;
        self.expires_at = next.expires_at;
        self.key_versions = next.key_versions;
        self.selected_db = index;
    }

    fn execute_set(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("set");
        }

        // parse options after key and value
        let options = match parse_set_options(&args[2..]) {
            Ok(options) => options,
            Err(reply) => return reply,
        };

        let expiration_deadline = match options.expiration {
            Some(duration) => match Instant::now().checked_add(duration) {
                Some(deadline) => Some(deadline),
                None => return invalid_expire_time(),
            },
            None => None,
        };

        let mut args = args;
        let key = args.remove(0);
        let value = args.remove(0);
        self.remove_if_expired(&key);

        let old_value = match self.values.get(&key) {
            Some(RedisValue::String(existing)) => Some(existing.to_vec()),
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            None => None,
        };
        let key_exists = old_value.is_some();
        let should_write = match options.condition {
            SetCondition::Always => true,
            SetCondition::OnlyIfAbsent => !key_exists,
            SetCondition::OnlyIfPresent => key_exists,
        };

        let get_reply = if options.get {
            match old_value {
                Some(value) => RespReply::BulkString(value),
                None => RespReply::NullBulkString,
            }
        } else {
            RespReply::NullBulkString
        };

        if !should_write {
            return get_reply;
        }

        match expiration_deadline {
            Some(deadline) => {
                self.expires_at.insert(key.to_vec(), deadline);
            }
            None => {
                self.expires_at.remove(&key);
            }
        }
        self.bump_key_version(&key);
        self.values.insert(key, RedisValue::String(value));
        if options.get {
            get_reply
        } else {
            RespReply::SimpleString("OK")
        }
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

    fn execute_mget(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.is_empty() {
            return wrong_arity("mget");
        }

        let mut replies = Vec::with_capacity(args.len());
        for key in args {
            self.remove_if_expired(&key);
            match self.values.get(&key) {
                Some(RedisValue::String(value)) => {
                    replies.push(RespReply::BulkString(value.to_vec()))
                }
                Some(RedisValue::List(_))
                | Some(RedisValue::Hash(_))
                | Some(RedisValue::Set(_))
                | Some(RedisValue::ZSet(_))
                | Some(RedisValue::Stream(_))
                | None => replies.push(RespReply::NullBulkString),
            }
        }
        RespReply::Array(replies)
    }

    fn execute_mset(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.is_empty() || args.len() % 2 != 0 {
            return wrong_arity("mset");
        }

        for pair in args.chunks(2) {
            self.remove_if_expired(&pair[0]);
        }
        for pair in args.chunks(2) {
            if matches!(
                self.values.get(&pair[0]),
                Some(RedisValue::List(_))
                    | Some(RedisValue::Hash(_))
                    | Some(RedisValue::Set(_))
                    | Some(RedisValue::ZSet(_))
                    | Some(RedisValue::Stream(_))
            ) {
                return wrong_type();
            }
        }

        let mut args = args.into_iter();
        while let Some(key) = args.next() {
            let value = args.next().expect("mset value after arity check");
            self.expires_at.remove(&key);
            self.bump_key_version(&key);
            self.values.insert(key, RedisValue::String(value));
        }
        RespReply::SimpleString("OK")
    }

    fn execute_append(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("append");
        }

        let mut args = args;
        let key = args.remove(0);
        let value = args.remove(0);
        self.remove_if_expired(&key);
        let len = match self.values.get_mut(&key) {
            Some(RedisValue::String(existing)) => {
                existing.extend(value);
                existing.len() as i64
            }
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            None => {
                let len = value.len() as i64;
                self.values.insert(key.to_vec(), RedisValue::String(value));
                len
            }
        };
        self.expires_at.remove(&key);
        self.bump_key_version(&key);
        RespReply::Integer(len)
    }

    fn execute_strlen(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("strlen");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(value)) => RespReply::Integer(value.len() as i64),
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Integer(0),
        }
    }

    fn execute_getrange(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("getrange");
        }

        let start = match parse_integer(&args[1]) {
            Some(start) => start,
            None => return integer_error(),
        };
        let end = match parse_integer(&args[2]) {
            Some(end) => end,
            None => return integer_error(),
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(value)) => match normalize_range(value.len(), start, end) {
                Some((start, end)) => RespReply::BulkString(value[start..=end].to_vec()),
                None => RespReply::BulkString(Vec::new()),
            },
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::BulkString(Vec::new()),
        }
    }

    fn execute_setrange(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("setrange");
        }

        let offset = match parse_scan_index(&args[1]) {
            Some(offset) => offset,
            None => return integer_error(),
        };
        let value_len = args[2].len();
        let target_len = match offset.checked_add(value_len) {
            Some(target_len) if target_len <= MAX_STRING_SIZE => target_len,
            _ => return RespReply::Error("ERR string exceeds maximum allowed size".to_string()),
        };

        let mut args = args;
        let key = args.remove(0);
        let _offset = args.remove(0);
        let value = args.remove(0);
        self.remove_if_expired(&key);
        if value.is_empty() {
            return match self.values.get(&key) {
                Some(RedisValue::String(existing)) => RespReply::Integer(existing.len() as i64),
                Some(RedisValue::List(_))
                | Some(RedisValue::Hash(_))
                | Some(RedisValue::Set(_))
                | Some(RedisValue::ZSet(_))
                | Some(RedisValue::Stream(_)) => wrong_type(),
                None => RespReply::Integer(0),
            };
        }

        let len = match self.values.get_mut(&key) {
            Some(RedisValue::String(existing)) => {
                if existing.len() < target_len {
                    existing.resize(target_len, 0);
                }
                existing[offset..offset + value.len()].copy_from_slice(&value);
                existing.len()
            }
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            None => {
                let mut next = vec![0; target_len];
                next[offset..offset + value.len()].copy_from_slice(&value);
                self.values.insert(key.to_vec(), RedisValue::String(next));
                target_len
            }
        };
        self.expires_at.remove(&key);
        self.bump_key_version(&key);
        RespReply::Integer(len as i64)
    }

    fn execute_getset(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("getset");
        }

        let mut args = args;
        let key = args.remove(0);
        let value = args.remove(0);
        self.remove_if_expired(&key);
        let old_value = match self.values.get(&key) {
            Some(RedisValue::String(existing)) => RespReply::BulkString(existing.to_vec()),
            Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            None => RespReply::NullBulkString,
        };

        self.expires_at.remove(&key);
        self.bump_key_version(&key);
        self.values.insert(key, RedisValue::String(value));
        old_value
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
                let remaining = deadline.saturating_duration_since(Instant::now());
                let mut ttl = remaining.as_secs() as i64;
                if remaining.subsec_nanos() > 0 {
                    ttl = ttl.saturating_add(1);
                }
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

    fn execute_llen(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("llen");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::List(list)) => RespReply::Integer(list.len() as i64),
            None => RespReply::Integer(0),
        }
    }

    fn execute_lindex(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("lindex");
        }

        let index = match parse_integer(&args[1]) {
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
            Some(RedisValue::List(list)) => match normalize_index(list.len(), index) {
                Some(index) => RespReply::BulkString(list[index].to_vec()),
                None => RespReply::NullBulkString,
            },
            None => RespReply::NullBulkString,
        }
    }

    fn execute_lset(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("lset");
        }

        let index = match parse_integer(&args[1]) {
            Some(value) => value,
            None => return integer_error(),
        };

        let mut args = args;
        let key = args.remove(0);
        let _index = args.remove(0);
        let element = args.remove(0);
        self.remove_if_expired(&key);
        match self.values.get_mut(&key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::List(list)) => match normalize_index(list.len(), index) {
                Some(index) => {
                    list[index] = element;
                    self.expires_at.remove(&key);
                    self.bump_key_version(&key);
                    RespReply::SimpleString("OK")
                }
                None => out_of_range(),
            },
            None => out_of_range(),
        }
    }

    fn execute_ltrim(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("ltrim");
        }

        let start = match parse_integer(&args[1]) {
            Some(value) => value,
            None => return integer_error(),
        };
        let stop = match parse_integer(&args[2]) {
            Some(value) => value,
            None => return integer_error(),
        };

        let key = &args[0];
        self.remove_if_expired(key);
        let (remove_key, mutated) = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::List(list)) => {
                let original_len = list.len();
                match normalize_range(original_len, start, stop) {
                    Some((start, stop)) => {
                        if start > 0 {
                            list.drain(0..start);
                        }
                        let keep_len = stop - start + 1;
                        list.truncate(keep_len);
                    }
                    None => list.clear(),
                }
                (list.is_empty(), list.len() != original_len)
            }
            None => return RespReply::SimpleString("OK"),
        };

        if remove_key {
            self.values.remove(key);
            self.expires_at.remove(key);
        } else if mutated {
            self.expires_at.remove(key);
        }
        if mutated {
            self.bump_key_version(key);
        }
        RespReply::SimpleString("OK")
    }

    fn execute_lrem(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("lrem");
        }

        let count = match parse_integer(&args[1]) {
            Some(value) => value,
            None => return integer_error(),
        };

        let key = &args[0];
        let element = &args[2];
        self.remove_if_expired(key);
        let mut remove_key = false;
        let removed = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::List(list)) => {
                let removed = remove_list_elements(list, count, element);
                remove_key = list.is_empty();
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

        RespReply::Integer(removed as i64)
    }

    fn execute_rpoplpush(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("rpoplpush");
        }

        let mut args = args;
        let source = args.remove(0);
        let destination = args.remove(0);
        self.execute_lmove_between_keys(source, destination, ListSide::Right, ListSide::Left)
    }

    fn execute_lmove(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 4 {
            return wrong_arity("lmove");
        }

        let from = match parse_list_side(&args[2]) {
            Some(side) => side,
            None => return syntax_error(),
        };
        let to = match parse_list_side(&args[3]) {
            Some(side) => side,
            None => return syntax_error(),
        };

        let mut args = args;
        let source = args.remove(0);
        let destination = args.remove(0);
        self.execute_lmove_between_keys(source, destination, from, to)
    }

    fn execute_lmove_between_keys(
        &mut self,
        source: Vec<u8>,
        destination: Vec<u8>,
        from: ListSide,
        to: ListSide,
    ) -> RespReply {
        self.remove_if_expired(&source);
        if source != destination {
            self.remove_if_expired(&destination);
        }

        if source == destination {
            return self.execute_same_key_lmove(source, from, to);
        }

        // Check types on source and destination before performing mutations to avoid
        // removing elements when the destination is a wrong type.
        match self.values.get(&source) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::List(_)) | None => {}
        }
        match self.values.get(&destination) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::List(_)) | None => {}
        }

        let mut remove_source = false;
        let value = match self.values.get_mut(&source) {
            Some(RedisValue::List(list)) => {
                let value = pop_list_value(list, from);
                remove_source = list.is_empty();
                value
            }
            _ => None,
        };
        let Some(value) = value else {
            return RespReply::NullBulkString;
        };

        match self.values.get_mut(&destination) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::List(list)) => push_list_value(list, to, value.to_vec()),
            None => {
                let mut list = Vec::new();
                push_list_value(&mut list, to, value.to_vec());
                self.values
                    .insert(destination.to_vec(), RedisValue::List(list));
            }
        }

        if remove_source {
            self.values.remove(&source);
            self.expires_at.remove(&source);
        } else {
            self.expires_at.remove(&source);
        }
        self.expires_at.remove(&destination);
        self.bump_key_version(&source);
        self.bump_key_version(&destination);
        RespReply::BulkString(value)
    }

    fn execute_same_key_lmove(&mut self, key: Vec<u8>, from: ListSide, to: ListSide) -> RespReply {
        let value = match self.values.get_mut(&key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::List(list)) => {
                let value = pop_list_value(list, from);
                if let Some(value) = &value {
                    push_list_value(list, to, value.to_vec());
                }
                value
            }
            None => None,
        };

        match value {
            Some(value) => {
                self.expires_at.remove(&key);
                self.bump_key_version(&key);
                RespReply::BulkString(value)
            }
            None => RespReply::NullBulkString,
        }
    }

    fn execute_blocking_pop(&mut self, args: Vec<Vec<u8>>, side: ListSide) -> RespReply {
        // Minimal non-blocking compatibility: scan keys in order and pop immediately if present.
        if args.len() < 2 {
            return wrong_arity(side.blocking_pop_command_name());
        }

        // Last argument is timeout; validate it but do not block.
        if let Err(reply) = parse_blocking_timeout(&args[args.len() - 1]) {
            return reply;
        }

        for key in &args[..args.len() - 1] {
            self.remove_if_expired(key);
            match self.values.get_mut(key) {
                Some(RedisValue::String(_))
                | Some(RedisValue::Hash(_))
                | Some(RedisValue::Set(_))
                | Some(RedisValue::ZSet(_))
                | Some(RedisValue::Stream(_)) => return wrong_type(),
                Some(RedisValue::List(list)) => {
                    let Some(value) = pop_list_value(list, side) else {
                        continue;
                    };
                    let remove_key = list.is_empty();
                    if remove_key {
                        self.values.remove(key);
                    }
                    self.expires_at.remove(key);
                    self.bump_key_version(key);
                    return RespReply::Array(vec![
                        RespReply::BulkString(key.to_vec()),
                        RespReply::BulkString(value),
                    ]);
                }
                None => {}
            }
        }

        RespReply::NullArray
    }

    fn execute_blmove(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        // BLMOVE source destination FROM TO timeout
        if args.len() != 5 {
            return wrong_arity("blmove");
        }

        let from = match parse_list_side(&args[2]) {
            Some(side) => side,
            None => return syntax_error(),
        };
        let to = match parse_list_side(&args[3]) {
            Some(side) => side,
            None => return syntax_error(),
        };

        if let Err(reply) = parse_blocking_timeout(&args[4]) {
            return reply;
        }

        let mut args = args;
        let source = args.remove(0);
        let destination = args.remove(0);
        self.execute_lmove_between_keys(source, destination, from, to)
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

    fn execute_hmget(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("hmget");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Hash(hash)) => RespReply::Array(
                args[1..]
                    .iter()
                    .map(|field| match hash.get(field) {
                        Some(value) => RespReply::BulkString(value.to_vec()),
                        None => RespReply::NullBulkString,
                    })
                    .collect(),
            ),
            None => RespReply::Array(
                args[1..]
                    .iter()
                    .map(|_| RespReply::NullBulkString)
                    .collect(),
            ),
        }
    }

    fn execute_hkeys(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("hkeys");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Hash(hash)) => RespReply::Array(
                hash.keys()
                    .map(|field| RespReply::BulkString(field.to_vec()))
                    .collect(),
            ),
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_hvals(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("hvals");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Hash(hash)) => RespReply::Array(
                hash.values()
                    .map(|value| RespReply::BulkString(value.to_vec()))
                    .collect(),
            ),
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_hlen(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("hlen");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Hash(hash)) => RespReply::Integer(hash.len() as i64),
            None => RespReply::Integer(0),
        }
    }

    fn execute_hincrby(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("hincrby");
        }

        let delta = match parse_integer(&args[2]) {
            Some(delta) => delta,
            None => return integer_error(),
        };
        let mut args = args;
        let key = args.remove(0);
        let field = args.remove(0);
        self.remove_if_expired(&key);

        match self.values.get_mut(&key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Hash(hash)) => {
                let current = match hash.get(&field) {
                    Some(value) => match parse_integer(value) {
                        Some(value) => value,
                        None => return integer_error(),
                    },
                    None => 0,
                };
                let Some(next) = current.checked_add(delta) else {
                    return RespReply::Error(
                        "ERR increment or decrement would overflow".to_string(),
                    );
                };
                hash.insert(field, next.to_string().into_bytes());
                self.expires_at.remove(&key);
                self.bump_key_version(&key);
                RespReply::Integer(next)
            }
            None => {
                let mut hash = BTreeMap::new();
                hash.insert(field, delta.to_string().into_bytes());
                self.bump_key_version(&key);
                self.values.insert(key, RedisValue::Hash(hash));
                RespReply::Integer(delta)
            }
        }
    }

    fn execute_hscan(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 && args.len() != 4 {
            return wrong_arity("hscan");
        }

        let cursor = match parse_scan_index(&args[1]) {
            Some(cursor) => cursor,
            None => return RespReply::Error("ERR invalid cursor".to_string()),
        };
        let count = if args.len() == 4 {
            if !args[2].eq_ignore_ascii_case(b"COUNT") {
                return RespReply::Error("ERR unsupported HSCAN option".to_string());
            }
            match parse_scan_index(&args[3]) {
                Some(0) | None => return RespReply::Error("ERR invalid COUNT".to_string()),
                Some(count) => Some(count),
            }
        } else {
            None
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Hash(hash)) => scan_hash_entries(hash, cursor, count),
            None if cursor == 0 => RespReply::Array(vec![
                RespReply::BulkString(b"0".to_vec()),
                RespReply::Array(Vec::new()),
            ]),
            None => RespReply::Error("ERR invalid cursor".to_string()),
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

    fn execute_scard(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("scard");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Set(set)) => RespReply::Integer(set.len() as i64),
            None => RespReply::Integer(0),
        }
    }

    fn execute_spop(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 && args.len() != 2 {
            return wrong_arity("spop");
        }

        let count = if args.len() == 2 {
            match parse_scan_index(&args[1]) {
                Some(count) => Some(count),
                None => return integer_error(),
            }
        } else {
            None
        };

        let key = &args[0];
        self.remove_if_expired(key);
        let mut remove_key = false;
        let popped = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
            Some(RedisValue::Set(set)) => match count {
                Some(count) => {
                    let mut values = Vec::new();
                    for _ in 0..count {
                        let Some(member) = set.pop_first() else {
                            break;
                        };
                        values.push(member);
                    }
                    remove_key = set.is_empty();
                    Some(values)
                }
                None => match set.pop_first() {
                    Some(member) => {
                        remove_key = set.is_empty();
                        Some(vec![member])
                    }
                    None => Some(Vec::new()),
                },
            },
            None => None,
        };

        match (count, popped) {
            (Some(_), None) => RespReply::Array(Vec::new()),
            (None, None) => RespReply::NullBulkString,
            (Some(_), Some(values)) => {
                if !values.is_empty() {
                    if remove_key {
                        self.values.remove(key);
                    }
                    self.expires_at.remove(key);
                    self.bump_key_version(key);
                }
                RespReply::Array(values.into_iter().map(RespReply::BulkString).collect())
            }
            (None, Some(mut values)) => match values.pop() {
                Some(member) => {
                    if remove_key {
                        self.values.remove(key);
                    }
                    self.expires_at.remove(key);
                    self.bump_key_version(key);
                    RespReply::BulkString(member)
                }
                None => RespReply::NullBulkString,
            },
        }
    }

    fn execute_srandmember(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 && args.len() != 2 {
            return wrong_arity("srandmember");
        }

        let count = if args.len() == 2 {
            match parse_integer(&args[1]) {
                Some(count) => Some(count),
                None => return integer_error(),
            }
        } else {
            None
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Set(set)) => match count {
                Some(count) if count >= 0 => RespReply::Array(
                    set.iter()
                        .take(count as usize)
                        .map(|member| RespReply::BulkString(member.to_vec()))
                        .collect(),
                ),
                Some(count) => {
                    let count = count.unsigned_abs() as usize;
                    let values = if set.is_empty() {
                        Vec::new()
                    } else {
                        set.iter()
                            .cycle()
                            .take(count)
                            .map(|member| RespReply::BulkString(member.to_vec()))
                            .collect()
                    };
                    RespReply::Array(values)
                }
                None => match set.first() {
                    Some(member) => RespReply::BulkString(member.to_vec()),
                    None => RespReply::NullBulkString,
                },
            },
            None => match count {
                Some(_) => RespReply::Array(Vec::new()),
                None => RespReply::NullBulkString,
            },
        }
    }

    fn execute_smove(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("smove");
        }

        let mut args = args;
        let source = args.remove(0);
        let destination = args.remove(0);
        let member = args.remove(0);
        self.remove_if_expired(&source);
        if source != destination {
            self.remove_if_expired(&destination);
        }

        match self.values.get(&source) {
            Some(RedisValue::Set(_)) | None => {}
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
        }
        match self.values.get(&destination) {
            Some(RedisValue::Set(_)) | None => {}
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => return wrong_type(),
        }

        if source == destination {
            return match self.values.get(&source) {
                Some(RedisValue::Set(set)) if set.contains(&member) => RespReply::Integer(1),
                _ => RespReply::Integer(0),
            };
        }

        let mut remove_source = false;
        let moved = match self.values.get_mut(&source) {
            Some(RedisValue::Set(set)) => {
                if set.remove(&member) {
                    remove_source = set.is_empty();
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        if !moved {
            return RespReply::Integer(0);
        }

        let destination_changed = match self.values.get_mut(&destination) {
            Some(RedisValue::Set(set)) => set.insert(member),
            None => {
                let mut set = BTreeSet::new();
                set.insert(member);
                self.values
                    .insert(destination.to_vec(), RedisValue::Set(set));
                true
            }
            Some(_) => unreachable!("destination type checked before mutation"),
        };

        if remove_source {
            self.values.remove(&source);
        }
        self.expires_at.remove(&source);
        self.bump_key_version(&source);
        if destination_changed {
            self.expires_at.remove(&destination);
            self.bump_key_version(&destination);
        }
        RespReply::Integer(1)
    }

    fn execute_set_read(&mut self, args: Vec<Vec<u8>>, operation: SetReadOp) -> RespReply {
        if args.is_empty() {
            return wrong_arity(operation.command_name());
        }

        for key in &args {
            self.remove_if_expired(key);
        }
        for key in &args {
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
            SetReadOp::Union => self.set_union(&args),
            SetReadOp::Intersection => self.set_intersection(&args),
            SetReadOp::Difference => self.set_difference(&args),
        };
        RespReply::Array(result.into_iter().map(RespReply::BulkString).collect())
    }

    fn execute_sscan(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 && args.len() != 4 {
            return wrong_arity("sscan");
        }

        let cursor = match parse_scan_index(&args[1]) {
            Some(cursor) => cursor,
            None => return RespReply::Error("ERR invalid cursor".to_string()),
        };
        let count = if args.len() == 4 {
            if !args[2].eq_ignore_ascii_case(b"COUNT") {
                return RespReply::Error("ERR unsupported SSCAN option".to_string());
            }
            match parse_scan_index(&args[3]) {
                Some(0) | None => return RespReply::Error("ERR invalid COUNT".to_string()),
                Some(count) => Some(count),
            }
        } else {
            None
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::ZSet(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            Some(RedisValue::Set(set)) => scan_set_members(set, cursor, count),
            None => RespReply::Array(vec![
                RespReply::BulkString(b"0".to_vec()),
                RespReply::Array(Vec::new()),
            ]),
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

    fn execute_zcard(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 1 {
            return wrong_arity("zcard");
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::ZSet(zset)) => RespReply::Integer(zset.len() as i64),
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Integer(0),
        }
    }

    fn execute_zcount(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("zcount");
        }

        let low = match parse_score_bound(&args[1]) {
            Some(b) => b,
            None => return syntax_error(),
        };
        let high = match parse_score_bound(&args[2]) {
            Some(b) => b,
            None => return syntax_error(),
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::ZSet(zset)) => {
                let count = zset
                    .iter()
                    .filter(|(_member, score)| score_in_bounds(**score, &low, &high))
                    .count();
                RespReply::Integer(count as i64)
            }
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Integer(0),
        }
    }

    fn execute_zrank(&mut self, args: Vec<Vec<u8>>, reverse: bool) -> RespReply {
        if args.len() != 2 {
            return wrong_arity(if reverse { "zrevrank" } else { "zrank" });
        }

        let key = &args[0];
        let member = &args[1];
        self.remove_if_expired(key);
        match self.values.get(key) {
            Some(RedisValue::ZSet(zset)) => {
                let mut entries: Vec<(&Vec<u8>, &i64)> = zset.iter().collect();
                entries.sort_by(|(lmem, lscore), (rmem, rscore)| {
                    lscore.cmp(rscore).then_with(|| lmem.cmp(rmem))
                });
                if reverse {
                    entries.reverse();
                }
                for (i, (m, _)) in entries.iter().enumerate() {
                    if *m == member {
                        return RespReply::Integer(i as i64);
                    }
                }
                RespReply::NullBulkString
            }
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::NullBulkString,
        }
    }

    fn execute_zrevrange(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 && args.len() != 4 {
            return wrong_arity("zrevrange");
        }

        let start = match parse_integer(&args[1]) {
            Some(v) => v,
            None => return integer_error(),
        };
        let stop = match parse_integer(&args[2]) {
            Some(v) => v,
            None => return integer_error(),
        };

        let mut with_scores = false;
        if args.len() == 4 {
            if !args[3].eq_ignore_ascii_case(b"WITHSCORES") {
                return RespReply::Error("ERR unsupported ZREVRANGE option".to_string());
            }
            with_scores = true;
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::ZSet(zset)) => match normalize_range(zset.len(), start, stop) {
                Some((s, e)) => {
                    let mut entries: Vec<(&Vec<u8>, &i64)> = zset.iter().collect();
                    entries.sort_by(|(left_member, left_score), (right_member, right_score)| {
                        left_score
                            .cmp(right_score)
                            .then_with(|| left_member.cmp(right_member))
                    });
                    entries.reverse();
                    {
                        if with_scores {
                            let mut out = Vec::new();
                            for (member, score) in &entries[s..=e] {
                                out.push(RespReply::BulkString(member.to_vec()));
                                out.push(RespReply::BulkString(score.to_string().into_bytes()));
                            }
                            RespReply::Array(out)
                        } else {
                            RespReply::Array(
                                entries[s..=e]
                                    .iter()
                                    .map(|(member, _score)| RespReply::BulkString(member.to_vec()))
                                    .collect(),
                            )
                        }
                    }
                }
                None => RespReply::Array(Vec::new()),
            },
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_zrangebyscore(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 3 {
            return wrong_arity("zrangebyscore");
        }

        let low = match parse_score_bound(&args[1]) {
            Some(b) => b,
            None => return syntax_error(),
        };
        let high = match parse_score_bound(&args[2]) {
            Some(b) => b,
            None => return syntax_error(),
        };

        // parse optional arguments: WITHSCORES and LIMIT offset count
        let mut with_scores = false;
        let mut limit: Option<(usize, usize)> = None;
        let mut index = 3;
        while index < args.len() {
            if args[index].eq_ignore_ascii_case(b"WITHSCORES") {
                if with_scores {
                    return RespReply::Error("ERR syntax error".to_string());
                }
                with_scores = true;
                index += 1;
            } else if args[index].eq_ignore_ascii_case(b"LIMIT") {
                if limit.is_some() || index + 2 >= args.len() {
                    return RespReply::Error("ERR syntax error".to_string());
                }
                let offset = match parse_scan_index(&args[index + 1]) {
                    Some(o) => o,
                    None => return RespReply::Error("ERR invalid LIMIT offset".to_string()),
                };
                let count = match parse_scan_index(&args[index + 2]) {
                    Some(0) | None => {
                        return RespReply::Error("ERR invalid LIMIT count".to_string());
                    }
                    Some(c) => c,
                };
                limit = Some((offset, count));
                index += 3;
            } else {
                return RespReply::Error("ERR unsupported ZRANGEBYSCORE option".to_string());
            }
        }

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::ZSet(zset)) => {
                let mut entries: Vec<(&Vec<u8>, &i64)> = zset
                    .iter()
                    .filter(|(_m, score)| score_in_bounds(**score, &low, &high))
                    .collect();
                entries.sort_by(|(lmem, lscore), (rmem, rscore)| {
                    lscore.cmp(rscore).then_with(|| lmem.cmp(rmem))
                });
                let mut result: Vec<RespReply> = Vec::new();
                let mut iter = entries.into_iter();
                if let Some((offset, count)) = limit {
                    for _ in 0..offset {
                        iter.next();
                    }
                    for _ in 0..count {
                        if let Some((member, score)) = iter.next() {
                            if with_scores {
                                result.push(RespReply::BulkString(member.to_vec()));
                                result.push(RespReply::BulkString(score.to_string().into_bytes()));
                            } else {
                                result.push(RespReply::BulkString(member.to_vec()));
                            }
                        } else {
                            break;
                        }
                    }
                } else {
                    for (member, score) in iter {
                        if with_scores {
                            result.push(RespReply::BulkString(member.to_vec()));
                            result.push(RespReply::BulkString(score.to_string().into_bytes()));
                        } else {
                            result.push(RespReply::BulkString(member.to_vec()));
                        }
                    }
                }
                RespReply::Array(result)
            }
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_zremrangebyrank(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("zremrangebyrank");
        }

        let start = match parse_integer(&args[1]) {
            Some(v) => v,
            None => return integer_error(),
        };
        let stop = match parse_integer(&args[2]) {
            Some(v) => v,
            None => return integer_error(),
        };

        self.remove_if_expired(&args[0]);
        match self.values.get_mut(&args[0]) {
            Some(RedisValue::ZSet(zset)) => match normalize_range(zset.len(), start, stop) {
                Some((s, e)) => {
                    let mut entries: Vec<(&Vec<u8>, &i64)> = zset.iter().collect();
                    entries.sort_by(|(lmem, lscore), (rmem, rscore)| {
                        lscore.cmp(rscore).then_with(|| lmem.cmp(rmem))
                    });
                    let to_remove: Vec<Vec<u8>> =
                        entries[s..=e].iter().map(|(m, _)| (*m).to_vec()).collect();
                    let removed = remove_zset_members(zset, &to_remove);
                    // expiration cleared on mutation and key removed if empty
                    if removed > 0 {
                        let key = &args[0];
                        if zset.is_empty() {
                            self.values.remove(key);
                            self.expires_at.remove(key);
                        } else {
                            self.expires_at.remove(key);
                        }
                        self.bump_key_version(key);
                    }
                    RespReply::Integer(removed as i64)
                }
                None => RespReply::Integer(0),
            },
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Integer(0),
        }
    }

    fn execute_zremrangebyscore(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("zremrangebyscore");
        }

        let low = match parse_score_bound(&args[1]) {
            Some(b) => b,
            None => return syntax_error(),
        };
        let high = match parse_score_bound(&args[2]) {
            Some(b) => b,
            None => return syntax_error(),
        };

        self.remove_if_expired(&args[0]);
        match self.values.get_mut(&args[0]) {
            Some(RedisValue::ZSet(zset)) => {
                let to_remove: Vec<Vec<u8>> = zset
                    .iter()
                    .filter(|(_m, score)| score_in_bounds(**score, &low, &high))
                    .map(|(m, _)| m.to_vec())
                    .collect();
                let removed = remove_zset_members(zset, &to_remove);
                if removed > 0 {
                    let key = &args[0];
                    if zset.is_empty() {
                        self.values.remove(key);
                        self.expires_at.remove(key);
                    } else {
                        self.expires_at.remove(key);
                    }
                    self.bump_key_version(key);
                }
                RespReply::Integer(removed as i64)
            }
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Integer(0),
        }
    }

    fn execute_zrangebylex(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("zrangebylex");
        }

        let min = parse_lex_bound(&args[1]);
        let max = parse_lex_bound(&args[2]);
        if min.is_none() || max.is_none() {
            return syntax_error();
        }
        let (min, max) = (min.unwrap(), max.unwrap());

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::ZSet(zset)) => {
                // only supported when all scores are equal
                let all_equal = zset.values().all(|s| *s == *zset.values().next().unwrap());
                if !all_equal {
                    return RespReply::Array(Vec::new());
                }
                let mut members: Vec<&Vec<u8>> = zset.keys().collect();
                members.sort();
                let result: Vec<RespReply> = members
                    .into_iter()
                    .filter(|m| lex_in_bounds(m, &min, &max))
                    .map(|m| RespReply::BulkString(m.to_vec()))
                    .collect();
                RespReply::Array(result)
            }
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_zlexcount(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("zlexcount");
        }
        let min = parse_lex_bound(&args[1]);
        let max = parse_lex_bound(&args[2]);
        if min.is_none() || max.is_none() {
            return syntax_error();
        }
        let (min, max) = (min.unwrap(), max.unwrap());

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::ZSet(zset)) => {
                let all_equal = zset.values().all(|s| *s == *zset.values().next().unwrap());
                if !all_equal {
                    return RespReply::Integer(0);
                }
                let mut members: Vec<&Vec<u8>> = zset.keys().collect();
                members.sort();
                let count = members
                    .into_iter()
                    .filter(|m| lex_in_bounds(m, &min, &max))
                    .count();
                RespReply::Integer(count as i64)
            }
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Integer(0),
        }
    }

    fn execute_zremrangebylex(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("zremrangebylex");
        }
        let min = parse_lex_bound(&args[1]);
        let max = parse_lex_bound(&args[2]);
        if min.is_none() || max.is_none() {
            return syntax_error();
        }
        let (min, max) = (min.unwrap(), max.unwrap());

        self.remove_if_expired(&args[0]);
        match self.values.get_mut(&args[0]) {
            Some(RedisValue::ZSet(zset)) => {
                let all_equal = zset.values().all(|s| *s == *zset.values().next().unwrap());
                if !all_equal {
                    return RespReply::Integer(0);
                }
                let mut members: Vec<Vec<u8>> = zset.keys().cloned().collect();
                members.sort();
                let to_remove: Vec<Vec<u8>> = members
                    .into_iter()
                    .filter(|m| lex_in_bounds(m, &min, &max))
                    .collect();
                let removed = remove_zset_members(zset, &to_remove);
                if removed > 0 {
                    let key = &args[0];
                    if zset.is_empty() {
                        self.values.remove(key);
                        self.expires_at.remove(key);
                    } else {
                        self.expires_at.remove(key);
                    }
                    self.bump_key_version(key);
                }
                RespReply::Integer(removed as i64)
            }
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Integer(0),
        }
    }

    fn execute_zscan(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 && args.len() != 4 {
            return wrong_arity("zscan");
        }

        let cursor = match parse_scan_index(&args[1]) {
            Some(c) => c,
            None => return RespReply::Error("ERR invalid cursor".to_string()),
        };
        let count = if args.len() == 4 {
            if !args[2].eq_ignore_ascii_case(b"COUNT") {
                return RespReply::Error("ERR unsupported ZSCAN option".to_string());
            }
            match parse_scan_index(&args[3]) {
                Some(0) | None => return RespReply::Error("ERR invalid COUNT".to_string()),
                Some(c) => Some(c),
            }
        } else {
            None
        };

        self.remove_if_expired(&args[0]);
        match self.values.get(&args[0]) {
            Some(RedisValue::ZSet(zset)) => scan_zset_entries(zset, cursor, count),
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::Stream(_)) => wrong_type(),
            None => RespReply::Array(vec![
                RespReply::BulkString(b"0".to_vec()),
                RespReply::Array(Vec::new()),
            ]),
        }
    }

    fn execute_xadd(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 4 || args.len() % 2 != 0 {
            return wrong_arity("xadd");
        }

        let mut args = args;
        let key = args.remove(0);
        let id = args.remove(0);
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
                let (parsed_id, entry_id) = match resolve_xadd_id(&id, &stream.entries) {
                    Some(resolved) => resolved,
                    None => return invalid_stream_id(),
                };
                stream.entries.insert(
                    parsed_id,
                    StreamEntry {
                        id: entry_id.to_vec(),
                        fields,
                    },
                );
                self.expires_at.remove(&key);
                self.bump_key_version(&key);
                RespReply::BulkString(entry_id)
            }
            None => {
                let empty_stream: BTreeMap<(u64, u64), StreamEntry> = BTreeMap::new();
                let (parsed_id, entry_id) = match resolve_xadd_id(&id, &empty_stream) {
                    Some(resolved) => resolved,
                    None => return invalid_stream_id(),
                };
                let mut stream = StreamData::default();
                stream.entries.insert(
                    parsed_id,
                    StreamEntry {
                        id: entry_id.to_vec(),
                        fields,
                    },
                );
                self.bump_key_version(&key);
                self.values.insert(key, RedisValue::Stream(stream));
                RespReply::BulkString(entry_id)
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
            Some(RedisValue::Stream(stream)) => RespReply::Integer(stream.entries.len() as i64),
            None => RespReply::Integer(0),
        }
    }

    fn execute_xrange(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 && args.len() != 5 {
            return wrong_arity("xrange");
        }
        let count = if args.len() == 5 {
            if !args[3].eq_ignore_ascii_case(b"COUNT") {
                return RespReply::Error("ERR unsupported XRANGE option".to_string());
            }
            match parse_scan_index(&args[4]) {
                Some(0) | None => return RespReply::Error("ERR invalid COUNT".to_string()),
                Some(count) => Some(count),
            }
        } else {
            None
        };

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
            Some(RedisValue::Stream(stream)) => {
                RespReply::Array(stream_range_reply(&stream.entries, start, end, count))
            }
            None => RespReply::Array(Vec::new()),
        }
    }

    fn execute_xread(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        let mut index = 0usize;
        let mut count = None;
        if args.len() >= 2 && args[0].eq_ignore_ascii_case(b"COUNT") {
            match parse_scan_index(&args[1]) {
                Some(0) | None => return RespReply::Error("ERR invalid COUNT".to_string()),
                Some(parsed) => count = Some(parsed),
            }
            index = 2;
        }
        if index >= args.len() || !args[index].eq_ignore_ascii_case(b"STREAMS") {
            return RespReply::Error("ERR syntax error".to_string());
        }

        let stream_args = &args[index + 1..];
        if stream_args.len() < 2 || stream_args.len() % 2 != 0 {
            return wrong_arity("xread");
        }
        let stream_count = stream_args.len() / 2;
        let keys = &stream_args[..stream_count];
        let ids = &stream_args[stream_count..];

        let mut parsed_ids = Vec::with_capacity(ids.len());
        for id in ids {
            if id == b"$" {
                parsed_ids.push(None);
            } else if let Some(parsed) = parse_stream_id(id) {
                parsed_ids.push(Some(parsed));
            } else {
                return invalid_stream_id();
            }
        }

        for key in keys {
            self.remove_if_expired(key);
        }

        let mut replies = Vec::new();
        for (key, parsed_id) in keys.iter().zip(parsed_ids.iter()) {
            match self.values.get(key) {
                Some(RedisValue::String(_))
                | Some(RedisValue::List(_))
                | Some(RedisValue::Hash(_))
                | Some(RedisValue::Set(_))
                | Some(RedisValue::ZSet(_)) => return wrong_type(),
                Some(RedisValue::Stream(stream)) => {
                    let start_after = match parsed_id {
                        Some(id) => *id,
                        None => stream
                            .entries
                            .keys()
                            .next_back()
                            .copied()
                            .unwrap_or((u64::MAX, u64::MAX)),
                    };
                    let entries = stream_after_reply(&stream.entries, start_after, count);
                    if !entries.is_empty() {
                        replies.push(RespReply::Array(vec![
                            RespReply::BulkString(key.to_vec()),
                            RespReply::Array(entries),
                        ]));
                    }
                }
                None => {}
            }
        }

        if replies.is_empty() {
            RespReply::NullArray
        } else {
            RespReply::Array(replies)
        }
    }

    fn execute_xdel(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 2 {
            return wrong_arity("xdel");
        }

        let key = &args[0];
        let mut ids = Vec::with_capacity(args.len() - 1);
        for id in &args[1..] {
            match parse_stream_id(id) {
                Some(parsed) => ids.push(parsed),
                None => return invalid_stream_id(),
            }
        }

        self.remove_if_expired(key);
        let mut should_remove_key = false;
        let removed = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => return wrong_type(),
            Some(RedisValue::Stream(stream)) => {
                let mut removed = 0usize;
                for id in ids {
                    if stream.entries.remove(&id).is_some() {
                        for group in stream.groups.values_mut() {
                            group.pending.remove(&id);
                        }
                        removed += 1;
                    }
                }
                should_remove_key = stream.entries.is_empty();
                removed
            }
            None => 0,
        };

        if removed > 0 {
            if should_remove_key {
                self.values.remove(key);
            }
            self.expires_at.remove(key);
            self.bump_key_version(key);
        }

        RespReply::Integer(removed as i64)
    }

    fn execute_xtrim(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("xtrim");
        }
        if !args[1].eq_ignore_ascii_case(b"MAXLEN") {
            return RespReply::Error("ERR unsupported XTRIM option".to_string());
        }
        let max_len = match parse_scan_index(&args[2]) {
            Some(value) => value,
            None => return RespReply::Error("ERR invalid MAXLEN".to_string()),
        };

        let key = &args[0];
        self.remove_if_expired(key);
        let mut should_remove_key = false;
        let removed = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => return wrong_type(),
            Some(RedisValue::Stream(stream)) => {
                let excess = stream.entries.len().saturating_sub(max_len);
                let ids: Vec<(u64, u64)> = stream.entries.keys().take(excess).copied().collect();
                for id in &ids {
                    stream.entries.remove(id);
                    for group in stream.groups.values_mut() {
                        group.pending.remove(id);
                    }
                }
                should_remove_key = stream.entries.is_empty();
                ids.len()
            }
            None => 0,
        };

        if removed > 0 {
            if should_remove_key {
                self.values.remove(key);
            }
            self.expires_at.remove(key);
            self.bump_key_version(key);
        }

        RespReply::Integer(removed as i64)
    }

    fn execute_xgroup(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.is_empty() {
            return wrong_arity("xgroup");
        }

        if args[0].eq_ignore_ascii_case(b"CREATE") {
            self.execute_xgroup_create(args)
        } else if args[0].eq_ignore_ascii_case(b"DESTROY") {
            self.execute_xgroup_destroy(args)
        } else if args[0].eq_ignore_ascii_case(b"CREATECONSUMER") {
            self.execute_xgroup_createconsumer(args)
        } else if args[0].eq_ignore_ascii_case(b"DELCONSUMER") {
            self.execute_xgroup_delconsumer(args)
        } else {
            RespReply::Error("ERR unsupported XGROUP subcommand".to_string())
        }
    }

    fn execute_xgroup_create(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 4 && args.len() != 5 {
            return wrong_arity("xgroup");
        }
        if args.len() == 5 && !args[4].eq_ignore_ascii_case(b"MKSTREAM") {
            return RespReply::Error("ERR syntax error".to_string());
        }

        let key = &args[1];
        let group_name = &args[2];
        let group_id = match parse_stream_group_id(&args[3]) {
            Some(id) => id,
            None => return invalid_stream_id(),
        };
        let mkstream = args.len() == 5;

        self.remove_if_expired(key);
        if !self.values.contains_key(key) && mkstream {
            self.values
                .insert(key.to_vec(), RedisValue::Stream(StreamData::default()));
        }

        let created = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => return wrong_type(),
            Some(RedisValue::Stream(stream)) => {
                if stream.groups.contains_key(group_name) {
                    return RespReply::Error(
                        "BUSYGROUP Consumer Group name already exists".to_string(),
                    );
                }
                stream.groups.insert(
                    group_name.to_vec(),
                    StreamConsumerGroup {
                        last_delivered_id: group_id,
                        consumers: BTreeSet::new(),
                        pending: BTreeMap::new(),
                    },
                );
                true
            }
            None => return RespReply::Error("ERR no such key".to_string()),
        };

        if created {
            self.bump_key_version(key);
        }
        RespReply::SimpleString("OK")
    }

    fn execute_xgroup_destroy(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 3 {
            return wrong_arity("xgroup");
        }
        let key = &args[1];
        let group_name = &args[2];

        self.remove_if_expired(key);
        let removed = match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => return wrong_type(),
            Some(RedisValue::Stream(stream)) => stream.groups.remove(group_name).is_some(),
            None => false,
        };
        if removed {
            self.bump_key_version(key);
        }
        RespReply::Integer(if removed { 1 } else { 0 })
    }

    fn execute_xgroup_createconsumer(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 4 {
            return wrong_arity("xgroup");
        }
        let key = &args[1];
        let group_name = &args[2];
        let consumer = &args[3];

        self.remove_if_expired(key);
        let created = match self.stream_group_mut(key, group_name) {
            Ok(group) => group.consumers.insert(consumer.to_vec()),
            Err(reply) => return reply,
        };
        if created {
            self.bump_key_version(key);
        }
        RespReply::Integer(if created { 1 } else { 0 })
    }

    fn execute_xgroup_delconsumer(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 4 {
            return wrong_arity("xgroup");
        }
        let key = &args[1];
        let group_name = &args[2];
        let consumer = &args[3];

        self.remove_if_expired(key);
        let removed_pending = match self.stream_group_mut(key, group_name) {
            Ok(group) => {
                group.consumers.remove(consumer);
                let ids: Vec<(u64, u64)> = group
                    .pending
                    .iter()
                    .filter_map(|(id, pending)| {
                        if pending.consumer == *consumer {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();
                for id in &ids {
                    group.pending.remove(id);
                }
                ids.len()
            }
            Err(reply) => return reply,
        };
        self.bump_key_version(key);
        RespReply::Integer(removed_pending as i64)
    }

    fn execute_xreadgroup(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 6 || !args[0].eq_ignore_ascii_case(b"GROUP") {
            return RespReply::Error("ERR syntax error".to_string());
        }
        let group_name = &args[1];
        let consumer = &args[2];
        let mut index = 3usize;
        let mut count = None;
        if index + 1 < args.len() && args[index].eq_ignore_ascii_case(b"COUNT") {
            match parse_scan_index(&args[index + 1]) {
                Some(0) | None => return RespReply::Error("ERR invalid COUNT".to_string()),
                Some(parsed) => count = Some(parsed),
            }
            index += 2;
        }
        if index >= args.len() || !args[index].eq_ignore_ascii_case(b"STREAMS") {
            return RespReply::Error("ERR syntax error".to_string());
        }
        let stream_args = &args[index + 1..];
        if stream_args.len() < 2 || stream_args.len() % 2 != 0 {
            return wrong_arity("xreadgroup");
        }

        let stream_count = stream_args.len() / 2;
        let keys = &stream_args[..stream_count];
        let ids = &stream_args[stream_count..];
        let mut parsed_ids = Vec::with_capacity(ids.len());
        for id in ids {
            if id == b">" {
                parsed_ids.push(None);
            } else if let Some(parsed) = parse_stream_id(id) {
                parsed_ids.push(Some(parsed));
            } else {
                return invalid_stream_id();
            }
        }

        for key in keys {
            self.remove_if_expired(key);
        }

        let mut mutated_keys = Vec::new();
        let mut replies = Vec::new();
        for (key, parsed_id) in keys.iter().zip(parsed_ids.iter()) {
            let stream = match self.values.get_mut(key) {
                Some(RedisValue::String(_))
                | Some(RedisValue::List(_))
                | Some(RedisValue::Hash(_))
                | Some(RedisValue::Set(_))
                | Some(RedisValue::ZSet(_)) => return wrong_type(),
                Some(RedisValue::Stream(stream)) => stream,
                None => return no_such_consumer_group(),
            };
            let Some(group) = stream.groups.get_mut(group_name) else {
                return no_such_consumer_group();
            };
            group.consumers.insert(consumer.to_vec());

            let mut entry_replies = Vec::new();
            match parsed_id {
                None => {
                    let start = (
                        group.last_delivered_id.0,
                        group.last_delivered_id.1.saturating_add(1),
                    );
                    for (id, entry) in stream.entries.range(start..) {
                        if count.is_some_and(|limit| entry_replies.len() >= limit) {
                            break;
                        }
                        entry_replies.push(stream_entry_reply(entry));
                        group.last_delivered_id = *id;
                        group.pending.insert(
                            *id,
                            StreamPendingEntry {
                                consumer: consumer.to_vec(),
                            },
                        );
                    }
                }
                Some(start_after) => {
                    for (id, pending) in &group.pending {
                        if *id <= *start_after || pending.consumer != *consumer {
                            continue;
                        }
                        if count.is_some_and(|limit| entry_replies.len() >= limit) {
                            break;
                        }
                        if let Some(entry) = stream.entries.get(id) {
                            entry_replies.push(stream_entry_reply(entry));
                        }
                    }
                }
            }

            if !entry_replies.is_empty() {
                mutated_keys.push((*key).to_vec());
                replies.push(RespReply::Array(vec![
                    RespReply::BulkString((*key).to_vec()),
                    RespReply::Array(entry_replies),
                ]));
            }
        }

        for key in mutated_keys {
            self.bump_key_version(&key);
        }

        if replies.is_empty() {
            RespReply::NullArray
        } else {
            RespReply::Array(replies)
        }
    }

    fn execute_xack(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 3 {
            return wrong_arity("xack");
        }
        let key = &args[0];
        let group_name = &args[1];
        let mut ids = Vec::with_capacity(args.len() - 2);
        for id in &args[2..] {
            match parse_stream_id(id) {
                Some(parsed) => ids.push(parsed),
                None => return invalid_stream_id(),
            }
        }

        self.remove_if_expired(key);
        let acked = match self.stream_group_mut(key, group_name) {
            Ok(group) => {
                let mut count = 0usize;
                for id in ids {
                    if group.pending.remove(&id).is_some() {
                        count += 1;
                    }
                }
                count
            }
            Err(reply) => return reply,
        };
        if acked > 0 {
            self.bump_key_version(key);
        }
        RespReply::Integer(acked as i64)
    }

    fn execute_xpending(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() != 2 {
            return wrong_arity("xpending");
        }
        let key = &args[0];
        let group_name = &args[1];

        self.remove_if_expired(key);
        let group = match self.stream_group(key, group_name) {
            Ok(group) => group,
            Err(reply) => return reply,
        };
        let smallest = group.pending.keys().next().copied();
        let greatest = group.pending.keys().next_back().copied();
        let mut per_consumer: BTreeMap<&Vec<u8>, usize> = BTreeMap::new();
        for pending in group.pending.values() {
            let count = per_consumer.entry(&pending.consumer).or_insert(0);
            *count += 1;
        }
        let consumers = per_consumer
            .into_iter()
            .map(|(consumer, count)| {
                RespReply::Array(vec![
                    RespReply::BulkString(consumer.to_vec()),
                    RespReply::BulkString(count.to_string().into_bytes()),
                ])
            })
            .collect();

        RespReply::Array(vec![
            RespReply::Integer(group.pending.len() as i64),
            optional_stream_id_reply(smallest),
            optional_stream_id_reply(greatest),
            RespReply::Array(consumers),
        ])
    }

    fn execute_xclaim(&mut self, args: Vec<Vec<u8>>) -> RespReply {
        if args.len() < 5 {
            return wrong_arity("xclaim");
        }
        let key = &args[0];
        let group_name = &args[1];
        let consumer = &args[2];
        if parse_scan_index(&args[3]).is_none() {
            return RespReply::Error("ERR invalid min-idle-time".to_string());
        }
        let mut ids = Vec::with_capacity(args.len() - 4);
        for id in &args[4..] {
            match parse_stream_id(id) {
                Some(parsed) => ids.push(parsed),
                None => return invalid_stream_id(),
            }
        }

        self.remove_if_expired(key);
        let mut claimed = Vec::new();
        let mut changed = false;
        match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => return wrong_type(),
            Some(RedisValue::Stream(stream)) => {
                let Some(group) = stream.groups.get_mut(group_name) else {
                    return no_such_consumer_group();
                };
                group.consumers.insert(consumer.to_vec());
                for id in ids {
                    if let Some(pending) = group.pending.get_mut(&id) {
                        pending.consumer = consumer.to_vec();
                        if let Some(entry) = stream.entries.get(&id) {
                            claimed.push(stream_entry_reply(entry));
                        }
                        changed = true;
                    }
                }
            }
            None => return no_such_consumer_group(),
        }
        if changed {
            self.bump_key_version(key);
        }
        RespReply::Array(claimed)
    }

    fn stream_group_mut(
        &mut self,
        key: &[u8],
        group_name: &[u8],
    ) -> Result<&mut StreamConsumerGroup, RespReply> {
        match self.values.get_mut(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => Err(wrong_type()),
            Some(RedisValue::Stream(stream)) => stream
                .groups
                .get_mut(group_name)
                .ok_or_else(no_such_consumer_group),
            None => Err(no_such_consumer_group()),
        }
    }

    fn stream_group(
        &self,
        key: &[u8],
        group_name: &[u8],
    ) -> Result<&StreamConsumerGroup, RespReply> {
        match self.values.get(key) {
            Some(RedisValue::String(_))
            | Some(RedisValue::List(_))
            | Some(RedisValue::Hash(_))
            | Some(RedisValue::Set(_))
            | Some(RedisValue::ZSet(_)) => Err(wrong_type()),
            Some(RedisValue::Stream(stream)) => stream
                .groups
                .get(group_name)
                .ok_or_else(no_such_consumer_group),
            None => Err(no_such_consumer_group()),
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
    MGet,
    MSet,
    Append,
    StrLen,
    GetRange,
    SetRange,
    GetSet,
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
    LLen,
    LIndex,
    LSet,
    LTrim,
    LRem,
    RPopLPush,
    LMove,
    BLPop,
    BRPop,
    BLMove,
    HSet,
    HGet,
    HMGet,
    HDel,
    HGetAll,
    HKeys,
    HVals,
    HLen,
    HIncrBy,
    HScan,
    SAdd,
    SRem,
    SIsMember,
    SMembers,
    SCard,
    SPop,
    SRandMember,
    SMove,
    SDiff,
    SInter,
    SUnion,
    SScan,
    SUnionStore,
    SInterStore,
    SDiffStore,
    ZAdd,
    ZRem,
    ZScore,
    ZRange,
    ZCard,
    ZCount,
    ZRank,
    ZRevRank,
    ZRevRange,
    ZRangeByScore,
    ZRemRangeByRank,
    ZRemRangeByScore,
    ZRangeByLex,
    ZLexCount,
    ZRemRangeByLex,
    ZScan,
    XAdd,
    XLen,
    XRange,
    XRead,
    XDel,
    XTrim,
    XGroup,
    XReadGroup,
    XAck,
    XPending,
    XClaim,
    Type,
    Rename,
    RenameNx,
    Keys,
    Scan,
    Hello,
    Select,
    DbSize,
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
    command_spec("MGET", CommandCategory::String, CommandKind::MGet),
    command_spec("MSET", CommandCategory::String, CommandKind::MSet),
    command_spec("APPEND", CommandCategory::String, CommandKind::Append),
    command_spec("STRLEN", CommandCategory::String, CommandKind::StrLen),
    command_spec("GETRANGE", CommandCategory::String, CommandKind::GetRange),
    command_spec("SETRANGE", CommandCategory::String, CommandKind::SetRange),
    command_spec("GETSET", CommandCategory::String, CommandKind::GetSet),
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
    command_spec("LLEN", CommandCategory::List, CommandKind::LLen),
    command_spec("LINDEX", CommandCategory::List, CommandKind::LIndex),
    command_spec("LSET", CommandCategory::List, CommandKind::LSet),
    command_spec("LTRIM", CommandCategory::List, CommandKind::LTrim),
    command_spec("LREM", CommandCategory::List, CommandKind::LRem),
    command_spec("RPOPLPUSH", CommandCategory::List, CommandKind::RPopLPush),
    command_spec("LMOVE", CommandCategory::List, CommandKind::LMove),
    command_spec("BLPOP", CommandCategory::List, CommandKind::BLPop),
    command_spec("BRPOP", CommandCategory::List, CommandKind::BRPop),
    command_spec("BLMOVE", CommandCategory::List, CommandKind::BLMove),
    command_spec("HSET", CommandCategory::Hash, CommandKind::HSet),
    command_spec("HGET", CommandCategory::Hash, CommandKind::HGet),
    command_spec("HMGET", CommandCategory::Hash, CommandKind::HMGet),
    command_spec("HDEL", CommandCategory::Hash, CommandKind::HDel),
    command_spec("HGETALL", CommandCategory::Hash, CommandKind::HGetAll),
    command_spec("HKEYS", CommandCategory::Hash, CommandKind::HKeys),
    command_spec("HVALS", CommandCategory::Hash, CommandKind::HVals),
    command_spec("HLEN", CommandCategory::Hash, CommandKind::HLen),
    command_spec("HINCRBY", CommandCategory::Hash, CommandKind::HIncrBy),
    command_spec("HSCAN", CommandCategory::Hash, CommandKind::HScan),
    command_spec("SADD", CommandCategory::Set, CommandKind::SAdd),
    command_spec("SREM", CommandCategory::Set, CommandKind::SRem),
    command_spec("SISMEMBER", CommandCategory::Set, CommandKind::SIsMember),
    command_spec("SMEMBERS", CommandCategory::Set, CommandKind::SMembers),
    command_spec("SCARD", CommandCategory::Set, CommandKind::SCard),
    command_spec("SPOP", CommandCategory::Set, CommandKind::SPop),
    command_spec(
        "SRANDMEMBER",
        CommandCategory::Set,
        CommandKind::SRandMember,
    ),
    command_spec("SMOVE", CommandCategory::Set, CommandKind::SMove),
    command_spec("SDIFF", CommandCategory::Set, CommandKind::SDiff),
    command_spec("SINTER", CommandCategory::Set, CommandKind::SInter),
    command_spec("SUNION", CommandCategory::Set, CommandKind::SUnion),
    command_spec("SSCAN", CommandCategory::Set, CommandKind::SScan),
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
    command_spec("ZCARD", CommandCategory::SortedSet, CommandKind::ZCard),
    command_spec("ZCOUNT", CommandCategory::SortedSet, CommandKind::ZCount),
    command_spec("ZRANK", CommandCategory::SortedSet, CommandKind::ZRank),
    command_spec(
        "ZREVRANK",
        CommandCategory::SortedSet,
        CommandKind::ZRevRank,
    ),
    command_spec(
        "ZREVRANGE",
        CommandCategory::SortedSet,
        CommandKind::ZRevRange,
    ),
    command_spec(
        "ZRANGEBYSCORE",
        CommandCategory::SortedSet,
        CommandKind::ZRangeByScore,
    ),
    command_spec(
        "ZREMRANGEBYRANK",
        CommandCategory::SortedSet,
        CommandKind::ZRemRangeByRank,
    ),
    command_spec(
        "ZREMRANGEBYSCORE",
        CommandCategory::SortedSet,
        CommandKind::ZRemRangeByScore,
    ),
    command_spec(
        "ZRANGEBYLEX",
        CommandCategory::SortedSet,
        CommandKind::ZRangeByLex,
    ),
    command_spec(
        "ZLEXCOUNT",
        CommandCategory::SortedSet,
        CommandKind::ZLexCount,
    ),
    command_spec(
        "ZREMRANGEBYLEX",
        CommandCategory::SortedSet,
        CommandKind::ZRemRangeByLex,
    ),
    command_spec("ZSCAN", CommandCategory::SortedSet, CommandKind::ZScan),
    command_spec("XADD", CommandCategory::Stream, CommandKind::XAdd),
    command_spec("XLEN", CommandCategory::Stream, CommandKind::XLen),
    command_spec("XRANGE", CommandCategory::Stream, CommandKind::XRange),
    command_spec("XREAD", CommandCategory::Stream, CommandKind::XRead),
    command_spec("XDEL", CommandCategory::Stream, CommandKind::XDel),
    command_spec("XTRIM", CommandCategory::Stream, CommandKind::XTrim),
    command_spec("XGROUP", CommandCategory::Stream, CommandKind::XGroup),
    command_spec(
        "XREADGROUP",
        CommandCategory::Stream,
        CommandKind::XReadGroup,
    ),
    command_spec("XACK", CommandCategory::Stream, CommandKind::XAck),
    command_spec("XPENDING", CommandCategory::Stream, CommandKind::XPending),
    command_spec("XCLAIM", CommandCategory::Stream, CommandKind::XClaim),
    command_spec("TYPE", CommandCategory::Keyspace, CommandKind::Type),
    command_spec("RENAME", CommandCategory::Keyspace, CommandKind::Rename),
    command_spec("RENAMENX", CommandCategory::Keyspace, CommandKind::RenameNx),
    command_spec("KEYS", CommandCategory::Keyspace, CommandKind::Keys),
    command_spec("SCAN", CommandCategory::Keyspace, CommandKind::Scan),
    command_spec("HELLO", CommandCategory::Connection, CommandKind::Hello),
    command_spec("SELECT", CommandCategory::Connection, CommandKind::Select),
    command_spec("DBSIZE", CommandCategory::Keyspace, CommandKind::DbSize),
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

#[derive(Debug, Copy, Clone)]
enum SetReadOp {
    Union,
    Intersection,
    Difference,
}

impl SetReadOp {
    fn command_name(self) -> &'static str {
        match self {
            Self::Union => "sunion",
            Self::Intersection => "sinter",
            Self::Difference => "sdiff",
        }
    }
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

    fn blocking_pop_command_name(self) -> &'static str {
        match self {
            Self::Left => "blpop",
            Self::Right => "brpop",
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

fn execute_hello(args: Vec<Vec<u8>>) -> RespReply {
    if args.len() != 1 {
        return wrong_arity("hello");
    }

    let Some(protocol_version) = parse_protocol_version(&args[0]) else {
        return unsupported_protocol_version();
    };

    hello_reply(protocol_version)
}

fn is_hello_command(args: &[Vec<u8>]) -> bool {
    args.first()
        .is_some_and(|command_name| command_name.eq_ignore_ascii_case(b"HELLO"))
}

fn parse_protocol_version(value: &[u8]) -> Option<RespProtocolVersion> {
    match value {
        b"2" => Some(RespProtocolVersion::Resp2),
        b"3" => Some(RespProtocolVersion::Resp3),
        _ => None,
    }
}

fn hello_reply(protocol_version: RespProtocolVersion) -> RespReply {
    RespReply::Array(vec![
        RespReply::BulkString(b"server".to_vec()),
        RespReply::BulkString(b"redis-mini".to_vec()),
        RespReply::BulkString(b"proto".to_vec()),
        RespReply::Integer(protocol_version.number()),
    ])
}

fn unsupported_protocol_version() -> RespReply {
    RespReply::Error("NOPROTO unsupported protocol version".to_string())
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

#[derive(Clone, Debug)]
struct ScoreBound {
    value: f64,
    exclusive: bool,
}

fn parse_score_bound(value: &[u8]) -> Option<ScoreBound> {
    if value.eq_ignore_ascii_case(b"-inf") {
        return Some(ScoreBound {
            value: f64::NEG_INFINITY,
            exclusive: false,
        });
    }
    if value.eq_ignore_ascii_case(b"+inf") || value.eq_ignore_ascii_case(b"inf") {
        return Some(ScoreBound {
            value: f64::INFINITY,
            exclusive: false,
        });
    }
    let s = std::str::from_utf8(value).ok()?;
    let (exclusive, part) = if s.starts_with('(') {
        (true, &s[1..])
    } else {
        (false, s)
    };
    let v = part.parse::<f64>().ok()?;
    Some(ScoreBound {
        value: v,
        exclusive,
    })
}

fn score_in_bounds(score: i64, low: &ScoreBound, high: &ScoreBound) -> bool {
    let s = score as f64;
    let low_ok = if low.value.is_infinite() && low.value.is_sign_negative() {
        true
    } else if low.exclusive {
        s > low.value
    } else {
        s >= low.value
    };
    let high_ok = if high.value.is_infinite() && high.value.is_sign_positive() {
        true
    } else if high.exclusive {
        s < high.value
    } else {
        s <= high.value
    };
    low_ok && high_ok
}

fn remove_zset_members(
    zset: &mut std::collections::BTreeMap<Vec<u8>, i64>,
    members: &[Vec<u8>],
) -> usize {
    let mut removed = 0usize;
    for m in members {
        if zset.remove(m).is_some() {
            removed += 1;
        }
    }
    removed
}

fn parse_lex_bound(value: &[u8]) -> Option<(Vec<u8>, bool, bool)> {
    // returns (value, inclusive, is_inf)
    if value == b"-" {
        return Some((Vec::new(), true, true));
    }
    if value == b"+" {
        return Some((Vec::new(), true, true));
    }
    let s = std::str::from_utf8(value).ok()?;
    let (inclusive, part) = if s.starts_with('(') {
        (false, &s[1..])
    } else if s.starts_with('[') {
        (true, &s[1..])
    } else {
        (true, s)
    };
    Some((part.as_bytes().to_vec(), inclusive, false))
}

fn lex_in_bounds(member: &[u8], min: &(Vec<u8>, bool, bool), max: &(Vec<u8>, bool, bool)) -> bool {
    // if min or max is inf marker, treat accordingly
    if min.2 == true || max.2 == true {
        // '-' or '+' marker handling: '-' means -inf, '+' means +inf
    }
    let min_ok = if min.2 && min.0.is_empty() {
        true
    } else {
        if min.1 {
            member >= &min.0
        } else {
            member > &min.0
        }
    };
    let max_ok = if max.2 && max.0.is_empty() {
        true
    } else {
        if max.1 {
            member <= &max.0
        } else {
            member < &max.0
        }
    };
    min_ok && max_ok
}

fn scan_zset_entries(
    zset: &std::collections::BTreeMap<Vec<u8>, i64>,
    cursor: usize,
    count: Option<usize>,
) -> RespReply {
    let mut entries: Vec<&Vec<u8>> = zset.keys().collect();
    entries.sort_by(|a, b| a.cmp(b));
    let total = entries.len();
    if total == 0 {
        return RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(Vec::new()),
        ]);
    }
    let start = if cursor >= total { 0usize } else { cursor };
    let cnt = count.unwrap_or(10usize);
    // Return member/score pairs like real Redis: inner array contains
    // [member, score, member, score, ...]
    let mut result: Vec<RespReply> = Vec::new();
    for i in 0..cnt {
        if start + i >= total {
            break;
        }
        let member = entries[start + i];
        let score = zset.get(member).copied().unwrap_or(0);
        result.push(RespReply::BulkString(member.to_vec()));
        result.push(RespReply::BulkString(score.to_string().into_bytes()));
    }
    let next = if start + cnt >= total {
        0usize
    } else {
        start + cnt
    };
    RespReply::Array(vec![
        RespReply::BulkString(next.to_string().into_bytes()),
        RespReply::Array(result),
    ])
}

fn parse_database_index(value: &[u8]) -> Option<usize> {
    parse_scan_index(value)
}

fn integer_error() -> RespReply {
    RespReply::Error("ERR value is not an integer or out of range".to_string())
}

fn syntax_error() -> RespReply {
    RespReply::Error("ERR syntax error".to_string())
}

fn invalid_expire_time() -> RespReply {
    RespReply::Error("ERR invalid expire time".to_string())
}

fn out_of_range() -> RespReply {
    RespReply::Error("ERR index out of range".to_string())
}

fn normalize_index(len: usize, index: i64) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let len = len as i64;
    let index = if index < 0 { len + index } else { index };
    if index < 0 || index >= len {
        None
    } else {
        Some(index as usize)
    }
}

fn parse_list_side(value: &[u8]) -> Option<ListSide> {
    if value.eq_ignore_ascii_case(b"LEFT") {
        Some(ListSide::Left)
    } else if value.eq_ignore_ascii_case(b"RIGHT") {
        Some(ListSide::Right)
    } else {
        None
    }
}

fn parse_blocking_timeout(value: &[u8]) -> Result<(), RespReply> {
    // Accept non-negative finite timeouts (integer or float). Negative or parse errors are invalid.
    let timeout = std::str::from_utf8(value)
        .ok()
        .and_then(|text| text.parse::<f64>().ok())
        .filter(|t| t.is_finite());

    match timeout {
        Some(t) if t >= 0.0 => Ok(()),
        _ => Err(integer_error()),
    }
}

fn pop_list_value(list: &mut Vec<Vec<u8>>, side: ListSide) -> Option<Vec<u8>> {
    match side {
        ListSide::Left => {
            if list.is_empty() {
                None
            } else {
                Some(list.remove(0))
            }
        }
        ListSide::Right => list.pop(),
    }
}

fn push_list_value(list: &mut Vec<Vec<u8>>, side: ListSide, value: Vec<u8>) {
    match side {
        ListSide::Left => list.insert(0, value),
        ListSide::Right => list.push(value),
    }
}

fn remove_list_elements(list: &mut Vec<Vec<u8>>, count: i64, element: &[u8]) -> usize {
    let mut removed = 0usize;
    if count >= 0 {
        let limit = count as usize;
        let mut index = 0;
        while index < list.len() {
            if list[index] == element && (limit == 0 || removed < limit) {
                list.remove(index);
                removed += 1;
                if limit != 0 && removed == limit {
                    break;
                }
            } else {
                index += 1;
            }
        }
    } else {
        let limit = count.unsigned_abs() as usize;
        let mut index = list.len();
        while index > 0 {
            index -= 1;
            if list[index] == element {
                list.remove(index);
                removed += 1;
                if removed == limit {
                    break;
                }
            }
        }
    }
    removed
}

fn scan_hash_entries(
    hash: &BTreeMap<Vec<u8>, Vec<u8>>,
    cursor: usize,
    count: Option<usize>,
) -> RespReply {
    if cursor > hash.len() {
        return RespReply::Error("ERR invalid cursor".to_string());
    }

    let end = match count {
        Some(count) => cursor.saturating_add(count).min(hash.len()),
        None => hash.len(),
    };
    let next_cursor = if end < hash.len() { end } else { 0 };
    let mut field_values = Vec::new();
    for (field, value) in hash.iter().skip(cursor).take(end - cursor) {
        field_values.push(RespReply::BulkString(field.to_vec()));
        field_values.push(RespReply::BulkString(value.to_vec()));
    }

    RespReply::Array(vec![
        RespReply::BulkString(next_cursor.to_string().into_bytes()),
        RespReply::Array(field_values),
    ])
}

fn scan_set_members(set: &BTreeSet<Vec<u8>>, cursor: usize, count: Option<usize>) -> RespReply {
    if cursor > set.len() {
        return RespReply::Error("ERR invalid cursor".to_string());
    }

    let end = match count {
        Some(count) => cursor.saturating_add(count).min(set.len()),
        None => set.len(),
    };
    let next_cursor = if end < set.len() { end } else { 0 };
    let members = set
        .iter()
        .skip(cursor)
        .take(end - cursor)
        .map(|member| RespReply::BulkString(member.to_vec()))
        .collect();

    RespReply::Array(vec![
        RespReply::BulkString(next_cursor.to_string().into_bytes()),
        RespReply::Array(members),
    ])
}

fn parse_set_options(args: &[Vec<u8>]) -> Result<SetOptions, RespReply> {
    let mut options = SetOptions {
        condition: SetCondition::Always,
        get: false,
        expiration: None,
    };
    let mut index = 0;

    while index < args.len() {
        let option = &args[index];
        if option.eq_ignore_ascii_case(b"NX") {
            if options.condition != SetCondition::Always {
                return Err(syntax_error());
            }
            options.condition = SetCondition::OnlyIfAbsent;
            index += 1;
        } else if option.eq_ignore_ascii_case(b"XX") {
            if options.condition != SetCondition::Always {
                return Err(syntax_error());
            }
            options.condition = SetCondition::OnlyIfPresent;
            index += 1;
        } else if option.eq_ignore_ascii_case(b"GET") {
            if options.get {
                return Err(syntax_error());
            }
            options.get = true;
            index += 1;
        } else if option.eq_ignore_ascii_case(b"EX") || option.eq_ignore_ascii_case(b"PX") {
            if options.expiration.is_some() || index + 1 >= args.len() {
                return Err(syntax_error());
            }
            let value = match parse_integer(&args[index + 1]) {
                Some(value) if value > 0 => value as u64,
                Some(_) => return Err(invalid_expire_time()),
                None => return Err(integer_error()),
            };
            options.expiration = if option.eq_ignore_ascii_case(b"EX") {
                Some(Duration::from_secs(value))
            } else {
                Some(Duration::from_millis(value))
            };
            index += 2;
        } else {
            return Err(syntax_error());
        }
    }

    Ok(options)
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

fn resolve_xadd_id(
    requested_id: &[u8],
    stream: &BTreeMap<(u64, u64), StreamEntry>,
) -> Option<((u64, u64), Vec<u8>)> {
    if requested_id == b"*" {
        let next_id = match stream.keys().next_back().copied() {
            Some((milliseconds, sequence)) => (milliseconds, sequence.checked_add(1)?),
            None => (1, 0),
        };
        return Some((next_id, format!("{}-{}", next_id.0, next_id.1).into_bytes()));
    }

    let parsed = parse_stream_id(requested_id)?;
    Some((parsed, requested_id.to_vec()))
}

fn stream_entry_reply(entry: &StreamEntry) -> RespReply {
    let mut field_values = Vec::with_capacity(entry.fields.len() * 2);
    for (field, value) in &entry.fields {
        field_values.push(RespReply::BulkString(field.to_vec()));
        field_values.push(RespReply::BulkString(value.to_vec()));
    }
    RespReply::Array(vec![
        RespReply::BulkString(entry.id.to_vec()),
        RespReply::Array(field_values),
    ])
}

fn stream_range_reply(
    stream: &BTreeMap<(u64, u64), StreamEntry>,
    start: (u64, u64),
    end: (u64, u64),
    count: Option<usize>,
) -> Vec<RespReply> {
    let entries = stream
        .range(start..=end)
        .map(|(_id, entry)| stream_entry_reply(entry));
    match count {
        Some(count) => entries.take(count).collect(),
        None => entries.collect(),
    }
}

fn stream_after_reply(
    stream: &BTreeMap<(u64, u64), StreamEntry>,
    start_after: (u64, u64),
    count: Option<usize>,
) -> Vec<RespReply> {
    let entries = stream
        .range((start_after.0, start_after.1.saturating_add(1))..)
        .map(|(_id, entry)| stream_entry_reply(entry));
    match count {
        Some(count) => entries.take(count).collect(),
        None => entries.collect(),
    }
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

fn parse_stream_group_id(value: &[u8]) -> Option<(u64, u64)> {
    if value == b"$" {
        Some((u64::MAX, u64::MAX))
    } else {
        parse_stream_id(value)
    }
}

fn optional_stream_id_reply(id: Option<(u64, u64)>) -> RespReply {
    match id {
        Some(id) => RespReply::BulkString(format!("{}-{}", id.0, id.1).into_bytes()),
        None => RespReply::NullBulkString,
    }
}

fn no_such_consumer_group() -> RespReply {
    RespReply::Error("NOGROUP No such key or consumer group".to_string())
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

fn encode_array(values: &[RespReply], protocol_version: RespProtocolVersion) -> Vec<u8> {
    let mut encoded = format!("*{}\r\n", values.len()).into_bytes();
    for value in values {
        encoded.extend_from_slice(&value.encode_with_protocol(protocol_version));
    }
    encoded
}

fn encode_error(message: &str) -> Vec<u8> {
    encode_prefixed_string(b'-', message.as_bytes())
}
