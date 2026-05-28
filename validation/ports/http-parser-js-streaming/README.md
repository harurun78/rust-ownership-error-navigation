# http-parser-js Streaming Validation

This target compares two Rust porting conditions for a JavaScript-origin HTTP/1 streaming parser shape inspired by `http-parser-js`.

- **Original language**: JavaScript
- **Upstream style**: callback-driven streaming parser with parser state and buffer slices
- **Validation goal**: measure whether deterministic design suggestions help distinguish local repair from Rust-native API prevention

## Tracks

- `tracks/compatibility/rust-port`: preserves a callback-oriented parser API similar to JavaScript parser hooks.
- `tracks/rust-native/rust-port`: preserves behavior but returns owned Rust request records through `Result`.

## First Slice

Iteration 001 parses a complete HTTP request head:

```text
GET /chat HTTP/1.1\r\nHost: example.test\r\nConnection: keep-alive\r\n\r\n
```

The slice intentionally excludes chunked bodies, pipelining, response parsing, and upgrade handling.