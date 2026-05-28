# Specification: http-parser-js Streaming A/B Porting Validation

## Target Choice

Use `http-parser-js` style HTTP/1 request parsing because it is not Rust, is not C/C++, and naturally combines parser state, caller callbacks, and input buffer slices.

## Hypothesis

- Compatibility-preserving Rust will experience ownership pressure around callback invocation, parser state mutation, and borrowed input slices.
- Rust-native Rust can avoid much of that pressure by returning owned request records and keeping input borrows short.
- The current application's deterministic design suggestions should surface prevention guidance such as long-lived buffer borrow avoidance and mutation phase splitting when diagnostics occur.

## Conditions

### Compatibility-Preserving

- Preserve a parser object with `execute` and callback registration.
- Preserve request-head callback behavior similar to `onHeadersComplete`.
- Preserve caller-visible parser state such as pause and complete status.
- Prefer borrowed callback event fields when feasible.

### Rust-Native

- Preserve observable parsing behavior only.
- Return owned `Request` records with owned headers.
- Use `Result` errors instead of callback status codes.
- Keep input borrows local to the parse call.

## Iteration 001 Scope

- Parse request line: method, path, HTTP version.
- Parse header fields until `\r\n\r\n`.
- Reject malformed request line, malformed headers, and incomplete input.
- Tests must pass in both tracks.

## Iteration 002 Completion Scope

- Parse `Content-Length` request bodies.
- Deliver body data through a compatibility callback as borrowed chunks.
- Return body data through the Rust-native owned `Request` record.
- Decode minimal HTTP/1 chunked request bodies.
- Reject malformed content length, unsupported transfer encoding, malformed chunks, and incomplete bodies.

This is the completion boundary for the current target. It covers the ownership-relevant parser state, callback, input-buffer, and owned-result surfaces without expanding into full Node.js compatibility.

## Non-Goals

- Response parsing.
- Pipelining.
- Real Node.js compatibility.
- Unsafe FFI.
