---
description: "Use when running A/B C/C++ to Rust porting validation: compatibility-preserving versus behavior-only Rust-native implementation, including navigation-effect measurement."
applyTo: 'validation/ports/**'
---

# Porting Comparison Validation Instructions

Use these rules when a validation target compares two porting conditions:

- **Compatibility-preserving port**: keep C API/ABI shape, long-lived context state, callbacks, allocator hooks, and observable call sequencing as much as the slice requires.
- **Behavior-only Rust-native port**: keep only input/output behavior and visible semantics; redesign APIs toward owned values, `Result`, builders, iterators, short borrow scopes, and explicit state machines.

## Purpose

This comparison measures two different navigation values:

1. Whether diagnostic navigation helps repair ownership/lifetime/mutable-aliasing errors when C compatibility pressure is high.
2. Whether Rust-native design suggestions can prevent ownership errors or reduce shortcut pressure before errors accumulate.

Do not treat clean compilation as a failed experiment. Clean compilation under the Rust-native condition is evidence that ownership pressure can be designed away.

## Required Target Layout

Each comparison target under `validation/ports/<target>/` should keep these artifacts:

```text
validation/ports/<target>/
├── spec.md
├── plan.md
├── tasks.md
├── quickstart.md
├── notes/
│   ├── iteration-log.md
│   └── comparison-matrix.md
├── tracks/
│   ├── compatibility/
│   │   └── rust-port/
│   └── rust-native/
│       └── rust-port/
└── reports/
    ├── compatibility/iteration-NNN/
    ├── rust-native/iteration-NNN/
    └── comparison-summary.md
```

If a target already has a single `rust-port/`, keep it as historical data and create `tracks/` only for the comparison experiment.

## Experimental Controls

For both tracks, keep these constant unless the prompt states otherwise:

- Same upstream version and license record.
- Same behavior requirements and fixture tests.
- Same model class or declared model identity.
- Same maximum slice size.
- Same iteration artifact requirements.
- Same prohibition on hidden human ownership fixes.

Only the porting condition should differ.

## Compatibility-Preserving Condition

Prompt the implementer to preserve:

- Public function names and call order when feasible.
- Opaque context structs or handles.
- Caller-visible mutable state across API calls.
- Callback registration and callback invocation behavior.
- Allocator/error hook surface, even if implemented as safe Rust placeholders.
- ABI notes for any place where true `extern "C"` or `unsafe` would be required.

Measurement focus:

- E0382, E0499, E0502, E0505, E0596, E0597, and lifetime-related diagnostics.
- Pressure to add `unsafe`, broad `.clone()`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, raw pointers, or global mutable state.
- Whether navigation reports reduce repeated diagnostics in the next iteration.

## Behavior-Only Rust-Native Condition

Prompt the implementer to preserve:

- Input/output fixtures.
- Error categories and visible behavior.
- Deterministic handling of invalid input.
- Test-observable state transitions.

Allow and encourage:

- Owned parse results instead of borrowed views into long-lived buffers.
- Builders or typestate/state-machine APIs instead of mutable C context structs.
- Iterator or callback boundaries that do not store borrowed callback data.
- Narrow mutation phases and explicit ownership transfer.
- `Result<T, E>` instead of C-style error jumps or out-parameters.

Measurement focus:

- Whether Rust-native design avoids ownership diagnostics.
- Whether total iterations, shortcut pressure, or API complexity decrease.
- Whether deterministic navigation suggestions correctly point toward Rust-native redesign patterns.

## Iteration Artifacts

For each track iteration, save:

- `cargo-check.jsonl`
- `ownership-report.json`
- `ownership-report.html`
- test output summary
- shortcut pressure notes
- whether report guidance changed the next implementation action

Use track-specific report paths, for example:

```bash
validation/ports/<target>/reports/compatibility/iteration-001/cargo-check.jsonl
validation/ports/<target>/reports/rust-native/iteration-001/cargo-check.jsonl
```

## Comparison Summary

At the end of a paired slice, update `reports/comparison-summary.md` with:

| Metric | Compatibility | Rust-Native | Interpretation |
| --- | ---: | ---: | --- |
| first cargo-check diagnostics | | | |
| ownership diagnostics | | | |
| non-ownership diagnostics | | | |
| repair iterations to pass tests | | | |
| shortcut pressure events | | | |
| tests passed | | | |
| navigation changed next action | yes/no | yes/no | |

Conclude separately on:

- **repair value**: navigation helped fix concrete compiler diagnostics.
- **prevention value**: Rust-native suggestions avoided diagnostics or shortcut pressure.

## Suggestion Implementation Constraint

Do not require an LLM service call for navigation suggestions. Prefer deterministic rules over the existing normalized rustc diagnostics, spans, evidence, code categories, and local pattern matching. If a suggestion would require semantic understanding not available locally, emit a conservative design note instead of inventing a precise fix.