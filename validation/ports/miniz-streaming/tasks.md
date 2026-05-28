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

## Phase 4: Paired Slice 002 - zlib Stored Block Decode

- [x] M011 Implement zlib header validation, DEFLATE stored block decode, LEN/NLEN validation, and Adler-32 validation in the compatibility track.
- [x] M012 Implement the same stored-block behavior with owned outputs and deterministic errors in the Rust-native track.
- [x] M013 Add tests for valid stored blocks, multiple blocks, output/buffer pressure, invalid lifecycle, and checksum failure.
- [x] M014 Save cargo diagnostics and navigation reports for both tracks.
- [x] M015 Update iteration log, comparison matrix, and comparison summary with iteration-002 results.

## Phase 5: Completion

- [x] M016 Mark the miniz-streaming comparison target complete at the stored-block zlib decode boundary.
