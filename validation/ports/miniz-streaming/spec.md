# miniz Streaming Porting Comparison Spec

## Purpose

This target evaluates whether ownership-error navigation is more useful under compatibility-preserving C API pressure than under behavior-only Rust-native design. It uses miniz/zlib-style streaming buffer state because this domain naturally contains long-lived input/output buffers, mutable stream structs, status codes, callbacks, allocator hooks, and incremental calls.

## Upstream Scope

- Repository: `https://github.com/richgel999/miniz.git`
- Tag: `3.1.1`
- Commit: `d10b03cc73475af673df40f06e5cefd1d5f940d9`
- Reference concepts:
  - `mz_stream`
  - `mz_inflateInit`
  - `mz_inflate`
  - `mz_inflateEnd`
  - `zalloc` / `zfree` allocator hook shape

## Comparison Conditions

### Compatibility-Preserving Track

Preserve miniz/zlib-like surface:

- `MzStream` context with caller-visible `next_in`, `avail_in`, `next_out`, `avail_out`, `total_in`, and `total_out` concepts.
- `mz_inflate_init`, `mz_inflate`, and `mz_inflate_end` style lifecycle.
- status-code enum instead of idiomatic error-only API.
- allocator hook surface, even if the first slice records it as unimplemented compatibility pressure.
- no Rust-native redesign to hide long-lived mutable stream state.

### Behavior-Only Rust-Native Track

Preserve only behavior:

- Same input bytes produce same output bytes.
- Invalid lifecycle states produce deterministic errors.
- API may use owned `Vec<u8>`, `Result`, builders, iterators, or state machines.
- No requirement to preserve `mz_stream`, out buffers, or allocator hook shape.

## Initial Slice

The first paired slice implements a pass-through streaming transform:

- Input bytes are copied to output bytes.
- Partial output capacity is handled deterministically.
- Both tracks expose testable lifecycle behavior.
- Both tracks save cargo diagnostics and navigation reports.

This slice intentionally avoids full deflate decoding so the comparison starts with ownership/API pressure rather than compression algorithm complexity.

## Stored-Block Completion Slice

The completion slice adds minimal real zlib/DEFLATE behavior while staying focused on ownership/API pressure:

- zlib header validation for deflate streams without preset dictionaries.
- DEFLATE stored block decoding for one or more blocks.
- LEN/NLEN validation.
- Adler-32 checksum validation.
- Compatibility track output-buffer pressure through caller-provided buffers.
- Rust-native track owned output and deterministic error values.

This is the completion boundary for the current comparison target. Full compressed Huffman block decoding, compression, preset dictionaries, and bit-level performance tuning are outside this target because they primarily measure algorithm implementation rather than ownership navigation.

## Acceptance Criteria

- Both tracks compile and test independently.
- Each track writes `reports/<track>/iteration-NNN/cargo-check.jsonl`.
- Each track generates `ownership-report.json` and `ownership-report.html`.
- `notes/iteration-log.md` records diagnostics, shortcut pressure, and navigation effect.
- `notes/comparison-matrix.md` compares first cargo-check diagnostics, ownership diagnostics, shortcut pressure, and prevention/repair value.
- No external LLM API call is required for any suggestion or evaluation step.
- Stored-block completion reports compare repair value and prevention value across both tracks.
