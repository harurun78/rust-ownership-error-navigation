# Iteration Log

## iteration-001

- Model condition: main agent implementation under A/B validation instructions.
- Slice: simple start tags, end tags, and text nodes.
- Human ownership hints: none beyond the track condition definitions.
- Compatibility prompt summary: preserve parser object, incremental `write`, queued event delivery, and borrowed views into parser-owned input storage.
- Rust-native prompt summary: preserve tokenization behavior with owned events and `Result` errors.
- Expected pressure: compatibility event queue should create borrow pressure when parser buffer is compacted after a queued view is stored.
- Compatibility result: `cargo check` produced 3 total diagnostics, including 2 supported ownership diagnostics (`E0502`).
- Navigation summary: recommended ending shared borrows before mutable borrows and emitted `avoid-long-lived-buffer-borrow` design suggestions.
- Rust-native result: after a non-ownership predicate fix, `cargo test` passed 3 tests and `cargo check` produced 0 diagnostics.

## iteration-002

- Slice: repair the compatibility event queue while keeping the same tokenization behavior.
- Compatibility repair: replace queued `&str` payloads with queued byte spans into the parser buffer, then create borrowed `Event<'_>` views only in `next_event`.
- Navigation effect: changed the implementation from long-lived borrowed event storage to span-based event storage.
- Compatibility result: `cargo test` passed 4 tests; `cargo check` produced 0 diagnostics.
- Rust-native result: `cargo test` passed 3 tests; `cargo check` produced 0 diagnostics.
- Shortcut pressure: no `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `.clone()` calls detected.
- Next slice candidate: attributes or incremental partial tags if more parser-state pressure is desired.

## iteration-003

- Slice: quoted start-tag attributes plus compatibility incremental partial tags.
- Compatibility condition: continue preserving queued event delivery with borrowed event and attribute views, backed by span storage rather than long-lived `&str` payloads.
- Rust-native condition: return owned `Event::StartTag { name, attributes }` records.
- Compatibility result: `cargo test` passed 7 tests; `cargo check` produced 0 diagnostics.
- Rust-native result: `cargo test` passed 6 tests; `cargo check` produced 0 diagnostics.
- Shortcut pressure: no `unsafe`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or broad `.clone()` calls detected.
- Navigation effect: no new repair signal after the iteration-002 span-queue repair; the same design continued to scale to attributes and partial tags.
- Completion decision: target complete at tags, text, quoted attributes, malformed tag/attribute rejection, and compatibility partial tag completion.
