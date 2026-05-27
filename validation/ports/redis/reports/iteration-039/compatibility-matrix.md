# Redis Compatibility Matrix — iteration-039

Phase 36 closes the lightweight Redis porting validation with a compatibility-oriented harness, smoke tests, and gap inventory. Status values are intentionally conservative: **complete** means covered for the scoped mini-Redis behavior, **partial** means representative Redis behavior exists with known caveats, **stub** means deterministic boundary behavior only, and **unsupported** means intentionally absent.

| Area | Status | Covered behavior | Key caveats |
|---|---|---|---|
| RESP2 parser | Complete | Inline and multibulk commands, partial input, pipelining, binary-safe bulk bytes, protocol errors | RESP3 parsing is not implemented; RESP3 is reply encoding only |
| TCP client session | Partial | `RedisMiniClientSession` and blocking TCP server loop preserve per-client parser/executor state | No async event loop, timeouts, client output buffering, TLS, or production networking |
| Strings | Partial | `PING`, `ECHO`, `SET` options, `GET`, `MGET`, `MSET`, `DEL`, `EXISTS`, `INCR*`, ranges, append/strlen/getset | Not exhaustive versus Redis 7.2 command surface |
| Lists | Partial | Push/pop/range/len/move and blocking-command placeholders | Blocking commands do not block; advanced list variants are limited |
| Hashes | Partial | `HSET`, `HGET`, `HDEL`, `HGETALL` and selected completion commands | Cursor and field-level command parity is incomplete |
| Sets | Partial | Add/remove/membership/members, algebra/store, deterministic ordering, scan-style subset | Randomized Redis behavior is deterministic in this port |
| Sorted sets | Partial | Integer-score `ZADD`, remove, score, rank/range/count/lex/scan subsets | Scores are integer-only; full floating-point and option parity is absent |
| Streams | Partial | `XADD`, `XLEN`, `XRANGE`, `XREAD`, trim/delete, consumer-group boundary commands | Blocking reads, complete pending/claim semantics, and ID edge cases are incomplete |
| Keyspace/database | Partial | `SELECT`, `DBSIZE`, `KEYS`, `SCAN`, `TYPE`, `RENAME`, expiration metadata | Eviction policies and full pattern/cursor semantics are not implemented |
| Transactions/watch | Partial | `MULTI`, `EXEC`, `DISCARD`, `WATCH`, `UNWATCH`, queued replies and watched-key invalidation | No Lua/replication integration beyond implemented executor state |
| Pub/Sub | Partial | Subscribe acknowledgements, subscribed-mode restrictions, pattern matching, in-memory broker delivery | No TCP fanout between live sockets; pattern matching is a simple glob subset |
| Persistence | Partial | Deterministic snapshot save/load and AOF append/replay for implemented value types | Format is a validation subset, not Redis RDB/AOF byte-compatible |
| ACL/auth/config/admin | Partial | `AUTH`, ACL user subset, category checks, `CONFIG`, `INFO`, `COMMAND`, `CLIENT`, `TIME`, `SLOWLOG` placeholder | No full ACL rule grammar, persistence, or operational configuration surface |
| Scripting/functions | Stub | `SCRIPT LOAD/EXISTS/FLUSH`, `EVAL`, `EVALSHA` deterministic key/argv and simple `redis.call` stubs | No Lua engine, sandbox, script replication, functions, or command effects beyond stubs |
| Replication | Stub | `ROLE`, `REPLICAOF`, `REPLCONF`, `PSYNC ? -1`, read-only replica mode, propagation log offsets | No actual replica sockets, backlog, RDB transfer, partial resync, or PSYNC variants |
| Cluster | Stub | CRC16 slots with hash tags, `CLUSTER KEYSLOT/SLOTS/INFO/NODES`, MOVED/ASK/CROSSSLOT smoke | No gossip, failover, slot migration state machine, replicas, or multi-node topology |
| Modules | Unsupported | None | Redis module ABI is outside validation scope |
| Exact Redis performance | Unsupported | None | Throughput, memory layout, SDS/object encodings, and allocator behavior are non-goals |

Conclusion: the port is suitable for deterministic compatibility validation and ownership-diagnostic experiments across parser, executor, persistence, pub/sub, replication, and cluster boundaries. It is not a production Redis replacement and intentionally leaves network fanout, byte-compatible persistence, full replication, real Lua/functions, module ABI, and cluster orchestration as known gaps.