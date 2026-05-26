use rust_port::{Command, ParseOutcome, RespCommandParser};

fn parse_complete(frame: &[u8]) -> Command {
    let mut parser = RespCommandParser::new();
    parser.append(frame);
    match parser.parse_available().expect("valid frame") {
        ParseOutcome::Complete(command) => command,
        ParseOutcome::Incomplete => panic!("expected complete command"),
    }
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
