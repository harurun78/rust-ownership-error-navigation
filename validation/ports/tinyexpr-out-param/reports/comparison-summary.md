# tinyexpr Out-Param Comparison Summary

## Status

Iterations 001 and 002 are complete. The target is complete at decimal numeric literals, variables, unary plus/minus, binary arithmetic, parentheses, deterministic parse error positions, a compatibility C-style out-param API, and a Rust-native `Result` API.

## Hypothesis

- Compatibility shape should expose type-boundary pressure when a natural Rust parser returns `Result<Expr, ParseError>` but the public API preserves `Option<Expr>` plus `error_out`.
- Rust-native shape should avoid this pressure by returning `Result` directly.

## Current Result

The compatibility first attempt produced E0308 at the parser/API boundary. The navigation report surfaced the diagnostic as a supported non-ownership record and emitted `owned-result` design guidance. The repair preserved the compatibility API by adding an explicit adapter from `Result` to `Option` plus error position. The Rust-native track compiled cleanly and passed tests from the first completed iteration.

| Metric | Compatibility | Rust-Native |
| --- | ---: | ---: |
| first cargo-check diagnostics | 2 | 0 |
| ownership diagnostics | 0 | 0 |
| non-ownership diagnostics | 1 | 0 |
| final cargo-check diagnostics | 0 | 0 |
| tests passed after repair | 3 | 3 |
| shortcut pressure events | 0 | 0 |

## Interpretation

The first compatibility attempt returned `Result<TeExpr, ParseError>` from a function declared as `Option<TeExpr>`, exactly the kind of boundary mismatch expected when C-style out-parameters meet Rust-native error handling. The report's `owned-result` suggestion was more directly applicable to the Rust-native track than to strict compatibility. For compatibility, the useful action was to keep the out-parameter surface but make the conversion explicit in one place.

## Assessment

- **Repair value**: medium. The E0308 report identified the precise type boundary, and the design suggestion pointed to the cleaner Rust-native shape. Compatibility still required a manual adapter decision.
- **Prevention value**: strong. The Rust-native `Result` API avoided the mismatch entirely and kept parse errors structured.
- **Target lesson**: E0308 owned-result guidance should distinguish strict compatibility adapters from behavior-only redesign. `Result` vs `Option`/out-param evidence is a high-value trigger, but the rendered advice should say whether the caller may change the public API.

## Completion Boundary

- decimal numeric literals
- named variables
- unary plus/minus
- binary `+`, `-`, `*`, `/`
- parentheses and precedence
- deterministic 1-based parse errors
- compatibility `te_compile` / `te_interp` with error-position out-param
- Rust-native `compile` / `evaluate` with `Result`
