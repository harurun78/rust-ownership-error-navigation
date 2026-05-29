# Iteration Log

## iteration-001

- Model condition: main agent implementation under A/B validation instructions.
- Slice: root tree, start tags, text nodes, end tags, parent links, and child lists.
- Human ownership hints: none beyond the track condition definitions.
- Compatibility prompt summary: preserve direct object-graph style parent and child references with an open-element stack.
- Rust-native prompt summary: preserve behavior with owned arena nodes and stable `NodeId` links.
- Expected pressure: direct parent/child references should create mutable aliasing and lifetime pressure when pushing children and updating the stack.
- Compatibility result: `cargo check` produced 8 total diagnostics, including 4 supported ownership diagnostics (`E0499`). It also surfaced unsupported-but-relevant self-referential ownership diagnostics (`E0515`, `E0505`).
- Navigation summary: recommended shortening mutable borrow scopes around `parent.children` and `self.stack`; the broader repair direction was to avoid storing direct references in the tree graph.
- Rust-native result: `cargo test` passed 2 tests and `cargo check` produced 0 diagnostics.

## iteration-002

- Slice: repair compatibility tree construction while preserving handler callbacks, open stack behavior, parent links, child lists, and root inspection.
- Compatibility repair: replace direct `&mut Node` parent/stack references with stable `NodeId` links into an owned `Vec<Node>` arena.
- Compatibility result: `cargo test` passed 3 tests; `cargo check` produced 0 diagnostics.
- Rust-native result: `cargo test` passed 2 tests; `cargo check` produced 0 diagnostics.
- Shortcut pressure: no `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `.clone()` calls detected.
- Navigation effect: changed the compatibility implementation from a direct object graph to arena-indexed tree storage.
- Completion decision: target complete at start tags, text nodes, end tags, parent links, child lists, and root child inspection.
