# miniz Streaming Comparison Summary

Date: 2026-05-27

## Status

Paired slice 001 is complete.

## Hypothesis

- Compatibility-preserving shape should create more ownership/lifetime pressure as slices add real streaming decode behavior.
- Rust-native shape should prevent some ownership diagnostics by moving data into owned outputs and short-lived borrow boundaries.

## Current Conclusion

The first miniz-streaming comparison produced the ownership-pressure contrast this validation was designed to measure.

- Compatibility-preserving track first check: E0502 from reading `output.len()` after storing a long-lived mutable output borrow in `MzStream`.
- Navigation report: supported E0502, recommended separating shared and mutable access.
- Report-guided repair: moved `output.len()` before assigning `next_out`; fixed check passed.
- Rust-native track first check: 0 diagnostics, because output ownership stays inside `Inflater` and update borrows are short-lived.

## Metrics

| Metric | Compatibility | Rust-Native |
| --- | ---: | ---: |
| first cargo-check diagnostics | 2 | 0 |
| ownership diagnostics | 1 | 0 |
| supported ownership code | E0502 | none |
| tests passed after iteration | 3 | 3 |
| shortcut pressure events | 0 | 0 |
| report-guided repair used | yes | not needed |

## Interpretation

Repair value is already visible in the compatibility track: the report pinpointed the shared borrow after the mutable borrow and the fix followed that evidence.

Prevention value is visible in the Rust-native track: owned output and short borrow scopes avoided the same class of error before navigation was needed.

## Next Slice

Use stored deflate block decode with incremental input/output pressure. This should preserve the fair comparison while adding enough parser state to test whether compatibility pressure grows beyond simple borrow ordering.