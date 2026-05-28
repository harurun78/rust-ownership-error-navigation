# sax-js Queued Events Comparison Summary

## Status

Iteration 001 diagnostics and iteration 002 repair are complete.

## Hypothesis

- Compatibility shape should expose long-lived borrow pressure around queued events that view parser buffers.
- Rust-native shape should avoid that pressure by returning owned event records.

## Current Result

The compatibility track produced ownership diagnostics on the first attempt, while the Rust-native track compiled cleanly. A navigation-guided compatibility repair then passed tests and cargo check.

| Metric | Compatibility | Rust-Native |
| --- | ---: | ---: |
| first cargo-check diagnostics | 3 | 0 |
| ownership diagnostics | 2 | 0 |
| final cargo-check diagnostics | 0 | 0 |
| tests passed after repair | 4 | 3 |
| shortcut pressure events | 0 | 0 |

## Interpretation

The compatibility first attempt queued borrowed `&str` event payloads into a parser-owned buffer and then attempted to mutate that buffer. Rust reported E0502 because the immutable buffer borrow was required to live for the parser event lifetime while `drain` requested mutable access.

The navigation report recommended ending shared borrows before mutable borrows and emitted `avoid-long-lived-buffer-borrow`. The repair followed that direction by storing byte spans in the queue and resolving them into borrowed event views only when the caller asks for the next event.

## Assessment

- **Repair value**: strong for this slice. Navigation directly changed the compatibility implementation strategy.
- **Prevention value**: also strong. The Rust-native owned event API avoided the borrow conflict entirely.
- **Next slice**: attributes or incremental partial tags can add more parser-state pressure while keeping the same target.
