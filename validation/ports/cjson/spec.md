# cJSON Porting Validation Spec

## Purpose

Evaluate whether ownership-error navigation helps a low-cost AI model port C code with manual memory management to Rust with fewer repeated ownership mistakes.

The first validation target is cJSON `v1.7.19` because its core parser is compact, widely used, and rich in tree ownership, string ownership, and cleanup-on-failure behavior.

## Scope

### In Scope

- A Rust crate under `validation/ports/cjson/rust-port/`
- A JSON value model for null, boolean, number, string, array, and object values
- Parsing JSON text into the Rust value model
- Basic parser tests derived from cJSON parser behavior
- Capturing failed `cargo check --message-format=json` output
- Generating this repository's JSON and HTML ownership reports from failed iterations

### Out Of Scope

- Full cJSON public API compatibility
- C ABI or FFI compatibility
- Serializer parity
- Custom allocator hooks
- Reference nodes equivalent to `cJSON_IsReference`
- `cJSON_StringIsConst` compatibility
- Performance parity and zero-copy parsing

## Upstream Baseline

- Upstream: `DaveGamble/cJSON`
- Version: `v1.7.19`
- Commit: `c859b25da02955fef659d658b8f324b5cde87be3`
- License: MIT
- Acquisition notes: [upstream/UPSTREAM.md](upstream/UPSTREAM.md)

## Porting Model

Represent JSON as owned Rust data first:

```rust
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
```

This intentionally avoids parent pointers, sibling linked lists, borrowed string storage, and custom allocation in the first pass. The first experiment should expose how much ownership guidance is still needed even after choosing an idiomatic owned representation.

## Phases

### Phase 1: Scalar Parser

- Create a minimal Rust crate in `rust-port/`.
- Implement null, true, false, number, and string parsing.
- Add tests for scalar success and failure cases.
- Capture diagnostics for every failed model-generated iteration.

### Phase 2: Arrays

- Implement `Array(Vec<JsonValue>)` parsing.
- Preserve cJSON's nesting-limit concern with an explicit recursion-depth limit.
- Add tests for empty arrays, mixed arrays, nested arrays, trailing comma rejection, and malformed input.

### Phase 3: Objects

- Implement `Object(Vec<(String, JsonValue)>)` parsing.
- Model the cJSON key-transfer behavior as direct ownership of `String` keys.
- Add tests for empty objects, string keys, nested objects, missing colon rejection, and malformed input.

### Phase 4: Diagnostic Navigation Evaluation

- Run a low-cost model without manual Rust ownership hints.
- Save each failed `cargo check --message-format=json` capture under `reports/`.
- Generate JSON and HTML reports with this repository's CLI.
- Track repeated E0382, E0499, and E0502 patterns in `notes/`.

## Commands

Generate Rust compiler diagnostics from the experimental crate:

```bash
cd validation/ports/cjson/rust-port
cargo check --message-format=json 2>/dev/null \
  | tee ../reports/cargo-check.jsonl
```

Generate ownership reports from the repository root:

```bash
npm run build
node dist/cli/main.js \
  --input validation/ports/cjson/reports/cargo-check.jsonl \
  --json-out validation/ports/cjson/reports/ownership-report.json \
  --html-out validation/ports/cjson/reports/ownership-report.html
```

## Evaluation Metrics

- number of failed iterations before `cargo check` succeeds
- number of E0382, E0499, and E0502 diagnostics per iteration
- repeated diagnostics after a report is generated
- human intervention count
- use of `clone`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or `unsafe`
- test pass rate after compile success

## Acceptance Criteria For The First Experiment

- `rust-port/` contains a compiling crate for scalar JSON values.
- At least one failed iteration's JSONL diagnostics are captured under `reports/`.
- This repository generates a JSON report and a static HTML report from that JSONL.
- `notes/` records whether the generated report helped resolve the ownership issue.
- No upstream source snapshot is committed to this repository.
