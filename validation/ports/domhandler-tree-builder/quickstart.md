# Quickstart: domhandler Tree Builder Validation

## Compatibility Track

```bash
cd validation/ports/domhandler-tree-builder/tracks/compatibility/rust-port
cargo check --message-format=json > ../../../reports/compatibility/iteration-001/cargo-check.jsonl
```

## Rust-Native Track

```bash
cd validation/ports/domhandler-tree-builder/tracks/rust-native/rust-port
cargo test
cargo check --message-format=json > ../../../reports/rust-native/iteration-001/cargo-check.jsonl
```

## Reports

```bash
npm run build
node dist/cli/main.js \
  --input validation/ports/domhandler-tree-builder/reports/compatibility/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/domhandler-tree-builder/reports/compatibility/iteration-001/ownership-report.json \
  --html-out validation/ports/domhandler-tree-builder/reports/compatibility/iteration-001/ownership-report.html \
  --audience intermediate

node dist/cli/main.js \
  --input validation/ports/domhandler-tree-builder/reports/rust-native/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/domhandler-tree-builder/reports/rust-native/iteration-001/ownership-report.json \
  --html-out validation/ports/domhandler-tree-builder/reports/rust-native/iteration-001/ownership-report.html \
  --audience intermediate
```

## Repair Iteration Reports

Use `iteration-002` paths after applying the NodeId arena compatibility repair:

```bash
node dist/cli/main.js \
  --input validation/ports/domhandler-tree-builder/reports/compatibility/iteration-002/cargo-check.jsonl \
  --json-out validation/ports/domhandler-tree-builder/reports/compatibility/iteration-002/ownership-report.json \
  --html-out validation/ports/domhandler-tree-builder/reports/compatibility/iteration-002/ownership-report.html \
  --audience intermediate

node dist/cli/main.js \
  --input validation/ports/domhandler-tree-builder/reports/rust-native/iteration-002/cargo-check.jsonl \
  --json-out validation/ports/domhandler-tree-builder/reports/rust-native/iteration-002/ownership-report.json \
  --html-out validation/ports/domhandler-tree-builder/reports/rust-native/iteration-002/ownership-report.html \
  --audience intermediate
```
