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

## Paired Slice 002 - zlib Stored Block Decode

| Metric                          |      Compatibility |    Rust-Native | Interpretation                                                                     |
| ------------------------------- | -----------------: | -------------: | ---------------------------------------------------------------------------------- |
| first cargo-check diagnostics   |                  0 |              0 | After the iteration-001 fix, both tracks handled stored-block decoding cleanly.    |
| ownership diagnostics           |                  0 |              0 | No new E0382/E0499/E0502 emerged in the stored-block slice.                        |
| non-ownership diagnostics       |                  0 |              0 | No type/name/control-flow blockers.                                                |
| repair iterations to pass tests |                  0 |              0 | Both tracks passed after implementation.                                           |
| shortcut pressure events        |                  0 |              0 | No `unsafe`, shared mutability wrappers, or broad clone shortcuts.                 |
| tests passed                    |                  5 |              5 | Both tracks decode single and multiple stored blocks and reject checksum failures. |
| navigation changed next action  | no new diagnostics | no diagnostics | Prevention remains the main signal in iteration-002.                               |

## Completion Note

The target is complete for minimal miniz/zlib streaming comparison. Iteration-001 provided repair evidence; iteration-002 showed both designs can implement stored-block behavior without additional ownership diagnostics once the compatibility borrow-order issue is fixed.
