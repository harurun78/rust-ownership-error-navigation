# Rust Ownership Error Navigation Constitution

## Core Principles

### I. Diagnostic Payload First

The project must treat `cargo check --message-format json` and rustc diagnostics as the source of truth. The tool may reinterpret spans for navigation, but it must not reimplement borrow checking or claim compiler-internal facts that are not evidenced by the diagnostic payload.

### II. Phase 1 Scope Discipline

Phase 1 mapping is limited to E0382, E0499, and E0502. Other captured diagnostics are compatibility corpus inputs and must be parsed and displayed without crashing, but they are not Phase 1 ownership-event mapping requirements.

### III. Evidence-Backed Events

Every derived `OwnershipEvent` must retain evidence and confidence. Mapping rules based on diagnostic code, primary spans, labels, child diagnostics, suggestions, or heuristics must be visible in output artifacts.

### IV. Reporter Independence

Parser, diagnostic model, event model, JSON reporter, and HTML reporter must remain separable. HTML layout and balance-sheet presentation must not leak into the shared event model used by future VS Code integration.

### V. Static, Local, Non-Mutating Operation

Phase 1 must operate on local JSONL input files and produce JSON / static HTML output. It must not modify the user Rust project, apply compiler suggestions, require rust-analyzer, require clippy, or require network access.

## Project Constraints

- Initial implementation target: TypeScript / Node.js CLI, package manager `npm` unless changed by plan review.
- Input command family: `cargo check --message-format json`.
- Phase 1 supported mapping targets: E0382 / E0499 / E0502.
- Compatibility corpus: follow-up ownership, advanced async/desugaring, and non-ownership compiler smoke JSONL fixtures.
- Phase 1 output artifacts: event JSON and static HTML report.
- Current repository stores both specifications and Phase 1 implementation code. Implementation tasks may target root-level `src/`, `test/`, `examples/`, and package configuration files in this repository.

## Development Workflow

- Start from fixture-backed specifications before implementation.
- Treat fixture JSONL as compatibility evidence, not as the only source of normative behavior.
- Use snapshot-style tests for parser and reporter behavior.
- Use expected event JSON for mapper acceptance on Phase 1 diagnostics.
- Keep unsupported diagnostics as display-only records instead of dropping them.

## Governance

This constitution governs speckit specs for the Rust Ownership Error Navigation project. Changes that expand Phase 1 mapping scope, introduce new input command families, or couple reporter-specific views into the event model require an explicit specification update and rationale.

**Version**: 0.1.1 | **Ratified**: 2026-05-25 | **Last Amended**: 2026-05-25
