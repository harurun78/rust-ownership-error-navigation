# Comparison Matrix

## Paired Slice 001 - DOM Tree Builder

| Metric                          | Compatibility | Rust-Native | Interpretation                                                                        |
| ------------------------------- | ------------: | ----------: | ------------------------------------------------------------------------------------- |
| first cargo-check diagnostics   |             8 |           0 | Compatibility exposed direct-reference tree pressure; Rust-native arena stayed clean. |
| ownership diagnostics           |             4 |           0 | Compatibility produced E0499 around child-list and stack mutable aliasing.            |
| non-ownership diagnostics       |             0 |           0 | Recorded comparison diagnostics were ownership/lifetime related.                      |
| repair iterations to pass tests |             1 |           0 | Compatibility needed an arena repair; Rust-native passed directly.                    |
| shortcut pressure events        |             0 |           0 | No `unsafe`, shared mutability wrappers, or broad clone shortcuts.                    |
| tests passed                    |             3 |           2 | Compatibility adds nested tree coverage; Rust-native covers owned arena behavior.     |
| navigation changed next action  |           yes |          no | The report pushed compatibility away from direct mutable references.                  |

## Slice 001 Interpretation

This target produced stronger tree-mutation pressure than the queued-event target. The direct-reference compatibility attempt failed before tests because the handler tried to store mutable parent references, push children, borrow children again, and update the open stack in overlapping phases. The repair changed tree identity from Rust references to stable `NodeId` values while preserving observable parent and child links.
