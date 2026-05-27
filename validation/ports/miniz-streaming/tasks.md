# Tasks: miniz Streaming Porting Comparison

## Phase 1: Setup

- [x] M001 Create `validation/ports/miniz-streaming/` target structure.
- [x] M002 Record upstream miniz repository, tag, commit, license, and acquisition command.
- [x] M003 Define comparison spec, plan, quickstart, iteration log, comparison matrix, and summary files.
- [x] M004 Create compatibility and Rust-native Rust crate skeletons.

## Phase 2: Paired Slice 001 - Stream Lifecycle Pass-Through

- [x] M005 Implement compatibility-preserving stream lifecycle API with caller-provided buffers.
- [x] M006 Implement behavior-only Rust-native pass-through inflator API.
- [x] M007 Add lifecycle and partial-output tests for both tracks.
- [x] M008 Save cargo diagnostics and navigation reports for both tracks.
- [x] M009 Update iteration log and comparison matrix with iteration-001 results.

## Phase 3: Next Slice Selection

- [x] M010 Select the next paired slice based on ownership pressure: stored deflate block decode with incremental input/output pressure.
