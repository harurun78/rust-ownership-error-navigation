# Feature Specification: Deterministic Design Suggestions

**Feature Branch**: `003-deterministic-design-suggestions`

**Created**: 2026-05-28

**Status**: Implementation started

**Input**: Porting validation showed that the tool has two kinds of value: local repair guidance after an ownership error appears, and prevention guidance that nudges C-style Rust ports toward Rust-native ownership boundaries.

## User Scenarios & Testing

### User Story 1 - Rust-native prevention direction appears in reports (Priority: P1)

Rust intermediate learnerとして、E0499/E0502 などの borrow conflict を見たとき、局所修正だけでなく、長い borrow を避ける設計方向も確認したい。

**Independent Test**: E0502 fixture report includes a deterministic `avoid-long-lived-buffer-borrow` or `split-mutation-phase` suggestion with evidence and confidence.

### User Story 2 - Agent can consume structured suggestion hints (Priority: P1)

Porting agentとして、JSON report から suggestion kind、confidence、evidence を機械的に読み取り、次 iteration の設計判断に使いたい。

**Independent Test**: JSON output includes `designSuggestions` on supported diagnostics without changing existing events, spans, learner summaries, or fix strategies.

### User Story 3 - HTML report shows design direction without applying fixes (Priority: P2)

Rust 学習者として、HTML report の `Design Direction` section で「いつ使うか」「注意点」を読めるが、ソースは自動変更されないことを確認したい。

**Independent Test**: HTML report renders a `Design Direction` section after fix strategies and before detailed causality sections.

## Requirements

- **FR-001**: System MUST add optional `designSuggestions` to diagnostic records without removing existing report fields.
- **FR-002**: System MUST derive suggestions deterministically from local diagnostic data, spans, labels, snippets, events, and audience mode.
- **FR-003**: System MUST NOT call an external LLM API or generate source patches.
- **FR-004**: System MUST implement at least `avoid-long-lived-buffer-borrow`, `split-mutation-phase`, and `owned-result` suggestion kinds.
- **FR-005**: System MUST include evidence and confidence on every suggestion.
- **FR-006**: System MUST render suggestions in JSON and static HTML.
- **FR-007**: System MUST preserve existing unsupported fallback behavior.

## Scope Boundaries

### In Scope

- Pure rule-based suggestion derivation.
- JSON/HTML report rendering.
- Mapper/reporter tests and documentation updates.

### Out of Scope

- Automatic code modification.
- rust-analyzer, MIR, clippy, or semantic whole-program analysis.
- External LLM API calls.

## Success Criteria

- **SC-001**: E0502/E0499 reports include conservative design suggestions when span evidence indicates borrow conflict pressure.
- **SC-002**: E0382/E0308 reports can include `owned-result` when local evidence suggests moved-value reuse or C-style type boundary pressure.
- **SC-003**: Existing unit, integration, lint, type-check, and build gates pass.