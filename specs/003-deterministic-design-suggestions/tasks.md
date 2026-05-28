# Tasks: Deterministic Design Suggestions

**Input**: Design documents from `/specs/003-deterministic-design-suggestions/`

**Issue Status**: GitHub Issue created as #58.

## GitHub Issue Mapping

- #58: T001-T012 deterministic design suggestions minimal implementation.

## Phase 1: Model And Rule Foundation

- [x] T001 Add `DesignSuggestionKind` and `DesignSuggestion` types to `src/mapper/ownership-event.ts`.
- [x] T002 Add `src/mapper/design-suggestion.ts` with pure deterministic derivation rules.
- [x] T003 Export design suggestion helpers from `src/index.ts`.

## Phase 2: Mapper Integration

- [x] T004 Attach design suggestions in `src/mapper/index.ts` after diagnostic-specific mapping and fix strategy generation.
- [x] T005 Thread `AudienceMode` into suggestion wording without changing diagnostic evidence.

## Phase 3: Reporter Output

- [x] T006 Preserve `designSuggestions` in JSON report output.
- [x] T007 Render `Design Direction` section in `src/reporter/html-reporter.ts`.

## Phase 4: Tests

- [x] T008 Add mapper tests for `split-mutation-phase` and `avoid-long-lived-buffer-borrow` using E0499/E0502 fixtures.
- [x] T009 Add mapper tests for `owned-result` using E0382/E0308 fixtures.
- [x] T010 Add JSON reporter test for `designSuggestions` preservation.
- [x] T011 Add HTML reporter test for `Design Direction` rendering.

## Phase 5: Docs And Verification

- [x] T012 Update `docs/navigation-suggestion-design.md`, `docs/quickstart.md`, examples, and the issue draft with the implemented slice and verification result.