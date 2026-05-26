use rust_port::{Command, ParseOutcome, RedisMiniDb, RespCommandParser, RespError, RespReply};

fn assert_incomplete(parser: &mut RespCommandParser) {
    assert_eq!(
        parser.parse_available().expect("valid partial frame"),
        ParseOutcome::Incomplete
    );
}

fn parse_complete(frame: &[u8]) -> Command {
    let mut parser = RespCommandParser::new();
    parser.append(frame);
    match parser.parse_available().expect("valid frame") {
        ParseOutcome::Complete(command) => command,
        ParseOutcome::Incomplete => panic!("expected complete command"),
    }
}

fn parse_next_complete(parser: &mut RespCommandParser) -> Command {
    match parser.parse_available().expect("valid buffered frame") {
        ParseOutcome::Complete(command) => command,
        ParseOutcome::Incomplete => panic!("expected complete command"),
    }
}

fn multibulk_frame(args: &[&[u8]]) -> Vec<u8> {
    let mut frame = format!("*{}\r\n", args.len()).into_bytes();
    for arg in args {
        frame.extend_from_slice(format!("${}\r\n", arg.len()).as_bytes());
        frame.extend_from_slice(arg);
        frame.extend_from_slice(b"\r\n");
    }
    frame
}

fn assert_parse_error(frame: &[u8], expected: RespError) {
    let mut parser = RespCommandParser::new();
    parser.append(frame);
    assert_eq!(parser.parse_available(), Err(expected));
}

fn command(args: &[&[u8]]) -> Command {
    Command::new(args.iter().map(|arg| arg.to_vec()).collect())
}

fn execute(db: &mut RedisMiniDb, args: &[&[u8]]) -> RespReply {
    db.execute(command(args))
}

#[test]
fn encodes_resp_replies() {
    assert_eq!(RespReply::SimpleString("OK").encode(), b"+OK\r\n".to_vec());
    assert_eq!(
        RespReply::BulkString(b"hello\0world".to_vec()).encode(),
        b"$11\r\nhello\0world\r\n".to_vec()
    );
    assert_eq!(RespReply::NullBulkString.encode(), b"$-1\r\n".to_vec());
    assert_eq!(RespReply::Integer(42).encode(), b":42\r\n".to_vec());
    assert_eq!(
        RespReply::Array(vec![
            RespReply::BulkString(b"one".to_vec()),
            RespReply::BulkString(b"two".to_vec()),
        ])
        .encode(),
        b"*2\r\n$3\r\none\r\n$3\r\ntwo\r\n".to_vec()
    );
    assert_eq!(
        RespReply::Error("ERR wrong number of arguments".to_string()).encode(),
        b"-ERR wrong number of arguments\r\n".to_vec()
    );
}

#[test]
fn executes_ping_and_echo() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"PING"]),
        RespReply::SimpleString("PONG")
    );
    assert_eq!(
        execute(&mut db, &[b"pInG", b"hello"]),
        RespReply::BulkString(b"hello".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"ECHO", b"hello world"]),
        RespReply::BulkString(b"hello world".to_vec())
    );
}

#[test]
fn executes_set_get_del_and_exists() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"GET", b"missing"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"key", b"value\0bytes"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::BulkString(b"value\0bytes".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"EXISTS", b"missing", b"key"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"DEL", b"missing", b"key"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::NullBulkString
    );
}

#[test]
fn executes_incr_decr_and_incrby_on_missing_keys() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"INCR", b"counter"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"counter"]),
        RespReply::BulkString(b"1".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"DECR", b"debits"]),
        RespReply::Integer(-1)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"debits"]),
        RespReply::BulkString(b"-1".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"INCRBY", b"steps", b"42"]),
        RespReply::Integer(42)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"steps"]),
        RespReply::BulkString(b"42".to_vec())
    );
}

#[test]
fn executes_incr_decr_and_incrby_on_existing_integer_values() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"counter", b"10"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"INCR", b"counter"]),
        RespReply::Integer(11)
    );
    assert_eq!(
        execute(&mut db, &[b"DECR", b"counter"]),
        RespReply::Integer(10)
    );
    assert_eq!(
        execute(&mut db, &[b"INCRBY", b"counter", b"-3"]),
        RespReply::Integer(7)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"counter"]),
        RespReply::BulkString(b"7".to_vec())
    );
}

