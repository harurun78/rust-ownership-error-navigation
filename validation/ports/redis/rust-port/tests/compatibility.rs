use std::fs;
use std::path::PathBuf;

use rust_port::{
    AofFsyncPolicy, Command, RedisMiniClientSession, RedisMiniDb, RedisPubSubBroker, RespReply,
};

fn compatibility_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("compatibility-tests")
        .join("iteration-039");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn decode_escapes(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\' && index + 1 < bytes.len() {
            match bytes[index + 1] {
                b'r' => output.push(b'\r'),
                b'n' => output.push(b'\n'),
                b't' => output.push(b'\t'),
                b'0' => output.push(0),
                b'\\' => output.push(b'\\'),
                other => {
                    output.push(b'\\');
                    output.push(other);
                }
            }
            index += 2;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }

    output
}

fn resp_frame(args: &[Vec<u8>]) -> Vec<u8> {
    let mut frame = format!("*{}\r\n", args.len()).into_bytes();
    for arg in args {
        frame.extend_from_slice(format!("${}\r\n", arg.len()).as_bytes());
        frame.extend_from_slice(arg);
        frame.extend_from_slice(b"\r\n");
    }
    frame
}

fn command(args: &[&[u8]]) -> Command {
    Command::new(args.iter().map(|arg| arg.to_vec()).collect())
}

fn parse_fixture_commands(input: &str) -> Vec<Vec<Vec<u8>>> {
    input
        .split('|')
        .map(|command_text| {
            command_text
                .split(',')
                .map(decode_escapes)
                .collect::<Vec<Vec<u8>>>()
        })
        .collect()
}

#[test]
fn fixture_transcripts_match_expected_resp_replies() {
    let fixture = include_str!("fixtures/compatibility-transcripts.resp");
    let mut covered_families = Vec::new();

    for (line_number, line) in fixture.lines().enumerate() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.splitn(4, '\t').collect();
        assert_eq!(
            fields.len(),
            4,
            "fixture line {} should have family, case, commands, and expected RESP",
            line_number + 1
        );

        let family = fields[0];
        let case_name = fields[1];
        let commands = parse_fixture_commands(fields[2]);
        let expected = decode_escapes(fields[3]);

        let mut input = Vec::new();
        for args in commands {
            input.extend(resp_frame(&args));
        }

        let mut client = RedisMiniClientSession::new();
        let actual = client
            .process_input(&input)
            .unwrap_or_else(|error| panic!("{family}/{case_name} failed to parse: {error:?}"));
        assert_eq!(actual, expected, "{family}/{case_name} transcript mismatch");
        covered_families.push(family.to_string());
    }

    covered_families.sort();
    covered_families.dedup();
    assert_eq!(
        covered_families,
        vec![
            "cluster",
            "hashes",
            "lists",
            "persistence",
            "pubsub",
            "replication",
            "scripting",
            "sets",
            "sorted-sets",
            "streams",
            "strings",
        ]
    );
}

#[test]
fn client_session_smoke_covers_parser_executor_pubsub_and_replication_boundaries() {
    let mut client = RedisMiniClientSession::new();
    let set_frame = resp_frame(&[b"SET".to_vec(), b"tcp".to_vec(), b"value".to_vec()]);

    assert_eq!(client.process_input(&set_frame[..8]).unwrap(), Vec::new());

    let mut remaining = set_frame[8..].to_vec();
    remaining.extend(resp_frame(&[b"GET".to_vec(), b"tcp".to_vec()]));
    assert_eq!(
        client.process_input(&remaining).unwrap(),
        b"+OK\r\n$5\r\nvalue\r\n".to_vec()
    );

    let mut pubsub = RedisMiniClientSession::new();
    let mut pubsub_input = resp_frame(&[b"SUBSCRIBE".to_vec(), b"news".to_vec()]);
    pubsub_input.extend(resp_frame(&[b"GET".to_vec(), b"news".to_vec()]));
    assert_eq!(
        pubsub.process_input(&pubsub_input).unwrap(),
        b"*3\r\n$9\r\nsubscribe\r\n$4\r\nnews\r\n:1\r\n-ERR only (P)SUBSCRIBE / (P)UNSUBSCRIBE / PING / QUIT / HELLO allowed in subscribed state\r\n".to_vec()
    );

    let mut replication = RedisMiniClientSession::new();
    let mut replication_input = resp_frame(&[
        b"REPLICAOF".to_vec(),
        b"127.0.0.1".to_vec(),
        b"6379".to_vec(),
    ]);
    replication_input.extend(resp_frame(&[b"ROLE".to_vec()]));
    assert_eq!(
        replication.process_input(&replication_input).unwrap(),
        b"+OK\r\n*5\r\n$7\r\nreplica\r\n$9\r\n127.0.0.1\r\n:6379\r\n$9\r\nconnected\r\n:0\r\n"
            .to_vec()
    );
}

#[test]
fn pubsub_broker_transcript_delivers_pending_messages_between_clients() {
    let mut broker = RedisPubSubBroker::new();
    let subscriber = broker.add_session();
    let publisher = broker.add_session();

    assert_eq!(
        broker.execute(subscriber, command(&[b"SUBSCRIBE", b"news"])),
        RespReply::Array(vec![
            RespReply::BulkString(b"subscribe".to_vec()),
            RespReply::BulkString(b"news".to_vec()),
            RespReply::Integer(1),
        ])
    );
    assert_eq!(
        broker.execute(publisher, command(&[b"PUBLISH", b"news", b"hello"])),
        RespReply::Integer(1)
    );
    assert_eq!(
        broker.drain_messages(subscriber),
        vec![RespReply::Array(vec![
            RespReply::BulkString(b"message".to_vec()),
            RespReply::BulkString(b"news".to_vec()),
            RespReply::BulkString(b"hello".to_vec()),
        ])]
    );
}

#[test]
fn persistence_snapshot_and_aof_smoke_roundtrip_fixture_commands() {
    let dir = compatibility_dir();
    let snapshot = dir.join("compatibility.rdb");
    let aof = dir.join("compatibility.aof");
    let _ = fs::remove_file(&snapshot);
    let _ = fs::remove_file(&aof);

    let mut db = RedisMiniDb::new();
    let commands = [
        command(&[b"SET", b"persist", b"value"]),
        command(&[b"RPUSH", b"queue", b"a", b"b"]),
        command(&[b"XADD", b"stream", b"1-0", b"f", b"v"]),
    ];

    for command in commands {
        db.append_aof(&aof, &command, AofFsyncPolicy::NoFsync)
            .expect("append compatibility aof");
        db.execute(command);
    }
    db.save_snapshot(&snapshot)
        .expect("save compatibility snapshot");

    let mut loaded_snapshot = RedisMiniDb::load_snapshot(&snapshot).expect("load snapshot");
    assert_eq!(
        loaded_snapshot.execute(command(&[b"GET", b"persist"])),
        RespReply::BulkString(b"value".to_vec())
    );

    let mut replayed = RedisMiniDb::new();
    replayed.replay_aof(&aof).expect("replay compatibility aof");
    assert_eq!(
        replayed.execute(command(&[b"LRANGE", b"queue", b"0", b"-1"])),
        RespReply::Array(vec![
            RespReply::BulkString(b"a".to_vec()),
            RespReply::BulkString(b"b".to_vec()),
        ])
    );
    assert_eq!(
        replayed.execute(command(&[b"XLEN", b"stream"])),
        RespReply::Integer(1)
    );
}
