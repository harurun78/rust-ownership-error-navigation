# Tasks: cJSON Scalar Parser Porting Validation

## Phase 1: Experiment Setup

- [ ] C001 Verify upstream checkout commit with `git -C validation/ports/cjson/upstream/cjson rev-parse HEAD`.
- [ ] C002 Confirm `validation/ports/cjson/upstream/cjson/` remains ignored by Git.
- [ ] C003 Create or update `validation/ports/cjson/notes/iteration-log.md` for model, prompt, human hints, command, diagnostics, report path, and next action.
- [ ] C004 Extract Phase 1 scalar behavior notes from upstream `tests/parse_value.c`, `tests/parse_number.c`, and `tests/parse_string.c`.
- [ ] C005 Use report folder naming convention `reports/iteration-001/`, `reports/iteration-002/`, etc.

## Phase 2: Rust Crate Skeleton

- [ ] C006 Initialize Rust library crate in `validation/ports/cjson/rust-port/`.
- [ ] C007 Add crate README or module docs stating Phase 1 scalar-only scope.
- [ ] C008 Define `JsonValue` in `rust-port/src/value.rs`.
- [ ] C009 Define `ParseError` in `rust-port/src/error.rs`.
- [ ] C010 Expose crate modules from `rust-port/src/lib.rs`.

## Phase 3: Scalar Parser Tests First

- [ ] C011 Add literal parser tests for `null`, `true`, and `false`.
- [ ] C012 Add number parser tests for zero, signed integers, decimals, and exponents.
- [ ] C013 Add large-number parser tests using upstream-style big numeric inputs.
- [ ] C014 Add string parser tests for empty/basic strings and common escapes.
- [ ] C015 Add Unicode escape tests for `\u20AC`, `\u732b`, and a surrogate pair.
- [ ] C016 Add invalid scalar tests for empty input, non-value input, malformed string, invalid escape, and trailing backslash.
- [ ] C017 Add Phase 1 boundary tests asserting array/object parsing is not implemented yet.

## Phase 4: Scalar Parser Implementation

- [ ] C018 Implement parser cursor/input state without self-referential borrowing.
- [ ] C019 Implement whitespace skipping at parser entry.
- [ ] C020 Implement literal parsing for `null`, `true`, and `false`.
- [ ] C021 Implement number token scanning and `f64` conversion.
- [ ] C022 Implement string parsing with owned `String` output.
- [ ] C023 Implement JSON escape decoding.
- [ ] C024 Implement Unicode escape decoding, including surrogate pairs.
- [ ] C025 Wire a public `parse_scalar` or `parse_value` entrypoint.
- [ ] C026 Run `cargo test` in `rust-port/`.

## Phase 5: Diagnostic Capture And Ownership Report

- [ ] C027 On the first failed compile, save raw diagnostics to `reports/iteration-001/cargo-check.jsonl`.
- [ ] C028 Build this repository's CLI with `npm run build`.
- [ ] C029 Generate `reports/iteration-001/ownership-report.json`.
- [ ] C030 Generate `reports/iteration-001/ownership-report.html`.
- [ ] C031 Record ownership diagnostics, repeated diagnostics, and report usefulness in `notes/iteration-log.md`.
- [ ] C032 Repeat C027-C031 for each failed model-generated iteration.

## Phase 6: Evaluation Closeout

- [ ] C033 Run final `cargo check` in `rust-port/`.
- [ ] C034 Run final `cargo test` in `rust-port/`.
- [ ] C035 Count iterations before compile success.
- [ ] C036 Count E0382, E0499, and E0502 occurrences per iteration.
- [ ] C037 Record human intervention count.
- [ ] C038 Record use of `clone`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, and `unsafe`.
- [ ] C039 Write scalar-phase conclusion in `notes/iteration-log.md`.
