use rust_port::{
    Command, CommandCategory, CommandMetadata, ParseOutcome, RedisMiniDb, RedisMiniServer,
    RedisMiniSession, RespCommandParser, RespError, RespProtocolVersion, RespReply,
    command_metadata, normalize_command_name,
};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

fn assert_incomplete(parser: &mut RespCommandParser) {
    assert_eq!(
        parser.parse_available().expect("valid partial frame"),
        ParseOutcome::Incomplete
    );
}

#[test]
fn executes_getrange_and_setrange_with_binary_safe_ranges() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"range", b"abcdef\0ghi"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GETRANGE", b"range", b"1", b"-2"]),
        RespReply::BulkString(b"bcdef\0gh".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"GETRANGE", b"range", b"20", b"30"]),
        RespReply::BulkString(Vec::new())
    );
    assert_eq!(
        execute(&mut db, &[b"GETRANGE", b"missing", b"0", b"-1"]),
        RespReply::BulkString(Vec::new())
    );

    assert_eq!(
        execute(&mut db, &[b"SETRANGE", b"pad", b"3", b"A\0B"]),
        RespReply::Integer(6)
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"pad"]),
        RespReply::BulkString(b"\0\0\0A\0B".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"SETRANGE", b"pad", b"1", b"xy"]),
        RespReply::Integer(6)
    );
    assert_eq!(
        execute(&mut db, &[b"GETRANGE", b"pad", b"0", b"-1"]),
        RespReply::BulkString(b"\0xyA\0B".to_vec())
    );
}

#[test]
fn executes_set_options_for_conditions_get_and_expiration() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"k", b"one", b"EX", b"10", b"GET"]),
        RespReply::NullBulkString
    );
    match execute(&mut db, &[b"TTL", b"k"]) {
        RespReply::Integer(ttl) => assert!((0..=10).contains(&ttl)),
        reply => panic!("expected integer ttl, got {reply:?}"),
    }
    assert_eq!(
        execute(&mut db, &[b"SET", b"k", b"two", b"NX", b"GET"]),
        RespReply::BulkString(b"one".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"k"]),
        RespReply::BulkString(b"one".to_vec())
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"SET", b"k", b"two", b"XX", b"GET", b"PX", b"10000"]
        ),
        RespReply::BulkString(b"one".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"k"]),
        RespReply::BulkString(b"two".to_vec())
    );
    match execute(&mut db, &[b"TTL", b"k"]) {
        RespReply::Integer(ttl) => assert!((0..=10).contains(&ttl)),
        reply => panic!("expected integer ttl, got {reply:?}"),
    }

    assert_eq!(
        execute(&mut db, &[b"SET", b"missing", b"v", b"XX", b"GET"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"missing"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"new", b"v", b"NX"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"new", b"again", b"NX"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"new", b"again", b"XX"]),
        RespReply::SimpleString("OK")
    );
}

