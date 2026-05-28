# Porting Targets

This directory tracks C/C++ to Rust porting experiments.

| Target | Upstream | Status | Why It Fits |
| --- | --- | --- | --- |
| cJSON | `DaveGamble/cJSON` | Selected | Compact C JSON parser with tree ownership, string ownership, allocation, and cleanup paths. |
| http-parser-js-streaming | `creationix/http-parser-js` | Completed | JavaScript callback-driven streaming parser shape with parser state, callback hooks, and input buffer slices. |
| Redis | `redis/redis` | In validation | Streaming RESP parser and server state with mutable buffers, owned command transfer, and multi-client ownership pressure. |
| sax-js-queued-events | `isaacs/sax-js` | Iteration 001 started | JavaScript XML streaming tokenizer with queued events, parser buffer views, and callback-style delivery. |
| libpng | `pnggroup/libpng` | Selected | Byte-level C parser with progressive input, signature/chunk state, allocated buffers, and error-path cleanup. |

## Directory Contract

Each target directory should include a `README.md` with:

- upstream repository and version or commit under evaluation
- target slice and explicit non-goals
- ownership risks expected during Rust migration
- commands for generating rustc JSONL diagnostics
- report output locations
- evaluation notes for low-cost model runs