#[test]
fn rejects_non_integer_existing_values_and_increment_arguments() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"counter", b"not-an-int"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"INCR", b"counter"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"counter"]),
        RespReply::BulkString(b"not-an-int".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"INCRBY", b"other", b"nope"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"other"]),
        RespReply::NullBulkString
    );
}

#[test]
fn rejects_integer_overflow_and_preserves_stored_values() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"max", b"9223372036854775807"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"INCR", b"max"]),
        RespReply::Error("ERR increment or decrement would overflow".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"max"]),
        RespReply::BulkString(b"9223372036854775807".to_vec())
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"min", b"-9223372036854775808"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"DECR", b"min"]),
        RespReply::Error("ERR increment or decrement would overflow".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"min"]),
        RespReply::BulkString(b"-9223372036854775808".to_vec())
    );

    assert_eq!(
        execute(&mut db, &[b"INCRBY", b"max", b"1"]),
        RespReply::Error("ERR increment or decrement would overflow".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"max"]),
        RespReply::BulkString(b"9223372036854775807".to_vec())
    );
}

#[test]
fn executes_lpush_and_lpop_on_missing_and_existing_lists() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"LPOP", b"missing"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"LPUSH", b"letters", b"a", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"LPUSH", b"letters", b"c"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"LPOP", b"letters"]),
        RespReply::BulkString(b"c".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LPOP", b"letters"]),
        RespReply::BulkString(b"b".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LPOP", b"letters"]),
        RespReply::BulkString(b"a".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LPOP", b"letters"]),
        RespReply::NullBulkString
    );
}

#[test]
fn executes_rpush_and_rpop_on_missing_and_existing_lists() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"RPOP", b"missing"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"letters", b"a", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"letters", b"c"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"RPOP", b"letters"]),
        RespReply::BulkString(b"c".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"RPOP", b"letters"]),
        RespReply::BulkString(b"b".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"RPOP", b"letters"]),
        RespReply::BulkString(b"a".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"RPOP", b"letters"]),
        RespReply::NullBulkString
    );
}

#[test]
fn executes_lrange_with_positive_and_negative_indexes() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"missing", b"0", b"-1"]),
        RespReply::Array(Vec::new())
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"letters", b"a", b"b", b"c", b"d"]),
        RespReply::Integer(4)
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"letters", b"0", b"2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"b".to_vec()),
            RespReply::BulkString(b"c".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"letters", b"-2", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"c".to_vec()),
            RespReply::BulkString(b"d".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"letters", b"3", b"1"]),
        RespReply::Array(Vec::new())
    );
}

#[test]
fn executes_hset_hget_and_hdel_on_missing_and_existing_hashes() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"HGET", b"missing", b"field"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"field", b"value"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HGET", b"hash", b"field"]),
        RespReply::BulkString(b"value".to_vec())
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"HSET", b"hash", b"field", b"new", b"other", b"value2"]
        ),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HGET", b"hash", b"field"]),
        RespReply::BulkString(b"new".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"HDEL", b"hash", b"missing", b"other"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HGET", b"hash", b"other"]),
        RespReply::NullBulkString
    );
}

#[test]
fn executes_hgetall_with_binary_safe_fields_and_values() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"HGETALL", b"missing"]),
        RespReply::Array(Vec::new())
    );
    assert_eq!(
        execute(
            &mut db,
            &[
                b"HSET",
                b"hash",
                b"field\0two",
                b"value\0two",
                b"field one",
                b"value one",
            ]
        ),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"HGET", b"hash", b"field\0two"]),
        RespReply::BulkString(b"value\0two".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"HGETALL", b"hash"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"field\0two".to_vec()),
            RespReply::BulkString(b"value\0two".to_vec()),
            RespReply::BulkString(b"field one".to_vec()),
            RespReply::BulkString(b"value one".to_vec()),
        ])
    );
}

#[test]
fn executes_expire_ttl_and_persist() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"TTL", b"missing"]),
        RespReply::Integer(-2)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"missing", b"10"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"PERSIST", b"missing"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"key", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"key"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"key", b"10"]),
        RespReply::Integer(1)
    );
    match execute(&mut db, &[b"TTL", b"key"]) {
        RespReply::Integer(ttl) => assert!((0..=10).contains(&ttl)),
        reply => panic!("expected integer ttl, got {reply:?}"),
    }
    assert_eq!(
        execute(&mut db, &[b"PERSIST", b"key"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"key"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"PERSIST", b"key"]),
        RespReply::Integer(0)
    );
}

