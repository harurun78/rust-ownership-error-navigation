Iteration 037 Replication smoke slice

Date: 2026-05-27

Summary:
- Implemented minimal replication role state on `RedisMiniSession`.
- Added `REPLICAOF`, `ROLE`, `REPLCONF`, and `PSYNC` command stubs.
- Added a simple propagation log (`propagation_log`) and `ReplicationCheckpoint` with monotonically increasing `offset` and a fixed `replication_id`.
- Writes performed in master mode append entries to the propagation log with increasing offsets; reads and admin commands do not.
- Replica mode enforces read-only behavior for write commands returning a `READONLY`-style error.
- Implemented deterministic `PSYNC ? -1` `FULLRESYNC` reply and `REPLCONF` `OK` stub.
- INFO replication section augmented with `role`, `master_replid`, and `master_repl_offset` fields.

Tests added:
- `replication.rs` covering metadata, role transitions, write propagation offsets, replica read-only enforcement, handshake stubs, arity checks, transaction compatibility, and TCP session command handling.

Diagnostics:
- `cargo check` successful; no E0382/E0499/E0502 diagnostics observed.
- `cargo test` passed; all tests passed.

Notes on ownership pressure and shortcuts:
- No use of `unsafe`, `Rc<RefCell<_>>`, or `Arc<Mutex<_>>`.
- Used narrow `to_vec()` copies where required for propagation log entries and reply construction.
- No broad clone shortcuts introduced.

Next steps:
- Extend partial-sync model with real checkpoint persistence and replication handshake state machine.
- Add a replication applier that can ingest the propagation log and apply to replicas (simulation).
- Consider introducing limited binary-safe checkpoint snapshots to test PSYNC resume behavior.
