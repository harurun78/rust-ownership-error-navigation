# Specification: tinyexpr Out-Param A/B Porting Validation

## Target Choice

Use a tinyexpr-style arithmetic expression evaluator because the original C API exposes a pointer-like output parameter for parse error position: `te_compile(expression, variables, count, &error)`. This is a compact target for validating E0308 and owned-result guidance.

## Hypothesis

- Compatibility-preserving Rust will experience type-boundary pressure when a Rust parser naturally returns `Result<Expr, ParseError>` but the public API wants `Option<Expr>` plus a mutable error-position out-parameter.
- Rust-native Rust can avoid that pressure by exposing `Result<Expr, ParseError>` directly and keeping parse/evaluation state owned.
- Navigation reports should classify E0308 and suggest converting out-parameter API shapes into owned results when behavior-only redesign is allowed.

## Conditions

### Compatibility-Preserving

- Preserve `te_compile(expression, variables, error_out)` and `te_interp(expression, variables, error_out)` style functions.
- Preserve caller-visible mutable error position state.
- Preserve owned expression trees returned after successful compilation.
- Avoid `unsafe`; document ABI gaps rather than implementing raw C pointers.

### Rust-Native

- Preserve arithmetic behavior and deterministic parse errors only.
- Return `Result<Expr, ParseError>` instead of using out-parameters.
- Keep variables as borrowed input during compilation and owned names in the compiled expression.

## Completion Boundary

- Decimal numeric literals.
- Variables supplied by name/value pairs.
- Unary plus/minus.
- Binary `+`, `-`, `*`, and `/` with precedence.
- Parenthesized expressions.
- Deterministic 1-based parse error positions.

## Non-Goals

- Full tinyexpr function table support.
- C ABI and raw pointer compatibility.
- Constant folding or optimizer behavior.