#[test]
fn string_completion_commands_reject_invalid_range_and_set_options() {
    let mut db = RedisMiniDb::new();
    let syntax = RespReply::Error("ERR syntax error".to_string());

    assert_eq!(
        execute(&mut db, &[b"GETRANGE", b"key", b"start", b"0"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SETRANGE", b"key", b"-1", b"value"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"SETRANGE", b"key", b"18446744073709551615", b"x"]
        ),
        RespReply::Error("ERR string exceeds maximum allowed size".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"k", b"v", b"NX", b"XX"]),
        syntax
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"k", b"v", b"GET", b"GET"]),
        syntax
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"k", b"v", b"EX", b"1", b"PX", b"1"]),
        syntax
    );
    assert_eq!(execute(&mut db, &[b"SET", b"k", b"v", b"PX"]), syntax);
    assert_eq!(execute(&mut db, &[b"SET", b"k", b"v", b"NOPE"]), syntax);
    assert_eq!(
        execute(&mut db, &[b"SET", b"k", b"v", b"EX", b"0"]),
        RespReply::Error("ERR invalid expire time".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"k", b"v", b"PX", b"not-int"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
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

fn tcp_exchange(input: &[u8]) -> Vec<u8> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
    let addr = listener.local_addr().expect("test listener address");
    let server = thread::spawn(move || RedisMiniServer::serve_listener(listener, 1));

    let mut stream = TcpStream::connect(addr).expect("connect to test server");
    stream.write_all(input).expect("write request");
    stream
        .shutdown(Shutdown::Write)
        .expect("close request side");

    let mut output = Vec::new();
    stream.read_to_end(&mut output).expect("read response");
    server
        .join()
        .expect("server thread finished")
        .expect("server ok");
    output
}

fn hello3_response() -> Vec<u8> {
    b"*4\r\n$6\r\nserver\r\n$10\r\nredis-mini\r\n$5\r\nproto\r\n:3\r\n".to_vec()
}

#[test]
fn tcp_server_replies_to_ping() {
    assert_eq!(
        tcp_exchange(&multibulk_frame(&[b"PING"])),
        b"+PONG\r\n".to_vec()
    );
}

#[test]
fn tcp_server_keeps_session_state_for_set_and_get() {
    let mut input = multibulk_frame(&[b"SET", b"key", b"value"]);
    input.extend(multibulk_frame(&[b"GET", b"key"]));

    assert_eq!(tcp_exchange(&input), b"+OK\r\n$5\r\nvalue\r\n".to_vec());
}

#[test]
fn tcp_server_handles_pipelined_commands() {
    let mut input = multibulk_frame(&[b"PING"]);
    input.extend(multibulk_frame(&[b"PING", b"hello"]));
    input.extend(multibulk_frame(&[b"GET", b"missing"]));

    assert_eq!(
        tcp_exchange(&input),
        b"+PONG\r\n$5\r\nhello\r\n$-1\r\n".to_vec()
    );
}

#[test]
fn tcp_server_negotiates_resp3_nulls_per_connection() {
    let mut input = multibulk_frame(&[b"HELLO", b"3"]);
    input.extend(multibulk_frame(&[b"GET", b"missing"]));

    let mut expected = hello3_response();
    expected.extend(b"_\r\n");
    assert_eq!(tcp_exchange(&input), expected);
}

#[test]
fn normalizes_known_command_names_without_allocating_errors_for_unknowns() {
    assert_eq!(normalize_command_name(b"ping"), Some("PING"));
    assert_eq!(normalize_command_name(b"pInG"), Some("PING"));
    assert_eq!(normalize_command_name(b"sunionstore"), Some("SUNIONSTORE"));
    assert_eq!(normalize_command_name(b"scan"), Some("SCAN"));
    assert_eq!(normalize_command_name(b"NOPE"), None);
}

#[test]
fn exposes_command_category_metadata_for_implemented_commands() {
    assert_eq!(
        command_metadata(b"PING"),
        Some(CommandMetadata {
            name: "PING",
            category: CommandCategory::Connection,
        })
    );
    assert_eq!(
        command_metadata(b"set").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"mget").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"mset").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"append").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"strlen").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"getrange").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"setrange").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"getset").unwrap().category.as_str(),
        "string"
    );
    assert_eq!(
        command_metadata(b"lpush").unwrap().category.as_str(),
        "list"
    );
    assert_eq!(
        command_metadata(b"llen").unwrap(),
        CommandMetadata {
            name: "LLEN",
            category: CommandCategory::List,
        }
    );
    assert_eq!(
        command_metadata(b"lmove").unwrap().category.as_str(),
        "list"
    );
    assert_eq!(
        command_metadata(b"blpop").unwrap(),
        CommandMetadata {
            name: "BLPOP",
            category: CommandCategory::List,
        }
    );
    assert_eq!(
        command_metadata(b"brpop").unwrap().category.as_str(),
        "list"
    );
    assert_eq!(
        command_metadata(b"blmove").unwrap().category.as_str(),
        "list"
    );
    assert_eq!(
        command_metadata(b"hgetall").unwrap().category.as_str(),
        "hash"
    );
    assert_eq!(command_metadata(b"sadd").unwrap().category.as_str(), "set");
    assert_eq!(
        command_metadata(b"zrange").unwrap().category.as_str(),
        "sorted-set"
    );
    assert_eq!(
        command_metadata(b"xadd").unwrap().category.as_str(),
        "stream"
    );
    assert_eq!(
        command_metadata(b"scan").unwrap().category.as_str(),
        "keyspace"
    );
    assert_eq!(
        command_metadata(b"select").unwrap().category.as_str(),
        "connection"
    );
    assert_eq!(
        command_metadata(b"dbsize").unwrap().category.as_str(),
        "keyspace"
    );
    assert_eq!(
        command_metadata(b"hello").unwrap().category.as_str(),
        "connection"
    );
    assert_eq!(
        command_metadata(b"multi").unwrap().category.as_str(),
        "transaction"
    );
    assert_eq!(command_metadata(b"unknown"), None);
}

#[test]
fn central_dispatcher_preserves_unknown_arity_and_transaction_queue_behavior() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"PING", b"one", b"two"]),
        RespReply::Error("ERR wrong number of arguments for 'ping' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"NOPE"]),
        RespReply::Error("ERR unknown command 'NOPE'".to_string())
    );

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"PING"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"NOPE"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::SimpleString("PONG"),
            RespReply::Error("ERR unknown command 'NOPE'".to_string()),
        ])
    );
}

