# Tasks: Redis RESP Parser Porting Validation

## Phase 1: Experiment Setup

- [x] R001 Create `validation/ports/redis/` target directory.
- [x] R002 Fetch Redis upstream into ignored checkout `validation/ports/redis/upstream/redis/`.
- [x] R003 Record upstream repository, tag, commit, license, and acquisition commands in `upstream/UPSTREAM.md`.
- [x] R004 Create Redis porting validation `spec.md`.
- [x] R005 Create Redis porting validation `plan.md`.
- [x] R006 Create Redis porting validation `tasks.md`.
- [x] R007 Create Redis porting validation `quickstart.md`.
- [x] R008 Create `notes/iteration-log.md` template.
- [x] R009 Verify upstream checkout remains ignored by Git before first implementation iteration.

## Phase 2: Rust Crate Skeleton

- [x] R010 Initialize Rust library crate in `validation/ports/redis/rust-port/`.
- [x] R011 Add crate README or module docs stating RESP parser-only scope.
- [x] R012 Define command argument model.
- [x] R013 Define parser error model.
- [x] R014 Expose crate modules from `rust-port/src/lib.rs`.

## Phase 3: RESP Multibulk Happy Path

- [x] R015 Add tests for `PING`, `GET key`, and `SET key value` as RESP multibulk frames.
- [x] R016 Add tests for binary-safe bulk strings containing spaces and null bytes.
- [x] R017 Implement multibulk length parsing.
- [x] R018 Implement bulk string length parsing.
- [x] R019 Implement command extraction into owned argument bytes.
- [x] R020 Save `cargo check --message-format=json` output to `reports/iteration-001/cargo-check.jsonl`.
- [x] R021 Generate `reports/iteration-001/ownership-report.json`.
- [x] R022 Generate `reports/iteration-001/ownership-report.html`.
- [x] R023 Record iteration-001 results and diagnostic counts in `notes/iteration-log.md`.
- [x] R024 Run final `cargo test` in `rust-port/`.

## Phase 4: Partial Input And State Retention

- [x] R025 Add tests for command frames split across multiple `append` calls.
- [x] R026 Add tests for incomplete multibulk length, bulk length, and bulk payload states.
- [x] R027 Preserve parser state without producing a command until complete.
- [x] R028 Save and report diagnostics for iteration-002.

## Phase 5: Multiple Commands And Buffer Compaction

- [x] R029 Add tests for two or more commands in one input buffer.
- [x] R030 Add tests that incomplete trailing bytes remain after complete commands are extracted.
- [x] R031 Implement consumed-byte compaction after successful parse.
- [x] R032 Save and report diagnostics for iteration-003.

## Phase 6: Protocol Errors

- [x] R033 Add tests for invalid multibulk length.
- [x] R034 Add tests for invalid bulk length.
- [x] R035 Add tests for expected `$` but got another byte.
- [x] R036 Add tests for overlarge inline or multibulk header strings.
- [x] R037 Implement stable protocol error variants.
- [x] R038 Save and report diagnostics for iteration-004.

## Phase 7: Inline Command Parsing

- [x] R039 Add tests for inline `PING`, `SET key value`, and quoted values.
- [x] R040 Add tests for unbalanced inline quotes.
- [x] R041 Implement representative `sdssplitargs`-style inline parsing.
- [x] R042 Save and report diagnostics for iteration-005.

## Phase 8: Ownership Pressure Slice

- [x] R043 Add tests for large bulk payload extraction.
- [x] R044 Add tests for compaction after extracting a large argument.
- [x] R045 Attempt to move owned byte ranges out of the parser buffer where practical.
- [x] R046 Record shortcut pressure: `clone`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, and `unsafe`.
- [x] R047 Save and report diagnostics for iteration-006.

## Phase 9: Minimal String Command Executor

- [x] R048 Add tests for RESP reply encoding: simple strings, bulk strings, null bulk strings, integers, and errors.
- [x] R049 Add tests for `PING` and `ECHO` command execution.
- [x] R050 Add tests for `SET`, `GET`, `DEL`, and `EXISTS` against an in-memory string DB.
- [x] R051 Add tests for wrong arity and unknown command errors.
- [x] R052 Implement a minimal command executor with owned byte-vector keys and values.
- [x] R053 Preserve parser behavior and existing parser tests.
- [x] R054 Save `cargo check --message-format=json` output to `reports/iteration-007/cargo-check.jsonl`.
- [x] R055 Generate `reports/iteration-007/ownership-report.json`.
- [x] R056 Generate `reports/iteration-007/ownership-report.html`.
- [x] R057 Record iteration-007 results and diagnostic counts in `notes/iteration-log.md`.
- [x] R058 Run final `cargo test` in `rust-port/`.

