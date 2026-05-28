# Specification: sax-js Queued Events A/B Porting Validation

## Target Choice

Use a `sax-js` style XML streaming tokenizer because it is JavaScript-origin, callback-driven, and naturally creates pressure around event queues and parser input buffers.

## Hypothesis

- Compatibility-preserving Rust will encounter ownership pressure when queued events hold borrowed views into a parser-owned buffer while parsing attempts to mutate or compact that buffer.
- Rust-native Rust can avoid that pressure by returning owned event records and keeping borrows local to tokenization.
- The navigation report should surface borrow-conflict guidance around long-lived buffer borrows and mutation phase splitting.

## Conditions

### Compatibility-Preserving

- Preserve a parser object with incremental `write` calls.
- Preserve queued event delivery after parsing.
- Prefer borrowed event fields that view parser-owned input storage.
- Preserve caller-visible queue and parser state where the slice requires it.

### Rust-Native

- Preserve observable tokenization behavior only.
- Return owned `Event` records.
- Use `Result<T, E>` for malformed tags.
- Keep source input borrows local to the parse function.

## Iteration 001 Scope

- Parse `<name>` start tags.
- Parse `</name>` end tags.
- Parse text between tags.
- Reject empty or whitespace-containing tag names.
- Save cargo-check diagnostics and navigation reports for both tracks.

## Iteration 002 Repair Scope

- Repair compatibility E0502 diagnostics from queued borrowed event views.
- Preserve queued event delivery while replacing long-lived buffer borrows with queued byte spans.
- Create borrowed `Event<'_>` views only at `next_event` delivery time.
- Keep Rust-native behavior unchanged as an owned output baseline.

## Iteration 003 Completion Scope

- Parse quoted start-tag attributes.
- Preserve compatibility queued event delivery with borrowed event views and borrowed attribute views.
- Return Rust-native owned attributes.
- Accept incremental partial tags in the compatibility parser, for example `write("<ro")` followed by `write("ot id=\"main\">")`.
- Reject invalid attributes and malformed end-tag names.

This is the completion boundary for the current target. It covers the parser-buffer, queued-event, borrowed-view, owned-output, and incremental-input ownership surfaces without expanding into full XML compliance.

## Non-Goals

- Full XML compliance.
- Namespaces.
- Entities and CDATA.
- Encoding detection.
- Real Node.js stream compatibility.
