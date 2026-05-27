# Quickstart: libpng Porting Validation

## 1. Verify Upstream Checkout

```bash
git -C validation/ports/libpng/upstream/libpng rev-parse HEAD
git -C validation/ports/libpng/upstream/libpng describe --tags --always
git -C validation/ports/libpng/upstream/libpng rev-parse v1.6.58
```

Expected:

```text
3061454d980de7d53608f594194cfac722721d2a
v1.6.58
fdc7185dfedbddce8c2487bc171f66af4fca24ab
```

The upstream checkout is local validation input and must not be committed.

## 2. Enter The Rust Port Crate

```bash
cd validation/ports/libpng/rust-port
```

## 3. Run Rust Checks

```bash
cargo fmt -- --check
cargo check
cargo test
```

During failed model iterations, capture diagnostics as JSONL:

```bash
mkdir -p ../reports/iteration-001
cargo check --message-format=json > ../reports/iteration-001/cargo-check.jsonl
```

Keep stdout as JSONL. If human-readable stderr is needed, save it separately in the same iteration folder.

## 4. Generate Ownership Reports

From the repository root:

```bash
npm run build
node dist/cli/main.js \
  --input validation/ports/libpng/reports/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/libpng/reports/iteration-001/ownership-report.json \
  --html-out validation/ports/libpng/reports/iteration-001/ownership-report.html
```

Expected:

- JSON report is created.
- Static HTML report is created.
- E0382, E0499, and E0502 diagnostics, if present, include ownership events.
- Unsupported diagnostics remain visible as display-only records.

## 5. Inspect And Record The Iteration

Record in `notes/iteration-log.md`:

- model used
- prompt or task slice
- whether ownership hints were given before the attempt
- `cargo check` result
- E0382, E0499, and E0502 counts
- repeated diagnostic patterns
- whether the generated report changed the next fix
- human intervention count
- shortcut pressure: `clone`, shared mutability, or `unsafe`

## 6. Compatibility Facade Smoke Usage

The current validation crate includes Rust-native libpng-style lifecycle helpers. A caller can exercise the compatibility facade without C ABI assumptions:

```rust
let mut reader = png_compat_create_read_struct();
png_compat_set_read_buffer(&mut reader, png_bytes);
let info = png_compat_read_info(&mut reader)?;
let rows = png_compat_read_image(&mut reader)?;
png_compat_destroy_read_struct(&mut reader);

let mut writer = png_compat_create_write_struct();
png_compat_write_image(&mut writer, &image)?;
let output = png_compat_write_output(&writer);
png_compat_destroy_write_struct(&mut writer);
```

The facade intentionally reports `RustNativeFacadeOnly` and `CAbiNotProvided` warnings. C ABI, allocator hooks, and setjmp/longjmp behavior are a separate compatibility track.

## 7. Current Completion Boundary

The Rust-native compatibility boundary is complete when:

- parse/decode/write/document APIs compile
- compatibility facade tests pass
- JSONL diagnostics are captured under the active `reports/iteration-NNN/`
- ownership report JSON and HTML are generated
- iteration notes record diagnostic counts and whether report feedback was useful
