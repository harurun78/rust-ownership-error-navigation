---
description: "Run one low-cost C/C++ to Rust porting validation iteration with saved rustc diagnostics and ownership navigation reports."
agent: porting-lowcost
---

# Low-Cost Porting Iteration

Run one validation iteration for the target named by the caller.

## Inputs To Provide

- Target directory, for example `validation/ports/cjson`
- Iteration id, for example `iteration-001`
- Task slice, for example `C006-C017 crate skeleton and scalar tests`
- Model identity used for the attempt
- Whether human ownership hints are allowed; default: no

## Required Steps

1. Read the target `spec.md`, `plan.md`, `tasks.md`, and `quickstart.md`.
2. Read only the upstream excerpts needed for the requested task slice.
3. Update `notes/iteration-log.md` with the iteration metadata before implementation.
4. Implement the smallest requested Rust slice.
5. Run from the target Rust crate:

   ```bash
   cargo check --message-format=json > ../reports/<iteration-id>/cargo-check.jsonl
   ```

6. From the repository root, generate navigation reports:

   ```bash
   npm run build
   node dist/cli/main.js \
     --input validation/ports/<target>/reports/<iteration-id>/cargo-check.jsonl \
     --json-out validation/ports/<target>/reports/<iteration-id>/ownership-report.json \
     --html-out validation/ports/<target>/reports/<iteration-id>/ownership-report.html
   ```

7. If `cargo check` succeeds, run `cargo test` and record the result.
8. Update `notes/iteration-log.md` with diagnostic counts, report paths, and next action.

## Measurement Rules

- Do not hide failed compiler output.
- Do not overwrite prior iteration artifacts.
- Do not manually fix ownership errors after report generation unless recording that as human intervention.
- Record any pressure to use `clone`, `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or `unsafe`.
