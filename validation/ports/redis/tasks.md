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
