# Plan: domhandler Tree Builder Validation

## Phase 1: Target Setup

- Add target documentation and upstream metadata.
- Create compatibility and Rust-native Rust crates.
- Add iteration log and comparison matrix.

## Phase 2: Paired Slice 001

- Implement direct-reference compatibility first attempt.
- Implement Rust-native arena baseline.
- Capture cargo-check JSONL and navigation reports.

## Phase 3: Repair Iteration 002

- Apply report-guided repair by moving compatibility tree storage to node IDs.
- Preserve handler-style callbacks and tree inspection behavior.
- Save clean cargo-check JSONL, reports, and test results.

## Phase 4: Completion

- Complete the target at start tags, text, end tags, parent links, child lists, and root child inspection.
- Record repair value and prevention value in the comparison summary.
