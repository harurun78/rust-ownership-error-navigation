# Implementation Plan: Deterministic Design Suggestions

**Branch**: `003-deterministic-design-suggestions` | **Date**: 2026-05-28 | **Spec**: [spec.md](spec.md)

## Summary

Add deterministic, local-only design suggestions to diagnostic records. The first slice turns validation findings from miniz/libpng into product behavior: keep repair guidance, and add prevention guidance for Rust-native API boundaries.

## Technical Context

- **Language**: TypeScript on Node.js 20.
- **Inputs**: Existing normalized rustc diagnostics, spans, events, fix strategies, learner summaries, and audience mode.
- **Outputs**: Existing JSON report and static HTML report.
- **Constraints**: No external network/LLM calls. No source patch generation. Suggestions are explanatory only.

## Implementation Steps

1. Add `DesignSuggestionKind` and `DesignSuggestion` model types to mapper/report records.
2. Add pure derivation rules in `src/mapper/design-suggestion.ts`.
3. Attach suggestions in `mapDiagnostic` after mapper/fix-strategy generation and before learner summary rendering.
4. Render `Design Direction` in HTML and preserve suggestions in JSON.
5. Add unit/reporter tests for E0499/E0502 and E0382/E0308 suggestion surfaces.
6. Update docs and examples enough to document the new optional field.

## Rules In First Slice

| Rule | Trigger | Suggestion |
| --- | --- | --- |
| DS001 | E0499/E0502 with conflict and context events | `split-mutation-phase` |
| DS002 | E0499/E0502 with span labels/snippets mentioning input/output/buffer/parser/stream | `avoid-long-lived-buffer-borrow` |
| DS003 | E0382 moved value reuse or E0308 type boundary mismatch | `owned-result` |

## Verification

- `npm run format:check`
- `npm run lint`
- `npm run type-check`
- `npm run test:run`
- `npm run test:integration`
- `npm run build`

## Risk

The main risk is over-suggesting from weak evidence. Keep rules conservative, evidence-backed, and optional.