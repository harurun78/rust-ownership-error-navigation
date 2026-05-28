# sax-js Queued Events Validation

This target ports a small `sax-js` style XML streaming tokenizer into Rust under two conditions.

- Upstream style: JavaScript `isaacs/sax-js` streaming tokenizer callbacks.
- Validation focus: queued parser events that want to reference parser input buffers after parsing continues.
- Current slice: simple start tags, end tags, and text nodes.

The target is intentionally selected after `http-parser-js-streaming` because it makes borrowed event views more likely to escape the immediate parser call.

## Tracks

- `tracks/compatibility/rust-port`: preserves mutable parser state and queued events with borrowed parser-buffer views where possible.
- `tracks/rust-native/rust-port`: returns owned event values and keeps parse borrows local.

## Report Paths

- `reports/compatibility/iteration-001/`
- `reports/rust-native/iteration-001/`