#[test]
fn encodes_resp_replies() {
    assert_eq!(RespReply::SimpleString("OK").encode(), b"+OK\r\n".to_vec());
    assert_eq!(
        RespReply::BulkString(b"hello\0world".to_vec()).encode(),
        b"$11\r\nhello\0world\r\n".to_vec()
    );
    assert_eq!(RespReply::NullBulkString.encode(), b"$-1\r\n".to_vec());
    assert_eq!(RespReply::NullArray.encode(), b"*-1\r\n".to_vec());
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
fn resp3_encoding_preserves_existing_frames_except_nulls() {
    assert_eq!(
        RespReply::SimpleString("OK").encode_with_protocol(RespProtocolVersion::Resp3),
        b"+OK\r\n".to_vec()
    );
    assert_eq!(
        RespReply::BulkString(b"hello".to_vec()).encode_with_protocol(RespProtocolVersion::Resp3),
        b"$5\r\nhello\r\n".to_vec()
    );
    assert_eq!(
        RespReply::Integer(42).encode_with_protocol(RespProtocolVersion::Resp3),
        b":42\r\n".to_vec()
    );
    assert_eq!(
        RespReply::Error("ERR sample".to_string()).encode_with_protocol(RespProtocolVersion::Resp3),
        b"-ERR sample\r\n".to_vec()
    );
    assert_eq!(
        RespReply::NullBulkString.encode_with_protocol(RespProtocolVersion::Resp3),
        b"_\r\n".to_vec()
    );
    assert_eq!(
        RespReply::NullArray.encode_with_protocol(RespProtocolVersion::Resp3),
        b"_\r\n".to_vec()
    );
    assert_eq!(
        RespReply::Array(vec![RespReply::NullBulkString])
            .encode_with_protocol(RespProtocolVersion::Resp2),
        b"*1\r\n$-1\r\n".to_vec()
    );
    assert_eq!(
        RespReply::Array(vec![RespReply::NullBulkString])
            .encode_with_protocol(RespProtocolVersion::Resp3),
        b"*1\r\n_\r\n".to_vec()
    );
}

#[test]
fn session_hello_switches_between_resp2_and_resp3() {
    let mut session = RedisMiniSession::new();

    assert_eq!(session.protocol_version(), RespProtocolVersion::Resp2);
    assert_eq!(
        session.execute(command(&[b"HELLO", b"3"])),
        RespReply::Array(vec![
            RespReply::BulkString(b"server".to_vec()),
            RespReply::BulkString(b"redis-mini".to_vec()),
            RespReply::BulkString(b"proto".to_vec()),
            RespReply::Integer(3),
        ])
    );
    assert_eq!(session.protocol_version(), RespProtocolVersion::Resp3);
    assert_eq!(
        session.execute_encoded(command(&[b"GET", b"missing"])),
        b"_\r\n".to_vec()
    );

    assert_eq!(
        session.execute(command(&[b"HELLO", b"2"])),
        RespReply::Array(vec![
            RespReply::BulkString(b"server".to_vec()),
            RespReply::BulkString(b"redis-mini".to_vec()),
            RespReply::BulkString(b"proto".to_vec()),
            RespReply::Integer(2),
        ])
    );
    assert_eq!(session.protocol_version(), RespProtocolVersion::Resp2);
    assert_eq!(
        session.execute_encoded(command(&[b"GET", b"missing"])),
        b"$-1\r\n".to_vec()
    );
}

#[test]
fn direct_hello_returns_simplified_structured_reply_without_session_state() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"HELLO", b"3"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"server".to_vec()),
            RespReply::BulkString(b"redis-mini".to_vec()),
            RespReply::BulkString(b"proto".to_vec()),
            RespReply::Integer(3),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"HELLO", b"4"]),
        RespReply::Error("NOPROTO unsupported protocol version".to_string())
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
fn executes_mget_mset_append_strlen_and_getset_with_binary_values() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"MSET", b"a", b"one\0", b"b", b"two words"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"MGET", b"missing", b"a", b"b"]),
        RespReply::Array(vec![
            RespReply::NullBulkString,
            RespReply::BulkString(b"one\0".to_vec()),
            RespReply::BulkString(b"two words".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"APPEND", b"a", b"more\0bytes"]),
        RespReply::Integer(14)
    );
    assert_eq!(execute(&mut db, &[b"STRLEN", b"a"]), RespReply::Integer(14));
    assert_eq!(
        execute(&mut db, &[b"APPEND", b"new", b"value"]),
        RespReply::Integer(5)
    );
    assert_eq!(
        execute(&mut db, &[b"GETSET", b"new", b"replacement"]),
        RespReply::BulkString(b"value".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"GETSET", b"absent", b"created"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"MGET", b"new", b"absent"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"replacement".to_vec()),
            RespReply::BulkString(b"created".to_vec()),
        ])
    );
}

#[test]
fn string_completion_commands_validate_arity_wrong_types_and_expiration() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(execute(&mut db, &[b"MGET"]), wrong_arity_reply("mget"));
    assert_eq!(
        execute(&mut db, &[b"MSET", b"key"]),
        wrong_arity_reply("mset")
    );
    assert_eq!(
        execute(&mut db, &[b"APPEND", b"key"]),
        wrong_arity_reply("append")
    );
    assert_eq!(execute(&mut db, &[b"STRLEN"]), wrong_arity_reply("strlen"));
    assert_eq!(
        execute(&mut db, &[b"GETSET", b"key"]),
        wrong_arity_reply("getset")
    );

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"x"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"MGET", b"list"]),
        RespReply::Array(vec![RespReply::NullBulkString])
    );
    assert_eq!(execute(&mut db, &[b"MSET", b"list", b"value"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"APPEND", b"list", b"value"]),
        wrong_type
    );
    assert_eq!(execute(&mut db, &[b"STRLEN", b"list"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"GETSET", b"list", b"value"]),
        wrong_type
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"exp", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"exp", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"MSET", b"exp", b"new"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"exp"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"exp", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"APPEND", b"exp", b"!"]),
        RespReply::Integer(4)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"exp"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"exp", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"GETSET", b"exp", b"final"]),
        RespReply::BulkString(b"new!".to_vec())
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"exp"]), RespReply::Integer(-1));
}

#[test]
fn string_completion_commands_invalidate_watches_and_run_in_transactions() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"WATCH", b"a"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"MSET", b"a", b"1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"GET", b"a"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::NullArray);

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"APPEND", b"a", b"2"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"STRLEN", b"a"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"GETSET", b"a", b"done"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"MGET", b"a", b"missing"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::Integer(2),
            RespReply::Integer(2),
            RespReply::BulkString(b"12".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"done".to_vec()),
                RespReply::NullBulkString,
            ]),
        ])
    );
}

fn wrong_arity_reply(command_name: &str) -> RespReply {
    RespReply::Error(format!(
        "ERR wrong number of arguments for '{}' command",
        command_name
    ))
}

#[test]
fn select_and_dbsize_isolate_keys_per_database() {
    let mut db = RedisMiniDb::new();

    assert_eq!(execute(&mut db, &[b"DBSIZE"]), RespReply::Integer(0));
    assert_eq!(
        execute(&mut db, &[b"SET", b"key", b"db0"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"DBSIZE"]), RespReply::Integer(1));

    assert_eq!(
        execute(&mut db, &[b"SELECT", b"1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::NullBulkString
    );
    assert_eq!(execute(&mut db, &[b"DBSIZE"]), RespReply::Integer(0));
    assert_eq!(
        execute(&mut db, &[b"SET", b"key", b"db1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::BulkString(b"db1".to_vec())
    );

    assert_eq!(
        execute(&mut db, &[b"SELECT", b"0"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::BulkString(b"db0".to_vec())
    );
}

#[test]
fn dbsize_counts_only_non_expired_keys_in_selected_database() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"gone", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"kept", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"gone", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"DBSIZE"]), RespReply::Integer(1));

    assert_eq!(
        execute(&mut db, &[b"SELECT", b"2"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"DBSIZE"]), RespReply::Integer(0));
}

#[test]
fn select_rejects_invalid_database_indexes() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SELECT", b"15"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SELECT", b"16"]),
        RespReply::Error("ERR invalid DB index".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SELECT", b"not-a-db"]),
        RespReply::Error("ERR invalid DB index".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SELECT"]),
        RespReply::Error("ERR wrong number of arguments for 'select' command".to_string())
    );
}

