# Iteration Log

## iteration-001

- Model condition: main agent implementation under A/B validation instructions.
- Slice: decimal numbers, variables, unary signs, binary arithmetic, parentheses, and deterministic parse errors.
- Human ownership hints: none beyond the track condition definitions.
- Compatibility prompt summary: preserve C-shaped `te_compile` and `te_interp` functions with mutable error-position out-parameters.
- Rust-native prompt summary: preserve behavior with `Result<Expr, ParseError>` and owned expression trees.
- Expected pressure: compatibility API should expose E0308 when parser `Result` is returned directly from a C-style `Option` plus out-param boundary.
- Compatibility result: `cargo check` produced 2 total diagnostics: one supported E0308 plus the rustc failure-note. The E0308 span showed `expected Option<TeExpr>, found Result<TeExpr, ParseError>` at the API return boundary.
- Navigation summary: the report emitted `owned-result` design guidance for E0308, which matched the Rust-native API and clarified the type-boundary mismatch. In the compatibility track, the repair kept the C-shaped API but explicitly adapted `Result` into `Option` plus `error_out`.
- Rust-native result: `cargo test` passed 3 tests and `cargo check` produced 0 diagnostics.

## iteration-002

- Slice: repair the compatibility API while preserving `te_compile`, `te_interp`, and mutable error-position out-parameters.
- Compatibility repair: match on `Parser::parse_expression()`, return `Some(expr)` and set `error = 0` on success, or return `None` and set `error = parse_error.position` on failure.
- Compatibility result: `cargo test` passed 3 tests and `cargo check` produced 0 diagnostics.
- Rust-native result: `cargo test` passed 3 tests and `cargo check` produced 0 diagnostics.
- Shortcut pressure: no `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `.clone()` calls in source.
- Navigation effect: useful as a type-boundary repair signal. It did not require ownership-event mapping, but it did surface the owned-result redesign direction for the Rust-native track and the explicit adapter boundary for compatibility.
- Completion decision: target complete at decimal literals, variables, unary signs, binary arithmetic, parentheses, deterministic parse errors, compatibility out-param API, and Rust-native `Result` API.
