# miniz Streaming Porting Comparison Plan

## Summary

Run a paired validation target with two implementations of the same miniz-style streaming behavior:

1. Compatibility-preserving: C-like stream struct and lifecycle functions.
2. Behavior-only Rust-native: owned output and idiomatic `Result` API.

The goal is to test whether navigation reports provide repair value under compatibility pressure and prevention value under Rust-native redesign.

## Phase 1: Target Setup

- Create target docs, notes, reports, and track directories.
- Pin upstream miniz metadata.
- Define first paired slice and metrics.

## Phase 2: Paired Slice 001 - Stream Lifecycle Pass-Through

- Compatibility track: implement `MzStream`, `mz_inflate_init`, `mz_inflate`, `mz_inflate_end` with caller-provided input/output buffers.
- Rust-native track: implement owned pass-through inflator API with deterministic partial output behavior.
- Run cargo checks, tests, and navigation report generation for both tracks.

## Phase 3: Assessment

- Compare diagnostics, shortcut pressure, and API complexity.
- Record whether Rust-native design avoided ownership pressure.
- Select next slice: stored deflate block decode, allocator hook behavior, or callback/user-data pressure.

## Verification Commands

From each track crate:

```bash
cargo fmt -- --check
cargo test
cargo check --message-format=json > ../../../reports/<track>/iteration-001/cargo-check.jsonl
```

From repository root:

```bash
npm run build
node dist/cli/main.js \
  --input validation/ports/miniz-streaming/reports/<track>/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/miniz-streaming/reports/<track>/iteration-001/ownership-report.json \
  --html-out validation/ports/miniz-streaming/reports/<track>/iteration-001/ownership-report.html \
  --audience intermediate
```
