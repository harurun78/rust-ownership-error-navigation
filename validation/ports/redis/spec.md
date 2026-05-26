# Redis RESP Parser Porting Validation Spec

## Purpose

This target measures whether the Rust ownership-error navigation tool helps a low-cost model port Redis C code to Rust when the source code has streaming buffers, cursor state, partial frames, and owned argument transfer.

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

Implement a Rust library that parses Redis client request frames into command arguments.

The initial port must support:

- RESP2 multibulk command arrays such as `*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n`.
- Partial input where a command is incomplete until more bytes are appended.
- Multiple commands in one input buffer.
- Buffer cursor advancement and compaction after parsed commands are consumed.
- Inline commands such as `PING\r\n` and `SET key value\r\n` after multibulk parsing is established.
- Protocol errors for malformed multibulk lengths, bulk lengths, missing `$`, invalid line endings, overlarge inline requests, and unbalanced inline quoting.

## Non-Goals

- No Redis server, networking, authentication, replication, ACLs, command execution, persistence, cluster, scripting, or pub/sub.
- No RESP replies in the first validation phase.
- No exact SDS allocator behavior or Redis object reference counting.
- No unsafe Rust unless an iteration explicitly records it as a measurement event.

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
- Each iteration records model, prompt summary, human ownership hints, command result, diagnostic counts, shortcut pressure, and next action in `notes/iteration-log.md`.
- `cargo fmt -- --check` and `cargo test` pass for completed iterations.
- Existing repository validation commands continue to pass when tracked files outside the Rust validation crate are changed.
