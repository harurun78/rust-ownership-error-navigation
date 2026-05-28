# Iteration Log

## iteration-001

- Slice: complete HTTP request head parsing.
- Compatibility condition: JavaScript-style parser object with callback registration.
- Rust-native condition: owned request result with local input borrows.
- Compatibility result: `cargo test` passed 4 tests; `cargo check` produced 0 diagnostics.
- Rust-native result: `cargo test` passed 4 tests; `cargo check` produced 0 diagnostics.
- Compatibility report: `totalDiagnostics: 0`, `ownershipDiagnostics: 0`.
- Rust-native report: `totalDiagnostics: 0`, `ownershipDiagnostics: 0`.
- Shortcut pressure: no `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `.clone()` calls detected.
- Navigation effect: no repair signal yet because the first request-head slice compiled cleanly in both tracks.
- Prevention effect: Rust-native owned request result keeps input borrows local; compatibility callback event also stayed safe because it invokes callbacks with short-lived request-head views.
- Next slice: add incremental body delivery or chunk callbacks to create stronger pressure around callback lifetimes and parser buffer ownership.

## iteration-002

- Slice: `Content-Length` body parsing plus minimal chunked body parsing.
- Compatibility condition: body data is delivered through `on_body` borrowed chunk callbacks while parser state remains caller-visible.
- Rust-native condition: body data is accumulated into an owned `Request.body` string.
- Compatibility result: `cargo test` passed 8 tests; `cargo check` produced 0 diagnostics.
- Rust-native result: `cargo test` passed 8 tests; `cargo check` produced 0 diagnostics after a local helper-name repair from duplicate `parse_request` definitions.
- Compatibility report: `totalDiagnostics: 0`, `ownershipDiagnostics: 0`.
- Rust-native report: `totalDiagnostics: 0`, `ownershipDiagnostics: 0`.
- Shortcut pressure: no `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `.clone()` calls detected.
- Navigation effect: no ownership repair signal; the only encountered compiler issue was a simple Rust function namespace collision in the Rust-native track, fixed by renaming the private helper.
- Prevention effect: compatibility remained safe by invoking callbacks over short-lived body chunk views; Rust-native stayed simpler by returning owned body data.
- Completion decision: target complete at request-head + `Content-Length` body + minimal chunked body behavior. Full response parsing, pipelining, and Node.js compatibility are out of scope.
