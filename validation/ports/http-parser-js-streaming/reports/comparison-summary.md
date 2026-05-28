# http-parser-js Streaming Comparison Summary

## Status

Iterations 001 and 002 are complete. The target is complete at request-head + `Content-Length` body + minimal chunked body behavior.

## Hypothesis

- Compatibility shape should concentrate borrow pressure around callback events and parser state.
- Rust-native shape should keep parse borrows short and return owned records.

## Current Result

Both tracks compiled and passed tests with zero diagnostics through the completion slice.

| Metric | Compatibility | Rust-Native |
| --- | ---: | ---: |
| cargo-check diagnostics | 0 | 0 |
| ownership diagnostics | 0 | 0 |
| tests passed | 8 | 8 |
| shortcut pressure events | 0 | 0 |

## Interpretation

The JavaScript-origin target is a good broadened benchmark because it is callback-driven and not C/C++. In this completion boundary, however, it is mostly prevention/baseline evidence rather than repair evidence: compatibility callbacks remained safe because all borrowed body views are short-lived and invoked within `execute`, while Rust-native output remains owned and straightforward.

## Final Assessment

- **Repair value**: limited. No ownership diagnostics were produced in saved reports.
- **Prevention value**: useful. The Rust-native API demonstrates a simpler owned result shape, and the compatibility API shows that callback designs are not inherently problematic when borrow scopes stay short.
- **Target lesson**: to stress deterministic design suggestions more strongly, the next target or future variant should force callbacks, views, or parser buffers to escape a single parser call.

## Completion Boundary

- request line and headers
- `Content-Length` request body
- minimal chunked request body
- malformed request line/header/body rejection
- callback-based compatibility delivery
- owned Rust-native request output