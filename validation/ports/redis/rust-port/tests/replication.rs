use rust_port::{Command, RedisMiniClientSession, RedisMiniSession, RespReply, command_metadata};

fn command(args: &[&[u8]]) -> Command {
    Command::new(args.iter().map(|a| a.to_vec()).collect())
}

fn execute(session: &mut RedisMiniSession, args: &[&[u8]]) -> RespReply {
    session.execute(command(args))
}

#[test]
fn replication_commands_expose_metadata_category() {
    assert_eq!(
        command_metadata(b"role").unwrap().category.as_str(),
        "replication"
    );
    assert_eq!(
        command_metadata(b"replicaof").unwrap().category.as_str(),
        "replication"
    );
    assert_eq!(
        command_metadata(b"replconf").unwrap().category.as_str(),
        "replication"
    );
    assert_eq!(
        command_metadata(b"psync").unwrap().category.as_str(),
        "replication"
    );
}

#[test]
fn role_and_replicaof_transition_between_master_and_replica() {
    let mut session = RedisMiniSession::new();

    assert_eq!(
        execute(&mut session, &[b"ROLE"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"master".to_vec()),
            RespReply::Integer(0),
            RespReply::Array(Vec::new())
        ])
    );

    assert_eq!(
        execute(&mut session, &[b"REPLICAOF", b"127.0.0.1", b"6379"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"ROLE"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"replica".to_vec()),
            RespReply::BulkString(b"127.0.0.1".to_vec()),
            RespReply::Integer(6379),
            RespReply::BulkString(b"connected".to_vec()),
            RespReply::Integer(0)
        ])
    );

    assert_eq!(
        execute(&mut session, &[b"REPLICAOF", b"NO", b"ONE"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"ROLE"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"master".to_vec()),
            RespReply::Integer(0),
            RespReply::Array(Vec::new())
        ])
    );
}

#[test]
fn master_writes_append_propagation_log_with_offsets_but_reads_do_not() {
    let mut session = RedisMiniSession::new();

    assert_eq!(
        execute(&mut session, &[b"PING"]),
        RespReply::SimpleString("PONG")
    );
    assert!(session.propagation_log().is_empty());

    assert_eq!(
        execute(&mut session, &[b"SET", b"k", b"v"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(session.replication_checkpoint().offset, 1);
    assert_eq!(session.propagation_log().len(), 1);
    assert_eq!(session.propagation_log()[0].offset, 1);
    assert_eq!(session.propagation_log()[0].command[0], b"SET".to_vec());

    assert_eq!(
        execute(&mut session, &[b"GET", b"k"]),
        RespReply::BulkString(b"v".to_vec())
    );
    assert_eq!(session.replication_checkpoint().offset, 1);
    assert_eq!(session.propagation_log().len(), 1);

    assert_eq!(
        execute(&mut session, &[b"DEL", b"k"]),
        RespReply::Integer(1)
    );
    assert_eq!(session.replication_checkpoint().offset, 2);
    assert_eq!(session.propagation_log().len(), 2);
    assert_eq!(session.propagation_log()[1].offset, 2);
}

#[test]
fn replica_rejects_local_writes_but_allows_reads() {
    let mut session = RedisMiniSession::new();
    assert_eq!(
        execute(&mut session, &[b"SET", b"k", b"v"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"REPLICAOF", b"master.example", b"6380"]),
        RespReply::SimpleString("OK")
    );

    match execute(&mut session, &[b"SET", b"k", b"new"]) {
        RespReply::Error(message) => assert!(message.starts_with("READONLY")),
        other => panic!("expected READONLY error, got {other:?}"),
    }
    assert_eq!(session.replication_checkpoint().offset, 1);
    assert_eq!(
        execute(&mut session, &[b"GET", b"k"]),
        RespReply::BulkString(b"v".to_vec())
    );
}

#[test]
fn replication_info_and_handshake_stubs_are_deterministic() {
    let mut session = RedisMiniSession::new();

    assert_eq!(
        execute(&mut session, &[b"REPLCONF", b"listening-port", b"6379"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"PSYNC", b"?", b"-1"]),
        RespReply::SimpleStringOwned(
            "FULLRESYNC 0000000000000000000000000000000000000001 0".to_string()
        )
    );

    let info = execute(&mut session, &[b"INFO", b"replication"]);
    match info {
        RespReply::BulkString(bytes) => {
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.contains("# Replication"));
            assert!(text.contains("role:master"));
            assert!(text.contains("master_replid:0000000000000000000000000000000000000001"));
            assert!(text.contains("master_repl_offset:0"));
        }
        other => panic!("expected INFO bulk string, got {other:?}"),
    }
}

#[test]
fn replication_commands_validate_arity_and_subcommands() {
    let mut session = RedisMiniSession::new();

    assert!(matches!(
        execute(&mut session, &[b"ROLE", b"extra"]),
        RespReply::Error(_)
    ));
    assert!(matches!(
        execute(&mut session, &[b"REPLICAOF", b"host"]),
        RespReply::Error(_)
    ));
    assert!(matches!(
        execute(&mut session, &[b"REPLICAOF", b"host", b"not-a-port"]),
        RespReply::Error(_)
    ));
    assert!(matches!(
        execute(&mut session, &[b"REPLCONF"]),
        RespReply::Error(_)
    ));
    assert!(matches!(
        execute(&mut session, &[b"PSYNC", b"?", b"0"]),
        RespReply::Error(_)
    ));
}

#[test]
fn replica_read_only_enforcement_applies_to_transactions() {
    let mut session = RedisMiniSession::new();

    assert_eq!(
        execute(&mut session, &[b"MULTI"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"GET", b"k"]),
        RespReply::SimpleString("QUEUED")
    );
    assert!(matches!(
        execute(&mut session, &[b"EXEC"]),
        RespReply::Array(_)
    ));

    assert_eq!(
        execute(&mut session, &[b"REPLICAOF", b"127.0.0.1", b"6379"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"MULTI"]),
        RespReply::SimpleString("OK")
    );
    match execute(&mut session, &[b"SET", b"k", b"v"]) {
        RespReply::Error(message) => assert!(message.starts_with("READONLY")),
        other => panic!("expected READONLY inside transaction, got {other:?}"),
    }
    assert_eq!(
        execute(&mut session, &[b"DISCARD"]),
        RespReply::SimpleString("OK")
    );
}

#[test]
fn tcp_session_handles_replication_commands() {
    let mut client = RedisMiniClientSession::new();
    let output = client
        .process_input(
            b"*1\r\n$4\r\nROLE\r\n*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6379\r\n",
        )
        .unwrap();
    assert!(output.starts_with(b"*3\r\n$6\r\nmaster\r\n:0\r\n*0\r\n"));
    assert!(output.ends_with(b"+OK\r\n"));
}
