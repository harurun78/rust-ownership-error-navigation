# Quickstart: tinyexpr Out-Param Validation

## Compatibility Track

`iteration-001` is a saved historical failure artifact. Do not regenerate it from the final source unless intentionally recreating the pre-repair E0308 mismatch.

Use `iteration-002` for final-source checks:

```bash
cd validation/ports/tinyexpr-out-param/tracks/compatibility/rust-port
cargo test
cargo check --message-format=json > ../../../reports/compatibility/iteration-002/cargo-check.jsonl
```

## Rust-Native Track

```bash
cd validation/ports/tinyexpr-out-param/tracks/rust-native/rust-port
cargo test
cargo check --message-format=json > ../../../reports/rust-native/iteration-001/cargo-check.jsonl
```

## Reports

```bash
npm run build
node dist/cli/main.js \
  --input validation/ports/tinyexpr-out-param/reports/compatibility/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/tinyexpr-out-param/reports/compatibility/iteration-001/ownership-report.json \
  --html-out validation/ports/tinyexpr-out-param/reports/compatibility/iteration-001/ownership-report.html \
  --audience intermediate
```
