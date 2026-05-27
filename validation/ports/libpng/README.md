# libpng Porting Validation

This target measures whether ownership-error navigation helps port selected libpng C parsing routines to Rust.

## Upstream

- Repository: `https://github.com/pnggroup/libpng.git`
- Tag: `v1.6.58`
- Tag object: `fdc7185dfedbddce8c2487bc171f66af4fca24ab`
- Peeled commit: `3061454d980de7d53608f594194cfac722721d2a`
- License: PNG Reference Library License version 2
- Local checkout: `validation/ports/libpng/upstream/libpng/` (ignored)

## Initial Slice

The first Rust slice ports PNG signature checking and a streaming chunk-header reader:

- `png_sig_cmp`-style partial signature comparison
- partial input buffering for file signature and chunk header bytes
- extraction of chunk length and 4-byte chunk type into owned Rust values
- deterministic errors for invalid signatures, malformed chunk type bytes, and excessive chunk length

## Ownership Risks

- Maintaining a mutable input buffer while returning owned chunk metadata.
- Compacting consumed bytes without keeping stale borrows into the buffer.
- Avoiding broad `clone`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, and `unsafe` shortcuts.

## Commands

```bash
cd validation/ports/libpng/rust-port
cargo fmt -- --check
cargo check --message-format=json > ../reports/iteration-001/cargo-check.jsonl
cargo test
```

Generate the navigation report from the repository root:

```bash
npm run build
node dist/cli/main.js \
  --input validation/ports/libpng/reports/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/libpng/reports/iteration-001/ownership-report.json \
  --html-out validation/ports/libpng/reports/iteration-001/ownership-report.html
```
