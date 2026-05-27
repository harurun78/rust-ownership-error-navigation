---
description: "Run one behavior-only Rust-native C/C++ to Rust porting validation iteration with saved diagnostics and navigation reports."
agent: porting-rust-native
---

# Behavior-Only Rust-Native Porting Iteration

Run one validation iteration for the behavior-only Rust-native track.

## Inputs To Provide

- Target directory, for example `validation/ports/<target>`.
- Track crate directory, normally `validation/ports/<target>/tracks/rust-native/rust-port`.
- Iteration id, for example `iteration-001`.
- Task slice.
- Model identity used for the attempt.
- Whether deterministic Rust-native design suggestions are allowed up front.

## Required Steps

1. Read target `spec.md`, `plan.md`, `tasks.md`, `quickstart.md`, and `notes/comparison-matrix.md`.
2. Read only upstream excerpts needed to understand behavior.
3. Record iteration metadata in `notes/iteration-log.md` before implementation.
4. Implement the smallest behavior-preserving Rust slice using Rust-native API design.
5. Run from the track crate:

   ```bash
   mkdir -p ../../reports/rust-native/<iteration-id>
   cargo check --message-format=json > ../../reports/rust-native/<iteration-id>/cargo-check.jsonl
   ```

   Adjust `../` depth if the crate layout differs, but keep the report path under `reports/rust-native/<iteration-id>/`.

6. From the repository root, generate navigation reports:

   ```bash
   npm run build
   node dist/cli/main.js \
     --input validation/ports/<target>/reports/rust-native/<iteration-id>/cargo-check.jsonl \
     --json-out validation/ports/<target>/reports/rust-native/<iteration-id>/ownership-report.json \
     --html-out validation/ports/<target>/reports/rust-native/<iteration-id>/ownership-report.html \
     --audience intermediate
   ```

7. If `cargo check` succeeds, run `cargo test` and record the result.
8. Update `notes/iteration-log.md` and `notes/comparison-matrix.md` with diagnostics, Rust-native patterns used, shortcut pressure, and next action.

## Measurement Rules

- Preserve behavior, not C API shape.
- Prefer owned parse results, builders, iterators, typestate/state-machine APIs, and short borrow scopes.
- If clean compilation occurs, record which Rust-native design choices likely prevented ownership diagnostics.
- If diagnostics occur, compare local fixes with redesign suggestions and record which direction is clearer.