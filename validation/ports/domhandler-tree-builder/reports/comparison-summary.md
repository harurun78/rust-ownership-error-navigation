# domhandler Tree Builder Comparison Summary

## Status

Iterations 001 and 002 are complete. The target is complete at start tags, text nodes, end tags, parent links, child lists, and root child inspection.

## Hypothesis

- Compatibility shape should expose mutable aliasing pressure around parent links, child lists, and the open-element stack.
- Rust-native shape should avoid that pressure by storing nodes in an arena and linking them by `NodeId`.

## Current Result

The compatibility direct-reference first attempt produced ownership diagnostics, while the Rust-native arena baseline compiled cleanly. A navigation-guided compatibility repair switched the tree representation to stable `NodeId` links and then passed tests and cargo check.

| Metric | Compatibility | Rust-Native |
| --- | ---: | ---: |
| first cargo-check diagnostics | 8 | 0 |
| ownership diagnostics | 4 | 0 |
| final cargo-check diagnostics | 0 | 0 |
| tests passed after repair | 3 | 2 |
| shortcut pressure events | 0 | 0 |

## Interpretation

The compatibility first attempt mirrored a JavaScript object graph too directly: nodes stored mutable parent references, parents owned child nodes, and the handler stack stored mutable node references. Rust reported E0499 when the implementation tried to borrow parent children and the open stack in overlapping mutable phases. It also reported self-referential construction issues around returning a root while keeping a mutable root reference in the stack.

The repair followed the navigation direction by eliminating long-lived mutable references from the stored graph. Nodes now live in an owned arena, and parent/child/stack relationships use stable `NodeId` values. Mutation happens in short phases: create node, append child ID, update stack.

## Assessment

- **Repair value**: strong. Navigation identified the overlapping mutable scopes that made the direct-reference tree shape untenable.
- **Prevention value**: strong. The Rust-native arena design avoided diagnostics from the beginning.
- **Target lesson**: DOM-like tree builders are a high-value validation class for explaining when Rust references are the wrong identity mechanism and arena IDs are a better design.

## Completion Boundary

- root document node
- element nodes
- text nodes
- parent links by ID
- child lists by ID
- open-element stack
- extra close-tag rejection
