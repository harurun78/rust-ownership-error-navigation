use rust_port::{
    Command, RedisMiniClientSession, RedisMiniSession, RespReply, command_metadata,
    redis_cluster_hash_slot,
};

fn command(args: &[&[u8]]) -> Command {
    Command::new(args.iter().map(|arg| arg.to_vec()).collect())
}

fn execute(session: &mut RedisMiniSession, args: &[&[u8]]) -> RespReply {
    session.execute(command(args))
}

#[test]
fn cluster_command_exposes_metadata_category() {
    assert_eq!(
        command_metadata(b"cluster").unwrap().category.as_str(),
        "cluster"
    );
}

#[test]
fn cluster_hash_slot_matches_redis_crc16_and_hash_tags() {
    assert_eq!(redis_cluster_hash_slot(b"123456789"), 12739);
    assert_eq!(redis_cluster_hash_slot(b"foo"), 12182);
    assert_eq!(redis_cluster_hash_slot(b"bar"), 5061);
    assert_eq!(redis_cluster_hash_slot(b"foo{bar}zap"), 5061);
    assert_eq!(redis_cluster_hash_slot(b"{bar}"), 5061);
    assert_eq!(redis_cluster_hash_slot(b"foo{}bar"), 14292);

    let mut session = RedisMiniSession::new();
    assert_eq!(
        execute(&mut session, &[b"CLUSTER", b"KEYSLOT", b"foo{bar}zap"]),
        RespReply::Integer(5061)
    );
}

#[test]
fn cluster_slots_info_and_nodes_are_deterministic() {
    let mut session = RedisMiniSession::new();
    session.enable_cluster_for_test(vec![(0, 100), (5000, 5100)]);

    assert_eq!(
        execute(&mut session, &[b"CLUSTER", b"SLOTS"]),
        RespReply::Array(vec![
            RespReply::Array(vec![
                RespReply::Integer(0),
                RespReply::Integer(100),
                RespReply::Array(vec![
                    RespReply::BulkString(b"127.0.0.1".to_vec()),
                    RespReply::Integer(6379),
                    RespReply::BulkString(b"0000000000000000000000000000000000000001".to_vec()),
                ]),
            ]),
            RespReply::Array(vec![
                RespReply::Integer(5000),
                RespReply::Integer(5100),
                RespReply::Array(vec![
                    RespReply::BulkString(b"127.0.0.1".to_vec()),
                    RespReply::Integer(6379),
                    RespReply::BulkString(b"0000000000000000000000000000000000000001".to_vec()),
                ]),
            ]),
        ])
    );

    match execute(&mut session, &[b"CLUSTER", b"INFO"]) {
        RespReply::BulkString(bytes) => {
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.contains("cluster_enabled:1"));
            assert!(text.contains("cluster_state:ok"));
            assert!(text.contains("cluster_slots_assigned:202"));
        }
        other => panic!("expected bulk info, got {other:?}"),
    }

    match execute(&mut session, &[b"CLUSTER", b"NODES"]) {
        RespReply::BulkString(bytes) => {
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.contains("myself,master"));
            assert!(text.contains("0-100 5000-5100"));
        }
        other => panic!("expected bulk nodes, got {other:?}"),
    }
}

#[test]
fn cluster_routing_returns_moved_or_ask_for_non_local_single_key_commands() {
    let mut moved = RedisMiniSession::new();
    moved.enable_cluster_for_test(vec![(0, 100)]);
    moved.set_cluster_moved_target_for_test("10.0.0.2", 7001);
    assert_eq!(
        execute(&mut moved, &[b"GET", b"foo"]),
        RespReply::Error("MOVED 12182 10.0.0.2:7001".to_string())
    );

    let mut asked = RedisMiniSession::new();
    asked.enable_cluster_for_test(vec![(0, 100)]);
    asked.set_cluster_ask_target_for_test("10.0.0.3", 7002);
    assert_eq!(
        execute(&mut asked, &[b"SET", b"foo", b"bar"]),
        RespReply::Error("ASK 12182 10.0.0.3:7002".to_string())
    );
}

#[test]
fn cluster_routing_allows_local_slots_and_preserves_standalone_mode() {
    let mut standalone = RedisMiniSession::new();
    assert_eq!(
        execute(&mut standalone, &[b"SET", b"foo", b"v"]),
        RespReply::SimpleString("OK")
    );

    let mut clustered = RedisMiniSession::new();
    clustered.enable_cluster_for_test(vec![(12182, 12182)]);
    assert_eq!(
        execute(&mut clustered, &[b"SET", b"foo", b"v"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut clustered, &[b"GET", b"foo"]),
        RespReply::BulkString(b"v".to_vec())
    );
}

#[test]
fn cluster_routing_validates_crossslot_multi_key_commands() {
    let mut session = RedisMiniSession::new();
    session.enable_cluster_for_test(vec![(0, 16383)]);

    assert_eq!(
        execute(&mut session, &[b"MGET", b"foo", b"bar"]),
        RespReply::Error("CROSSSLOT Keys in request don't hash to the same slot".to_string())
    );
    assert_eq!(
        execute(
            &mut session,
            &[b"MSET", b"left{tag}", b"1", b"right{tag}", b"2"]
        ),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"MGET", b"left{tag}", b"right{tag}"]),
        RespReply::Array(vec![
            RespReply::BulkString(b"1".to_vec()),
            RespReply::BulkString(b"2".to_vec()),
        ])
    );
}

#[test]
fn cluster_routing_applies_inside_transaction_queue_boundary() {
    let mut session = RedisMiniSession::new();
    session.enable_cluster_for_test(vec![(0, 16383)]);

    assert_eq!(
        execute(&mut session, &[b"MULTI"]),
        RespReply::SimpleString("OK")
    );
    assert_eq!(
        execute(&mut session, &[b"MGET", b"foo", b"bar"]),
        RespReply::Error("CROSSSLOT Keys in request don't hash to the same slot".to_string())
    );
    assert_eq!(
        execute(&mut session, &[b"SET", b"foo", b"v"]),
        RespReply::SimpleString("QUEUED")
    );
    assert!(matches!(
        execute(&mut session, &[b"EXEC"]),
        RespReply::Array(_)
    ));
}

#[test]
fn cluster_command_validates_arity_and_subcommands() {
    let mut session = RedisMiniSession::new();

    assert!(matches!(
        execute(&mut session, &[b"CLUSTER"]),
        RespReply::Error(_)
    ));
    assert!(matches!(
        execute(&mut session, &[b"CLUSTER", b"KEYSLOT"]),
        RespReply::Error(_)
    ));
    assert!(matches!(
        execute(&mut session, &[b"CLUSTER", b"SLOTS", b"extra"]),
        RespReply::Error(_)
    ));
    assert_eq!(
        execute(&mut session, &[b"CLUSTER", b"NOPE"]),
        RespReply::Error("ERR unsupported CLUSTER subcommand".to_string())
    );
}

#[test]
fn tcp_session_handles_cluster_keyslot_command() {
    let mut client = RedisMiniClientSession::new();
    let output = client
        .process_input(b"*3\r\n$7\r\nCLUSTER\r\n$7\r\nKEYSLOT\r\n$3\r\nfoo\r\n")
        .unwrap();
    assert_eq!(output, b":12182\r\n");
}
