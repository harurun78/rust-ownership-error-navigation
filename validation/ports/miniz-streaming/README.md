# miniz Streaming Porting Comparison

This target compares two C/C++ to Rust porting conditions using a miniz/zlib-style streaming API slice:

- `tracks/compatibility/rust-port`: preserve C-style stream state, input/output buffer fields, status codes, and allocator hook surface.
- `tracks/rust-native/rust-port`: preserve behavior while using owned output, `Result`, and short borrow scopes.

The first slice intentionally uses a pass-through stream transform rather than full deflate decoding. Its purpose is to start measuring API shape, buffer ownership, and mutation pressure before adding compression semantics.