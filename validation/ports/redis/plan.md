# Redis Rust Porting Plan

## Strategy

Start with a narrow RESP parser crate, then expand in measured slices toward a complete Rust Redis-compatible server. Every slice remains compile-checkable and records ownership diagnostics so the navigation tool can be evaluated against real porting pressure.

## Target Architecture

- `rust-port/src/lib.rs`: public exports.
- `rust-port/src/error.rs`: protocol and parser error variants.
- `rust-port/src/command.rs`: command argument model.
- `rust-port/src/parser.rs`: streaming parser with input buffer, cursor, and compaction.
- `rust-port/src/executor.rs`: command dispatcher, in-memory DB, RESP replies, and command families.
- Future modules: server/client sessions, persistence, pub/sub, replication, cluster, ACL/auth, config, and compatibility harnesses.
- `rust-port/tests/resp_parser.rs`: tests derived from Redis request parser behavior.

## Iteration Plan

1. Parser and command argument extraction.
2. In-memory executor for core data structures.
3. Transactions, WATCH/UNWATCH, scans, sorted sets, and streams.
4. Command dispatch metadata and database/server separation.
5. TCP server loop, clients, pipelining, and integration fixtures.
6. Full command-family expansion by Redis data type.
7. Persistence, pub/sub, blocking commands, ACL/auth, config, observability.
8. Replication, cluster basics, compatibility harness, and final gap report.

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
- Keep each full-port phase behind tests and recorded diagnostic artifacts.

## Risks

- A fully owned Rust buffer design may still be too easy for the lightweight model.
- Exact Redis inline parsing via `sdssplitargs` has nuanced quoting behavior; the first inline slice should cover only representative behavior.
- Optimizing large bulk transfer too early may distract from measuring ownership diagnostics.

## Completion Definition

The full Redis validation port is complete when the task ledger has no remaining implementation phases, the Rust server can run compatibility fixtures through the TCP interface, persistence and replication smoke tests pass, and the final report records zero unresolved supported ownership diagnostics.
