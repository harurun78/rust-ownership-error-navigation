# libpng Porting Validation Plan

## Objective

Run libpng as the next C-to-Rust validation target after cJSON and Redis, focusing on byte parser state, chunk metadata ownership, image decode/write behavior, and libpng-style lifecycle compatibility.

## Phase 1: Setup

1. Fetch upstream `pnggroup/libpng` at `v1.6.58` into ignored `upstream/libpng/`.
2. Record repository, tag, commit, license, and acquisition commands in `upstream/UPSTREAM.md`.
3. Create `spec.md`, `plan.md`, `tasks.md`, `quickstart.md`, and `notes/iteration-log.md`.
4. Initialize a Rust library crate under `rust-port/`.

## Phase 2: Signature And Chunk Header Slice

1. Add tests for full and partial PNG signature checks.
2. Add tests for invalid signature bytes.
3. Add tests for chunk type property helpers using `IHDR`, `IDAT`, `tEXt`, and invalid reserved bits.
4. Add tests for streaming input that produces `NeedMoreData`, `SignatureComplete`, and `ChunkHeader` outcomes.
5. Implement the smallest Rust API that satisfies these tests.
6. Capture `cargo check --message-format=json`, generate reports, and record iteration notes.

## Phase 3: Chunk Payload And CRC Slice

1. Add tests for partial payload retention after header parse.
2. Add CRC calculation or verification boundary with a small deterministic implementation or documented dependency decision.
3. Return owned chunk records after full payload and CRC bytes are available.
4. Record ownership diagnostics and shortcut pressure.

## Phase 4: IHDR Validation Slice

1. Parse IHDR payload fields into owned metadata.
2. Validate width, height, bit depth, color type, compression, filter, and interlace fields.
3. Record unsupported or non-ownership diagnostics surfaced during implementation.

## Phase 5: Evaluation

1. Compare diagnostics across iterations.
2. Track repeated E0382, E0499, and E0502 counts.
3. Record whether reports changed the next implementation prompt.
4. Summarize whether libpng produces more useful ownership-navigation cases than cJSON and Redis.

## Phase 6: Rust-Native PNG Parity

1. Extend decoding to IDAT inflation, filter reconstruction, color types 0/2/3/4/6, packed samples, 16-bit samples, tRNS, PLTE, and Adam7.
2. Extend metadata support to gAMA, cHRM, sRGB, iCCP, pHYs, tIME, tEXt, zTXt, and iTXt.
3. Add image/document writing, indexed writing, packed indexed writing, explicit/adaptive filters, and byte-aligned Adam7 output.
4. Add row callback style decode and unknown ancillary preservation.

## Phase 7: libpng Compatibility Facade

1. Add Rust-native read/write structs that mirror libpng lifecycle concepts.
2. Add read-info/read-image and write-image/write-document/write-indexed operations.
3. Expose compatibility warnings for Rust-native facade semantics and missing C ABI behavior.
4. Keep C ABI, allocator hooks, setjmp/longjmp, and exact warning recovery as a separate compatibility track rather than hidden behavior in the validation crate.

## Phase 8: Compatibility Behavior Controls

1. Add libpng-style read transform setters for 16-bit stripping, low-bit grayscale expansion, palette-to-RGB, and tRNS-to-alpha behavior.
2. Add a Rust-native warning callback hook so callers can observe compatibility warnings without C longjmp semantics.
3. Add writer unknown ancillary copy policy controls for safe-only, all ancillary, and none.
4. Record which compatibility behaviors are implemented in safe Rust and which remain true C ABI concerns.

## Validation Commands

```bash
cd validation/ports/libpng/rust-port
cargo fmt -- --check
cargo check --message-format=json > ../reports/iteration-026/cargo-check.jsonl
cargo test
```

```bash
cd /workspaces/rust-ownership-error-navigation
npm run build
node dist/cli/main.js \
  --input validation/ports/libpng/reports/iteration-026/cargo-check.jsonl \
  --json-out validation/ports/libpng/reports/iteration-026/ownership-report.json \
  --html-out validation/ports/libpng/reports/iteration-026/ownership-report.html
```
