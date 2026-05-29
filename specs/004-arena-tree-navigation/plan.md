# Implementation Plan: Arena And Tree Navigation Suggestions

## Technical Context

- Language: TypeScript
- Runtime: Node.js 20
- Test framework: Vitest
- Existing surfaces:
  - `src/mapper/design-suggestion.ts`
  - `src/mapper/ownership-event.ts`
  - `src/mapper/index.ts`
  - `src/mapper/unsupported.ts`
  - `src/reporter/html-reporter.ts`
  - `docs/navigation-suggestion-design.md`

## Validation Inputs

- `validation/ports/sax-js-queued-events/reports/comparison-summary.md`
- `validation/ports/domhandler-tree-builder/reports/comparison-summary.md`
- `validation/ports/domhandler-tree-builder/reports/compatibility/iteration-001/cargo-check.jsonl`

## Phases

### Phase 1 - Deterministic Arena Suggestions

- Extend `DesignSuggestionKind` with arena/tree kinds.
- Add E0499/E0502 tree-pressure rule using local diagnostic text and span evidence.
- Add mapper tests using synthetic diagnostics or domhandler report-derived fixtures.

### Phase 2 - Self-Referential Guidance

- Add conservative guidance for E0505/E0515 diagnostics when local evidence indicates returning a value that borrows local state or moving after borrow.
- Decide whether E0505/E0515 become supported diagnostics or unsupported records with design guidance.
- Add JSON and HTML reporter tests.

### Phase 3 - HTML Design Translation

- Improve the report rendering for design suggestions with explicit design translation wording.
- Keep static HTML simple and compatible with existing snapshots.

### Phase 4 - Documentation And Verification

- Update navigation suggestion design documentation.
- Run lint, format, type-check, tests, and build.

## Risk And Mitigation

- False positives: keep rules conservative and evidence-triggered.
- Unsupported diagnostic widening: start with guidance-only behavior before deeper event mapping.
- UI churn: add small, focused rendering changes rather than redesigning the full report.