#[test]
fn scan_and_keys_operate_on_current_database_only() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"a", b"db0"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SELECT", b"1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"b", b"db1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"KEYS", b"*"]),
        RespReply::Array(vec![RespReply::BulkString(b"b".to_vec())])
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(vec![RespReply::BulkString(b"b".to_vec())]),
        ])
    );

    assert_eq!(
        execute(&mut db, &[b"SELECT", b"0"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"KEYS", b"*"]),
        RespReply::Array(vec![RespReply::BulkString(b"a".to_vec())])
    );
}

#[test]
fn select_clears_watches_and_is_rejected_inside_multi() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"WATCH", b"watched"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SELECT", b"1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SELECT", b"0"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"changed"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"SELECT", b"1"]),
        RespReply::Error("ERR SELECT inside MULTI is not allowed".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"watched"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![RespReply::BulkString(b"changed".to_vec())])
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
fn executes_llen_lindex_and_lset_with_binary_safe_negative_indexes() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"LLEN", b"missing"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"LINDEX", b"missing", b"0"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a\0", b"b", b"c"]),
        RespReply::Integer(3)
    );
    assert_eq!(execute(&mut db, &[b"LLEN", b"list"]), RespReply::Integer(3));
    assert_eq!(
        execute(&mut db, &[b"LINDEX", b"list", b"-1"]),
        RespReply::BulkString(b"c".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LINDEX", b"list", b"3"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"LSET", b"list", b"-2", b"B\0B"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"list", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a\0".to_vec()),
            RespReply::BulkString(b"B\0B".to_vec()),
            RespReply::BulkString(b"c".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"LSET", b"list", b"4", b"x"]),
        RespReply::Error("ERR index out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"LSET", b"missing", b"0", b"x"]),
        RespReply::Error("ERR index out of range".to_string())
    );
}

#[test]
fn executes_ltrim_and_lrem_with_negative_ranges_and_removal_counts() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"LTRIM", b"missing", b"0", b"-1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a", b"b", b"a", b"c", b"a"]),
        RespReply::Integer(5)
    );
    assert_eq!(
        execute(&mut db, &[b"LTRIM", b"list", b"1", b"-2"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"list", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"b".to_vec()),
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"c".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"LREM", b"list", b"1", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"b", b"a", b"a"]),
        RespReply::Integer(5)
    );
    assert_eq!(
        execute(&mut db, &[b"LREM", b"list", b"-1", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"LREM", b"list", b"0", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"list", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"c".to_vec()),
            RespReply::BulkString(b"a".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"LTRIM", b"list", b"5", b"1"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXISTS", b"list"]),
        RespReply::Integer(0)
    );
}

#[test]
fn executes_rpoplpush_and_lmove_across_and_within_lists() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"RPOPLPUSH", b"missing", b"dest"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"source", b"a", b"b", b"c"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"RPOPLPUSH", b"source", b"dest"]),
        RespReply::BulkString(b"c".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"dest", b"0", b"-1"]),
        RespReply::Array(vec![RespReply::BulkString(b"c".to_vec())])
    );
    assert_eq!(
        execute(&mut db, &[b"LMOVE", b"source", b"dest", b"LEFT", b"RIGHT"]),
        RespReply::BulkString(b"a".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"dest", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"c".to_vec()),
            RespReply::BulkString(b"a".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"LMOVE", b"dest", b"dest", b"RIGHT", b"LEFT"]),
        RespReply::BulkString(b"a".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"dest", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"c".to_vec()),
        ])
    );
}

#[test]
fn list_completion_validates_errors_expiration_watches_and_transactions() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(execute(&mut db, &[b"LLEN"]), wrong_arity_reply("llen"));
    assert_eq!(
        execute(&mut db, &[b"LINDEX", b"k", b"bad"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"LMOVE", b"a", b"b", b"NOPE", b"LEFT"]),
        RespReply::Error("ERR syntax error".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    for command in [
        vec![b"LLEN".as_slice(), b"string"],
        vec![b"LINDEX", b"string", b"0"],
        vec![b"LSET", b"string", b"0", b"x"],
        vec![b"LTRIM", b"string", b"0", b"-1"],
        vec![b"LREM", b"string", b"0", b"x"],
        vec![b"RPOPLPUSH", b"string", b"dest"],
        vec![b"LMOVE", b"string", b"dest", b"LEFT", b"LEFT"],
    ] {
        assert_eq!(execute(&mut db, &command), wrong_type);
    }

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"list", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"LSET", b"list", b"0", b"A"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"list"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"list", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"LREM", b"list", b"0", b"A"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"list"]), RespReply::Integer(-1));

    assert_eq!(
        execute(&mut db, &[b"WATCH", b"list"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"LTRIM", b"list", b"1", b"0"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"LLEN", b"list"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::NullArray);

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"tx", b"one", b"two"]),
        RespReply::Integer(2)
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"RPOPLPUSH", b"tx", b"out"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"LMOVE", b"tx", b"out", b"LEFT", b"RIGHT"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"two".to_vec()),
            RespReply::BulkString(b"one".to_vec()),
        ])
    );
}

