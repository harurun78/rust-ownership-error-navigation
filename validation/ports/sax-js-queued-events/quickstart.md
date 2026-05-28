# Quickstart: sax-js Queued Events Validation

## Compatibility Track

```bash
cd validation/ports/sax-js-queued-events/tracks/compatibility/rust-port
cargo check --message-format=json > ../../../reports/compatibility/iteration-001/cargo-check.jsonl
cargo test
```

## Rust-Native Track

```bash
cd validation/ports/sax-js-queued-events/tracks/rust-native/rust-port
cargo check --message-format=json > ../../../reports/rust-native/iteration-001/cargo-check.jsonl
cargo test
```

## Reports

```bash
npm run build
node dist/cli/main.js \
  --input validation/ports/sax-js-queued-events/reports/compatibility/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/sax-js-queued-events/reports/compatibility/iteration-001/ownership-report.json \
  --html-out validation/ports/sax-js-queued-events/reports/compatibility/iteration-001/ownership-report.html \
  --audience intermediate

node dist/cli/main.js \
  --input validation/ports/sax-js-queued-events/reports/rust-native/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/sax-js-queued-events/reports/rust-native/iteration-001/ownership-report.json \
  --html-out validation/ports/sax-js-queued-events/reports/rust-native/iteration-001/ownership-report.html \
  --audience intermediate
```

## Repair Iteration Reports

Use `iteration-002` paths after applying the span-based compatibility repair:

```bash
node dist/cli/main.js \
  --input validation/ports/sax-js-queued-events/reports/compatibility/iteration-002/cargo-check.jsonl \
  --json-out validation/ports/sax-js-queued-events/reports/compatibility/iteration-002/ownership-report.json \
  --html-out validation/ports/sax-js-queued-events/reports/compatibility/iteration-002/ownership-report.html \
  --audience intermediate

node dist/cli/main.js \
  --input validation/ports/sax-js-queued-events/reports/rust-native/iteration-002/cargo-check.jsonl \
  --json-out validation/ports/sax-js-queued-events/reports/rust-native/iteration-002/ownership-report.json \
  --html-out validation/ports/sax-js-queued-events/reports/rust-native/iteration-002/ownership-report.html \
  --audience intermediate
```

## Completion Iteration Reports

Use `iteration-003` paths after adding attributes and compatibility partial tags:

```bash
node dist/cli/main.js \
  --input validation/ports/sax-js-queued-events/reports/compatibility/iteration-003/cargo-check.jsonl \
  --json-out validation/ports/sax-js-queued-events/reports/compatibility/iteration-003/ownership-report.json \
  --html-out validation/ports/sax-js-queued-events/reports/compatibility/iteration-003/ownership-report.html \
  --audience intermediate

node dist/cli/main.js \
  --input validation/ports/sax-js-queued-events/reports/rust-native/iteration-003/cargo-check.jsonl \
  --json-out validation/ports/sax-js-queued-events/reports/rust-native/iteration-003/ownership-report.json \
  --html-out validation/ports/sax-js-queued-events/reports/rust-native/iteration-003/ownership-report.html \
  --audience intermediate
```
