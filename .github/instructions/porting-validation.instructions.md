---
applyTo: 'validation/ports/**'
---

# Porting Validation Instructions

Use these rules when working under `validation/ports/**`.

## Purpose

Porting validation exists to measure whether ownership-error navigation helps low-cost or lightweight AI models migrate C/C++ code to Rust.

## Required Workflow

1. Keep upstream source checkouts local and ignored unless the target explicitly chooses vendoring.
2. Track selected upstream repository, tag, commit, license, and acquisition commands in `upstream/UPSTREAM.md`.
3. Keep each target's executable plan in `spec.md`, `plan.md`, `tasks.md`, and `quickstart.md`.
4. Before each model-generated Rust implementation attempt, record the model, prompt summary, task slice, and human hints in `notes/iteration-log.md`.
5. For each failed Rust compile, save raw compiler JSONL under `reports/iteration-NNN/cargo-check.jsonl`.
6. Run this repository's CLI against the saved JSONL and save:
   - `reports/iteration-NNN/ownership-report.json`
   - `reports/iteration-NNN/ownership-report.html`
7. Feed the saved report summary back into the next lightweight-model attempt.
8. Record whether repeated E0382, E0499, and E0502 diagnostics decreased after report use.

## Lightweight Model Constraints

When delegating implementation to a low-cost model:

- Provide only the relevant target spec, tasks, and a small upstream excerpt.
- Ask for the smallest compile-checkable slice.
- Do not give manual Rust ownership fixes unless recording them as human intervention.
- Prefer tests-first tasks when possible.
- Treat use of `unsafe`, broad `clone`, `Rc<RefCell<_>>`, or `Arc<Mutex<_>>` as measurement events to log.

## Artifact Rules

- Do not commit upstream source snapshots from ignored checkout directories.
- Do not commit generated `target/` directories.
- Generated report artifacts may be committed only when they are part of a named validation iteration.
- Keep iteration artifacts reproducible from the saved JSONL.