#[test]
fn blocking_list_pops_scan_immediately_without_sleeping() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"BLPOP", b"missing", b"0"]),
        RespReply::NullArray
    );
    assert_eq!(
        execute(&mut db, &[b"LPUSH", b"empty", b"x"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"LPOP", b"empty"]),
        RespReply::BulkString(b"x".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"second", b"a", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"BLPOP", b"missing", b"empty", b"second", b"1.5"]
        ),
        RespReply::Array(vec![
            RespReply::BulkString(b"second".to_vec()),
            RespReply::BulkString(b"a".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"BRPOP", b"second", b"0"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"second".to_vec()),
            RespReply::BulkString(b"b".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"BRPOP", b"second", b"0"]),
        RespReply::NullArray
    );
}

#[test]
fn blocking_list_move_is_immediate_and_session_compatible() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(
            &mut db,
            &[b"BLMOVE", b"missing", b"dest", b"LEFT", b"RIGHT", b"0"]
        ),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"source", b"one", b"two"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"BLMOVE", b"source", b"dest", b"RIGHT", b"LEFT", b"2"]
        ),
        RespReply::BulkString(b"two".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"dest", b"0", b"-1"]),
        RespReply::Array(vec![RespReply::BulkString(b"two".to_vec())])
    );

    let mut input = multibulk_frame(&[b"RPUSH", b"tcp", b"x"]);
    input.extend(multibulk_frame(&[b"BLPOP", b"tcp", b"0"]));
    assert_eq!(
        tcp_exchange(&input),
        b":1\r\n*2\r\n$3\r\ntcp\r\n$1\r\nx\r\n".to_vec()
    );
}

#[test]
fn blocking_list_commands_validate_errors_and_wrong_types() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"BLPOP", b"k"]),
        wrong_arity_reply("blpop")
    );
    assert_eq!(
        execute(&mut db, &[b"BRPOP", b"k"]),
        wrong_arity_reply("brpop")
    );
    assert_eq!(
        execute(&mut db, &[b"BLMOVE", b"s", b"d", b"LEFT", b"RIGHT"]),
        wrong_arity_reply("blmove")
    );
    assert_eq!(
        execute(&mut db, &[b"BLPOP", b"k", b"bad"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"BRPOP", b"k", b"-1"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"BLMOVE", b"s", b"d", b"SIDE", b"RIGHT", b"0"]),
        RespReply::Error("ERR syntax error".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"BLMOVE", b"s", b"d", b"LEFT", b"RIGHT", b"bad"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"BLPOP", b"missing", b"string", b"0"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"source", b"x"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"BLMOVE", b"source", b"string", b"LEFT", b"RIGHT", b"0"]
        ),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"LRANGE", b"source", b"0", b"-1"]),
        RespReply::Array(vec![RespReply::BulkString(b"x".to_vec())])
    );
}

#[test]
fn blocking_list_mutations_clear_expiration_invalidate_watches_and_queue() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"watched", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"watched", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"WATCH", b"watched"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"BLPOP", b"watched", b"0"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"watched".to_vec()),
            RespReply::BulkString(b"a".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"TTL", b"watched"]),
        RespReply::Integer(-2)
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"GET", b"watched"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::NullArray);

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"src", b"one"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"src", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(
            &mut db,
            &[b"BLMOVE", b"src", b"dst", b"LEFT", b"LEFT", b"0"]
        ),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"BRPOP", b"dst", b"0"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"one".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"dst".to_vec()),
                RespReply::BulkString(b"one".to_vec()),
            ]),
        ])
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"src"]), RespReply::Integer(-2));
    assert_eq!(execute(&mut db, &[b"TTL", b"dst"]), RespReply::Integer(-2));
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
fn executes_hash_completion_reads_in_deterministic_order() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"HMGET", b"missing", b"a", b"b"]),
        RespReply::Array(vec![RespReply::NullBulkString, RespReply::NullBulkString])
    );
    assert_eq!(
        execute(&mut db, &[b"HLEN", b"missing"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"HKEYS", b"missing"]),
        RespReply::Array(Vec::new())
    );
    assert_eq!(
        execute(&mut db, &[b"HVALS", b"missing"]),
        RespReply::Array(Vec::new())
    );
    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"b", b"two", b"a\0", b"one\0"]),
        RespReply::Integer(2)
    );

    assert_eq!(
        execute(&mut db, &[b"HMGET", b"hash", b"a\0", b"missing", b"b"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"one\0".to_vec()),
            RespReply::NullBulkString,
            RespReply::BulkString(b"two".to_vec()),
        ])
    );
    assert_eq!(execute(&mut db, &[b"HLEN", b"hash"]), RespReply::Integer(2));
    assert_eq!(
        execute(&mut db, &[b"HKEYS", b"hash"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a\0".to_vec()),
            RespReply::BulkString(b"b".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"HVALS", b"hash"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"one\0".to_vec()),
            RespReply::BulkString(b"two".to_vec()),
        ])
    );
}

#[test]
fn executes_hincrby_with_integer_errors_overflow_expiration_and_watch_invalidation() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"counter", b"5"]),
        RespReply::Integer(5)
    );
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"counter", b"-2"]),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"HGET", b"hash", b"counter"]),
        RespReply::BulkString(b"3".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"text", b"nope"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"text", b"1"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"counter", b"bad"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"max", b"9223372036854775807"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"max", b"1"]),
        RespReply::Error("ERR increment or decrement would overflow".to_string())
    );

    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"hash", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"counter", b"0"]),
        RespReply::Integer(3)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"hash"]), RespReply::Integer(-1));

    assert_eq!(
        execute(&mut db, &[b"WATCH", b"hash"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"counter", b"1"]),
        RespReply::Integer(4)
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"GET", b"queued"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::NullArray);
}

