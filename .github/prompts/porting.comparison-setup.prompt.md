---
description: "Set up a reusable A/B porting validation target comparing compatibility-preserving and behavior-only Rust-native conditions."
---

# Porting Comparison Setup

Set up a validation target that compares two C/C++ to Rust porting conditions:

1. **compatibility-preserving**: keep C API/ABI shape and lifecycle concepts.
2. **rust-native**: preserve only behavior and redesign APIs idiomatically.

## Inputs To Provide

- Target name, for example `libpng-abi-benchmark`.
- Upstream repository/tag/commit/license.
- Source excerpts or files to use as reference.
- Behavior fixtures or observable requirements.
- First paired slice to implement.

## Required Steps

1. Create or update `validation/ports/<target>/` with:
   - `spec.md`
   - `plan.md`
   - `tasks.md`
   - `quickstart.md`
   - `notes/iteration-log.md`
   - `notes/comparison-matrix.md`
   - `tracks/compatibility/rust-port/`
   - `tracks/rust-native/rust-port/`
   - `reports/comparison-summary.md`
2. Define shared behavior requirements that both tracks must satisfy.
3. Define compatibility-only constraints: API names, call ordering, handles, callbacks, allocator/error hooks, and ABI notes.
4. Define Rust-native allowances: owned results, `Result`, builders, iterators, typestate/state-machine APIs, and short borrow scopes.
5. Define metrics:
   - first cargo-check diagnostics
   - ownership diagnostics
   - non-ownership diagnostics
   - repair iterations to pass tests
   - shortcut pressure events
   - navigation changed next action
6. Do not implement the Rust slice unless explicitly requested after setup.

## Output

Return the created files, the first paired task slice, and the exact commands/prompts to run both tracks.