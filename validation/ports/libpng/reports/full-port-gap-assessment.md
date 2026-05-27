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
- Non-interlaced 16-bit decode for color types 0 and 2, preserving big-endian sample bytes
- PLTE parsing and indexed-color expansion to RGB pixels
- tRNS transparency expansion for grayscale, truecolor, and indexed-color images

## Remaining Gaps Versus Full libpng Parity

This is not a full libpng replacement. Major remaining gaps include:

- Packed bit depths 1, 2, and 4 for grayscale/indexed paths
- 16-bit grayscale-alpha and truecolor-alpha decode paths
- PLTE ordering and cardinality rules beyond the basic decode path
- Adam7 interlace reconstruction
- Progressive row callbacks and true streaming row decode
- Color management chunks: gAMA, cHRM, sRGB, iCCP
- Text/time/physical metadata chunks: tEXt, zTXt, iTXt, tIME, pHYs
- Unknown ancillary chunk preservation/copy policy
- Error recovery and warning model closer to libpng
- Write/encode APIs
- C ABI, allocator hooks, setjmp/longjmp behavior, and full public API parity

## Next Slice Decision

The next highest-value slice is packed bit-depth expansion for 1/2/4-bit scanlines. It is smaller than Adam7 interlace, builds directly on the current row reconstruction path, and will force explicit handling of sub-byte samples.

Adam7 interlace and progressive row callbacks are better handled after bit-depth expansion because they require broader data-flow changes and more substantial test fixtures.
