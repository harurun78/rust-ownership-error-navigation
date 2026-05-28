# Zlib Dependency Decision for libpng Rust Port

PNG IDAT data is compressed with zlib-wrapped deflate. For this validation port, a pure Rust decoder dependency is preferred over implementing deflate from scratch.

- Chosen crate: `flate2 = "1.0"`
- Rationale: Widely used, maintained, supports zlib streams, keeps this validation focused on PNG parser ownership and row reconstruction rather than compression internals.
- Alternatives considered: manual deflate implementation, `miniz_oxide` directly, full `png` crate. Manual deflate is out of scope; direct `miniz_oxide` use is lower level than needed; the full `png` crate would hide the porting surface we want to measure.
- Usage: `flate2::read::ZlibDecoder` to inflate concatenated IDAT payload bytes before PNG scanline filter reconstruction.
