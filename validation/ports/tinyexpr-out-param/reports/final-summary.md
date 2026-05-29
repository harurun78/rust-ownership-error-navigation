# tinyexpr Out-Param Validation Final Summary

Date: 2026-05-29

## Completion Boundary

This validation target ports a tinyexpr-style arithmetic evaluator through two tracks: a compatibility-preserving C-shaped API and a behavior-only Rust-native API. The completed slice supports decimal numeric literals, variables, unary plus/minus, binary arithmetic with precedence, parentheses, deterministic 1-based parse error positions, compatibility `te_compile` / `te_interp` out-parameter behavior, and Rust-native `Result`-returning compile/evaluate behavior.

## Iteration Results

| Track | Iteration | Result | Tests | E0308 | Ownership Diagnostics | Notes |
| --- | --- | --- | ---: | ---: | ---: | --- |
| compatibility | iteration-001 | cargo-check failed | 0 | 1 | 0 | Returned `Result<TeExpr, ParseError>` from `Option<TeExpr>` API boundary. |
| rust-native | iteration-001 | compile/test pass | 3 | 0 | 0 | Exposed `Result<Expr, ParseError>` directly. |
| compatibility | iteration-002 | compile/test pass | 3 | 0 | 0 | Added explicit `Result` to `Option` plus `error_out` adapter. |
| rust-native | iteration-002 | compile/test pass | 3 | 0 | 0 | Rechecked clean baseline after compatibility repair. |

## Shortcut Pressure

Final source scan found no `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `.clone()` calls in the tinyexpr Rust port source. Expression names are owned with `to_owned()` at compile time; evaluation uses borrowed expression trees.

## Navigation App Effect Check

The compatibility iteration-001 report contains one supported non-ownership E0308 diagnostic and one rustc failure note. It also emits `owned-result` design guidance for the `Result`/`Option` type boundary.

Observed activation:

| Track/Iteration | Total Diagnostics | Supported | Non-Ownership | Design Suggestion |
| --- | ---: | ---: | ---: | --- |
| compatibility iteration-001 | 2 | 1 | 1 | `owned-result` |
| compatibility iteration-002 | 0 | 0 | 0 | none |
| rust-native iteration-001 | 0 | 0 | 0 | none |
| rust-native iteration-002 | 0 | 0 | 0 | none |

Interpretation:

- The target confirms E0308 out-param/API-boundary pressure is a useful validation class for #60.
- The existing `owned-result` suggestion is directionally correct for Rust-native redesign.
- For compatibility-preserving ports, the next product improvement is wording or trigger refinement that distinguishes "change the API to Result" from "add an adapter at the C-shaped boundary."

## Completion Decision

The target is complete for this validation cycle. Remaining tinyexpr surface area such as custom function tables and exact C ABI layout should be tracked as a future compatibility extension only if #60 requires more evidence.