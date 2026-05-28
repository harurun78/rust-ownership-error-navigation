# Tasks: http-parser-js Streaming A/B Porting Validation

## Phase 1: Target Setup

- [x] H001 Select JavaScript-origin `http-parser-js` style streaming parser target.
- [x] H002 Add target spec, plan, quickstart, upstream metadata, and notes skeleton.
- [x] H003 Create compatibility and Rust-native Rust crates.

## Phase 2: Paired Slice 001

- [x] H004 Implement compatibility request-head parser with callback API.
- [x] H005 Implement Rust-native request-head parser with owned results.
- [x] H006 Add equivalent tests for valid, malformed, and incomplete request heads.

## Phase 3: Diagnostics And Reports

- [x] H007 Save cargo-check JSONL for both tracks.
- [x] H008 Generate ownership/navigation reports for both tracks.
- [x] H009 Update iteration log, comparison matrix, and comparison summary.

## Phase 4: Verification

- [x] H010 Run formatting, tests, report generation, and repository format checks.

## Phase 5: Paired Slice 002 - Body Completion

- [x] H011 Implement `Content-Length` body parsing in both tracks.
- [x] H012 Implement minimal chunked body parsing in both tracks.
- [x] H013 Preserve compatibility body delivery through borrowed callbacks.
- [x] H014 Preserve Rust-native owned request body output.
- [x] H015 Add tests for valid body delivery, incomplete bodies, chunked bodies, and malformed chunks.
- [x] H016 Save iteration-002 cargo-check JSONL and navigation reports for both tracks.
- [x] H017 Update iteration log, comparison matrix, and comparison summary.

## Phase 6: Completion

- [x] H018 Mark the target complete at request-head + `Content-Length` body + minimal chunked body behavior.
