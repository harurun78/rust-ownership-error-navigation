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
