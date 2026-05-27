# miniz Streaming Comparison Matrix

## Paired Slice 001 - Stream Lifecycle Pass-Through

| Metric                          |                                    Compatibility |    Rust-Native | Interpretation                                                                                   |
| ------------------------------- | -----------------------------------------------: | -------------: | ------------------------------------------------------------------------------------------------ |
| first cargo-check diagnostics   |                                                2 |              0 | Compatibility shape immediately exposed borrow-order pressure; Rust-native owned output did not. |
| ownership diagnostics           |                                                1 |              0 | Compatibility first check produced supported E0502.                                              |
| non-ownership diagnostics       |                                                0 |              0 | No non-ownership blockers in either track.                                                       |
| repair iterations to pass tests |                              1 report-guided fix |              0 | Compatibility needed one local reorder; Rust-native passed first.                                |
| shortcut pressure events        | 0 shortcuts, 1 long-lived output borrow pressure |              0 | Compatibility retained caller output borrow by design.                                           |
| tests passed                    |                                                3 |              3 | Both tracks satisfy the pass-through lifecycle behavior.                                         |
| navigation changed next action  |                                              yes | no diagnostics | Compatibility fix followed report evidence.                                                      |

## Notes

- Compatibility track intentionally preserves long-lived input/output buffer fields.
- Rust-native track intentionally uses owned output and short borrow scopes.
- The first paired slice already validates the comparison design: compatibility preservation produced E0502, while Rust-native design avoided it without shortcuts.
