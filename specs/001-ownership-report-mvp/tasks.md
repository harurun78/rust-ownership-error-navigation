# Tasks: Rust Ownership Diagnostic Report MVP

**Input**: Design documents from `/specs/001-ownership-report-mvp/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md, contracts/diagnostic-report.schema.json

**Tests**: Included. The feature specification and implementation plan explicitly require parser unit tests, mapper tests, reporter snapshots, JSON contract validation, fixture compatibility tests, and independent user-story tests.

**Organization**: Tasks are grouped by user story so each story can be implemented and tested independently after shared foundations are complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files and does not depend on incomplete tasks
- **[Story]**: User story label for story-specific phases only
- Every task includes exact file paths

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the CLI/library project shape and test dependencies for implementation.

- [x] T001 Create source module entry files in src/cli/main.ts, src/parser/cargo-message-parser.ts, src/diagnostics/rustc-diagnostic.ts, src/diagnostics/diagnostic-span.ts, src/mapper/ownership-event.ts, src/reporter/json-reporter.ts, and src/reporter/html-reporter.ts
- [x] T002 Add direct JSON schema validation dependency and npm scripts for CLI verification in package.json
- [x] T003 [P] Configure TypeScript build output and declaration settings for CLI/library modules in tsconfig.json
- [x] T004 [P] Configure Vitest coverage and snapshot behavior for reporter tests in vitest.config.ts
- [x] T005 [P] Export public parser, diagnostic, mapper, and reporter modules from src/index.ts

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement shared parser, diagnostic model, normalized span model, mapper registry, fixture helpers, and base reporter contracts required by every user story.

**CRITICAL**: No user story work can begin until this phase is complete.

- [x] T006 [P] Define CargoMessage, RustcDiagnostic, RustcChildDiagnostic, and RustcSpan raw payload types in src/diagnostics/rustc-diagnostic.ts
- [x] T007 [P] Define DiagnosticSpan, Evidence, Confidence, and DiagnosticSpanRole types in src/diagnostics/diagnostic-span.ts
- [x] T008 [P] Define OwnershipEvent, DiagnosticRecord, DiagnosticReport, and supported event kind types in src/mapper/ownership-event.ts
- [x] T009 [P] Add fixture loading helpers for JSONL diagnostics in test/helpers/diagnostic-fixtures.ts
- [x] T010 [P] Add parser unit tests for compiler-message extraction and non-compiler-message skipping in test/parser/cargo-message-parser.test.ts
- [x] T011 Implement JSONL parser with malformed-line errors that include line numbers in src/parser/cargo-message-parser.ts
- [x] T012 [P] Add diagnostic normalization tests for code, level, message, spans, children, rendered, and 1-based locations in test/diagnostics/normalizer.test.ts
- [x] T013 Implement rustc diagnostic and span normalization in src/diagnostics/normalizer.ts
- [x] T014 Implement mapper registry for E0382, E0499, E0502, and unsupported fallback dispatch in src/mapper/index.ts
- [x] T015 [P] Add shared HTML escaping and stable ID helper tests in test/reporter/reporter-utils.test.ts
- [x] T016 Implement shared reporter utilities for HTML escaping, stable IDs, and summary counts in src/reporter/reporter-utils.ts

**Checkpoint**: Parser, normalized diagnostic model, mapper dispatch, and shared reporter utilities are ready for user stories.

---

## Phase 3: User Story 1 - E0382 moved value cause report (Priority: P1) MVP

**Goal**: Show where a value was moved, where it was used after move, and any clone/possible-fix evidence in JSON and HTML.

**Independent Test**: Run the CLI against the baseline fixture filtered to E0382 and verify JSON plus HTML include move cause, use conflict, source locations, evidence, confidence, and possible_fix.

### Tests for User Story 1

- [x] T017 [P] [US1] Add E0382 mapper expectation tests for move cause, use conflict, context, evidence, confidence, and possible_fix in test/mapper/e0382.test.ts
- [x] T018 [P] [US1] Add E0382 JSON reporter snapshot test using ownership-baseline-2026-05-24.jsonl in test/reporter/json-reporter.e0382.test.ts
- [x] T019 [P] [US1] Add E0382 HTML reporter snapshot test for Summary, Causality Timeline, Source Spans, Evidence, and possible_fix rows in test/reporter/html-reporter.e0382.test.ts
- [x] T020 [P] [US1] Add E0382 CLI integration test for json-out and html-out files in test/integration/e0382-report.integration.test.ts

### Implementation for User Story 1

- [x] T021 [US1] Implement E0382 span role classification and ownership event mapping in src/mapper/e0382.ts
- [x] T022 [US1] Implement rustc child suggestion and possible_fix extraction in src/mapper/rustc-suggestions.ts
- [x] T023 [US1] Implement supported diagnostic JSON report output with schemaVersion 0.1.0, input, summary, diagnostics, spans, and events in src/reporter/json-reporter.ts
- [x] T024 [US1] Implement static HTML report sections for supported diagnostics in src/reporter/html-reporter.ts
- [x] T025 [US1] Implement CLI argument parsing for --input, --json-out, and --html-out in src/cli/main.ts
- [x] T026 [US1] Wire parser, normalizer, mapper registry, JSON reporter, and HTML reporter in src/cli/main.ts
- [x] T027 [US1] Export E0382 mapper and reporter APIs from src/index.ts

**Checkpoint**: User Story 1 is independently functional and testable as the MVP.

---

## Phase 4: User Story 2 - E0499 and E0502 borrow conflicts in causal order (Priority: P1)

**Goal**: Show first borrow, conflicting borrow/use, and later context for mutable and shared-vs-mutable borrow diagnostics.

**Independent Test**: Run mapper and CLI tests with baseline E0499/E0502 diagnostics and verify cause, conflict, and context roles plus event kinds are correct.

### Tests for User Story 2

- [x] T028 [P] [US2] Add E0499 mapper tests for first mutable borrow cause, second mutable borrow conflict, later use context, evidence, and confidence in test/mapper/e0499.test.ts
- [x] T029 [P] [US2] Add E0502 mapper tests for immutable borrow cause, mutable borrow conflict, later immutable use context, evidence, and confidence in test/mapper/e0502.test.ts
- [x] T030 [P] [US2] Add borrow-conflict CLI integration test for E0499 and E0502 JSON/HTML output in test/integration/borrow-conflicts.integration.test.ts
- [x] T031 [P] [US2] Add HTML reporter snapshot test for causal ordering of borrow conflict events in test/reporter/html-reporter.borrow-conflicts.test.ts

### Implementation for User Story 2

- [x] T032 [US2] Implement E0499 span role classification and ownership event mapping in src/mapper/e0499.ts
- [x] T033 [US2] Implement E0502 span role classification and ownership event mapping in src/mapper/e0502.ts
- [x] T034 [US2] Register E0499 and E0502 mappers in src/mapper/index.ts
- [x] T035 [US2] Update HTML reporter event ordering for cause, conflict, context, and possible_fix in src/reporter/html-reporter.ts
- [x] T036 [US2] Update JSON reporter summary counts for multiple supported diagnostics in src/reporter/json-reporter.ts

**Checkpoint**: User Stories 1 and 2 both work independently against the baseline fixture.

---

## Phase 5: User Story 3 - Unsupported diagnostics are retained display-only (Priority: P2)

**Goal**: Preserve non-Phase-1 diagnostics and null-code diagnostics without mapper failure, then show them in JSON and HTML summaries.

**Independent Test**: Run follow-up, advanced, and non-ownership smoke fixtures through the CLI and verify unsupported diagnostics remain visible in JSON and HTML.

### Tests for User Story 3

- [ ] T037 [P] [US3] Add compatibility integration tests for ownership-followup-2026-05-25.jsonl, ownership-advanced-2026-05-25.jsonl, and rustc-non-ownership-smoke-2026-05-25.jsonl in test/integration/unsupported-diagnostics.integration.test.ts
- [ ] T038 [P] [US3] Add parser edge-case tests for null diagnostic code, empty children, missing primary span, null labels, macro expansion, and mixed Windows/POSIX paths in test/parser/edge-cases.test.ts
- [ ] T039 [P] [US3] Add malformed JSONL error tests that assert line number reporting in test/parser/malformed-jsonl.test.ts
- [ ] T040 [P] [US3] Add unsupported diagnostic reporter snapshot tests in test/reporter/unsupported-diagnostics.test.ts

### Implementation for User Story 3

- [ ] T041 [US3] Implement unsupported diagnostic fallback record creation in src/mapper/unsupported.ts
- [ ] T042 [US3] Preserve unsupported diagnostic code, message, spans, children, and rendered output in src/reporter/json-reporter.ts
- [ ] T043 [US3] Render unsupported diagnostics display-only section in src/reporter/html-reporter.ts
- [ ] T044 [US3] Ensure CLI exits successfully for unsupported diagnostics and exits nonzero for malformed JSONL in src/cli/main.ts
- [ ] T045 [US3] Register unsupported fallback behavior in src/mapper/index.ts

**Checkpoint**: Compatibility fixtures parse and report without widening Phase 1 mapping scope.

---

## Phase 6: User Story 4 - Borrow Sheet learning view (Priority: P3)

**Goal**: Add a static HTML Borrow Sheet section that reuses ownership events while preserving Rust event kind labels.

**Independent Test**: Pass E0382 event JSON to the HTML reporter and verify both causality view and Borrow Sheet section appear with move, borrow, use, and conflict labels unchanged.

### Tests for User Story 4

- [ ] T046 [P] [US4] Add Borrow Sheet HTML snapshot test for E0382 event JSON in test/reporter/borrow-sheet.e0382.test.ts
- [ ] T047 [P] [US4] Add Rust event kind label preservation test for move, borrow_shared, borrow_mut, use, conflict, possible_fix, and context in test/reporter/borrow-sheet-labels.test.ts

### Implementation for User Story 4

- [ ] T048 [US4] Implement Borrow Sheet row model generation in src/reporter/borrow-sheet.ts
- [ ] T049 [US4] Integrate Borrow Sheet section into static HTML output in src/reporter/html-reporter.ts
- [ ] T050 [US4] Export Borrow Sheet reporter helpers from src/index.ts

**Checkpoint**: HTML report includes both causality and Borrow Sheet views for supported diagnostics.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Contract validation, quickstart verification, documentation, and final quality checks across all stories.

- [ ] T051 [P] Add JSON schema contract validation tests against specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json in test/reporter/diagnostic-report-schema.test.ts
- [ ] T052 [P] Add README CLI usage examples for baseline and compatibility fixture workflows in README.md
- [ ] T053 [P] Add generated output examples for JSON and HTML reports in examples/ownership-report.json and examples/ownership-report.html
- [ ] T054 [P] Document unsupported diagnostic behavior, Phase 1 mapping scope, and rustc 1-based positions versus future VS Code 0-based adapter conversion in docs/quickstart.md
- [ ] T055 Add end-to-end quickstart validation test covering json-out, html-out, schema validation, and non-mutating behavior outside requested outputs in test/integration/quickstart.integration.test.ts
- [ ] T056 Run npm ci, npm run lint, npm run format:check, npm run type-check, npm run test:run, npm run test:integration, and npm run build; record the result in tasks/todo.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; can start immediately.
- **Foundational (Phase 2)**: Depends on Setup; blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational; MVP delivery slice.
- **User Story 2 (Phase 4)**: Depends on Foundational; can proceed in parallel with US1 after shared contracts are stable, but priority is P1 after US1 for MVP validation.
- **User Story 3 (Phase 5)**: Depends on Foundational and reporter surfaces from US1; preserves compatibility without expanding mapping scope.
- **User Story 4 (Phase 6)**: Depends on supported ownership events from US1/US2 and HTML reporter from US1.
- **Polish (Phase 7)**: Depends on selected user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational; no dependency on other stories.
- **US2 (P1)**: Can start after Foundational; independent mapper tests, shares reporter surfaces.
- **US3 (P2)**: Can start after Foundational; reporting tasks benefit from US1 JSON/HTML reporter completion.
- **US4 (P3)**: Requires ownership event JSON and HTML reporter output from earlier stories.

### Within Each User Story

- Write tests first and verify they fail before implementation.
- Implement mapper logic before wiring mapper registry changes.
- Implement reporter behavior before CLI integration assertions are finalized.
- Complete each story checkpoint before moving to the next priority if working sequentially.

---

## Parallel Opportunities

- T003, T004, and T005 can run in parallel after T001 is understood.
- T006, T007, T008, T009, T010, T012, and T015 can run in parallel during Foundational work.
- US1 test tasks T017, T018, T019, and T020 can run in parallel.
- US2 test tasks T028, T029, T030, and T031 can run in parallel.
- US3 test tasks T037, T038, T039, and T040 can run in parallel.
- US4 test tasks T046 and T047 can run in parallel.
- Polish tasks T051, T052, T053, and T054 can run in parallel after the corresponding implementation surfaces exist.

---

## Parallel Example: User Story 1

```bash
Task: "T017 [P] [US1] Add E0382 mapper expectation tests in test/mapper/e0382.test.ts"
Task: "T018 [P] [US1] Add E0382 JSON reporter snapshot test in test/reporter/json-reporter.e0382.test.ts"
Task: "T019 [P] [US1] Add E0382 HTML reporter snapshot test in test/reporter/html-reporter.e0382.test.ts"
Task: "T020 [P] [US1] Add E0382 CLI integration test in test/integration/e0382-report.integration.test.ts"
```

## Parallel Example: User Story 2

```bash
Task: "T028 [P] [US2] Add E0499 mapper tests in test/mapper/e0499.test.ts"
Task: "T029 [P] [US2] Add E0502 mapper tests in test/mapper/e0502.test.ts"
Task: "T030 [P] [US2] Add borrow-conflict CLI integration test in test/integration/borrow-conflicts.integration.test.ts"
Task: "T031 [P] [US2] Add HTML reporter causal ordering snapshot in test/reporter/html-reporter.borrow-conflicts.test.ts"
```

## Parallel Example: User Story 3

```bash
Task: "T037 [P] [US3] Add compatibility integration tests in test/integration/unsupported-diagnostics.integration.test.ts"
Task: "T038 [P] [US3] Add parser edge-case tests in test/parser/edge-cases.test.ts"
Task: "T039 [P] [US3] Add malformed JSONL tests in test/parser/malformed-jsonl.test.ts"
Task: "T040 [P] [US3] Add unsupported reporter snapshots in test/reporter/unsupported-diagnostics.test.ts"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 setup.
2. Complete Phase 2 foundational parser, model, mapper registry, fixture helpers, and reporter utilities.
3. Complete Phase 3 User Story 1.
4. Stop and validate E0382 independently through mapper, JSON reporter, HTML reporter, and CLI integration tests.
5. Demo with ownership-baseline-2026-05-24.jsonl filtered to E0382 or with an E0382-only fixture.

### Incremental Delivery

1. Setup + Foundational: parse and normalize diagnostics with shared contracts.
2. US1: E0382 moved value report as MVP.
3. US2: E0499/E0502 borrow conflicts using the same event/report surfaces.
4. US3: Unsupported diagnostics retained as display-only records.
5. US4: Borrow Sheet HTML learning view.
6. Polish: schema validation, docs, examples, and full verification.

### Parallel Team Strategy

1. Complete Setup and Foundational tasks together.
2. After Foundational checkpoint, split by test files first: one developer writes mapper tests, another writes reporter snapshots, another writes CLI integration tests.
3. Implement US1 and US2 mappers in parallel because they use separate files.
4. Keep reporter file edits coordinated because json-reporter.ts and html-reporter.ts are shared surfaces.

---

## Notes

- Tests are included because the specification requires independent tests, parser unit tests, mapper tests, reporter snapshots, schema validation, and compatibility fixture tests.
- Unsupported diagnostics must remain display-only and must not receive Phase 1 ownership event kinds.
- Core line and column values stay 1-based; future VS Code adapter conversion is out of scope.
- Do not run clippy, rust-analyzer, or automatic fix application for Phase 1.