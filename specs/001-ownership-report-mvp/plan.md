# Implementation Plan: Rust Ownership Diagnostic Report MVP

**Branch**: `001-ownership-report-mvp` | **Date**: 2026-05-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-ownership-report-mvp/spec.md`

**Note**: This plan targets the current repository as both the specification repository and the Phase 1 implementation repository.

## Summary

Build a TypeScript / Node.js CLI that reads Cargo JSONL diagnostics, maps Phase 1 Rust ownership errors E0382 / E0499 / E0502 into evidence-backed ownership events, and emits both JSON and static HTML reports. Future diagnostics are included as compatibility corpus and must be parsed/displayed without requiring Phase 1 event mapping.

## Technical Context

**Language/Version**: TypeScript on Node.js 20 LTS with npm.

**Primary Dependencies**: Minimal Node CLI stack. JSONL parsing can use standard Node streams; HTML rendering should start with local template/string rendering before adding dependencies.

**Storage**: Local files only. Input JSONL, output JSON, and output HTML.

**Testing**: Unit tests for parser/mapper, snapshot tests for JSON and HTML reporter, fixture compatibility tests.

**Target Platform**: Local developer machines on Windows/macOS/Linux.

**Project Type**: CLI plus reusable parser/mapper library modules.

**Performance Goals**: Process fixture-scale JSONL instantly for human use; no Phase 1 hard performance target beyond streaming-friendly design.

**Constraints**: Must not modify Rust source projects, must not require rust-analyzer, must not require clippy, must not apply suggestions.

**Scale/Scope**: Phase 1 supports E0382 / E0499 / E0502 mapping and display-only fallback for captured compatibility fixtures.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle                             | Check                                                          | Status |
| ------------------------------------- | -------------------------------------------------------------- | ------ |
| Diagnostic Payload First              | Uses cargo/rustc JSON only; no borrow checker reimplementation | PASS   |
| Phase 1 Scope Discipline              | Mapping limited to E0382 / E0499 / E0502                       | PASS   |
| Evidence-Backed Events                | Event schema requires evidence and confidence                  | PASS   |
| Reporter Independence                 | Parser/model/mapper/reporter separated                         | PASS   |
| Static, Local, Non-Mutating Operation | Local JSONL input and static outputs only                      | PASS   |

## Project Structure

### Documentation (this feature)

```text
specs/001-ownership-report-mvp/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    └── diagnostic-report.schema.json
```

### Source Code (this repository)

```text
src/
├── cli/
│   └── main.ts
├── parser/
│   └── cargo-message-parser.ts
├── diagnostics/
│   ├── rustc-diagnostic.ts
│   └── diagnostic-span.ts
├── mapper/
│   ├── ownership-event.ts
│   ├── e0382.ts
│   ├── e0499.ts
│   └── e0502.ts
└── reporter/
    ├── json-reporter.ts
    └── html-reporter.ts

test/
├── fixtures/
├── parser/
├── mapper/
└── reporter/
```

**Structure Decision**: Keep implementation modular so that future VS Code extension can reuse parser, diagnostic model, and event model without depending on HTML reporter.

## Phase Plan

## Cargo JSONL Parser

### スコープ

Read local JSONL files and extract `reason = compiler-message` diagnostics.

### 受け入れ条件

- [ ] Valid JSONL fixture lines are parsed into CargoMessage records.
- [ ] Non-compiler-message records are skipped or retained as ignored metadata without causing failure.
- [ ] Malformed lines produce a clear parser error that includes line number.

### スコープ外

- Running `cargo check` directly.
- Watching Rust projects.

### 検証方法

- Parser unit tests with baseline and compatibility fixture JSONL.

## Rustc Diagnostic And Span Model

### スコープ

Normalize rustc diagnostics and spans into L0 model while preserving source location and evidence.

### 受け入れ条件

- [ ] `code`, `level`, `message`, `spans`, `children`, and `rendered` are preserved.
- [ ] `line_start` / `column_start` remain 1-based in the core model.
- [ ] span `expansion` is represented as `hasExpansion` plus raw metadata when needed.

### スコープ外

- VS Code `Range` conversion implementation.

### 検証方法

- Snapshot tests for normalized diagnostics from fixture corpus.

## Ownership Event Mapper For E0382/E0499/E0502

### スコープ

Map Phase 1 diagnostics into L1 ownership events using diagnostic code, span labels, primary flags, children, and suggestions.

### 受け入れ条件

- [ ] E0382 maps move cause and use conflict events.
- [ ] E0499 maps first/second mutable borrow and later use events.
- [ ] E0502 maps immutable borrow cause, mutable borrow conflict, and later immutable use context.
- [ ] Every event includes evidence and confidence.

### スコープ外

- Full mapping for E0505 / E0506 / E0507 / E0596 / E0597 / E0716.
- Async, macro, trait-bound, and type-inference event mapping.

### 検証方法

- Expected event JSON snapshots for baseline fixture.

## Unsupported Diagnostic Fallback

### スコープ

Preserve non-Phase-1 diagnostics as display-only records.

### 受け入れ条件

- [ ] Follow-up ownership fixture parses without mapper failure.
- [ ] Advanced async/desugaring fixture parses without mapper failure.
- [ ] Non-ownership compiler smoke fixture parses without mapper failure.
- [ ] Unsupported diagnostics appear in JSON and HTML summary.

### スコープ外

- Assigning ownership event kind to unsupported diagnostics.

### 検証方法

- Compatibility tests that assert diagnostics are retained.

## JSON Reporter

### スコープ

Emit a machine-readable report containing supported events and unsupported diagnostics.

### 受け入れ条件

- [ ] Output conforms to [contracts/diagnostic-report.schema.json](contracts/diagnostic-report.schema.json).
- [ ] Report includes input summary and diagnostic counts.
- [ ] Supported diagnostics contain mapped events.
- [ ] Unsupported diagnostics preserve code, message, spans, children, and rendered output when available.

### スコープ外

- Stable public API versioning beyond `schemaVersion = 0.1.0`.

### 検証方法

- JSON schema validation and snapshot tests.

## Static HTML Reporter

### スコープ

Generate a static HTML report for supported and unsupported diagnostics.

### 受け入れ条件

- [ ] HTML includes Summary, Causality Timeline, Source Spans, Evidence, and Borrow Sheet sections for supported diagnostics.
- [ ] Unsupported diagnostics are shown in a display-only section.
- [ ] HTML can be opened as a local static file.

### スコープ外

- Client-side filtering, collapsible UI, or interactive navigation.

### 検証方法

- HTML snapshot tests for required section headings and representative rows.

## Complexity Tracking

No constitution violations are currently required.
