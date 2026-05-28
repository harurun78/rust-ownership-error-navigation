# Navigation Suggestion Design Without LLM API Calls

Date: 2026-05-27

## Conclusion

Rust-native design suggestions can be implemented without calling an external LLM service. The first version should be deterministic: map normalized rustc diagnostics, span roles, labels, suggestions, and local code-shape hints to conservative suggestion templates.

This will not replace semantic program analysis, but it is sufficient for the product's current value proposition: help Rust learners and porting agents choose safer ownership boundaries before reaching for `clone`, shared mutability, or `unsafe`.

## Product Goal

The navigation report should not only say how to fix a local error. It should also identify when the code shape looks like a C-style port and suggest a Rust-native redesign direction.

Examples:

- Replace long-lived borrowed parser views with owned parse records.
- Replace C out-parameters with return values or builders.
- Replace shared mutable context with a short mutation phase plus immutable result.
- Replace callback state stored in a struct with callback invocation over short-lived row/event views.
- Replace ad hoc state flags with an enum state machine or typestate builder.

## Inputs Available Locally

The current CLI already has enough local data for rule-based suggestions:

- rustc diagnostic code, level, message, children, and suggestions.
- normalized spans with file, line, label, snippet, primary/cause/conflict roles, and macro expansion flags.
- existing mapped ownership events for E0382, E0499, and E0502.
- non-ownership navigation records for E0308, E0004, and E0425.
- audience mode: `beginner`, `intermediate`, or `agent`.
- optional fixture or validation metadata when reports are generated from `validation/ports/**`.

No network call is needed for these rules.

## Suggested Data Model

Add an optional array to each diagnostic navigation record:

```ts
type DesignSuggestionKind =
  | "owned-result"
  | "builder"
  | "state-machine"
  | "short-borrow-callback"
  | "split-mutation-phase"
  | "avoid-c-style-out-param"
  | "avoid-long-lived-buffer-borrow";

interface DesignSuggestion {
  kind: DesignSuggestionKind;
  title: string;
  why: string;
  whenToUse: string;
  caution: string;
  confidence: "high" | "medium" | "low";
  evidence: Evidence[];
}
```

Keep suggestions non-mutating. Do not emit patches in this feature.

## Deterministic Rule Examples

| Trigger | Suggestion | Confidence |
| --- | --- | --- |
| E0502 with labels containing immutable borrow later used plus mutable borrow | split mutation phase or owned snapshot | high |
| E0499 with first mutable borrow later used | shorten first mutable borrow; consider state-machine step API | high |
| E0382 with moved value later reused and child note suggests borrowing | accept borrowed parameter or return owned value from builder | high |
| span snippets include `&mut self` across multiple conflicting methods | split context into phase-specific structs | medium |
| snippets include `Option<&`, `Vec<&`, or struct fields with borrowed data in validation target | prefer owned parse records over long-lived buffer borrows | medium |
| E0308 around expected mutable reference or pointer-like type | replace C out-parameter shape with return value when behavior-only track allows it | medium |
| E0425 after compatibility-preserving port references C helper name | resolve missing API shim before ownership redesign | high |

## Audience Rendering

- `beginner`: explain the local cause first, then one Rust-native direction in plain language.
- `intermediate`: show trade-offs: allocation, API boundary, mutation phase, callback lifetime.
- `agent`: output compact structured hints with rule id, confidence, and evidence fields.

## Implementation Sketch

1. Add `src/mapper/design-suggestion.ts` with pure rule functions.
2. Extend ownership and non-ownership mappers to call `deriveDesignSuggestions(record, context)`.
3. Add JSON reporter support for optional `designSuggestions`.
4. Add HTML section after fix strategies titled `Design Direction`.
5. Add tests using synthetic diagnostics and validation fixtures.
6. Add comparison validation metrics for suggestion presence and whether it changed the next iteration.

## Limits

- The rules should be conservative. If evidence is weak, emit no suggestion or use low confidence.
- These suggestions cannot prove semantic equivalence.
- They should not tell the user that C ABI compatibility can be preserved by a Rust-native redesign. In compatibility-preserving tracks, show the Rust-native direction as prevention value, not as an allowed fix unless the experiment condition changes.
- Full semantic analysis can be considered later with rust-analyzer or MIR-based tooling, still without an LLM API call.

## Recommended First Slice

Implement three deterministic suggestions first:

1. `avoid-long-lived-buffer-borrow` for borrow conflicts around parser/input buffers.
2. `split-mutation-phase` for E0499/E0502 involving `&mut self` or conflicting context mutation.
3. `owned-result` for E0382/E0308 cases that look like C out-parameter or moved-value reuse patterns.

This is enough to test the product idea discovered during libpng validation: Rust-native design can prevent many ownership errors before local repairs are needed.

## Implemented Minimal Slice

The first deterministic slice is implemented as local mapper rules. It does not call an external LLM API and does not generate patches.

- `split-mutation-phase`: emitted for E0499/E0502 records with cause, conflict, and context events.
- `avoid-long-lived-buffer-borrow`: emitted for E0499/E0502 records when local text evidence mentions parser, stream, input, output, or buffer pressure.
- `owned-result`: emitted for E0382 moved-value reuse and E0308 type-boundary pressure.

Reports expose these under optional `designSuggestions` in JSON and a `Design Direction` section in static HTML.