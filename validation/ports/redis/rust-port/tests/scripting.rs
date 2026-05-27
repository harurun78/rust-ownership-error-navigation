use rust_port::{Command, RedisMiniDb, RespReply, command_metadata};

fn command(args: &[&[u8]]) -> Command {
    Command::new(args.iter().map(|a| a.to_vec()).collect())
}

fn execute(db: &mut RedisMiniDb, args: &[&[u8]]) -> RespReply {
    db.execute(command(args))
}

#[test]
fn scripting_commands_expose_server_metadata() {
    assert_eq!(
        command_metadata(b"eval").unwrap().category.as_str(),
        "server"
    );
    assert_eq!(
        command_metadata(b"evalsha").unwrap().category.as_str(),
        "server"
    );
    assert_eq!(
        command_metadata(b"script").unwrap().category.as_str(),
        "server"
    );
}

#[test]
fn script_load_exists_flush_and_eval_behaviors() {
    let mut db = RedisMiniDb::new();

    // Load simple script
    let load = execute(&mut db, &[b"SCRIPT", b"LOAD", b"return KEYS[1]"]);
    match load {
        RespReply::BulkString(ref sha) => assert_eq!(sha.len(), 16),
        other => panic!("expected bulk sha, got {other:?}"),
    }

    // Exists should be 1
    if let RespReply::BulkString(sha) = load {
        let exists = execute(&mut db, &[b"SCRIPT", b"EXISTS", &sha]);
        assert_eq!(exists, RespReply::Array(vec![RespReply::Integer(1)]));

        // Flush clears cache
        let flush = execute(&mut db, &[b"SCRIPT", b"FLUSH"]);
        assert_eq!(flush, RespReply::SimpleString("OK"));

        let exists2 = execute(&mut db, &[b"SCRIPT", b"EXISTS", &sha]);
        assert_eq!(exists2, RespReply::Array(vec![RespReply::Integer(0)]));
    }
}

#[test]
fn eval_key_and_arg_parsing_and_returning() {
    let mut db = RedisMiniDb::new();

    // return {KEYS[1],ARGV[1]} with 1 key and one arg
    let reply = execute(
        &mut db,
        &[b"EVAL", b"return {KEYS[1],ARGV[1]}", b"1", b"k", b"v"],
    );
    assert_eq!(
        reply,
        RespReply::Array(vec![
            RespReply::BulkString(b"k".to_vec()),
            RespReply::BulkString(b"v".to_vec())
        ])
    );
}

#[test]
fn eval_redis_call_get_set_via_stub() {
    let mut db = RedisMiniDb::new();

    // SET via EVAL
    let set_reply = execute(
        &mut db,
        &[
            b"EVAL",
            b"return redis.call('SET', KEYS[1], ARGV[1])",
            b"1",
            b"mykey",
            b"hello",
        ],
    );
    assert_eq!(set_reply, RespReply::SimpleString("OK"));

    // GET via EVAL
    let get_reply = execute(
        &mut db,
        &[
            b"EVAL",
            b"return redis.call('GET', KEYS[1])",
            b"1",
            b"mykey",
        ],
    );
    assert_eq!(get_reply, RespReply::BulkString(b"hello".to_vec()));
}

#[test]
fn evalsha_success_and_noscript() {
    let mut db = RedisMiniDb::new();
    let load = execute(&mut db, &[b"SCRIPT", b"LOAD", b"return ARGV[1]"]);
    let sha = match load {
        RespReply::BulkString(s) => s,
        other => panic!("{other:?}"),
    };

    // EVALSHA with known sha
    let reply = execute(&mut db, &[b"EVALSHA", &sha, b"0", b"arg1"]);
    assert_eq!(reply, RespReply::BulkString(b"arg1".to_vec()));

    // EVALSHA with unknown sha
    let miss = execute(&mut db, &[b"EVALSHA", b"deadbeefdeadbeef", b"0"]);
    match miss {
        RespReply::Error(msg) => assert!(msg.starts_with("NOSCRIPT")),
        other => panic!("expected NOSCRIPT error, got {other:?}"),
    }
}

#[test]
fn eval_invalid_numkeys_returns_error() {
    let mut db = RedisMiniDb::new();
    let bad = execute(&mut db, &[b"EVAL", b"return KEYS[1]", b"2", b"onlyone"]);
    match bad {
        RespReply::Error(msg) => assert!(msg.contains("invalid number of keys")),
        other => panic!("expected invalid numkeys error, got {other:?}"),
    }
}
