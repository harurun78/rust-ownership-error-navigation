# Implementation Plan: cJSON Scalar Parser Porting Validation

## Summary

Validate whether this repository's ownership-error navigation workflow helps a low-cost AI model port C parser code to Rust with fewer repeated ownership mistakes.

The first slice ports only cJSON scalar parsing into an owned Rust model under `rust-port/`: `null`, booleans, numbers, and strings. Arrays and objects are deferred to later phases so the initial experiment can establish a clean baseline.

Each failed model-generated Rust iteration should save `cargo check --message-format=json` output under `reports/`, then generate this repository's JSON and static HTML ownership reports. The experiment measures whether generated reports reduce repeated E0382, E0499, and E0502 mistakes across iterations.

## Technical Context

- Upstream: `DaveGamble/cJSON`
- Version: `v1.7.19`
- Commit: `c859b25da02955fef659d658b8f324b5cde87be3`
- Upstream checkout: `upstream/cjson/` local only, ignored by Git
- Rust target: `rust-port/`
- Initial crate type: Rust library crate with tests
- Diagnostic capture: `cargo check --message-format=json`
- Report output: `reports/iteration-NNN/ownership-report.json` and `reports/iteration-NNN/ownership-report.html`

## Initial Rust Model

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

Array and object variants may exist to preserve the eventual data model, but parser support for `[` and `{` is out of scope for Phase 1.

## Relevant Upstream Areas

- `cJSON.h`: `struct cJSON`, type flags, memory-management contract
- `cJSON.c`: `parse_value`, `parse_number`, `parse_string`, `parse_array`, `parse_object`, `cJSON_Delete`
- `tests/parse_value.c`: scalar value behavior
- `tests/parse_number.c`: number behavior
- `tests/parse_string.c`: string and escape behavior

## Project Structure

```text
validation/ports/cjson/
  spec.md
  plan.md
  tasks.md
  quickstart.md
  README.md
  notes/
    upstream-analysis.md
    iteration-log.md
  reports/
    iteration-001/
      cargo-check.jsonl
      ownership-report.json
      ownership-report.html
      notes.md
  rust-port/
    Cargo.toml
    src/
      lib.rs
      parser.rs
      value.rs
      error.rs
    tests/
      scalar_parser.rs
  upstream/
    UPSTREAM.md
    cjson/          # local checkout, ignored by Git
```

## Phase Plan

### Phase 0: Baseline And Experiment Protocol

- Verify upstream commit and ignored checkout.
- Identify scalar parser behavior from cJSON tests.
- Define iteration naming convention.
- Define what counts as human intervention.
- Define repeated ownership diagnostic tracking.

Acceptance:

- upstream commit verified
- iteration log template exists
- scalar acceptance cases listed

### Phase 1: Scalar Parser Crate

- Create a minimal Rust library crate.
- Implement owned `JsonValue`.
- Add parser entrypoint such as `parse_scalar(input: &str) -> Result<JsonValue, ParseError>`.
- Implement scalar-only `parse_value`.
- Implement number parsing into `f64`.
- Implement string parsing with common JSON escapes and Unicode escape support.
- Implement whitespace handling at top level.

Acceptance:

- `cargo test` passes for scalar success and failure cases
- arrays and objects return a clear unsupported or malformed result in Phase 1
- no `unsafe`
- no `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `clone` use unless recorded as intervention debt

### Phase 2: Diagnostic Capture Workflow

For every failed model iteration:

- run `cargo check --message-format=json`
- save JSONL to an iteration folder
- run this repository's CLI to produce JSON and HTML reports
- record whether the report changed the next prompt or fix

Acceptance:

- at least one failed iteration is captured, unless the first model-generated attempt compiles cleanly
- ownership report JSON and HTML are generated from captured JSONL
- iteration notes classify repeated E0382, E0499, and E0502 patterns

### Phase 3: Evaluation Summary

Summarize whether the navigation helped:

- failed iterations before compile success
- ownership diagnostic count per iteration
- repeated diagnostic count after report use
- human intervention count
- `clone`, shared mutability, and `unsafe` pressure
- final scalar test pass rate

Acceptance:

- notes contain a short conclusion
- report artifacts are reproducible from saved JSONL

## Initial Scalar Acceptance Cases

- literals: `null`, `true`, `false`
- numbers: `0`, `0.0`, `-0`, positive and negative integers, decimals, `e` and `E` exponents
- strings: empty string, escaped quote, escaped backslash, escaped slash, `\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX`, and surrogate pairs
- invalid strings: non-quoted input, empty input, invalid escape, trailing backslash
- parser entry behavior: skip leading whitespace and optional UTF-8 BOM if implemented in entrypoint

## Risks

- If the first low-cost model attempt compiles cleanly, report effectiveness is harder to measure.
- Scalar-only parsing does not yet exercise cJSON's linked child/sibling ownership and partial cleanup behavior.
- Unicode escape handling can dominate the work with specification bugs rather than ownership diagnostics.
- Rust `str::parse::<f64>()` does not guarantee exact cJSON `strtod` parity.
- Extra human prompts can blur the effect of the generated ownership report.

## Non-Goals

- array and object parser implementation in Phase 1
- full cJSON public API compatibility
- C ABI or FFI compatibility
- custom allocator hooks
- serializer parity
- zero-copy parsing
- performance parity
- clippy or rust-analyzer as required diagnostic sources
- committing the upstream checkout
