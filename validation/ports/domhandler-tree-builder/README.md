# domhandler Tree Builder Validation

This target ports a small `domhandler` style DOM tree builder into Rust under two conditions.

- Upstream style: JavaScript/TypeScript `fb55/domhandler`, commonly used with `htmlparser2`.
- Validation focus: parent links, child lists, mutable open-element stacks, and node views that can escape tree mutation.
- Current completion boundary: start tags, text nodes, end tags, parent-child relationships, and root child inspection.

The target is selected after `sax-js-queued-events` because it adds arena/tree mutation pressure on top of callback and queue pressure.

## Tracks

- `tracks/compatibility/rust-port`: preserves a handler object, open-element stack, parent links, and caller-visible tree nodes.
- `tracks/rust-native/rust-port`: uses owned nodes with stable `NodeId` references and short mutation phases.

## Report Paths

- `reports/compatibility/iteration-001/`
- `reports/rust-native/iteration-001/`
- `reports/compatibility/iteration-002/`
- `reports/rust-native/iteration-002/`
