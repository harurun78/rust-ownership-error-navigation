# Porting Targets

This directory tracks C/C++ to Rust porting experiments.

| Target | Upstream | Status | Why It Fits |
| --- | --- | --- | --- |
| cJSON | `DaveGamble/cJSON` | Selected | Compact C JSON parser with tree ownership, string ownership, allocation, and cleanup paths. |
| Redis | `redis/redis` | In validation | Streaming RESP parser and server state with mutable buffers, owned command transfer, and multi-client ownership pressure. |
| libpng | `pnggroup/libpng` | Selected | Byte-level C parser with progressive input, signature/chunk state, allocated buffers, and error-path cleanup. |

## Directory Contract

Each target directory should include a `README.md` with:

- upstream repository and version or commit under evaluation
- target slice and explicit non-goals
- ownership risks expected during Rust migration
- commands for generating rustc JSONL diagnostics
- report output locations
- evaluation notes for low-cost model runs