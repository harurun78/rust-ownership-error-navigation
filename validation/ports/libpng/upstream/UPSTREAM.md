# libpng Upstream

## Source

- Repository: `https://github.com/pnggroup/libpng.git`
- Selected tag: `v1.6.58`
- Tag object: `fdc7185dfedbddce8c2487bc171f66af4fca24ab`
- Selected commit: `3061454d980de7d53608f594194cfac722721d2a`
- License: PNG Reference Library License version 2
- Local checkout path: `validation/ports/libpng/upstream/libpng/`

The local checkout is intentionally ignored by Git. This repository tracks the selected upstream version and acquisition commands, not a vendored copy of libpng.

## Acquisition

```bash
git clone --depth 1 --branch v1.6.58 \
  https://github.com/pnggroup/libpng.git \
  validation/ports/libpng/upstream/libpng
```

Verify the selected version:

```bash
git -C validation/ports/libpng/upstream/libpng rev-parse HEAD
git -C validation/ports/libpng/upstream/libpng describe --tags --always
git -C validation/ports/libpng/upstream/libpng rev-parse v1.6.58
```

Expected output:

```text
3061454d980de7d53608f594194cfac722721d2a
v1.6.58
fdc7185dfedbddce8c2487bc171f66af4fca24ab
```

## Files Of Interest

- `png.c`: `png_sig_cmp`, signature constants, ICC validation helpers
- `pngread.c`: high-level read initialization and signature check flow
- `pngrutil.c`: read-side signature and chunk utility logic
- `pngpread.c`: progressive read signature handling
- `pngpriv.h`: internal read helpers and chunk definitions
- `png.h`: public APIs, chunk and signature documentation
- `pngtest.c`: smoke-style file read/write validation reference

## Ownership Signals

- libpng C state is carried through `png_struct` and `png_info` pointers with error jumps.
- Read code mutates input state while exposing parsed metadata through caller-owned structures.
- Progressive read paths keep partial signature/chunk state between calls.
- Chunk data and row buffers are allocated, resized, and freed across multiple error paths.
- The first Rust slice should avoid raw pointer/state-machine shortcuts and use owned metadata plus explicit parser state.