## Phase 10: Integer String Commands

- [x] R059 Add tests for `INCR`, `DECR`, and `INCRBY` on missing and existing keys.
- [x] R060 Add tests for integer parse errors when the existing value or increment is not an integer.
- [x] R061 Add tests for integer overflow errors.
- [x] R062 Implement integer string command execution with binary-safe stored values.
- [x] R063 Preserve existing parser and command executor behavior.
- [x] R064 Save `cargo check --message-format=json` output to `reports/iteration-008/cargo-check.jsonl`.
- [x] R065 Generate `reports/iteration-008/ownership-report.json`.
- [x] R066 Generate `reports/iteration-008/ownership-report.html`.
- [x] R067 Record iteration-008 results and diagnostic counts in `notes/iteration-log.md`.
- [x] R068 Run final `cargo test` in `rust-port/`.

## Phase 11: Minimal List Commands

- [x] R069 Add tests for `LPUSH`, `RPUSH`, `LPOP`, and `RPOP` on missing and existing list keys.
- [x] R070 Add tests for list reply encoding with RESP arrays.
- [x] R071 Add tests for `LRANGE` with positive and negative indexes.
- [x] R072 Add tests for wrong-type errors when string commands touch lists or list commands touch strings.
- [x] R073 Refactor the minimal DB value model to support strings and lists.
- [x] R074 Preserve existing string command, integer command, and parser behavior.
- [x] R075 Save `cargo check --message-format=json` output to `reports/iteration-009/cargo-check.jsonl`.
- [x] R076 Generate `reports/iteration-009/ownership-report.json`.
- [x] R077 Generate `reports/iteration-009/ownership-report.html`.
- [x] R078 Record iteration-009 results and diagnostic counts in `notes/iteration-log.md`.
- [x] R079 Run final `cargo test` in `rust-port/`.

## Phase 12: Minimal Hash Commands

- [x] R080 Add tests for `HSET`, `HGET`, and `HDEL` on missing and existing hash keys.
- [x] R081 Add tests for `HGETALL` RESP array replies and binary-safe field/value pairs.
- [x] R082 Add tests for wrong-type errors between hashes, strings, and lists.
- [x] R083 Refactor the DB value model to support hash values without regressing strings or lists.
- [x] R084 Implement minimal `HSET`, `HGET`, `HDEL`, and `HGETALL` behavior.
- [x] R085 Preserve existing string, integer, list, parser, and reply encoding behavior.
- [x] R086 Save `cargo check --message-format=json` output to `reports/iteration-010/cargo-check.jsonl`.
- [x] R087 Generate `reports/iteration-010/ownership-report.json`.
- [x] R088 Generate `reports/iteration-010/ownership-report.html`.
- [x] R089 Record iteration-010 results and diagnostic counts in `notes/iteration-log.md`.
- [x] R090 Run final `cargo test` in `rust-port/`.

## Phase 13: Minimal Expiration Commands

- [x] R091 Add tests for `EXPIRE`, `TTL`, and `PERSIST` on missing and existing keys.
- [x] R092 Add tests for immediate expiration (`EXPIRE key 0`) removing string, list, and hash values.
- [x] R093 Add tests that writes clear any existing expiration for the key.
- [x] R094 Add tests that `DEL` also removes expiration metadata.
- [x] R095 Add expiration metadata to the DB without regressing value typing.
- [x] R096 Implement minimal `EXPIRE`, `TTL`, and `PERSIST` behavior with lazy expiration checks.
- [x] R097 Preserve existing string, integer, list, hash, parser, and reply encoding behavior.
- [x] R098 Save `cargo check --message-format=json` output to `reports/iteration-011/cargo-check.jsonl`.
- [x] R099 Generate `reports/iteration-011/ownership-report.json`.
- [x] R100 Generate `reports/iteration-011/ownership-report.html`.
- [x] R101 Record iteration-011 results and diagnostic counts in `notes/iteration-log.md` and run final `cargo test`.

## Phase 14: Minimal Set Commands

- [x] R102 Add tests for `SADD`, `SREM`, and `SISMEMBER` on missing and existing set keys.
- [x] R103 Add tests for `SMEMBERS` RESP array replies with deterministic ordering.
- [x] R104 Add tests for binary-safe set members.
- [x] R105 Add tests for wrong-type errors between sets, strings, lists, and hashes.
- [x] R106 Refactor the DB value model to support set values without regressing existing value types.
- [x] R107 Implement minimal `SADD`, `SREM`, `SISMEMBER`, and `SMEMBERS` behavior.
- [x] R108 Preserve existing expiration behavior for set writes and reads.
- [x] R109 Save `cargo check --message-format=json` output to `reports/iteration-012/cargo-check.jsonl`.
- [x] R110 Generate `reports/iteration-012/ownership-report.json`.
- [x] R111 Generate `reports/iteration-012/ownership-report.html`.
- [x] R112 Record iteration-012 results and diagnostic counts in `notes/iteration-log.md` and run final `cargo test`.

