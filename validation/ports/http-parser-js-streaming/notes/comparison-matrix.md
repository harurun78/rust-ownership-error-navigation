# Comparison Matrix

## Paired Slice 001 - Request Head Parsing

| Metric                          | Compatibility | Rust-Native | Interpretation                                                             |
| ------------------------------- | ------------: | ----------: | -------------------------------------------------------------------------- |
| first cargo-check diagnostics   |             0 |           0 | Request-head parsing did not create compiler pressure in either condition. |
| ownership diagnostics           |             0 |           0 | No E0382/E0499/E0502 repair signal yet.                                    |
| non-ownership diagnostics       |             0 |           0 | No type/name/control-flow blockers.                                        |
| repair iterations to pass tests |             0 |           0 | Both tracks passed after initial implementation.                           |
| shortcut pressure events        |             0 |           0 | No `unsafe`, shared mutability wrappers, or broad clone shortcuts.         |
| tests passed                    |             4 |           4 | Both tracks cover valid, malformed, and incomplete request heads.          |
| navigation changed next action  |            no |          no | Next action is target-pressure selection rather than report-guided repair. |

## Slice 001 Interpretation

The request-head slice is useful as a baseline but too shallow to trigger ownership navigation. The next slice should preserve the same A/B shape while adding body or chunk callbacks so compatibility code has a stronger reason to keep caller-visible parser state and borrowed buffers alive across callback boundaries.

## Paired Slice 002 - Body Completion

| Metric                          | Compatibility | Rust-Native | Interpretation                                                                                         |
| ------------------------------- | ------------: | ----------: | ------------------------------------------------------------------------------------------------------ |
| first cargo-check diagnostics   |             0 |           0 | Final saved checks are clean in both tracks.                                                           |
| ownership diagnostics           |             0 |           0 | No E0382/E0499/E0502 repair signal emerged even with body callbacks.                                   |
| non-ownership diagnostics       |             0 |           0 | The Rust-native helper-name collision was fixed before saved cargo-check artifacts.                    |
| repair iterations to pass tests |             0 |           1 | Rust-native required one local helper rename; compatibility passed after implementation.               |
| shortcut pressure events        |             0 |           0 | No `unsafe`, shared mutability wrappers, or broad clone shortcuts.                                     |
| tests passed                    |             8 |           8 | Both tracks cover request heads, content-length bodies, chunked bodies, and malformed inputs.          |
| navigation changed next action  |            no |          no | Clean compilation means this target mainly contributes prevention/baseline evidence, not repair value. |

## Completion Note

The JavaScript-origin callback parser is complete for the chosen ownership-navigation boundary. It did not produce ownership diagnostics, which is still useful evidence: callback-style parsing can remain safe when borrowed views are short-lived, while Rust-native owned records keep the API simpler. A stronger future target should force callbacks or views to escape parser execution if repair-signal generation is the goal.
