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

## Phase 7: Array And Object Parser Tests

- [x] C040 Add array parser tests for empty arrays.
- [x] C041 Add array parser tests for mixed scalar arrays.
- [x] C042 Add array parser tests for nested arrays.
- [x] C043 Add array parser rejection tests for trailing commas and missing closing brackets.
- [x] C044 Add object parser tests for empty objects.
- [x] C045 Add object parser tests for string keys and scalar values.
- [x] C046 Add object parser tests for nested arrays and objects.
- [x] C047 Add object parser rejection tests for missing colon, trailing comma, and non-string keys.

## Phase 8: Array And Object Parser Implementation

- [x] C048 Implement array parsing with `Vec<JsonValue>`.
- [x] C049 Implement object parsing with `Vec<(String, JsonValue)>`.
- [x] C050 Preserve recursive depth limit behavior for arrays and objects.
- [x] C051 Preserve Phase 1 scalar parser behavior.
- [x] C052 Save `cargo check --message-format=json` output to `reports/iteration-002/cargo-check.jsonl`.
- [x] C053 Generate `reports/iteration-002/ownership-report.json`.
- [x] C054 Generate `reports/iteration-002/ownership-report.html`.
- [x] C055 Record iteration-002 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C056 Run final `cargo test` in `rust-port/`.

## Phase 9: Mutable Tree Editing And Detach Operations

- [x] C057 Add tests for appending values to arrays.
- [x] C058 Add tests for inserting and replacing object members.
- [x] C059 Add tests for detaching array items by index and returning owned values.
- [x] C060 Add tests for detaching object members by key and returning owned values.
- [x] C061 Implement mutable array append helpers.
- [x] C062 Implement object insert/replace helpers.
- [x] C063 Implement array detach helpers that transfer ownership out of the tree.
- [x] C064 Implement object detach helpers that transfer ownership out of the tree.
- [x] C065 Save `cargo check --message-format=json` output to `reports/iteration-003/cargo-check.jsonl`.
- [x] C066 Generate `reports/iteration-003/ownership-report.json`.
- [x] C067 Generate `reports/iteration-003/ownership-report.html`.
- [x] C068 Record iteration-003 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C069 Run final `cargo test` in `rust-port/`.

## Phase 10: Path-Based Lookup And Mutation

- [x] C070 Add tests for immutable path lookup through arrays and objects.
- [x] C071 Add tests for mutable path lookup through arrays and objects.
- [x] C072 Add tests for replacing a value at a nested path and returning the old owned value.
- [x] C073 Add tests for missing path behavior.
- [x] C074 Implement path segment model for object keys and array indexes.
- [x] C075 Implement immutable path lookup.
- [x] C076 Implement mutable path lookup without broad shared mutability.
- [x] C077 Implement nested value replacement that transfers ownership of the replaced value.
- [x] C078 Save `cargo check --message-format=json` output to `reports/iteration-004/cargo-check.jsonl`.
- [x] C079 Generate `reports/iteration-004/ownership-report.json`.
- [x] C080 Generate `reports/iteration-004/ownership-report.html`.
- [x] C081 Record iteration-004 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C082 Run final `cargo test` in `rust-port/`.

## Phase 11: Compact JSON Printing

- [x] C083 Add tests for printing scalar values.
- [x] C084 Add tests for string escaping during printing.
- [x] C085 Add tests for printing arrays and objects.
- [x] C086 Add tests for round-tripping parsed values through compact printing.
- [x] C087 Implement compact JSON printing for `JsonValue`.
- [x] C088 Implement string escaping for compact printing.
- [x] C089 Save `cargo check --message-format=json` output to `reports/iteration-005/cargo-check.jsonl`.
- [x] C090 Generate `reports/iteration-005/ownership-report.json`.
- [x] C091 Generate `reports/iteration-005/ownership-report.html`.
- [x] C092 Record iteration-005 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C093 Run final `cargo test` in `rust-port/`.

