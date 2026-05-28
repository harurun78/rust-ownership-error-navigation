# Plan: sax-js Queued Events Validation

## Phase 1: Target Setup

- Add target documentation and upstream metadata.
- Create compatibility and Rust-native Rust crates.
- Add iteration log and comparison matrix.

## Phase 2: Paired Slice 001

- Implement compatibility parser with queued borrowed events.
- Implement Rust-native parser with owned events.
- Add equivalent tests for start tags, end tags, text, and malformed names.

## Phase 3: Diagnostics And Reports

- Capture cargo-check JSONL for both tracks.
- Generate JSON and HTML navigation reports.
- Record whether navigation changes the compatibility implementation approach.

## Phase 4: Next Iteration Decision

- If compatibility fails due long-lived buffer borrows, use report guidance to split event storage from buffer mutation.
- If both tracks compile cleanly, expand the slice to attributes or incremental partial tags.

## Phase 5: Repair Iteration 002

- Replace queued `&str` event payloads in the compatibility track with queued byte spans.
- Resolve spans into borrowed event views only when `next_event` is called.
- Save clean cargo-check JSONL and navigation reports after repair.
- Compare the repair value against the Rust-native owned event baseline.
