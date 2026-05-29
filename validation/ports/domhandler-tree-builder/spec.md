# Specification: domhandler Tree Builder A/B Porting Validation

## Target Choice

Use a `domhandler` style DOM tree builder because it is JavaScript/TypeScript-origin and naturally combines mutable tree construction, parent links, child lists, and an open-element stack.

## Hypothesis

- Compatibility-preserving Rust will encounter ownership pressure when it tries to store mutable parent references and child references while also mutating the tree.
- Rust-native Rust can avoid the pressure by using stable node IDs over an owned arena.
- The navigation report should surface mutable aliasing and long-lived reference guidance that points toward arena/index storage.

## Conditions

### Compatibility-Preserving

- Preserve a handler object with `on_open_tag`, `on_text`, and `on_close_tag` callbacks.
- Preserve a root node and an open-element stack.
- Preserve parent links and child lists in the observable tree.
- First attempt may use direct references to mirror JavaScript object graph identity.

### Rust-Native

- Preserve tree-building behavior only.
- Use owned nodes in an arena with `NodeId` links.
- Use short mutation phases and no parent/child Rust references stored in nodes.
- Return deterministic errors for mismatched close tags.

## Iteration 001 Scope

- Build a root tree from `<name>`, text, and `</name>` events.
- Preserve parent-child relationships.
- Capture cargo-check diagnostics for a direct-reference compatibility attempt.
- Save navigation reports for both tracks.

## Iteration 002 Completion Scope

- Repair the compatibility track by replacing direct parent/child references with stable node IDs.
- Preserve handler callbacks, open stack behavior, parent links, child lists, and root inspection.
- Keep Rust-native arena output as the prevention baseline.
- Save clean cargo-check diagnostics and reports for both tracks.

This is the completion boundary for the current target. It covers escaping node views, parent-child graph shape, mutable stack updates, and arena/tree mutation without implementing a full HTML parser.

## Non-Goals

- Full HTML parsing.
- Attributes and namespaces.
- HTML implied tag rules.
- DOM query APIs.
- Browser compatibility.