## Phase 12: Typed Accessors And Predicates

- [x] C094 Add tests for type predicates such as null, bool, number, string, array, and object.
- [x] C095 Add tests for typed value accessors.
- [x] C096 Add tests for object member and array item accessor helpers.
- [x] C097 Implement type predicates for `JsonValue`.
- [x] C098 Implement typed accessors for scalar and container values.
- [x] C099 Implement object member and array item accessor helpers.
- [x] C100 Save `cargo check --message-format=json` output to `reports/iteration-006/cargo-check.jsonl`.
- [x] C101 Generate `reports/iteration-006/ownership-report.json`.
- [x] C102 Generate `reports/iteration-006/ownership-report.html`.
- [x] C103 Record iteration-006 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C104 Run final `cargo test` in `rust-port/`.

## Phase 13: Pretty JSON Printing

- [x] C105 Add tests for pretty printing scalars, arrays, and objects.
- [x] C106 Add tests for nested pretty printing indentation.
- [x] C107 Add tests that pretty printing preserves JSON string escaping.
- [x] C108 Implement pretty JSON printing for `JsonValue`.
- [x] C109 Save `cargo check --message-format=json` output to `reports/iteration-007/cargo-check.jsonl`.
- [x] C110 Generate `reports/iteration-007/ownership-report.json`.
- [x] C111 Generate `reports/iteration-007/ownership-report.html`.
- [x] C112 Record iteration-007 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C113 Run final `cargo test` in `rust-port/`.

## Phase 14: JSON Minify Utility

- [x] C114 Add tests for removing insignificant whitespace.
- [x] C115 Add tests for preserving whitespace inside strings.
- [x] C116 Add tests for removing C-style block and line comments like cJSON minify.
- [x] C117 Add tests for malformed comments and unterminated strings.
- [x] C118 Implement JSON minify utility.
- [x] C119 Save `cargo check --message-format=json` output to `reports/iteration-008/cargo-check.jsonl`.
- [x] C120 Generate `reports/iteration-008/ownership-report.json`.
- [x] C121 Generate `reports/iteration-008/ownership-report.html`.
- [x] C122 Record iteration-008 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C123 Run final `cargo test` in `rust-port/`.

## Phase 15: Path-Based Detach And Delete

- [x] C124 Add tests for detaching nested array items by path and returning owned values.
- [x] C125 Add tests for detaching nested object members by path and returning owned values.
- [x] C126 Add tests for missing parent paths, missing terminal items, empty paths, and non-container parents.
- [x] C127 Implement path-based detach/delete helper without broad shared mutability.
- [x] C128 Preserve existing path lookup, path replacement, and top-level detach behavior.
- [x] C129 Save `cargo check --message-format=json` output to `reports/iteration-009/cargo-check.jsonl`.
- [x] C130 Generate `reports/iteration-009/ownership-report.json`.
- [x] C131 Generate `reports/iteration-009/ownership-report.html`.
- [x] C132 Record iteration-009 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C133 Run final `cargo test` in `rust-port/`.

## Phase 16: Merge Patch Utility

- [x] C134 Add tests for applying object merge patches to existing objects.
- [x] C135 Add tests for null-valued merge patch entries deleting object members.
- [x] C136 Add tests for nested object merge patches.
- [x] C137 Add tests for non-object patches replacing the whole target value.
- [x] C138 Implement merge patch utility that transfers patch values into the target where practical.
- [x] C139 Save `cargo check --message-format=json` output to `reports/iteration-010/cargo-check.jsonl`.
- [x] C140 Generate `reports/iteration-010/ownership-report.json`.
- [x] C141 Generate `reports/iteration-010/ownership-report.html`.
- [x] C142 Record iteration-010 results and diagnostic counts in `notes/iteration-log.md`.
- [x] C143 Run final `cargo test` in `rust-port/`.
