# Quickstart: Rust Ownership Diagnostic Report MVP

This quickstart describes the intended implementation repository workflow. This spec repository does not contain implementation code.

## 1. Prepare fixture input

Use reference JSONL from the specnote idea corpus or regenerate it with Docker.

Reference files:

- `ideas/draft/rust-ownership-error-navigation/references/diagnostics/ownership-baseline-2026-05-24.jsonl`
- `ideas/draft/rust-ownership-error-navigation/references/diagnostics/ownership-followup-2026-05-25.jsonl`
- `ideas/draft/rust-ownership-error-navigation/references/diagnostics/ownership-advanced-2026-05-25.jsonl`
- `ideas/draft/rust-ownership-error-navigation/references/diagnostics/rustc-non-ownership-smoke-2026-05-25.jsonl`

## 2. Run CLI on Phase 1 fixture

```sh
rust-ownership-report \
  --input fixtures/ownership-baseline-2026-05-24.jsonl \
  --json-out out/ownership-report.json \
  --html-out out/ownership-report.html
```

Expected result:

- JSON report is created.
- Static HTML report is created.
- E0382 / E0499 / E0502 have ownership events.
- Unsupported count is zero for the baseline fixture.

## 3. Run compatibility fixture checks

```sh
rust-ownership-report \
  --input fixtures/ownership-followup-2026-05-25.jsonl \
  --json-out out/followup-report.json \
  --html-out out/followup-report.html
```

Expected result:

- CLI exits successfully.
- Unsupported diagnostics are preserved as display-only records.
- No Phase 1 mapper is required to classify every diagnostic.

## 4. Validate JSON contract

```sh
validate-json-schema \
  specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json \
  out/ownership-report.json
```

Tool choice is implementation-specific. The acceptance condition is contract conformance, not a specific validator.

## 5. Inspect HTML

Open `out/ownership-report.html` in a browser.

Required sections:

- Summary
- Causality Timeline
- Source Spans
- Evidence
- Borrow Sheet
- Unsupported Diagnostics, when applicable

## Phase 1 Non-Goals

- Do not run clippy.
- Do not call rust-analyzer.
- Do not apply suggestions to user source files.
- Do not implement VS Code extension UI.
