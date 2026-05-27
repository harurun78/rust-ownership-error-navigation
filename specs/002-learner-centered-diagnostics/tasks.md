# Tasks: Learner-Centered Diagnostic Navigation

**Input**: Design documents from `/specs/002-learner-centered-diagnostics/`

**Prerequisites**: plan.md, spec.md, existing MVP implementation, Redis validation roadmap

**Tests**: Required. Each user story must include mapper/reporter/integration coverage because this feature changes learner-facing report behavior.

**Organization**: Tasks are grouped by user story so each story can be implemented and tested independently after shared foundations are complete.

## GitHub Issue Mapping

- #48: T001-T004 shared learner diagnostic model foundation
- #49: T005-T012 learner summary cards
- #50: T013-T019 beginner / intermediate / agent audience modes
- #51: T020-T029 high-frequency non-ownership diagnostics
- #52: T030-T036 first-fix order and fix strategy guidance

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files or isolated tests.
- **[Story]**: User story label for story-specific phases.
- Every task includes exact file paths.

---

## Phase 1: Setup And Shared Model

**Purpose**: Add the shared learner summary, audience, diagnostic navigation, and fix strategy model surfaces used by all stories.

- [x] T001 Define `AudienceMode`, `LearnerSummary`, `FixStrategy`, and `RecommendedFirstFix` types in `src/mapper/ownership-event.ts` or a new `src/mapper/diagnostic-navigation.ts`
- [x] T002 [P] Add model unit tests for learner summary and fix strategy shape in `test/mapper/diagnostic-navigation.test.ts`
- [x] T003 Add report schema expectations for optional learner fields in `specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json` or a new versioned contract file
- [x] T004 [P] Add reporter fixture helper for generating reports under different audience modes in `test/helpers/diagnostic-fixtures.ts`

**Checkpoint**: Report model can carry learner-oriented fields without breaking MVP diagnostics.

---

## Phase 2: User Story 1 - Learner summary cards (Priority: P1) MVP

**Goal**: Show what happened, why it matters, and the next step for supported ownership diagnostics.

**Independent Test**: Generate a beginner report from `ownership-baseline-2026-05-24.jsonl` and verify JSON/HTML learner summaries for E0382 / E0499 / E0502.

### Tests for User Story 1

- [x] T005 [P] [US1] Add learner summary mapper tests for E0382 in `test/mapper/learner-summary.e0382.test.ts`
- [x] T006 [P] [US1] Add learner summary mapper tests for E0499/E0502 in `test/mapper/learner-summary.borrow-conflicts.test.ts`
- [x] T007 [P] [US1] Add JSON reporter snapshot coverage for learner summaries in `test/reporter/json-reporter.learner-summary.test.ts`
- [x] T008 [P] [US1] Add HTML reporter snapshot coverage for summary cards in `test/reporter/html-reporter.learner-summary.test.ts`

### Implementation for User Story 1

- [x] T009 [US1] Implement learner summary generation for E0382/E0499/E0502 in `src/mapper/learner-summary.ts`
- [x] T010 [US1] Attach learner summaries to supported diagnostic records in `src/mapper/index.ts` or `src/reporter/json-reporter.ts`
- [x] T011 [US1] Render learner summary cards before detailed event sections in `src/reporter/html-reporter.ts`
- [x] T012 [US1] Export learner summary helpers from `src/index.ts`

**Checkpoint**: Beginner-facing summary cards are visible while existing causality, spans, evidence, and Borrow Sheet sections still render.

---

## Phase 3: User Story 2 - Audience modes (Priority: P1)

**Goal**: Let reports target beginner, intermediate, or agent audiences without changing the underlying diagnostic evidence.

**Independent Test**: Run the same fixture with `beginner`, `intermediate`, and `agent` modes and compare summary surface while preserving events/spans.

### Tests for User Story 2

- [x] T013 [P] [US2] Add CLI option parsing tests for `--audience` in `test/cli/main.test.ts`
- [x] T014 [P] [US2] Add audience mode snapshot tests in `test/reporter/audience-modes.test.ts`
- [x] T015 [P] [US2] Add integration test for beginner/intermediate/agent output in `test/integration/audience-modes.integration.test.ts`

### Implementation for User Story 2

- [x] T016 [US2] Add `--audience beginner|intermediate|agent` CLI parsing and default behavior in `src/cli/main.ts`
- [x] T017 [US2] Thread `AudienceMode` through mapper/reporter generation in `src/cli/main.ts` and reporter APIs
- [x] T018 [US2] Add audience-specific summary wording in `src/mapper/learner-summary.ts`
- [x] T019 [US2] Document audience mode usage in `docs/quickstart.md` and `README.md`

