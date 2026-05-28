# sax-js Queued Events Validation

This target ports a small `sax-js` style XML streaming tokenizer into Rust under two conditions.

- Upstream style: JavaScript `isaacs/sax-js` streaming tokenizer callbacks.
- Validation focus: queued parser events that want to reference parser input buffers after parsing continues.
- Completed slice: start tags, end tags, text nodes, quoted attributes, and incremental partial tags.

The target is intentionally selected after `http-parser-js-streaming` because it makes borrowed event views more likely to escape the immediate parser call.

## Tracks

- `tracks/compatibility/rust-port`: preserves mutable parser state and queued events with borrowed parser-buffer views where possible.
- `tracks/rust-native/rust-port`: returns owned event values and keeps parse borrows local.

## Report Paths

- `reports/compatibility/iteration-001/`
- `reports/rust-native/iteration-001/`
- `reports/compatibility/iteration-002/`
- `reports/rust-native/iteration-002/`
- `reports/compatibility/iteration-003/`
- `reports/rust-native/iteration-003/`

## Completion Boundary

- simple start tags and end tags
- text nodes between tags
- quoted start-tag attributes
- incremental partial tag completion in the compatibility parser
- malformed tag and attribute rejection