#[test]
fn hscan_uses_existing_deterministic_cursor_style() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"missing", b"0"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(Vec::new()),
        ])
    );
    assert_eq!(
        execute(
            &mut db,
            &[b"HSET", b"hash", b"b", b"two", b"a", b"one", b"c", b"three"]
        ),
        RespReply::Integer(3)
    );
    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"hash", b"0", b"COUNT", b"2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"2".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"a".to_vec()),
                RespReply::BulkString(b"one".to_vec()),
                RespReply::BulkString(b"b".to_vec()),
                RespReply::BulkString(b"two".to_vec()),
            ]),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"hash", b"2", b"count", b"2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"c".to_vec()),
                RespReply::BulkString(b"three".to_vec()),
            ]),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"hash", b"bad"]),
        RespReply::Error("ERR invalid cursor".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"hash", b"4"]),
        RespReply::Error("ERR invalid cursor".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"hash", b"0", b"MATCH", b"*"]),
        RespReply::Error("ERR unsupported HSCAN option".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"hash", b"0", b"COUNT", b"0"]),
        RespReply::Error("ERR invalid COUNT".to_string())
    );
}

#[test]
fn hash_completion_commands_reject_wrong_arity_wrong_type_and_queue_transactions() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"HMGET", b"string", b"f"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HKEYS", b"string"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HVALS", b"string"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HLEN", b"string"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"string", b"f", b"1"]),
        wrong_type
    );
    assert_eq!(execute(&mut db, &[b"HSCAN", b"string", b"0"]), wrong_type);

    assert_eq!(
        execute(&mut db, &[b"HMGET", b"hash"]),
        RespReply::Error("ERR wrong number of arguments for 'hmget' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HKEYS", b"hash", b"extra"]),
        RespReply::Error("ERR wrong number of arguments for 'hkeys' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"hash", b"field"]),
        RespReply::Error("ERR wrong number of arguments for 'hincrby' command".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"HSCAN", b"hash"]),
        RespReply::Error("ERR wrong number of arguments for 'hscan' command".to_string())
    );

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"HINCRBY", b"queued-hash", b"f", b"2"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"HMGET", b"queued-hash", b"f"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::Integer(2),
            RespReply::Array(vec![RespReply::BulkString(b"2".to_vec())]),
        ])
    );
}

#[test]
fn tcp_server_executes_hash_completion_commands() {
    let mut input = multibulk_frame(&[b"HINCRBY", b"tcp-hash", b"f", b"2"]);
    input.extend(multibulk_frame(&[b"HMGET", b"tcp-hash", b"f", b"missing"]));

    assert_eq!(
        tcp_exchange(&input),
        b":2\r\n*2\r\n$1\r\n2\r\n$-1\r\n".to_vec()
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
fn executes_zadd_zrem_and_zscore_on_missing_and_existing_zsets() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"ZSCORE", b"missing", b"member"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"2", b"b", b"1", b"a"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"3", b"b", b"4", b"c"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"ZSCORE", b"z", b"b"]),
        RespReply::BulkString(b"3".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"ZSCORE", b"z", b"missing"]),
        RespReply::NullBulkString
    );
    assert_eq!(
        execute(&mut db, &[b"ZREM", b"z", b"missing", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"ZSCORE", b"z", b"a"]),
        RespReply::NullBulkString
    );
}

#[test]
fn executes_zrange_with_score_member_ordering_and_negative_indexes() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"missing", b"0", b"-1"]),
        RespReply::Array(Vec::new())
    );
    assert_eq!(
        execute(
            &mut db,
            &[
                b"ZADD", b"z", b"2", b"c", b"1", b"b", b"1", b"a", b"3", b"d"
            ]
        ),
        RespReply::Integer(4)
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z", b"0", b"2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"b".to_vec()),
            RespReply::BulkString(b"c".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z", b"-2", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"c".to_vec()),
            RespReply::BulkString(b"d".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z", b"3", b"1"]),
        RespReply::Array(Vec::new())
    );
}

#[test]
fn sorted_sets_keep_binary_members_and_reject_invalid_scores() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(
            &mut db,
            &[b"ZADD", b"z", b"-5", b"a\0member", b"0", b"a member"]
        ),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a\0member".to_vec()),
            RespReply::BulkString(b"a member".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"not-int", b"bad"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z", b"0", b"-1"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"a\0member".to_vec()),
            RespReply::BulkString(b"a member".to_vec()),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z", b"nope", b"-1"]),
        RespReply::Error("ERR value is not an integer or out of range".to_string())
    );
}

#[test]
fn sorted_set_writes_clear_expiration_and_remove_empty_keys() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"1", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"z", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"2", b"b"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"TTL", b"z"]), RespReply::Integer(-1));
    assert_eq!(
        execute(&mut db, &[b"ZREM", b"z", b"a", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(execute(&mut db, &[b"EXISTS", b"z"]), RespReply::Integer(0));
}

#[test]
fn executes_xadd_xlen_and_xrange_with_ordered_ids() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"XLEN", b"stream"]),
        RespReply::Integer(0)
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"2-0", b"f", b"v2"]),
        RespReply::BulkString(b"2-0".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"1-1", b"a", b"b"]),
        RespReply::BulkString(b"1-1".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"XLEN", b"stream"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"XRANGE", b"stream", b"-", b"+"]),
        RespReply::Array(vec![
            RespReply::Array(vec![
                RespReply::BulkString(b"1-1".to_vec()),
                RespReply::Array(vec![
                    RespReply::BulkString(b"a".to_vec()),
                    RespReply::BulkString(b"b".to_vec()),
                ]),
            ]),
            RespReply::Array(vec![
                RespReply::BulkString(b"2-0".to_vec()),
                RespReply::Array(vec![
                    RespReply::BulkString(b"f".to_vec()),
                    RespReply::BulkString(b"v2".to_vec()),
                ]),
            ]),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"XRANGE", b"stream", b"2-0", b"2-0"]),
        RespReply::Array(vec![RespReply::Array(vec![
            RespReply::BulkString(b"2-0".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"f".to_vec()),
                RespReply::BulkString(b"v2".to_vec()),
            ]),
        ])])
    );
}

