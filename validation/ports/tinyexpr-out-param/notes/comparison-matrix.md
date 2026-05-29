# Comparison Matrix

| Metric                          | Compatibility | Rust-Native | Interpretation                                                                           |
| ------------------------------- | ------------: | ----------: | ---------------------------------------------------------------------------------------- |
| first cargo-check diagnostics   |             2 |           0 | Compatibility exposed one E0308 plus a rustc failure-note; Rust-native compiled cleanly. |
| ownership diagnostics           |             0 |           0 | This target primarily validated non-ownership type-boundary navigation.                  |
| non-ownership diagnostics       |             1 |           0 | E0308 appeared at the `Result` to `Option` out-param API boundary.                       |
| repair iterations to pass tests |             2 |           1 | Compatibility needed one repair; Rust-native passed in the first completed iteration.    |
| shortcut pressure events        |             0 |           0 | No `unsafe`, shared mutability wrappers, or broad clones were used.                      |
| tests passed                    |             3 |           3 | Both final tracks cover arithmetic, variables/unary behavior, and error reporting.       |
| navigation changed next action  |           yes |         yes | Compatibility added an adapter; Rust-native kept `Result` as the public API.             |
