Iteration 038 - Phase 35 Cluster Basics

Summary:
- Implemented Redis CRC16 hash slot calculation with support for hash tags `{...}`.
- Added `CLUSTER` command subset: `KEYSLOT`, `SLOTS`, `INFO`, `NODES`.
- Cluster test helpers on `RedisMiniSession` to enable deterministic local slot assignments and set MOVED/ASK redirect targets.
- In cluster mode, single-key commands for keys not in local slots return deterministic `MOVED` or `ASK` errors depending on configured targets. Multi-key commands validate same-slot; otherwise return `CROSSSLOT` error.

Files changed:
- src/executor.rs: cluster state, CRC16 and helpers, routing, CLUSTER command handling, tests integration.
- src/lib.rs: exported `redis_cluster_hash_slot`.
- tests/cluster.rs: new tests covering slot calculation, CLUSTER commands, MOVED/ASK, CROSSSLOT, transactions, and TCP session compatibility.

Commands run:
- `cargo fmt`
- `cargo check --message-format=json > ../reports/iteration-038/cargo-check.jsonl`
- `cargo test`
- `node ../../../../dist/cli/main.js --input ../reports/iteration-038/cargo-check.jsonl --json-out ../reports/iteration-038/ownership-report.json --html-out ../reports/iteration-038/ownership-report.html`

Result:
- `cargo check`: success
- `cargo test`: success (all tests passed)
- Ownership report: generated with zero diagnostics

Notes:
- Minimal, deterministic cluster behavior added for validation; not a full Redis cluster implementation.
- No unsafe or shared-mutable shortcuts were introduced.

Next steps:
- Expand CLUSTER SLOTS behavior to simulate slot migration and ASK handling over TCP.
- Add tests for multi-key command routing across transaction boundaries and pipelined TCP scenarios.