#[test]
fn streams_preserve_binary_field_values_and_validate_arguments() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(
            &mut db,
            &[
                b"XADD",
                b"stream",
                b"0-1",
                b"field\0one",
                b"value\0one",
                b"field two",
                b"value two",
            ]
        ),
        RespReply::BulkString(b"0-1".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"XRANGE", b"stream", b"-", b"+"]),
        RespReply::Array(vec![RespReply::Array(vec![
            RespReply::BulkString(b"0-1".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"field\0one".to_vec()),
                RespReply::BulkString(b"value\0one".to_vec()),
                RespReply::BulkString(b"field two".to_vec()),
                RespReply::BulkString(b"value two".to_vec()),
            ]),
        ])])
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"bad", b"f", b"v"]),
        RespReply::Error("ERR Invalid stream ID specified as stream command argument".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"1-", b"f", b"v"]),
        RespReply::Error("ERR Invalid stream ID specified as stream command argument".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"1-2", b"field"]),
        RespReply::Error("ERR wrong number of arguments for 'xadd' command".to_string())
    );
}

#[test]
fn streams_wrong_type_expiration_watch_and_transactions_work() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"string", b"1-0", b"f", b"v"]),
        wrong_type
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"1-0", b"f", b"v"]),
        RespReply::BulkString(b"1-0".to_vec())
    );
    assert_eq!(execute(&mut db, &[b"GET", b"stream"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SET", b"stream", b"value"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"stream", b"10"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"2-0", b"f", b"v"]),
        RespReply::BulkString(b"2-0".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"TTL", b"stream"]),
        RespReply::Integer(-1)
    );

    assert_eq!(
        execute(&mut db, &[b"WATCH", b"stream"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"3-0", b"f", b"v"]),
        RespReply::BulkString(b"3-0".to_vec())
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"XADD", b"queued", b"1-0", b"f", b"v"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::NullArray);

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"XADD", b"queued", b"1-0", b"f", b"v"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"XLEN", b"queued"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"1-0".to_vec()),
            RespReply::Integer(1)
        ])
    );
}

#[test]
fn queued_sorted_set_commands_execute_in_order() {
    let mut db = RedisMiniDb::new();

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"2", b"b", b"1", b"a"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z", b"0", b"-1"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::Integer(2),
            RespReply::Array(vec![
                RespReply::BulkString(b"a".to_vec()),
                RespReply::BulkString(b"b".to_vec()),
            ]),
        ])
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
        execute(&mut db, &[b"ZADD", b"zset", b"1", b"member"]),
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
    assert_eq!(
        execute(&mut db, &[b"TYPE", b"zset"]),
        RespReply::SimpleString("zset")
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

    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"1", b"a", b"2", b"b"]),
        RespReply::Integer(2)
    );
    assert_eq!(
        execute(&mut db, &[b"RENAME", b"z", b"z2"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"z2", b"0", b"-1"]),
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
        execute(&mut db, &[b"ZADD", b"zset", b"1", b"member"]),
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
            RespReply::BulkString(b"zset".to_vec()),
        ])
    );
}

#[test]
fn scan_zero_returns_all_current_key_names_in_deterministic_order() {
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
        execute(&mut db, &[b"ZADD", b"zset", b"1", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"XADD", b"stream", b"1-0", b"field", b"value"]),
        RespReply::BulkString(b"1-0".to_vec())
    );

    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"hash".to_vec()),
                RespReply::BulkString(b"list".to_vec()),
                RespReply::BulkString(b"set".to_vec()),
                RespReply::BulkString(b"stream".to_vec()),
                RespReply::BulkString(b"z".to_vec()),
                RespReply::BulkString(b"zset".to_vec()),
            ]),
        ])
    );
}

#[test]
fn scan_count_returns_stable_cursor_batches() {
    let mut db = RedisMiniDb::new();

    for key in [b"a".as_slice(), b"b", b"c", b"d", b"e"] {
        assert_eq!(
            execute(&mut db, &[b"SET", key, b"value"]),
            RespReply::SimpleString("OK")
        );
    }

    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0", b"COUNT", b"2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"2".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"a".to_vec()),
                RespReply::BulkString(b"b".to_vec()),
            ]),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"2", b"COUNT", b"2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"4".to_vec()),
            RespReply::Array(vec![
                RespReply::BulkString(b"c".to_vec()),
                RespReply::BulkString(b"d".to_vec()),
            ]),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"4", b"COUNT", b"2"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(vec![RespReply::BulkString(b"e".to_vec())]),
        ])
    );
}

