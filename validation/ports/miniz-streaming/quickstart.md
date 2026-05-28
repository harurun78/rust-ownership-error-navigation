# Quickstart: miniz Streaming Porting Comparison

## Compatibility Track

```bash
cd validation/ports/miniz-streaming/tracks/compatibility/rust-port
cargo fmt -- --check
cargo test
mkdir -p ../../../reports/compatibility/iteration-001
cargo check --message-format=json > ../../../reports/compatibility/iteration-001/cargo-check.jsonl
```

## Rust-Native Track

```bash
cd validation/ports/miniz-streaming/tracks/rust-native/rust-port
cargo fmt -- --check
cargo test
mkdir -p ../../../reports/rust-native/iteration-001
cargo check --message-format=json > ../../../reports/rust-native/iteration-001/cargo-check.jsonl
```

## Generate Navigation Reports

From repository root:

```bash
npm run build

node dist/cli/main.js \
  --input validation/ports/miniz-streaming/reports/compatibility/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/miniz-streaming/reports/compatibility/iteration-001/ownership-report.json \
  --html-out validation/ports/miniz-streaming/reports/compatibility/iteration-001/ownership-report.html \
  --audience intermediate

node dist/cli/main.js \
  --input validation/ports/miniz-streaming/reports/rust-native/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/miniz-streaming/reports/rust-native/iteration-001/ownership-report.json \
  --html-out validation/ports/miniz-streaming/reports/rust-native/iteration-001/ownership-report.html \
  --audience intermediate
```

## Evaluation

Update:

- `notes/iteration-log.md`
- `notes/comparison-matrix.md`
- `reports/comparison-summary.md`

Record both repair value and prevention value. Clean Rust-native compilation is prevention evidence, not an empty result.

## Completion Boundary

This target is complete when iteration-002 passes in both tracks:

- compatibility track decodes zlib stored blocks into caller-provided output buffers
- Rust-native track decodes zlib stored blocks into owned outputs
- both tracks validate Adler-32 and LEN/NLEN stored block metadata
- both tracks save cargo-check JSONL and ownership reports
- comparison summary separates repair value from prevention value
