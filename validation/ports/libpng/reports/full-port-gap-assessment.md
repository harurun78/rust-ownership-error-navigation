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
- Basic PNG writing for non-interlaced grayscale, truecolor, grayscale-alpha, and truecolor-alpha images at 8/16-bit depths
- Document-level decode returning image data, metadata, and unknown ancillary chunks as owned records

## Remaining Gaps Versus Full libpng Parity

This is not a full libpng replacement. Major remaining gaps include:

- Progressive row callbacks and true streaming row decode
- Remaining color management chunks: cHRM, iCCP, and full color transform behavior
- Remaining text metadata chunks: zTXt and iTXt
- Full unknown ancillary copy policy during writing
- Error recovery and warning model closer to libpng
- Broader write/encode APIs, including indexed palette output, metadata emission, filtering choices, and interlaced output
- C ABI, allocator hooks, setjmp/longjmp behavior, and full public API parity

## Next Slice Decision

The implementation has now moved past the validation read-path boundary into selected full-parity gaps: metadata inspection, basic writing, and ancillary preservation. Remaining items are still broad libpng API parity work rather than ownership-navigation validation blockers.

Further work should switch to a different validation target or an intentionally failed libpng branch if the goal is measuring diagnostic navigation effectiveness.
