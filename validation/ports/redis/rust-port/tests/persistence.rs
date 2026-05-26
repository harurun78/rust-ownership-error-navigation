use std::fs;
use std::path::PathBuf;

use rust_port::Command;
use rust_port::RedisMiniDb;

fn reports_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("persistence-tests")
        .join("iteration-034");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn read_bytes(path: &PathBuf) -> Vec<u8> {
    fs::read(path).expect("read bytes")
}

#[test]
fn snapshot_roundtrip_and_determinism() {
    let dir = reports_dir();
    let snap1 = dir.join("snap1.rdb");
    let snap2 = dir.join("snap2.rdb");

    let mut db = RedisMiniDb::new();
    // set string
    db.execute(Command::new(vec![
        b"SET".to_vec(),
        b"key1".to_vec(),
        b"value".to_vec(),
    ]));
    // list
    db.execute(Command::new(vec![
        b"RPUSH".to_vec(),
        b"l".to_vec(),
        b"a".to_vec(),
        b"b".to_vec(),
    ]));
    // hash
    db.execute(Command::new(vec![
        b"HSET".to_vec(),
        b"h".to_vec(),
        b"f".to_vec(),
        b"v".to_vec(),
    ]));
    // set
    db.execute(Command::new(vec![
        b"SADD".to_vec(),
        b"s".to_vec(),
        b"m".to_vec(),
    ]));
    // zset
    db.execute(Command::new(vec![
        b"ZADD".to_vec(),
        b"z".to_vec(),
        b"1".to_vec(),
        b"m".to_vec(),
    ]));
    // stream
    db.execute(Command::new(vec![
        b"XADD".to_vec(),
        b"x".to_vec(),
        b"*".to_vec(),
        b"f".to_vec(),
        b"v".to_vec(),
    ]));

    db.save_snapshot(&snap1).expect("save snap1");
    db.save_snapshot(&snap2).expect("save snap2");

    let a = read_bytes(&snap1);
    let b = read_bytes(&snap2);
    assert_eq!(a, b, "snapshot should be deterministic and identical");

    let mut loaded = RedisMiniDb::load_snapshot(&snap1).expect("load snapshot");
    assert_eq!(
        loaded.execute(Command::new(vec![b"GET".to_vec(), b"key1".to_vec()])),
        rust_port::RespReply::BulkString(b"value".to_vec())
    );
    assert_eq!(
        loaded.execute(Command::new(vec![
            b"LRANGE".to_vec(),
            b"l".to_vec(),
            b"0".to_vec(),
            b"-1".to_vec(),
        ])),
        rust_port::RespReply::Array(vec![
            rust_port::RespReply::BulkString(b"a".to_vec()),
            rust_port::RespReply::BulkString(b"b".to_vec()),
        ])
    );
    assert_eq!(
        loaded.execute(Command::new(vec![
            b"HGET".to_vec(),
            b"h".to_vec(),
            b"f".to_vec(),
        ])),
        rust_port::RespReply::BulkString(b"v".to_vec())
    );
    assert_eq!(
        loaded.execute(Command::new(vec![
            b"SISMEMBER".to_vec(),
            b"s".to_vec(),
            b"m".to_vec(),
        ])),
        rust_port::RespReply::Integer(1)
    );
    assert_eq!(
        loaded.execute(Command::new(vec![
            b"ZSCORE".to_vec(),
            b"z".to_vec(),
            b"m".to_vec(),
        ])),
        rust_port::RespReply::BulkString(b"1".to_vec())
    );
    assert_eq!(
        loaded.execute(Command::new(vec![b"XLEN".to_vec(), b"x".to_vec()])),
        rust_port::RespReply::Integer(1)
    );
}

#[test]
fn load_rejects_malformed() {
    let dir = reports_dir();
    let bad = dir.join("bad.rdb");
    fs::write(&bad, b"not a valid header").expect("write bad");
    let res = RedisMiniDb::load_snapshot(&bad);
    assert!(res.is_err(), "malformed snapshot should error");
}

#[test]
fn aof_append_and_replay() {
    let dir = reports_dir();
    let aof = dir.join("test.aof");
    let snap_a = dir.join("aof_a.rdb");
    let snap_b = dir.join("aof_b.rdb");
    let _ = fs::remove_file(&aof);

    let mut db = RedisMiniDb::new();
    let cmds = vec![
        Command::new(vec![b"SET".to_vec(), b"k".to_vec(), b"v".to_vec()]),
        Command::new(vec![b"RPUSH".to_vec(), b"l".to_vec(), b"x".to_vec()]),
    ];
    for cmd in &cmds {
        let exec_cmd = Command::new(cmd.args.clone());
        db.execute(exec_cmd);
        db.append_aof(
            &aof,
            &Command::new(cmd.args.clone()),
            rust_port::AofFsyncPolicy::NoFsync,
        )
        .expect("append aof");
    }
    db.save_snapshot(&snap_a).expect("save a");

    let mut db2 = RedisMiniDb::new();
    db2.replay_aof(&aof).expect("replay");
    db2.save_snapshot(&snap_b).expect("save b");

    let a = read_bytes(&snap_a);
    let b = read_bytes(&snap_b);
    assert_eq!(
        a, b,
        "AOF replay should produce equivalent database snapshot"
    );
}
