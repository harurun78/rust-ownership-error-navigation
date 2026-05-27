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

## Phase 7: CRC Validation Slice

- [x] L039 Add a documented CRC32 dependency decision for chunk verification. (iteration-005)
- [x] L040 Add tests for valid and mismatched PNG chunk CRC values. (iteration-005)
- [x] L041 Implement CRC validation over chunk type bytes plus payload bytes. (iteration-005)
- [x] L042 Record iteration-005 diagnostics and ownership report. (iteration-005)

## Phase 8: Non-Interlaced Image Decode Slice

- [x] L043 Add a documented zlib/deflate dependency decision for IDAT inflation. (iteration-006)
- [x] L044 Add tests for decoding a tiny non-interlaced grayscale PNG image. (iteration-006)
- [x] L045 Add tests for decoding a tiny non-interlaced truecolor PNG image. (iteration-006)
- [x] L046 Implement IDAT concatenation, zlib inflation, and PNG filter reconstruction for color types 0 and 2 at 8-bit depth. (iteration-006)
- [x] L047 Record iteration-006 diagnostics and ownership report. (iteration-006)

## Phase 9: Full-Port Boundary Reassessment

- [x] L048 Summarize remaining gaps versus full libpng parity and decide the next slice: palette/tRNS, Adam7 interlace, row streaming, or metadata chunks. (post-iteration-008)

## Phase 10: Alpha Channel Decode Slice

- [x] L049 Add tests for decoding a tiny non-interlaced grayscale-alpha PNG image. (iteration-007)
- [x] L050 Add tests for decoding a tiny non-interlaced truecolor-alpha PNG image. (iteration-007)
- [x] L051 Extend 8-bit decode support to color types 4 and 6. (iteration-007)
- [x] L052 Record iteration-007 diagnostics and ownership report. (iteration-007)

## Phase 11: Indexed Palette Decode Slice

- [x] L053 Add tests for PLTE chunk parsing. (iteration-008)
- [x] L054 Add tests for decoding a tiny 8-bit indexed-color PNG image. (iteration-008)
- [x] L055 Implement PLTE parsing and indexed-color expansion to RGB pixels. (iteration-008)
- [x] L056 Add deterministic errors for missing PLTE and invalid palette indices. (iteration-008)
- [x] L057 Record iteration-008 diagnostics and ownership report. (iteration-008)

## Phase 12: Transparency Chunk Decode Slice

- [x] L058 Add tests for grayscale tRNS transparency expansion. (iteration-009)
- [x] L059 Add tests for truecolor tRNS transparency expansion. (iteration-009)
- [x] L060 Add tests for indexed-color tRNS alpha expansion. (iteration-009)
- [x] L061 Implement tRNS parsing for grayscale, truecolor, and indexed-color images. (iteration-009)
- [x] L062 Add deterministic errors for invalid tRNS lengths and disallowed tRNS color types. (iteration-009)
- [x] L063 Record iteration-009 diagnostics and ownership report. (iteration-009)

## Phase 13: 16-bit Decode Slice

- [x] L064 Add tests for decoding a tiny non-interlaced 16-bit grayscale PNG image. (iteration-010)
- [x] L065 Add tests for decoding a tiny non-interlaced 16-bit truecolor PNG image. (iteration-010)
- [x] L066 Implement 16-bit row byte reconstruction for color types 0 and 2 while preserving big-endian sample bytes. (iteration-010)
- [x] L067 Record iteration-010 diagnostics and ownership report. (iteration-010)
