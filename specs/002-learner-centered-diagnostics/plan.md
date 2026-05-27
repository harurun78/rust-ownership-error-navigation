# Implementation Plan: Learner-Centered Diagnostic Navigation

**Branch**: `002-learner-centered-diagnostics` | **Date**: 2026-05-27 | **Spec**: [spec.md](spec.md)

## Summary

Extend the implemented MVP report from ownership event extraction into a learner-centered diagnostic navigation tool. The first post-MVP slice adds learner summary cards, audience modes, high-frequency non-ownership diagnostic navigation, recommended first-fix ordering, and fix strategy trade-offs.

## Technical Context

**Language/Version**: TypeScript on Node.js 20 LTS with npm.

**Primary Dependencies**: Existing project dependencies. Avoid new runtime dependencies unless the implementation requires schema validation or rendering support that cannot be handled locally.

**Storage**: Local input JSONL and local output JSON/HTML. Future cargo wrapper output remains local files.

**Testing**: Unit tests for summary generation, audience mode behavior, non-ownership mappers, priority ordering, reporter snapshots, and integration tests using fixtures.

**Target Platform**: Local developer machines on Windows/macOS/Linux.

**Project Type**: CLI plus reusable parser/mapper/reporter library modules.

**Constraints**: Preserve non-mutating behavior. Do not apply fixes to Rust source. Preserve unsupported fallback behavior.

## Constitution Check

| Principle                             | Check                                                                 | Status |
| ------------------------------------- | --------------------------------------------------------------------- | ------ |
| Diagnostic Payload First              | Uses cargo/rustc JSON and captured validation fixtures                | PASS   |
| Phase Discipline                      | Post-MVP scope is limited to learner summaries and prioritized codes  | PASS   |
| Evidence-Backed Events                | New summaries must link back to existing evidence and spans           | PASS   |
| Reporter Independence                 | Summary, mapper, and reporter code remain separable                   | PASS   |
| Static, Local, Non-Mutating Operation | Outputs remain local JSON/HTML; no source modification                | PASS   |

## Project Structure

```text
src/
├── cli/
│   └── main.ts
├── diagnostics/
│   ├── diagnostic-span.ts
│   ├── normalizer.ts
│   └── rustc-diagnostic.ts
├── mapper/
│   ├── diagnostic-navigation.ts       # new generalized records if needed
│   ├── e0004.ts                       # new
│   ├── e0308.ts                       # new
│   ├── e0382.ts
│   ├── e0425.ts                       # new
│   ├── e0499.ts
│   ├── e0502.ts
│   ├── fix-strategy.ts                # new
│   ├── learner-summary.ts             # new
│   └── index.ts
└── reporter/
    ├── html-reporter.ts
    ├── json-reporter.ts
    └── reporter-utils.ts

test/
├── fixtures/diagnostics/porting/       # selected validation corpus fixtures
├── mapper/
├── reporter/
└── integration/
```

## Phase Plan

### Phase 1: Learner Summary Foundation

Add report model fields and pure summary generation helpers. Preserve existing event mapping and unsupported fallback.

### Phase 2: Audience Modes

Add CLI option parsing and report generation mode selection for beginner, intermediate, and agent output.

### Phase 3: Non-Ownership Diagnostic Navigation

Add mappers/records for E0308, E0004, and E0425 using Redis validation fixtures as realistic coverage.

### Phase 4: Multi-Diagnostic Prioritization

Add deterministic recommended first-fix ordering across ownership and non-ownership diagnostics.

### Phase 5: Fix Strategy Guidance

Add fix strategy taxonomy and trade-off notes for ownership diagnostics.

### Phase 6: Documentation And Verification

Update quickstart/docs, add fixtures, and run the full npm verification gate.

## Complexity Tracking

No new architectural violation is expected. If the report schema changes require broader migration, keep schema versioning explicit and update contract tests in the same slice.
