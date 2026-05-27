---
description: "Run one compatibility-preserving C/C++ to Rust porting validation iteration with saved diagnostics and navigation reports."
agent: porting-compatibility
---

# Compatibility-Preserving Porting Iteration

Run one validation iteration for the compatibility-preserving track.

## Inputs To Provide

- Target directory, for example `validation/ports/<target>`.
- Track crate directory, normally `validation/ports/<target>/tracks/compatibility/rust-port`.
- Iteration id, for example `iteration-001`.
- Task slice.
- Model identity used for the attempt.
- Whether navigation report guidance from a previous iteration is allowed.

## Required Steps

1. Read target `spec.md`, `plan.md`, `tasks.md`, `quickstart.md`, and `notes/comparison-matrix.md`.
2. Read only upstream excerpts needed for the requested compatibility surface.
3. Record iteration metadata in `notes/iteration-log.md` before implementation.
4. Implement the smallest compatibility-preserving Rust slice.
5. Run from the track crate:

   ```bash
   mkdir -p ../../reports/compatibility/<iteration-id>
   cargo check --message-format=json > ../../reports/compatibility/<iteration-id>/cargo-check.jsonl
   ```

   Adjust `../` depth if the crate layout differs, but keep the report path under `reports/compatibility/<iteration-id>/`.

6. From the repository root, generate navigation reports:

   ```bash
   npm run build
   node dist/cli/main.js \
     --input validation/ports/<target>/reports/compatibility/<iteration-id>/cargo-check.jsonl \
     --json-out validation/ports/<target>/reports/compatibility/<iteration-id>/ownership-report.json \
     --html-out validation/ports/<target>/reports/compatibility/<iteration-id>/ownership-report.html \
     --audience intermediate
   ```

7. If `cargo check` succeeds, run `cargo test` and record the result.
8. Update `notes/iteration-log.md` and `notes/comparison-matrix.md` with diagnostics, shortcut pressure, compatibility pressure, and next action.

## Measurement Rules

- Preserve C API/ABI shape even if it increases ownership pressure.
- Do not hide `unsafe` or shared-mutability pressure; record it.
- Do not switch to a Rust-native API to make the slice pass.
- If the report suggests a Rust-native redesign, record it as prevention value but do not apply it in this track unless the task explicitly allows divergence.