# Quickstart: http-parser-js Streaming Validation

## Compatibility Track

```sh
cd validation/ports/http-parser-js-streaming/tracks/compatibility/rust-port
cargo fmt -- --check
cargo test
cargo check --message-format=json > ../../../reports/compatibility/iteration-002/cargo-check.jsonl
```

## Rust-Native Track

```sh
cd validation/ports/http-parser-js-streaming/tracks/rust-native/rust-port
cargo fmt -- --check
cargo test
cargo check --message-format=json > ../../../reports/rust-native/iteration-002/cargo-check.jsonl
```

## Generate Reports

```sh
npm run build
node dist/cli/main.js \
  --input validation/ports/http-parser-js-streaming/reports/compatibility/iteration-002/cargo-check.jsonl \
  --json-out validation/ports/http-parser-js-streaming/reports/compatibility/iteration-002/ownership-report.json \
  --html-out validation/ports/http-parser-js-streaming/reports/compatibility/iteration-002/ownership-report.html \
  --audience intermediate

node dist/cli/main.js \
  --input validation/ports/http-parser-js-streaming/reports/rust-native/iteration-002/cargo-check.jsonl \
  --json-out validation/ports/http-parser-js-streaming/reports/rust-native/iteration-002/ownership-report.json \
  --html-out validation/ports/http-parser-js-streaming/reports/rust-native/iteration-002/ownership-report.html \
  --audience intermediate
```

## Completion Boundary

The target is complete when both tracks pass tests and reports for:

- request line parsing
- header parsing
- incomplete request-head rejection
- `Content-Length` body parsing
- minimal chunked body parsing
- malformed body/chunk rejection
