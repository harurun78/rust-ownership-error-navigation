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

## Phase 14: Packed Bit-Depth Decode Slice

- [x] L068 Add tests for decoding 1-bit and 4-bit grayscale PNG images. (iteration-011)
- [x] L069 Add tests for decoding a 2-bit indexed-color PNG image. (iteration-011)
- [x] L070 Implement packed sample expansion for bit depths 1, 2, and 4. (iteration-011)
- [x] L071 Record iteration-011 diagnostics and ownership report. (iteration-011)

## Phase 15: 16-bit Alpha Decode Slice

- [x] L072 Add tests for decoding a tiny non-interlaced 16-bit grayscale-alpha PNG image. (iteration-012)
- [x] L073 Add tests for decoding a tiny non-interlaced 16-bit truecolor-alpha PNG image. (iteration-012)
- [x] L074 Verify 16-bit alpha color types preserve big-endian channel bytes through row reconstruction. (iteration-012)
- [x] L075 Record iteration-012 diagnostics and ownership report. (iteration-012)

## Phase 16: PLTE Structure Validation Slice

- [x] L076 Add tests for indexed-color PNG requiring PLTE before IDAT. (iteration-013)
- [x] L077 Add tests for duplicate PLTE and PLTE after IDAT errors. (iteration-013)
- [x] L078 Implement PLTE ordering/cardinality validation in stream structure checks. (iteration-013)
- [x] L079 Record iteration-013 diagnostics and ownership report. (iteration-013)

## Phase 17: Adam7 Interlace Completion Slice

- [x] L080 Add tests for decoding a tiny Adam7 interlaced grayscale PNG image. (iteration-014)
- [x] L081 Implement Adam7 pass reconstruction for byte-aligned decoded samples. (iteration-014)
- [x] L082 Record iteration-014 diagnostics and ownership report. (iteration-014)
- [x] L083 Write final validation-port completion assessment. (iteration-014)

## Phase 18: Metadata Chunk Inspection Slice

- [x] L084 Add tests for parsing gAMA, sRGB, pHYs, tIME, and tEXt chunks. (iteration-015)
- [x] L085 Implement metadata extraction over parsed chunk records. (iteration-015)
- [x] L086 Add deterministic errors for malformed metadata payloads. (iteration-015)
- [x] L087 Record iteration-015 diagnostics and ownership report. (iteration-015)

## Phase 19: Basic PNG Write Slice

- [x] L088 Add tests for encoding simple grayscale and truecolor PNG images. (iteration-016)
- [x] L089 Implement IHDR/IDAT/IEND writing with CRC and zlib compression for non-interlaced images. (iteration-016)
- [x] L090 Add encode/decode round-trip coverage. (iteration-016)
- [x] L091 Record iteration-016 diagnostics and ownership report. (iteration-016)

## Phase 20: Document Decode and Ancillary Preservation Slice

- [x] L092 Add tests for preserving unknown ancillary chunk payloads. (iteration-017)
- [x] L093 Implement document-level decode that returns image data, metadata, and unknown ancillary chunks. (iteration-017)
- [x] L094 Record iteration-017 diagnostics and ownership report. (iteration-017)

## Phase 21: Rich Metadata Chunk Slice

- [x] L095 Add tests for cHRM, zTXt, iTXt, and iCCP metadata chunks. (iteration-018)
- [x] L096 Implement owned parsing for chromaticities, compressed text, international text, and ICC profile payloads. (iteration-018)
- [x] L097 Add deterministic errors for unsupported metadata compression methods and malformed rich metadata payloads. (iteration-018)
- [x] L098 Record iteration-018 diagnostics and ownership report. (iteration-018)

## Phase 22: Document Write and Copy Policy Slice

- [x] L099 Add tests for writing document metadata and unknown ancillary chunks. (iteration-019)
- [x] L100 Implement document-level encoder that writes metadata, safe ancillary chunks, IDAT, and IEND. (iteration-019)
- [x] L101 Add encode/decode document round-trip coverage. (iteration-019)
- [x] L102 Record iteration-019 diagnostics and ownership report. (iteration-019)

## Phase 23: Indexed Palette Write Slice

- [x] L103 Add tests for encoding 8-bit indexed PNG images with PLTE. (iteration-020)
- [x] L104 Add tests for optional indexed tRNS alpha output. (iteration-020)
- [x] L105 Implement indexed PNG writer with palette/index validation. (iteration-020)
- [x] L106 Record iteration-020 diagnostics and ownership report. (iteration-020)

## Phase 24: Packed Indexed Write Slice

- [x] L107 Add tests for encoding packed 1/2/4-bit indexed PNG rows. (iteration-021)
- [x] L108 Implement packed palette index row emission and palette cardinality validation. (iteration-021)
- [x] L109 Record iteration-021 diagnostics and ownership report. (iteration-021)

## Phase 25: Writer Filter Strategy Slice

- [x] L110 Add tests for explicit Sub/Up/Average/Paeth writer filter strategies. (iteration-022)
- [x] L111 Implement filter-strategy IDAT emission for non-interlaced image writing. (iteration-022)
- [x] L112 Record iteration-022 diagnostics and ownership report. (iteration-022)

## Phase 26: Row Callback API Slice

- [x] L113 Add tests for row callback style decode. (iteration-023)
- [x] L114 Implement row callback API over decoded image rows. (iteration-023)
- [x] L115 Record iteration-023 diagnostics and ownership report. (iteration-023)

## Phase 27: Adaptive Writer Filter Slice

- [x] L116 Add tests for adaptive writer filter selection. (iteration-024)
- [x] L117 Implement row-local adaptive filter heuristic over None/Sub/Up/Average/Paeth. (iteration-024)
- [x] L118 Record iteration-024 diagnostics and ownership report. (iteration-024)

## Phase 28: Adam7 Interlaced Writer Slice

- [x] L119 Add tests for Adam7 interlaced image output. (iteration-025)
- [x] L120 Implement byte-aligned Adam7 writer pass emission. (iteration-025)
- [x] L121 Record iteration-025 diagnostics and ownership report. (iteration-025)

## Phase 29: libpng Compatibility Facade Slice

- [x] L122 Add tests for libpng-style read lifecycle: create reader, set input, read info, read image rows, destroy reader. (iteration-026)
- [x] L123 Add tests for libpng-style write lifecycle: create writer, write image/document/indexed output, retrieve output, destroy writer. (iteration-026)
- [x] L124 Implement a Rust-native compatibility facade that mirrors libpng read/write lifecycle concepts without C ABI exposure. (iteration-026)
- [x] L125 Record iteration-026 diagnostics and ownership report. (iteration-026)

## Phase 30: Compatibility Transform And Copy Policy Slice

- [x] L126 Add tests for libpng-style read transforms: strip 16-bit samples, expand low-bit grayscale info to 8-bit, palette-to-RGB, and tRNS-to-alpha. (iteration-027)
- [x] L127 Add a Rust-native warning callback hook for compatibility warnings. (iteration-027)
- [x] L128 Add writer unknown ancillary copy policy controls for safe-only, all ancillary, and none. (iteration-027)
- [x] L129 Implement compatibility warnings for transform application and unsafe ancillary copy policy. (iteration-027)
- [x] L130 Record iteration-027 diagnostics and ownership report. (iteration-027)
