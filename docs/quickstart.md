# Quickstart: Rust Ownership Diagnostic Report MVP

This quickstart describes the workflow for this repository, which stores both the feature specification and the Phase 1 TypeScript implementation.

## 0. Install and build

```sh
npm ci
npm run build
```

## 1. Prepare fixture input

Use the reference JSONL fixtures committed in this repository or regenerate them with Docker.

Reference files:

- `test/fixtures/diagnostics/ownership-baseline-2026-05-24.jsonl`
- `test/fixtures/diagnostics/ownership-followup-2026-05-25.jsonl`
- `test/fixtures/diagnostics/ownership-advanced-2026-05-25.jsonl`
- `test/fixtures/diagnostics/rustc-non-ownership-smoke-2026-05-25.jsonl`

## 2. Run CLI on Phase 1 fixture

```sh
node dist/cli/main.js \
  --input test/fixtures/diagnostics/ownership-baseline-2026-05-24.jsonl \
  --json-out out/ownership-report.json \
  --html-out out/ownership-report.html
```

Audience mode can be selected explicitly:

```sh
node dist/cli/main.js \
  --input test/fixtures/diagnostics/ownership-baseline-2026-05-24.jsonl \
  --json-out out/ownership-report.intermediate.json \
  --html-out out/ownership-report.intermediate.html \
  --audience intermediate
```

Supported audience modes are `beginner`, `intermediate`, and `agent`. The default is `beginner`. The mode changes learner summary wording while preserving diagnostic evidence, spans, and ownership events.

Expected result:

- JSON report is created.
- Static HTML report is created.
- E0382 / E0499 / E0502 have ownership events.
- Supported diagnostics include learner summaries.
- Supported diagnostics may include deterministic design suggestions.
- Unsupported count is 2 for the baseline fixture because rustc failure-note diagnostics are preserved display-only.

## 3. Run compatibility fixture checks

```sh
node dist/cli/main.js \
  --input test/fixtures/diagnostics/ownership-followup-2026-05-25.jsonl \
  --json-out out/followup-report.json \
  --html-out out/followup-report.html
```

Expected result:

- CLI exits successfully.
- Unsupported diagnostics are preserved as display-only records.
- No Phase 1 mapper is required to classify every diagnostic.

## 4. Validate JSON contract

```sh
npm run test:run -- test/reporter/diagnostic-report-schema.test.ts
```

The project uses Ajv in tests to validate the generated report shape against `specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json`.

## 4.5 Run implementation verification

```sh
npm run lint
npm run format:check
npm run type-check
npm run test:run
npm run test:integration
npm run build
```

## 5. Inspect HTML

Open `out/ownership-report.html` in a browser.

Required sections:

- Summary
- Learner Summaries
- Design Direction
- Causality Timeline
- Source Spans
- Evidence
- Borrow Sheet
- Unsupported Diagnostics, when applicable

## Design Suggestions

Reports can include optional `designSuggestions` on supported diagnostics and a matching `Design Direction` HTML section. These suggestions are deterministic local rules over rustc diagnostics, spans, labels, snippets, events, and audience mode.

The first implemented slice includes:

- `split-mutation-phase` for E0499/E0502 records with cause, conflict, and context evidence.
- `avoid-long-lived-buffer-borrow` for E0499/E0502 records involving parser, stream, input, output, or buffer pressure.
- `owned-result` for E0382 moved-value reuse and E0308 type-boundary pressure.

The CLI does not call an external LLM API and does not apply source edits.

## Unsupported Diagnostics

Phase 1 maps only E0382, E0499, and E0502. Other rustc diagnostics, warnings, failure notes, and null-code diagnostics remain in the JSON and HTML reports with `supported: false`, empty `events`, and an `unsupportedReason`.

This display-only fallback intentionally does not infer ownership events for unsupported diagnostics. It keeps compatibility fixtures inspectable without widening the Phase 1 mapping scope.

## Position Semantics

Core report positions preserve rustc's 1-based `line_start`, `line_end`, `column_start`, and `column_end` values as `lineStart`, `lineEnd`, `columnStart`, and `columnEnd`.

Future VS Code adapters should convert to VS Code's 0-based `Range` model at the adapter boundary, not in the parser, mapper, JSON reporter, or HTML reporter.

## Phase 1 Non-Goals

- Do not run clippy.
- Do not call rust-analyzer.
- Do not apply suggestions to user source files.
- Do not implement VS Code extension UI.

## Post-MVP Direction

The Redis and miniz porting validations showed that learner value should expand beyond ownership-only reports. The implemented post-MVP slices now include learner summaries, beginner/intermediate/agent display modes, E0308/E0004/E0425 triage, fix strategy trade-offs, and deterministic design suggestions.

See [application-roadmap-from-redis-validation.md](../tasks/application-roadmap-from-redis-validation.md) for the prioritized task list.