#[test]
fn immediate_expiration_removes_string_list_and_hash_values() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"string", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"string"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"TTL", b"string"]),
        RespReply::Integer(-2)
    );

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"list", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"list", b"0", b"-1"]),
        RespReply::Array(Vec::new())
    );

    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"field", b"value"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"hash", b"-1"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HGET", b"hash", b"field"]),
        RespReply::NullBulkString
    );
}

#[test]
fn writes_clear_existing_expiration() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"string", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"INCR", b"string"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"TTL", b"string"]),
        RespReply::Integer(-1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"string", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"new"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"TTL", b"string"]),
        RespReply::Integer(-1)
    );

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"list", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"LPUSH", b"list", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"list"]), RespReply::Integer(-1));

    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"field", b"value"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"hash", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"field", b"new"]),
        RespReply::Integer(0)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"hash"]), RespReply::Integer(-1));
}

#[test]
fn executes_sadd_srem_and_sismember_on_missing_and_existing_sets() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SISMEMBER", b"missing", b"a"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"a", b"b", b"a"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"b", b"c"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SISMEMBER", b"set", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SISMEMBER", b"set", b"missing"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"SREM", b"set", b"missing", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SISMEMBER", b"set", b"a"]),
        RespReply::Integer(0)
    );
}

#[test]
fn executes_smembers_with_deterministic_order_and_binary_safe_members() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"missing"]),
        RespReply::Array(Vec::new())
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"z", b"a\0member", b"a member"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"set"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a\0member".to_vec()),
            RespReply::BulkString(b"a member".to_vec()),
            RespReply::BulkString(b"z".to_vec()),
        ])
    );
}

#[test]
fn set_writes_clear_existing_expiration_and_set_reads_observe_expiration() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"set", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"b"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"set"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"set", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SISMEMBER", b"set", b"a"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"set"]),
        RespReply::Array(Vec::new())
    );
}

#[test]
fn sunionstore_stores_union_from_existing_and_missing_sources() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SADD", b"a", b"one", b"two"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"b", b"two", b"three"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"out", b"a", b"missing", b"b"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"out"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"one".to_vec()),
            RespReply::BulkString(b"three".to_vec()),
            RespReply::BulkString(b"two".to_vec()),
        ])
    );
}

#[test]
fn set_store_commands_allow_destination_as_source() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SADD", b"dest", b"a", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"other", b"b", b"c"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"dest", b"dest", b"other"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"dest"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"b".to_vec()),
            RespReply::BulkString(b"c".to_vec()),
        ])
    );

    assert_eq!(
        execute(&mut db, &[b"SINTERSTORE", b"dest", b"dest", b"other"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"dest"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"b".to_vec()),
            RespReply::BulkString(b"c".to_vec()),
        ])
    );

    assert_eq!(
        execute(&mut db, &[b"SDIFFSTORE", b"dest", b"dest", b"other"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"EXISTS", b"dest"]),
        RespReply::Integer(0)
    );
}

#[test]
fn sinterstore_and_sdiffstore_store_expected_results() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SADD", b"a", b"one", b"two", b"three"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"b", b"two", b"three", b"four"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"c", b"three", b"four"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"SINTERSTORE", b"inter", b"a", b"b", b"c"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"inter"]),
        RespReply::Array(vec![RespReply::BulkString(b"three".to_vec())])
    );
    assert_eq!(
        execute(&mut db, &[b"SINTERSTORE", b"empty", b"a", b"missing", b"b"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"EXISTS", b"empty"]),
        RespReply::Integer(0)
    );

    assert_eq!(
        execute(&mut db, &[b"SDIFFSTORE", b"diff", b"a", b"b", b"c"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"diff"]),
        RespReply::Array(vec![RespReply::BulkString(b"one".to_vec())])
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"SDIFFSTORE", b"missing-first", b"missing", b"a"]
        ),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"EXISTS", b"missing-first"]),
        RespReply::Integer(0)
    );
}

