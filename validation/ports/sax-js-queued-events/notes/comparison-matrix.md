# Comparison Matrix

## Paired Slice 001 - Queued Tag And Text Events

| Metric                          | Compatibility | Rust-Native | Interpretation                                                                           |
| ------------------------------- | ------------: | ----------: | ---------------------------------------------------------------------------------------- |
| first cargo-check diagnostics   |             3 |           0 | Compatibility surfaced queued-buffer borrow pressure; Rust-native stayed clean.          |
| ownership diagnostics           |             2 |           0 | Compatibility produced E0502 for mutating `self.buffer` after queued borrowed views.     |
| non-ownership diagnostics       |             0 |           0 | The unrelated predicate mismatch was fixed before recorded comparison.                   |
| repair iterations to pass tests |             1 |           0 | Compatibility needed a navigation-guided span queue repair; Rust-native passed directly. |
| shortcut pressure events        |             0 |           0 | No `unsafe`, shared mutability wrappers, or broad clone shortcuts.                       |
| tests passed                    |             4 |           3 | Compatibility has queued delivery tests; Rust-native has owned parse output tests.       |
| navigation changed next action  |           yes |          no | The report pointed compatibility away from long-lived buffer borrows.                    |

## Slice 001 Interpretation

This target produced the repair signal missing from the previous HTTP parser target. The compatibility shape tried to keep queued `&str` payloads into a parser-owned buffer and then mutate that buffer, yielding E0502. The deterministic design suggestion correctly identified long-lived buffer borrows, and the repair changed queued storage to byte spans with short-lived borrowed views at delivery time.

## Paired Slice 003 - Attributes And Partial Tags

| Metric                          | Compatibility | Rust-Native | Interpretation                                                                           |
| ------------------------------- | ------------: | ----------: | ---------------------------------------------------------------------------------------- |
| cargo-check diagnostics         |             0 |           0 | The span-queue repair continued to compile cleanly after attributes and partial tags.    |
| ownership diagnostics           |             0 |           0 | No E0382/E0499/E0502 surfaced in the completion slice.                                   |
| non-ownership diagnostics       |             0 |           0 | Both tracks passed saved cargo-check captures.                                           |
| repair iterations to pass tests |             0 |           0 | No additional repair was needed after the iteration-002 design change.                   |
| shortcut pressure events        |             0 |           0 | No `unsafe`, shared mutability wrappers, or broad clone shortcuts.                       |
| tests passed                    |             7 |           6 | Compatibility covers incremental partial tags; Rust-native covers whole-input behavior.  |
| navigation changed next action  |            no |          no | The earlier navigation-guided span queue was sufficient for the expanded completion set. |

## Completion Note

The target is complete for the selected ownership-navigation boundary. It produced a useful repair signal in the first compatibility attempt, then showed that the report-guided span queue design scales to attributes and incremental parser input without additional ownership diagnostics.
