# Redis RESP Parser Porting Plan

## Strategy

Start with a narrow RESP parser crate rather than the full Redis server. The implementation will mirror Redis request parsing behavior enough to test ownership navigation around mutable buffers and partial input.

## Target Architecture

- `rust-port/src/lib.rs`: public exports.
- `rust-port/src/error.rs`: protocol and parser error variants.
- `rust-port/src/command.rs`: command argument model.
- `rust-port/src/parser.rs`: streaming parser with input buffer, cursor, and compaction.
- `rust-port/tests/resp_parser.rs`: tests derived from Redis request parser behavior.

## Iteration Plan

1. Crate skeleton and RESP multibulk happy path.
2. Partial multibulk input and parser state retention.
3. Multiple commands in one buffer plus buffer compaction.
4. Protocol error handling for invalid multibulk and bulk lengths.
5. Inline command parsing and quoting behavior.
6. Ownership-pressure slice: large bulk transfer and compaction without broad copying.

## Diagnostic Loop

For every low-cost implementation attempt:

1. Run `cargo check --message-format=json > ../reports/iteration-NNN/cargo-check.jsonl` from `rust-port/`.
2. If `cargo check` fails, generate this repository's ownership report before giving the model any fix guidance.
3. If `cargo check` succeeds, run `cargo test`.
4. Record E0382, E0499, E0502 counts and whether the report affected the next attempt.

## Design Constraints

- Keep upstream Redis checkout ignored and read-only.
- Keep each slice compile-checkable and testable.
- Prefer owned `Vec<u8>` command arguments at API boundaries.
- Avoid `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, and broad `.clone()` shortcuts unless recorded as measurement events.
- Do not port networking or server command execution in this validation phase.

## Risks

- A fully owned Rust buffer design may still be too easy for the lightweight model.
- Exact Redis inline parsing via `sdssplitargs` has nuanced quoting behavior; the first inline slice should cover only representative behavior.
- Optimizing large bulk transfer too early may distract from measuring ownership diagnostics.

## Completion Definition

The Redis validation setup is ready when upstream metadata, spec, plan, tasks, quickstart, and iteration log exist, and the upstream checkout is verified ignored by Git.
