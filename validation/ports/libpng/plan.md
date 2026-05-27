# libpng Porting Validation Plan

## Objective

Run libpng as the next C-to-Rust validation target after cJSON and Redis, focusing on byte parser state and chunk metadata ownership.

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

## Validation Commands

```bash
cd validation/ports/libpng/rust-port
cargo fmt -- --check
cargo check --message-format=json > ../reports/iteration-001/cargo-check.jsonl
cargo test
```

```bash
cd /workspaces/rust-ownership-error-navigation
npm run build
node dist/cli/main.js \
  --input validation/ports/libpng/reports/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/libpng/reports/iteration-001/ownership-report.json \
  --html-out validation/ports/libpng/reports/iteration-001/ownership-report.html
```