**Checkpoint**: Audience mode is selectable from CLI and produces stable learner/agent surfaces.

---

## Phase 4: User Story 3 - High-frequency non-ownership diagnostics (Priority: P1)

**Goal**: Promote E0308, E0004, and E0425 from display-only fallback to first-class diagnostic navigation records.

**Independent Test**: Use Redis validation derived fixtures and verify first-class records plus grouped summary output.

### Tests for User Story 3

- [x] T020 [P] [US3] Add selected Redis validation JSONL fixtures under `test/fixtures/diagnostics/porting/`
- [x] T021 [P] [US3] Add E0308 mapper tests in `test/mapper/e0308.test.ts`
- [x] T022 [P] [US3] Add E0004 mapper tests in `test/mapper/e0004.test.ts`
- [x] T023 [P] [US3] Add E0425 mapper tests in `test/mapper/e0425.test.ts`
- [x] T024 [P] [US3] Add reporter snapshot tests for non-ownership diagnostic grouping in `test/reporter/non-ownership-diagnostics.test.ts`

### Implementation for User Story 3

- [x] T025 [US3] Implement E0308 type mismatch navigation in `src/mapper/e0308.ts`
- [x] T026 [US3] Implement E0004 non-exhaustive pattern navigation in `src/mapper/e0004.ts`
- [x] T027 [US3] Implement E0425 unresolved name navigation in `src/mapper/e0425.ts`
- [x] T028 [US3] Register non-ownership mappers in `src/mapper/index.ts`
- [x] T029 [US3] Render ownership and non-ownership diagnostic groups in `src/reporter/html-reporter.ts`

**Checkpoint**: Redis-derived non-ownership blockers are no longer only unsupported display records.

---

## Phase 5: User Story 4 - Multi-diagnostic first-fix order and fix strategies (Priority: P2)

**Goal**: Help learners decide where to start and understand fix trade-offs.

**Independent Test**: Mixed diagnostic report includes deterministic recommended first fixes and fix strategy trade-off notes.

### Tests for User Story 4

- [x] T030 [P] [US4] Add recommended first-fix ordering tests in `test/mapper/recommended-first-fixes.test.ts`
- [x] T031 [P] [US4] Add fix strategy tests for E0382/E0499/E0502 in `test/mapper/fix-strategy.test.ts`
- [x] T032 [P] [US4] Add HTML reporter snapshot tests for first fixes and strategy trade-offs in `test/reporter/fix-strategy.test.ts`

### Implementation for User Story 4

- [x] T033 [US4] Implement deterministic first-fix ordering in `src/mapper/recommended-first-fixes.ts`
- [x] T034 [US4] Implement fix strategy taxonomy and ownership diagnostic mapping in `src/mapper/fix-strategy.ts`
- [x] T035 [US4] Add summary output for recommended first fixes in `src/reporter/json-reporter.ts` and `src/reporter/html-reporter.ts`
- [x] T036 [US4] Add fix strategy trade-off rendering in `src/reporter/html-reporter.ts`

**Checkpoint**: Multi-diagnostic reports tell learners what to try first and why.

---

## Phase 6: Polish And Verification

**Purpose**: Keep docs, examples, and quality gates aligned with the new learner-centered behavior.

- [ ] T037 [P] Update example reports in `examples/ownership-report.json` and `examples/ownership-report.html`
- [ ] T038 [P] Update `docs/spec.md`, `docs/data-model.md`, and `docs/quickstart.md` with learner summary, audience modes, and non-ownership navigation behavior
- [ ] T039 [P] Update `test/fixtures/diagnostics/README.md` with porting validation fixture rationale
- [ ] T040 Run `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, and `npm run build`; record results in `tasks/todo.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- Phase 1 blocks all user stories.
- Phase 2 can start after Phase 1 and is the first implementation slice.
- Phase 3 depends on Phase 2 because audience modes use learner summaries.
- Phase 4 depends on Phase 1 and can proceed after summary model is stable.
- Phase 5 depends on Phase 4 for mixed diagnostic ordering.
- Phase 6 depends on completed target stories.

### Recommended First Issues

1. T001-T004 shared learner model setup.
2. T005-T012 learner summary cards.
3. T013-T019 audience modes.
4. T020-T029 high-frequency non-ownership diagnostics.
5. T030-T036 first-fix order and fix strategy guidance.

## Implementation Strategy

Start with learner summary cards because they add direct human value while preserving the MVP mapper/report architecture. Then add audience modes, followed by Redis-derived non-ownership diagnostics and prioritization.
