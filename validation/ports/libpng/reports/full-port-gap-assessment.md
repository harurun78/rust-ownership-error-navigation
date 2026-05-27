# libpng Full-Port Gap Assessment

Date: 2026-05-27

## Current Implementation Boundary

The validation port now covers a practical minimal PNG read path:

- PNG signature comparison and progressive parser state
- Chunk type/header parsing
- Owned chunk payload and CRC field extraction
- CRC32 validation over chunk type bytes plus payload
- IHDR parsing and validation
- PNG stream structure validation: IHDR first, IDAT before IEND, IEND final, unknown critical chunk rejection
- IDAT concatenation and zlib inflation
- PNG scanline filter reconstruction for filter types 0-4
- Non-interlaced 8-bit decode for color types 0, 2, 3, 4, and 6
- Packed 1/2/4-bit sample expansion for grayscale and indexed-color images
- Non-interlaced 16-bit decode for color types 0 and 2, preserving big-endian sample bytes
- Non-interlaced 16-bit decode for color types 4 and 6, preserving big-endian sample bytes
- PLTE parsing and indexed-color expansion to RGB pixels
- PLTE ordering and cardinality validation for indexed-color conformance
- tRNS transparency expansion for grayscale, truecolor, and indexed-color images
- Adam7 pass reconstruction for byte-aligned decoded samples
- Metadata extraction for gAMA, sRGB, pHYs, tIME, and tEXt chunks
- Rich metadata extraction for cHRM, zTXt, iTXt, and iCCP chunks
- Basic PNG writing for non-interlaced grayscale, truecolor, grayscale-alpha, and truecolor-alpha images at 8/16-bit depths
- Indexed PNG writing with PLTE and optional tRNS alpha for 8-bit palette indices
- Packed indexed PNG writing for 1/2/4-bit palette indices
- Explicit writer filter strategy selection for None, Sub, Up, Average, and Paeth scanlines
- Adaptive writer filter heuristic over row-local filter scores
- Byte-aligned Adam7 interlaced output
- Callback-style row decode over decoded image rows
- Document-level decode returning image data, metadata, and unknown ancillary chunks as owned records
- Document-level writing for metadata chunks and safe-to-copy unknown ancillary chunks
- Rust-native libpng-style read/write lifecycle facade with explicit compatibility warnings

## Remaining Gaps Versus C libpng Compatibility

The Rust-native validation port now includes a libpng-style lifecycle facade. Remaining gaps are compatibility with the binary C library surface rather than missing Rust validation slices:

- Full color transform behavior from color-management metadata
- Full unknown ancillary copy policy with exact ordering and transform-aware safe/unsafe handling
- Error recovery and warning model closer to libpng
- C ABI, allocator hooks, setjmp/longjmp behavior, and full public API parity

## Proposed C Compatibility Track

If this project needs drop-in libpng compatibility, create a separate C-compatibility layer with these milestones:

1. Map libpng public read/write APIs to the Rust facade and identify unsupported calls explicitly.
2. Define allocator hook and error callback semantics without relying on hidden panics.
3. Decide whether to expose `extern "C"` symbols and where `unsafe` is acceptable.
4. Add C fixture tests that compile a small libpng-style caller against the compatibility layer.
5. Validate warning/error behavior against selected upstream libpng fixtures.

## Next Slice Decision

The implementation has now moved past the validation read-path boundary into a complete Rust-native validation target: rich metadata inspection, image/document writing, metadata emission, indexed writing, filter strategy selection, row callbacks, Adam7 output, ancillary preservation, and read/write lifecycle facade coverage are included.

Further ownership-navigation measurement should switch to a different validation target or an intentionally failed libpng branch. C ABI compatibility can continue, but should be tracked as a separate product track with explicit `unsafe` and ABI decisions.