#[test]
fn set_store_overwrites_destination_and_clears_expiration() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SADD", b"source", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"dest", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"dest", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"dest", b"source"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"dest"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"dest"]),
        RespReply::Array(vec![RespReply::BulkString(b"member".to_vec())])
    );

    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"dest", b"missing"]),
        RespReply::Integer(0)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"dest"]), RespReply::Integer(-2));
}

#[test]
fn set_store_commands_reject_wrong_type_sources_without_overwriting_destination() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"SADD", b"good", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"bad", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"dest", b"old"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"dest", b"good", b"bad"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SINTERSTORE", b"dest", b"good", b"bad"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SDIFFSTORE", b"dest", b"good", b"bad"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"dest"]),
        RespReply::Array(vec![RespReply::BulkString(b"old".to_vec())])
    );
}

#[test]
fn executes_type_across_supported_value_kinds() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"TYPE", b"missing"]),
        RespReply::SimpleString("none")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"field", b"value"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"member"]),
        RespReply::Integer(1)
    );

    assert_eq!(
        execute(&mut db, &[b"TYPE", b"string"]),
        RespReply::SimpleString("string")
    );
    assert_eq!(
        execute(&mut db, &[b"TYPE", b"list"]),
        RespReply::SimpleString("list")
    );
    assert_eq!(
        execute(&mut db, &[b"TYPE", b"hash"]),
        RespReply::SimpleString("hash")
    );
    assert_eq!(
        execute(&mut db, &[b"TYPE", b"set"]),
        RespReply::SimpleString("set")
    );
}

#[test]
fn rename_moves_values_across_supported_value_kinds() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"RENAME", b"string", b"string2"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"string"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"string2"]),
        RespReply::BulkString(b"value".to_vec())
    );

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"RENAME", b"list", b"list2"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"list2", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"b".to_vec())
        ])
    );

    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"field", b"value"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"RENAME", b"hash", b"hash2"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"HGET", b"hash2", b"field"]),
        RespReply::BulkString(b"value".to_vec())
    );

    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"b", b"a"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"RENAME", b"set", b"set2"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SMEMBERS", b"set2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"b".to_vec())
        ])
    );
}

#[test]
fn rename_overwrites_destination_and_moves_expiration_metadata() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"source", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"source", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"destination", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"destination", b"20"]),
        RespReply::Integer(1)
    );

    assert_eq!(
        execute(&mut db, &[b"RENAME", b"source", b"destination"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"source"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"destination"]),
        RespReply::BulkString(b"value".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"TTL", b"source"]),
        RespReply::Integer(-2)
    );
    match execute(&mut db, &[b"TTL", b"destination"]) {
        RespReply::Integer(ttl) => assert!((0..=10).contains(&ttl)),
        reply => panic!("expected integer ttl, got {reply:?}"),
    }
}

#[test]
fn renamenx_moves_only_to_absent_destinations() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"source", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"destination", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"RENAMENX", b"source", b"destination"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"source"]),
        RespReply::BulkString(b"value".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"destination"]),
        RespReply::BulkString(b"old".to_vec())
    );

    assert_eq!(
        execute(&mut db, &[b"RENAMENX", b"source", b"moved"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"source"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"moved"]),
        RespReply::BulkString(b"value".to_vec())
    );
}

#[test]
fn keyspace_commands_observe_lazy_expiration() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"expired", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"expired", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"TYPE", b"expired"]),
        RespReply::SimpleString("none")
    );
    assert_eq!(
        execute(&mut db, &[b"RENAME", b"expired", b"moved"]),
        RespReply::Error("ERR no such key".to_string())
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"source", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"destination", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"destination", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"RENAMENX", b"source", b"destination"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"destination"]),
        RespReply::BulkString(b"value".to_vec())
    );
}

#[test]
fn keys_star_returns_deterministic_current_key_names() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"z", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"field", b"value"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"gone", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"gone", b"0"]),
        RespReply::Integer(1)
    );

    assert_eq!(
        execute(&mut db, &[b"KEYS", b"*"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"hash".to_vec()),
            RespReply::BulkString(b"list".to_vec()),
            RespReply::BulkString(b"set".to_vec()),
            RespReply::BulkString(b"z".to_vec()),
        ])
    );
}

#[test]
fn srem_removes_empty_set_key() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SREM", b"set", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXISTS", b"set"]),
        RespReply::Integer(0)
    );
}

