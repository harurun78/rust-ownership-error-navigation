# Tasks: sax-js Queued Events A/B Porting Validation

## Phase 1: Target Setup

- [x] S001 Select JavaScript-origin `sax-js` style queued event target.
- [x] S002 Add target spec, plan, quickstart, upstream metadata, and notes skeleton.
- [x] S003 Create compatibility and Rust-native Rust crates.

## Phase 2: Paired Slice 001

- [x] S004 Implement compatibility first attempt with queued borrowed event views.
- [x] S005 Implement Rust-native owned event parser.
- [x] S006 Add equivalent tests for start tags, end tags, text, and malformed names.

## Phase 3: Diagnostics And Reports

- [x] S007 Save cargo-check JSONL for both tracks.
- [x] S008 Generate ownership/navigation reports for both tracks.
- [x] S009 Update iteration log, comparison matrix, and comparison summary.

## Phase 4: Repair Iteration

- [x] S010 Apply navigation-guided compatibility repair after iteration 001 produced E0502 diagnostics.
- [x] S011 Save iteration-002 cargo-check JSONL and navigation reports for both tracks.
- [x] S012 Verify repaired compatibility tests and Rust-native baseline tests.

## Phase 5: Completion Iteration

- [x] S013 Add quoted attribute parsing to both tracks.
- [x] S014 Add compatibility incremental partial tag behavior.
- [x] S015 Add tests for attributes, partial tags, and invalid attributes.
- [x] S016 Save iteration-003 cargo-check JSONL and navigation reports for both tracks.
- [x] S017 Mark the target complete at tags, text, attributes, and partial tags.