## Phase 15: Minimal Keyspace Commands

- [x] R113 Add tests for `TYPE` across missing, string, list, hash, and set keys.
- [x] R114 Add tests for `RENAME` moving values and expiration metadata between keys.
- [x] R115 Add tests for `RENAMENX` preserving destination keys and moving only when absent.
- [x] R116 Add tests for `KEYS *` returning deterministic key names after lazy expiration cleanup.
- [x] R117 Implement minimal `TYPE`, `RENAME`, `RENAMENX`, and `KEYS` behavior.
- [x] R118 Preserve existing expiration, string, list, hash, set, parser, and reply encoding behavior.
- [x] R119 Save `cargo check --message-format=json` output to `reports/iteration-013/cargo-check.jsonl`.
- [x] R120 Generate `reports/iteration-013/ownership-report.json`.
- [x] R121 Generate `reports/iteration-013/ownership-report.html`.
- [x] R122 Record iteration-013 results and diagnostic counts in `notes/iteration-log.md` and run final `cargo test`.

## Phase 16: Set Algebra Store Commands

- [x] R123 Add tests for `SUNIONSTORE` with missing and existing source sets.
- [x] R124 Add tests for `SINTERSTORE` and `SDIFFSTORE` with multiple source sets.
- [x] R125 Add tests that store commands overwrite destination values and expiration metadata.
- [x] R126 Add tests for wrong-type errors when source keys are not sets.
- [x] R127 Implement minimal `SUNIONSTORE`, `SINTERSTORE`, and `SDIFFSTORE` behavior.
- [x] R128 Preserve existing expiration, keyspace, string, list, hash, set, parser, and reply encoding behavior.
- [x] R129 Save `cargo check --message-format=json` output to `reports/iteration-014/cargo-check.jsonl`.
- [x] R130 Generate `reports/iteration-014/ownership-report.json`.
- [x] R131 Generate `reports/iteration-014/ownership-report.html`.
- [x] R132 Record iteration-014 results and diagnostic counts in `notes/iteration-log.md` and run final `cargo test`.

## Phase 17: Minimal Transaction Commands

- [x] R133 Add tests for `MULTI`, queued command replies, `EXEC`, and `DISCARD`.
- [x] R134 Add tests that `EXEC` returns an array of command replies and applies queued writes in order.
- [x] R135 Add tests for transaction errors: nested `MULTI`, `EXEC` without `MULTI`, and `DISCARD` without `MULTI`.
- [x] R136 Add tests that queued commands preserve binary-safe argument bytes and existing expiration behavior.
- [x] R137 Add minimal transaction queue state to the DB.
- [x] R138 Implement minimal `MULTI`, `EXEC`, and `DISCARD` behavior without regressing existing commands.
- [x] R139 Save `cargo check --message-format=json` output to `reports/iteration-015/cargo-check.jsonl`.
- [x] R140 Generate `reports/iteration-015/ownership-report.json`.
- [x] R141 Generate `reports/iteration-015/ownership-report.html`.
- [x] R142 Record iteration-015 results and diagnostic counts in `notes/iteration-log.md` and run final `cargo test`.

## Phase 18: Minimal Sorted Set Commands

- [x] R143 Add tests for `ZADD`, `ZREM`, and `ZSCORE` on missing and existing sorted set keys.
- [x] R144 Add tests for `ZRANGE` with score/member deterministic ordering and negative indexes.
- [x] R145 Add tests for binary-safe sorted set members and integer score parsing errors.
- [x] R146 Add tests for wrong-type errors between sorted sets and existing value types.
- [x] R147 Refactor the DB value model to support sorted set values without regressing existing value types.
- [x] R148 Implement minimal `ZADD`, `ZREM`, `ZSCORE`, and `ZRANGE` behavior using integer scores.
- [x] R149 Preserve existing expiration and transaction behavior for sorted set commands.
- [x] R150 Save `cargo check --message-format=json` output to `reports/iteration-016/cargo-check.jsonl`.
- [x] R151 Generate `reports/iteration-016/ownership-report.json`.
- [x] R152 Generate `reports/iteration-016/ownership-report.html`.
- [x] R153 Record iteration-016 results and diagnostic counts in `notes/iteration-log.md` and run final `cargo test`.
