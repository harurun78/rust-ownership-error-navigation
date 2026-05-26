Iteration 034: Phase 31 Persistence

Summary:
- Implemented deterministic snapshot save/load (RDB-like subset header `RDBLIKEv1`).
- Snapshot includes string/list/hash/set/zset/stream entry payloads with deterministic ordering and expiry as remaining seconds.
- Implemented simple AOF append/replay framing (length-prefixed args) and placeholder fsync policy.
- Added integration tests for snapshot determinism, malformed input rejection, and AOF append/replay.

Diagnostics:
- cargo check succeeded; tests passed.
- Ownership report is clean: total diagnostics 0, supported diagnostics 0, unsupported diagnostics 0.
- Main verification fixed the initial unused `now` warning, corrected snapshot header parsing, and added a valid snapshot load roundtrip assertion before regenerating this report.

Compatibility notes / boundary:
- This implements an RDB-like deterministic subset (header `RDBLIKEv1`) but is not byte-for-byte compatible with upstream Redis RDB format.
- Streams: only entry list and fields are serialized; consumer group state (`groups` and `pending`) is omitted in this iteration.
- Expiration TTLs are stored as remaining seconds at snapshot time; on load these are reconstituted as `Instant::now() + ttl`.
- AOF uses a simple binary length-prefixed framing for commands; replay constructs `Command` values and executes them sequentially.

Next steps:
- Optional: include stream consumer group and pending metadata in snapshot for full recovery.
- Optional: implement configurable fsync behavior for AOF (use `sync`/`fdatasync`).
