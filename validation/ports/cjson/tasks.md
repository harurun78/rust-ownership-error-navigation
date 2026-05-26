# Tasks: cJSON Scalar Parser Porting Validation

## Phase 1: Experiment Setup

- [x] C001 Verify upstream checkout commit with `git -C validation/ports/cjson/upstream/cjson rev-parse HEAD`.
- [x] C002 Confirm `validation/ports/cjson/upstream/cjson/` remains ignored by Git.
- [x] C003 Create or update `validation/ports/cjson/notes/iteration-log.md` for model, prompt, human hints, command, diagnostics, report path, and next action.
- [x] C004 Extract Phase 1 scalar behavior notes from upstream `tests/parse_value.c`, `tests/parse_number.c`, and `tests/parse_string.c`.
- [x] C005 Use report folder naming convention `reports/iteration-001/`, `reports/iteration-002/`, etc.

## Phase 2: Rust Crate Skeleton

- [x] C006 Initialize Rust library crate in `validation/ports/cjson/rust-port/`.
- [x] C007 Add crate README or module docs stating Phase 1 scalar-only scope.
- [x] C008 Define `JsonValue` in `rust-port/src/value.rs`.
- [x] C009 Define `ParseError` in `rust-port/src/error.rs`.
- [x] C010 Expose crate modules from `rust-port/src/lib.rs`.

## Phase 3: Scalar Parser Tests First

- [x] C011 Add literal parser tests for `null`, `true`, and `false`.
- [x] C012 Add number parser tests for zero, signed integers, decimals, and exponents.
- [x] C013 Add large-number parser tests using upstream-style big numeric inputs.
- [x] C014 Add string parser tests for empty/basic strings and common escapes.
- [x] C015 Add Unicode escape tests for `\u20AC`, `\u732b`, and a surrogate pair.
- [x] C016 Add invalid scalar tests for empty input, non-value input, malformed string, invalid escape, and trailing backslash.
- [x] C017 Add Phase 1 boundary tests asserting array/object parsing is not implemented yet.

## Phase 4: Scalar Parser Implementation

- [x] C018 Implement parser cursor/input state without self-referential borrowing.
- [x] C019 Implement whitespace skipping at parser entry.
- [x] C020 Implement literal parsing for `null`, `true`, and `false`.
- [x] C021 Implement number token scanning and `f64` conversion.
- [x] C022 Implement string parsing with owned `String` output.
- [x] C023 Implement JSON escape decoding.
- [x] C024 Implement Unicode escape decoding, including surrogate pairs.
- [x] C025 Wire a public `parse_scalar` or `parse_value` entrypoint.
- [x] C026 Run `cargo test` in `rust-port/`.

## Phase 5: Diagnostic Capture And Ownership Report

- [x] C027 Save `cargo check --message-format=json` output to `reports/iteration-001/cargo-check.jsonl`.
- [x] C028 Build this repository's CLI with `npm run build`.
- [x] C029 Generate `reports/iteration-001/ownership-report.json`.
- [x] C030 Generate `reports/iteration-001/ownership-report.html`.
- [x] C031 Record ownership diagnostics, repeated diagnostics, and report usefulness in `notes/iteration-log.md`.
- [x] C032 Repeat C027-C031 for each failed model-generated iteration. Not needed for Phase 1 because iteration-001 compiled successfully.

## Phase 6: Evaluation Closeout

- [x] C033 Run final `cargo check` in `rust-port/`.
- [x] C034 Run final `cargo test` in `rust-port/`.
- [x] C035 Count iterations before compile success.
- [x] C036 Count E0382, E0499, and E0502 occurrences per iteration.
- [x] C037 Record human intervention count.
- [x] C038 Record use of `clone`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, and `unsafe`.
- [x] C039 Write scalar-phase conclusion in `notes/iteration-log.md`.
