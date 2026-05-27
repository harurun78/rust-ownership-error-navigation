# Tasks: libpng Signature And Chunk Header Porting Validation

## Phase 1: Experiment Setup

- [x] L001 Create `validation/ports/libpng/` target directory.
- [x] L002 Fetch libpng upstream into ignored checkout `validation/ports/libpng/upstream/libpng/`.
- [x] L003 Record upstream repository, tag, commit, license, and acquisition commands in `upstream/UPSTREAM.md`.
- [x] L004 Create libpng porting validation `spec.md`.
- [x] L005 Create libpng porting validation `plan.md`.
- [x] L006 Create libpng porting validation `tasks.md`.
- [x] L007 Create libpng porting validation `quickstart.md`.
- [x] L008 Create `notes/iteration-log.md` template.
- [x] L009 Verify upstream checkout remains ignored by Git before first implementation iteration.

## Phase 2: Rust Crate Skeleton

- [x] L010 Initialize Rust library crate in `validation/ports/libpng/rust-port/`.
- [x] L011 Add crate README or module docs stating signature/chunk-header scope.
- [x] L012 Define PNG signature helper API.
- [x] L013 Define chunk type and chunk header model.
- [x] L014 Define streaming parser outcome and error model.

## Phase 3: Signature And Chunk Header Initial Slice

- [x] L015 Add tests for full and partial PNG signature comparison.
- [x] L016 Add tests for invalid PNG signature bytes.
- [x] L017 Add tests for chunk type property helpers and invalid reserved bit.
- [x] L018 Add tests for partial streaming input through signature and chunk header parse.
- [x] L019 Implement signature comparison.
- [x] L020 Implement chunk type and chunk header parsing.
- [x] L021 Implement streaming parser with internal buffer compaction.
- [x] L022 Save `cargo check --message-format=json` output to `reports/iteration-001/cargo-check.jsonl`.
- [x] L023 Generate `reports/iteration-001/ownership-report.json`.
- [x] L024 Generate `reports/iteration-001/ownership-report.html`.
- [x] L025 Record iteration-001 results and diagnostic counts in `notes/iteration-log.md`.
- [x] L026 Run final `cargo fmt -- --check` and `cargo test` in `rust-port/`.

## Phase 4: Next Candidate Slice

- [x] L027 Add chunk payload and CRC boundary tests.
- [x] L028 Implement full chunk extraction without borrowing from the parser buffer.
- [x] L029 Record iteration-002 diagnostics and ownership report.

## Phase 5: IHDR Validation Slice

- [x] L030 Add IHDR payload parsing tests for width, height, bit depth, color type, compression, filter, and interlace fields.
- [x] L031 Add IHDR validation tests for zero dimensions, invalid bit depth/color type combinations, compression/filter methods, and interlace method.
- [x] L032 Implement IHDR metadata parsing and validation.
- [x] L033 Record iteration-003 diagnostics and ownership report.

## Phase 6: Minimal PNG Stream Structure Completion

- [x] L034 Add tests for ordered PNG chunk structure: signature, IHDR first, IDAT before IEND, IEND final.
- [x] L035 Add tests for unknown ancillary chunks, invalid critical chunks, and trailing bytes after IEND.
- [x] L036 Implement a minimal PNG structure validator over owned chunk records.
- [x] L037 Record iteration-004 diagnostics and ownership report.
- [x] L038 Write final libpng validation summary including whether navigation app feature additions improved usefulness.