#[test]
fn scan_rejects_invalid_cursor_count_and_options() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SCAN", b"-1"]),
        RespReply::Error("ERR invalid cursor".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"not-a-cursor"]),
        RespReply::Error("ERR invalid cursor".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0", b"COUNT", b"0"]),
        RespReply::Error("ERR invalid COUNT".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0", b"COUNT", b"nope"]),
        RespReply::Error("ERR invalid COUNT".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0", b"MATCH", b"*"]),
        RespReply::Error("ERR unsupported SCAN option".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0", b"COUNT"]),
        RespReply::Error("ERR wrong number of arguments for 'scan' command".to_string())
    );
}

#[test]
fn scan_observes_lazy_expiration_and_does_not_invalidate_watches() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"expired", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"expired", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"WATCH", b"watched"]),
        RespReply::SimpleString("OK")
    );

    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(vec![RespReply::BulkString(b"watched".to_vec())]),
        ])
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"SCAN", b"0"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![RespReply::Array(vec![
            RespReply::BulkString(b"0".to_vec()),
            RespReply::Array(vec![RespReply::BulkString(b"watched".to_vec())]),
        ])])
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
fn rejects_wrong_type_access_between_zsets_and_other_value_kinds() {
    let mut db = RedisMiniDb::new();
    let wrong_type = RespReply::Error(
        "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
    );

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"ZADD", b"string", b"1", b"x"]),
        wrong_type
    );
    assert_eq!(execute(&mut db, &[b"ZREM", b"string", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"ZSCORE", b"string", b"x"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"ZRANGE", b"string", b"0", b"-1"]),
        wrong_type
    );

    assert_eq!(
        execute(&mut db, &[b"RPUSH", b"list", b"a"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"ZADD", b"list", b"1", b"x"]),
        wrong_type
    );

    assert_eq!(
        execute(&mut db, &[b"HSET", b"hash", b"f", b"v"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"ZREM", b"hash", b"x"]), wrong_type);

    assert_eq!(
        execute(&mut db, &[b"SADD", b"set", b"x"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"ZSCORE", b"set", b"x"]), wrong_type);

    assert_eq!(
        execute(&mut db, &[b"ZADD", b"z", b"1", b"x"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"GET", b"z"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SET", b"z", b"value"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"INCR", b"z"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"LPUSH", b"z", b"x"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"HGET", b"z", b"f"]), wrong_type);
    assert_eq!(execute(&mut db, &[b"SADD", b"z", b"x"]), wrong_type);
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"out", b"z"]),
        wrong_type
    );
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
fn transaction_queues_writes_until_exec_and_returns_replies() {
    let mut db = RedisMiniDb::new();

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"SET", b"key", b"value"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXISTS", b"key"]),
        RespReply::SimpleString("QUEUED")
    );

    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::SimpleString("OK"),
            RespReply::BulkString(b"value".to_vec()),
            RespReply::Integer(1),
        ])
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::BulkString(b"value".to_vec())
    );
}

#[test]
fn discard_drops_queued_writes() {
    let mut db = RedisMiniDb::new();

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"SET", b"key", b"value"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"DISCARD"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"key"]),
        RespReply::NullBulkString
    );
}

#[test]
fn transaction_control_commands_return_errors_when_misused() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Error("ERR EXEC without MULTI".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"DISCARD"]),
        RespReply::Error("ERR DISCARD without MULTI".to_string())
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"MULTI"]),
        RespReply::Error("ERR MULTI calls can not be nested".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"DISCARD"]),
        RespReply::SimpleString("OK")
    );
}

#[test]
fn queued_commands_preserve_binary_arguments_and_execute_expiration_lazily() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"expires", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"expires", b"0"]),
        RespReply::Integer(1)
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"GET", b"expires"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"bin\0key", b"hello \0 world"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"bin\0key"]),
        RespReply::SimpleString("QUEUED")
    );

    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::NullBulkString,
            RespReply::SimpleString("OK"),
            RespReply::BulkString(b"hello \0 world".to_vec()),
        ])
    );
}

#[test]
fn watch_unwatch_and_transaction_state_are_tracked() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"WATCH", b"a", b"b"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"UNWATCH"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"WATCH", b"a"]),
        RespReply::Error("ERR WATCH inside MULTI is not allowed".to_string())
    );
    assert_eq!(
        execute(&mut db, &[b"DISCARD"]),
        RespReply::SimpleString("OK")
    );
}

#[test]
fn changed_watched_key_aborts_exec_and_drops_queued_writes() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"WATCH", b"watched"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"changed"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"SET", b"queued", b"value"]),
        RespReply::SimpleString("QUEUED")
    );

    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::NullArray);
    assert_eq!(
        execute(&mut db, &[b"GET", b"watched"]),
        RespReply::BulkString(b"changed".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"queued"]),
        RespReply::NullBulkString
    );
}

#[test]
fn unchanged_watched_keys_allow_exec_and_clear_watches() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"WATCH", b"watched"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"new"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"GET", b"watched"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(
        execute(&mut db, &[b"EXEC"]),
        RespReply::Array(vec![
            RespReply::SimpleString("OK"),
            RespReply::BulkString(b"new".to_vec()),
        ])
    );

    assert_eq!(
        execute(&mut db, &[b"WATCH", b"watched"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"DISCARD"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"watched", b"after-discard"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::Array(Vec::new()));
}

#[test]
fn writes_across_command_families_invalidate_watched_keys() {
    let mut db = RedisMiniDb::new();

    assert_eq!(
        execute(&mut db, &[b"SET", b"string", b"0"]),
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
        execute(&mut db, &[b"ZADD", b"zset", b"1", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SADD", b"source", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"source-name", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"rename-dest", b"old"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"SET", b"expire-now", b"value"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(
            &mut db,
            &[
                b"WATCH",
                b"string",
                b"list",
                b"hash",
                b"set",
                b"zset",
                b"store-dest",
                b"source-name",
                b"rename-dest",
                b"expire-now",
            ]
        ),
        RespReply::SimpleString("OK")
    );

    assert_eq!(
        execute(&mut db, &[b"INCR", b"string"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"LPOP", b"list"]),
        RespReply::BulkString(b"a".to_vec())
    );
    assert_eq!(
        execute(&mut db, &[b"HDEL", b"hash", b"field"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SREM", b"set", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"ZREM", b"zset", b"member"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"SUNIONSTORE", b"store-dest", b"source"]),
        RespReply::Integer(1)
    );
    assert_eq!(
        execute(&mut db, &[b"RENAME", b"source-name", b"rename-dest"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut db, &[b"EXPIRE", b"expire-now", b"0"]),
        RespReply::Integer(1)
    );

    assert_eq!(execute(&mut db, &[b"MULTI"]), RespReply::SimpleString("OK"));
    assert_eq!(
        execute(&mut db, &[b"SET", b"queued", b"value"]),
        RespReply::SimpleString("QUEUED")
    );
    assert_eq!(execute(&mut db, &[b"EXEC"]), RespReply::NullArray);
    assert_eq!(
        execute(&mut db, &[b"GET", b"queued"]),
        RespReply::NullBulkString
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
