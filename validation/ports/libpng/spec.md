# libpng Rust Porting Validation Spec

## Purpose

This target measures whether the Rust ownership-error navigation tool helps port libpng C parsing, decoding, writing, and lifecycle concepts to Rust. libpng is a useful target because it combines byte-level parsing, mutable decoder state, progressive input, chunk metadata extraction, allocation boundaries, CRC validation, interlacing, row filters, and long-lived read/write structs.

## Upstream Scope

- Repository: `https://github.com/pnggroup/libpng.git`
- Tag: `v1.6.58`
- Tag object: `fdc7185dfedbddce8c2487bc171f66af4fca24ab`
- Commit: `3061454d980de7d53608f594194cfac722721d2a`
- License: PNG Reference Library License version 2, recorded in `upstream/UPSTREAM.md`
- Primary C source:
  - `png.c`: `png_sig_cmp`
  - `pngrutil.c`: read signature and chunk utility flow
  - `pngpread.c`: progressive signature checking
  - `png.h` / `pngpriv.h`: public/internal contracts

## Functional Scope

Implement a Rust validation port in small compile-checkable slices. The scope now extends beyond parsing into a Rust-native libpng compatibility layer: read/decode/write/document APIs, rich metadata, filters, indexed color, Adam7, row callbacks, and a lifecycle facade that mirrors libpng read/write concepts.

### Initial Slice

- PNG signature constant and `png_sig_cmp`-style partial comparison.
- Streaming parser with an internal byte buffer and cursor.
- Parser outcomes for incomplete input, complete signature, and complete chunk header.
- Chunk header model containing length and owned 4-byte chunk type.
- Chunk type property helpers for critical/ancillary, public/private, reserved-bit validity, and safe-to-copy.
- Deterministic errors for invalid signature, invalid chunk type bytes, and length overflow policy.

### Compatibility Slice

- `png_compat_create_read_struct` / `png_compat_set_read_buffer` / `png_compat_read_info` / `png_compat_read_image` / `png_compat_destroy_read_struct` lifecycle coverage.
- `png_compat_create_write_struct` / `png_compat_write_image` / `png_compat_write_document` / `png_compat_write_indexed_image` / `png_compat_write_output` / `png_compat_destroy_write_struct` lifecycle coverage.
- Explicit compatibility warnings for Rust-native facade semantics and missing C ABI behavior.
- Documentation that separates Rust-native compatibility from true C ABI/setjmp/allocator compatibility.

## Non-Goals

- Binary-compatible C ABI or FFI replacement in this validation crate.
- Exact libpng allocator/error-jump behavior in this validation crate.
- Pretending setjmp/longjmp behavior exists behind safe Rust APIs.
- `unsafe`, `Rc<RefCell<_>>`, or `Arc<Mutex<_>>` unless recorded as a measurement event.

## Data Model Expectations

The Rust port should expose a small parser API, for example:

```rust
pub struct PngStreamParser { /* internal buffer and cursor */ }

pub enum ParseOutcome {
    NeedMoreData,
    SignatureComplete,
    ChunkHeader(ChunkHeader),
}

pub struct ChunkHeader {
    pub length: u32,
    pub chunk_type: ChunkType,
}
```

The exact names may change, but the API should make these states observable:

- signature accepted only after enough bytes are available
- invalid signatures reported deterministically
- chunk headers extracted as owned metadata without borrowing from the parser buffer
- unconsumed bytes retained for later parser slices

## Ownership-Pressure Points

- Appending partial byte input into mutable parser state.
- Returning owned chunk metadata after inspecting bytes inside the parser buffer.
- Draining consumed bytes without retaining stale borrows.
- Avoiding broad copies of the full input buffer.
- Preparing future slices where chunk payload extraction and CRC checks happen after header parsing.

## Acceptance Criteria

- Each implementation iteration saves `cargo check --message-format=json` to `reports/iteration-NNN/cargo-check.jsonl`.
- Each iteration generates `ownership-report.json` and `ownership-report.html` from the saved JSONL.
- If a report contains E0382, E0499, or E0502, the next lightweight-model attempt must receive the generated report as the primary fix guide and record whether diagnostics decreased.
- Each iteration records model, prompt summary, human ownership hints, command result, diagnostic counts, shortcut pressure, and next action in `notes/iteration-log.md`.
- `cargo fmt -- --check` and `cargo test` pass for completed iterations.
- No upstream source snapshot is committed to this repository.
