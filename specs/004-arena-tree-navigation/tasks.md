# Tasks: Arena And Tree Navigation Suggestions

## Phase 1: Specification And Issue Setup

- [x] A001 Create feature spec, implementation plan, and task ledger.
- [x] A002 Create GitHub issues for P1/P2 implementation slices: #61, #62, #63.

## Phase 2: P1 Arena/Stable-ID Suggestions

Issue: #61

- [ ] A003 Extend `DesignSuggestionKind` in `src/mapper/ownership-event.ts` with `arena-backed-tree`, `stable-node-id`, and `avoid-self-referential-struct`.
- [ ] A004 Add deterministic E0499/E0502 arena/tree suggestion rules in `src/mapper/design-suggestion.ts`.
- [ ] A005 Add mapper tests for DOM-like E0499 evidence in `test/mapper/design-suggestion.test.ts`.
- [ ] A006 Update `docs/navigation-suggestion-design.md` with arena/tree rules.

## Phase 3: P1 Self-Referential Guidance

Issue: #62

- [ ] A007 Add E0505/E0515 guidance path for self-referential struct diagnostics.
- [ ] A008 Add tests using domhandler iteration-001 diagnostic evidence.
- [ ] A009 Ensure unsupported diagnostics can still carry guidance without pretending full ownership mapping is available.

## Phase 4: P2 HTML Design Translation

Issue: #63

- [ ] A010 Improve `src/reporter/html-reporter.ts` rendering for design translation language.
- [ ] A011 Add HTML reporter coverage for arena/tree suggestions.

## Phase 5: Verification

- [ ] A012 Run `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, and `npm run build`.
