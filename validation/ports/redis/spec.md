# Redis Rust Porting Validation Spec

## Purpose

This target measures whether the Rust ownership-error navigation tool helps a low-cost model port Redis C code to Rust when the source code has streaming buffers, cursor state, partial frames, owned argument transfer, mutable in-memory data structures, transactions, persistence, networking, and multi-client server state.

The cJSON validation target has stayed too easy because the chosen Rust design used an owned tree with straightforward `Vec` ownership. Redis RESP parsing should create more ownership pressure through mutable input buffers, cursor advancement, partial reads, command extraction, and buffer compaction.

## Upstream Scope

- Repository: `https://github.com/redis/redis.git`
- Tag: `7.2.4`
- Commit: `d2c8a4b91e8c0e6aefd1f5bc0bf582cddbe046b7`
- License: BSD-3-Clause, recorded in `upstream/UPSTREAM.md`
- Primary C source:
  - `src/networking.c`: `processInputBuffer`, `processInlineBuffer`, `processMultibulkBuffer`
  - `src/server.h`: `PROTO_INLINE_MAX_SIZE`, `PROTO_MBULK_BIG_ARG`, `PROTO_REQ_INLINE`, `PROTO_REQ_MULTIBULK`
  - `src/sds.c` and `src/util.c`: useful references for string splitting and integer parsing behavior

## Functional Scope

Implement a Rust Redis-compatible validation port in incremental slices. The first milestones built a parser and in-memory command executor. The expanded goal is to grow this into a complete Rust Redis server implementation suitable for compatibility validation against Redis behavior.

Already completed parser scope:

- RESP2 multibulk command arrays such as `*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n`.
- Partial input where a command is incomplete until more bytes are appended.
- Multiple commands in one input buffer.
- Buffer cursor advancement and compaction after parsed commands are consumed.
- Inline commands such as `PING\r\n` and `SET key value\r\n` after multibulk parsing is established.
- Protocol errors for malformed multibulk lengths, bulk lengths, missing `$`, invalid line endings, overlarge inline requests, and unbalanced inline quoting.

Expanded full-port scope:

- Command dispatch and metadata for all supported commands.
- RESP2 and RESP3 protocol surfaces.
- TCP server loop, client sessions, command pipelining, and graceful shutdown.
- Multi-database keyspace, expiration, eviction, and memory-policy behavior.
- Complete core data type command families for strings, lists, hashes, sets, sorted sets, streams, transactions, keyspace, scanning, and scripting-facing behavior.
- Pub/Sub, blocking commands, stream consumer groups, ACL/auth, config, and observability commands.
- RDB snapshot loading/saving and AOF append/replay.
- Replication protocol, partial sync, and cluster slot/routing behavior.
- Compatibility fixtures and integration tests derived from Redis upstream behavior.

## Non-Goals

- Exact drop-in performance parity with Redis C in early validation phases.
- Redis module ABI compatibility until the core server reaches compatibility milestones.
- Unsafe Rust unless an iteration explicitly records it as a measurement event and is reviewed.
- No exact SDS allocator behavior or Redis object reference counting.

## Data Model Expectations

The Rust port should expose a small parser API, for example:

```rust
pub struct RespCommandParser { /* internal buffer and cursor */ }

pub enum ParseOutcome {
    Complete(Vec<Command>),
    Incomplete,
}

pub struct Command {
    pub args: Vec<Vec<u8>>,
}
```

The exact names may change, but the API should make these states observable:

- command completed with owned argument bytes
- parser needs more bytes
- protocol error with a stable error variant

## Ownership-Pressure Points

This target should deliberately exercise Rust ownership behavior that cJSON avoided:

- Maintaining a mutable buffer while returning owned command arguments.
- Parsing partial frames without holding stale borrows across mutation.
- Compacting or draining consumed bytes after command extraction.
- Moving large bulk byte ranges out of the parser where practical.
- Parsing multiple commands in a loop while preserving parser state for incomplete trailing bytes.

## Acceptance Criteria

- Each implementation iteration saves `cargo check --message-format=json` to `reports/iteration-NNN/cargo-check.jsonl`.
- Each iteration generates `ownership-report.json` and `ownership-report.html` from the saved JSONL.
- If a report contains E0382, E0499, or E0502, the next lightweight-model attempt must receive the generated report as the primary fix guide and must record whether diagnostics decreased.
- Each iteration records model, prompt summary, human ownership hints, command result, diagnostic counts, shortcut pressure, and next action in `notes/iteration-log.md`.
- `cargo fmt -- --check` and `cargo test` pass for completed iterations.
- Existing repository validation commands continue to pass when tracked files outside the Rust validation crate are changed.
- Completion requires command compatibility, TCP server operation, persistence, replication, cluster basics, ACL/auth, observability, and a final compatibility report set documenting remaining known gaps.
