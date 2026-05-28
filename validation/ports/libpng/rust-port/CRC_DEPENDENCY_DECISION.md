# CRC32 Dependency Decision for libpng Rust Port

For PNG CRC validation, a pure Rust implementation is preferred for safety and auditability. The `crc32fast` crate is widely used, no-std compatible, and maintained. It exposes a simple API for one-shot and incremental CRC32 computation, matching PNG's requirements.

- Chosen crate: `crc32fast = "1.4"`
- Rationale: Pure Rust, no-std, fast, maintained, minimal dependencies, widely used in ecosystem (e.g., flate2, png crate).
- Alternatives considered: `crc`, `digest`, manual implementation (not justified for this scope).
- Usage: `crc32fast::hash(&[chunk_type, payload])` for PNG chunk CRC validation.

---

This decision should be referenced in Cargo.toml and code comments for traceability.
