# Plan: http-parser-js Streaming A/B Porting Validation

## Phase 1: Setup

- Create target docs, upstream metadata, track directories, notes, and reports.
- Create two Rust library crates with no external dependencies.

## Phase 2: Paired Slice 001

- Compatibility track: implement callback-driven request-head parsing.
- Rust-native track: implement owned request-head parsing.
- Add equivalent tests for valid request head and malformed/incomplete inputs.

## Phase 3: Reports

- Run `cargo check --message-format=json` for each track.
- Generate ownership/navigation JSON and HTML reports with `--audience intermediate`.
- Record diagnostics, design suggestions, test counts, and shortcut pressure.

## Phase 4: Next Slice Selection

Choose the next paired slice based on iteration-001 evidence:

- callback/user-data pressure
- incremental input buffering
- chunked body callbacks versus owned body events

## Phase 5: Paired Slice 002 - Body Completion

- Add `Content-Length` body parsing to both tracks.
- Add minimal chunked body parsing to both tracks.
- Keep compatibility body delivery callback-based and borrowed.
- Keep Rust-native output owned and `Result` based.
- Generate iteration-002 reports and finalize comparison summary.

## Phase 6: Completion

- Treat request-head, content-length body, and minimal chunked body parsing as complete for this target.
- Leave response parsing, pipelining, and full Node.js compatibility out of scope.
