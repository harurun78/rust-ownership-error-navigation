# miniz Streaming Comparison Summary

Date: 2026-05-27

## Status

Paired slices 001 and 002 are complete. The target is complete at the zlib stored-block decode boundary.

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

No further slice is required for this target. Full Huffman-coded deflate, compression, preset dictionaries, and performance tuning are intentionally outside scope because they would mostly measure compression algorithm implementation rather than ownership-navigation value.

## Iteration 002 Result

The second slice added minimal real zlib/DEFLATE behavior:

- zlib header validation
- DEFLATE stored block decoding
- LEN/NLEN validation
- Adler-32 validation
- single and multiple stored-block tests
- compatibility output-buffer pressure
- Rust-native output-limit errors

Both tracks compiled and passed tests with zero diagnostics.

| Metric | Compatibility | Rust-Native |
| --- | ---: | ---: |
| iteration-002 cargo-check diagnostics | 0 | 0 |
| iteration-002 ownership diagnostics | 0 | 0 |
| tests passed | 5 | 5 |
| shortcut pressure events | 0 | 0 |

## Final Assessment

The comparison validated both dimensions of the new evaluation design:

- **Repair value**: compatibility-preserving shape produced E0502 in iteration-001, and the navigation report directly guided the fix.
- **Prevention value**: Rust-native shape avoided that borrow conflict from the start by using owned output and short borrow scopes.

The stored-block completion slice showed that once the compatibility borrow-order issue is resolved, both tracks can implement a small real streaming decode behavior without shortcuts. This supports the product direction: navigation should keep local repair guidance, but it should also surface Rust-native design suggestions as prevention guidance.