#[test]
fn del_clears_expiration_metadata() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"key", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"key", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"DEL", b"key"]), RespReply::Integer(1));
    assert_eq!(
        execute(&mut db, &[b"PERSIST", b"key"]),
        RespReply::Integer(0)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"key"]), RespReply::Integer(-2));
}

#[test]
fn rejects_wrong_type_access_between_strings_and_lists() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"LPUSH", b"string", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"RPUSH", b"string", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"LPOP", b"string"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"RPOP", b"string"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"string", b"0", b"-1"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"GET", b"list"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"INCR", b"list"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SET", b"list", b"value"]), wrong_type);
}

#[test]
fn rejects_wrong_type_access_between_hashes_strings_and_lists() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"HGET", b"string", b"f"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"HSET", b"string", b"f", b"v"]),
        wrong_type
    );
    assert_eq!(execute(&mut db, &[b"HDEL", b"string", b"f"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HGETALL", b"string"]), wrong_type);

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"HGET", b"list", b"f"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"HSET", b"list", b"f", b"v"]),
        wrong_type
    );
    assert_eq!(execute(&mut db, &[b"HDEL", b"list", b"f"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HGETALL", b"list"]), wrong_type);

    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"f", b"v"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"GET", b"hash"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SET", b"hash", b"value"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"INCR", b"hash"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"LPUSH", b"hash", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"RPUSH", b"hash", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"LPOP", b"hash"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"RPOP", b"hash"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"hash", b"0", b"-1"]),
        wrong_type
    );
}

#[test]
fn rejects_wrong_type_access_between_sets_strings_lists_and_hashes() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"SADD", b"string", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SREM", b"string", b"x"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"SISMEMBER", b"string", b"x"]),
        wrong_type
    );
    assert_eq!(execute(&mut db, &[b"SMEMBERS", b"string"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"out", b"string"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SINTERSTORE", b"out", b"string"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SDIFFSTORE", b"out", b"string"]),
        wrong_type
    );

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"SADD", b"list", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SREM", b"list", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SISMEMBER", b"list", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SMEMBERS", b"list"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"out", b"list"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SINTERSTORE", b"out", b"list"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SDIFFSTORE", b"out", b"list"]),
        wrong_type
    );

    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"f", b"v"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"SADD", b"hash", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SREM", b"hash", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SISMEMBER", b"hash", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SMEMBERS", b"hash"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"out", b"hash"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SINTERSTORE", b"out", b"hash"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"SDIFFSTORE", b"out", b"hash"]),
        wrong_type
    );

    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"x"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"GET", b"set"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SET", b"set", b"value"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"INCR", b"set"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"LPUSH", b"set", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"RPUSH", b"set", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"LPOP", b"set"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"RPOP", b"set"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"set", b"0", b"-1"]),
        wrong_type
    );
    assert_eq!(execute(&mut db, &[b"HGET", b"set", b"f"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HSET", b"set", b"f", b"v"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HDEL", b"set", b"f"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HGETALL", b"set"]), wrong_type);
}

#[test]
fn returns_wrong_arity_and_unknown_command_errors() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"PING", b"one", b"two"]),
        RespReply::Error("ERR wrong number of arguments for 'ping' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"ECHO"]),
        RespReply::Error("ERR wrong number of arguments for 'echo' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"key"]),
        RespReply::Error("ERR wrong number of arguments for 'set' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"INCR"]),
        RespReply::Error("ERR wrong number of arguments for 'incr' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"INCRBY", b"key"]),
        RespReply::Error("ERR wrong number of arguments for 'incrby' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"NOPE"]),
        RespReply::Error("ERR unknown command 'NOPE'".to_string())
    );
}

#[test]
fn parses_ping_multibulk() {
    let command = parse_complete(b"*1\r\n$4\r\nPING\r\n");
    assert_eq!(command.args, vec![b"PING".to_vec()]);
}

#[test]
fn parses_get_key_multibulk() {
    let command = parse_complete(b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n");
    assert_eq!(command.args, vec![b"GET".to_vec(), b"key".to_vec()]);
}

#[test]
fn parses_set_key_value_multibulk() {
    let command = parse_complete(b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n");
    assert_eq!(
        command.args,
        vec![b"SET".to_vec(), b"key".to_vec(), b"value".to_vec()]
    );
}

#[test]
fn parses_binary_safe_bulk_string() {
    let command = parse_complete(b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$12\r\nhello \0world\r\n");
    assert_eq!(
        command.args,
        vec![b"SET".to_vec(), b"key".to_vec(), b"hello \0world".to_vec()]
    );
}

#[test]
fn parses_large_bulk_payload() {
    let payload = vec![b'x'; 64 * 1024];
    let frame = multibulk_frame(&[b"ECHO", payload.as_slice()]);

    let command = parse_complete(&frame);

    assert_eq!(command.args[0], b"ECHO".to_vec());
    assert_eq!(command.args[1].len(), payload.len());
    assert_eq!(command.args[1], payload);
}

#[test]
fn parses_ping_inline() {
    let command = parse_complete(b"PING\r\n");
    assert_eq!(command.args, vec![b"PING".to_vec()]);
}

#[test]
fn parses_set_key_value_inline() {
    let command = parse_complete(b"SET key value\r\n");
    assert_eq!(
        command.args,
        vec![b"SET".to_vec(), b"key".to_vec(), b"value".to_vec()]
    );
}

#[test]
fn parses_double_quoted_inline_value() {
    let command = parse_complete(b"SET key \"hello world\"\r\n");
    assert_eq!(
        command.args,
        vec![b"SET".to_vec(), b"key".to_vec(), b"hello world".to_vec()]
    );
}

#[test]
fn parses_single_quoted_inline_value() {
    let command = parse_complete(b"SET key 'hello world'\r\n");
    assert_eq!(
        command.args,
        vec![b"SET".to_vec(), b"key".to_vec(), b"hello world".to_vec()]
    );
}

#[test]
fn rejects_unbalanced_double_quote_inline_value() {
    assert_parse_error(b"SET key \"hello world\r\n", RespError::UnbalancedQuote);
}

#[test]
fn rejects_unbalanced_single_quote_inline_value() {
    assert_parse_error(b"SET key 'hello world\r\n", RespError::UnbalancedQuote);
}

#[test]
fn waits_for_command_split_across_appends() {
    let mut parser = RespCommandParser::new();

    parser.append(b"*2\r\n$3\r\nGET\r\n");
    assert_incomplete(&mut parser);

    parser.append(b"$3\r\nkey\r\n");
    let command = match parser.parse_available().expect("valid completed frame") {
        ParseOutcome::Complete(command) => command,
        ParseOutcome::Incomplete => panic!("expected complete command after final append"),
    };

    assert_eq!(command.args, vec![b"GET".to_vec(), b"key".to_vec()]);
}

#[test]
fn retains_incomplete_multibulk_length_state() {
    let mut parser = RespCommandParser::new();

    parser.append(b"*1");
    assert_incomplete(&mut parser);
    assert_incomplete(&mut parser);

    parser.append(b"\r\n$4\r\nPING\r\n");
    let command = match parser.parse_available().expect("valid completed frame") {
        ParseOutcome::Complete(command) => command,
        ParseOutcome::Incomplete => panic!("expected complete command after multibulk length"),
    };

    assert_eq!(command.args, vec![b"PING".to_vec()]);
}

#[test]
fn retains_incomplete_bulk_length_state() {
    let mut parser = RespCommandParser::new();

    parser.append(b"*1\r\n$4");
    assert_incomplete(&mut parser);
    assert_incomplete(&mut parser);

    parser.append(b"\r\nPING\r\n");
    let command = match parser.parse_available().expect("valid completed frame") {
        ParseOutcome::Complete(command) => command,
        ParseOutcome::Incomplete => panic!("expected complete command after bulk length"),
    };

    assert_eq!(command.args, vec![b"PING".to_vec()]);
}

#[test]
fn retains_incomplete_bulk_payload_state() {
    let mut parser = RespCommandParser::new();

    parser.append(b"*1\r\n$4\r\nPI");
    assert_incomplete(&mut parser);
    assert_incomplete(&mut parser);

    parser.append(b"NG\r\n");
    let command = match parser.parse_available().expect("valid completed frame") {
        ParseOutcome::Complete(command) => command,
        ParseOutcome::Incomplete => panic!("expected complete command after bulk payload"),
    };

    assert_eq!(command.args, vec![b"PING".to_vec()]);
}

#[test]
fn parses_multiple_complete_commands_from_one_buffer() {
    let mut parser = RespCommandParser::new();

    parser.append(b"*1\r\n$4\r\nPING\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n");

    let first = parse_next_complete(&mut parser);
    let second = parse_next_complete(&mut parser);

    assert_eq!(first.args, vec![b"PING".to_vec()]);
    assert_eq!(second.args, vec![b"GET".to_vec(), b"key".to_vec()]);
    assert_incomplete(&mut parser);
}

#[test]
fn keeps_incomplete_trailing_command_after_complete_command() {
    let mut parser = RespCommandParser::new();

    parser.append(b"*1\r\n$4\r\nPING\r\n*2\r\n$3\r\nGET\r\n$3\r\n");

    let first = parse_next_complete(&mut parser);
    assert_eq!(first.args, vec![b"PING".to_vec()]);
    assert_incomplete(&mut parser);

    parser.append(b"key\r\n");
    let second = parse_next_complete(&mut parser);

    assert_eq!(second.args, vec![b"GET".to_vec(), b"key".to_vec()]);
    assert_incomplete(&mut parser);
}

#[test]
fn compacts_after_large_argument_before_next_command() {
    let payload = vec![b'y'; 64 * 1024];
    let first_frame = multibulk_frame(&[b"ECHO", payload.as_slice()]);
    let second_frame = multibulk_frame(&[b"PING"]);
    let mut parser = RespCommandParser::new();

    parser.append(&first_frame);
    parser.append(&second_frame);

    let first = parse_next_complete(&mut parser);
    assert_eq!(first.args[0], b"ECHO".to_vec());
    assert_eq!(first.args[1], payload);
    assert_eq!(parser.buffer_len(), second_frame.len());

    let second = parse_next_complete(&mut parser);
    assert_eq!(second.args, vec![b"PING".to_vec()]);
    assert_eq!(parser.buffer_len(), 0);
}

#[test]
fn keeps_incomplete_trailing_command_after_large_command() {
    let payload = vec![b'z'; 64 * 1024];
    let first_frame = multibulk_frame(&[b"ECHO", payload.as_slice()]);
    let trailing = b"*2\r\n$3\r\nGET\r\n$3\r\n";
    let mut parser = RespCommandParser::new();

    parser.append(&first_frame);
    parser.append(trailing);

    let first = parse_next_complete(&mut parser);
    assert_eq!(first.args[0], b"ECHO".to_vec());
    assert_eq!(first.args[1], payload);
    assert_eq!(parser.buffer_len(), trailing.len());
    assert_incomplete(&mut parser);

    parser.append(b"key\r\n");
    let second = parse_next_complete(&mut parser);
    assert_eq!(second.args, vec![b"GET".to_vec(), b"key".to_vec()]);
    assert_eq!(parser.buffer_len(), 0);
}

#[test]
fn rejects_invalid_multibulk_length_non_digits() {
    assert_parse_error(b"*abc\r\n", RespError::InvalidMultibulkLength);
}

#[test]
fn rejects_zero_and_negative_multibulk_length() {
    assert_parse_error(b"*0\r\n", RespError::InvalidMultibulkLength);
    assert_parse_error(b"*-1\r\n", RespError::InvalidMultibulkLength);
}

#[test]
fn rejects_invalid_bulk_length_non_digits() {
    assert_parse_error(b"*1\r\n$abc\r\n", RespError::InvalidBulkLength);
}

#[test]
fn rejects_negative_bulk_length() {
    assert_parse_error(b"*1\r\n$-1\r\n", RespError::InvalidBulkLength);
}

#[test]
fn rejects_missing_bulk_string_marker() {
    assert_parse_error(b"*1\r\n+4\r\nPING\r\n", RespError::ExpectedBulkString);
}

#[test]
fn rejects_overlarge_inline_header() {
    let mut parser = RespCommandParser::with_max_line_length(4);
    parser.append(b"PINGX");

    assert_eq!(parser.parse_available(), Err(RespError::LineTooLong));
}

#[test]
fn rejects_overlarge_multibulk_header() {
    let mut parser = RespCommandParser::with_max_line_length(2);
    parser.append(b"*123");

    assert_eq!(parser.parse_available(), Err(RespError::LineTooLong));
}
