# Redis Porting Validation Quickstart

## Verify Upstream

```sh
git -C validation/ports/redis/upstream/redis rev-parse HEAD
git -C validation/ports/redis/upstream/redis describe --tags --exact-match
git check-ignore -v validation/ports/redis/upstream/redis/src/networking.c
```

Expected upstream commit and tag:

- Commit: `d2c8a4b91e8c0e6aefd1f5bc0bf582cddbe046b7`
- Tag: `7.2.4`

## Start First Rust Iteration

```sh
cd validation/ports/redis
cargo new --lib rust-port
cd rust-port
cargo test
```

## Capture Diagnostics

From `validation/ports/redis/rust-port/`:

```sh
mkdir -p ../reports/iteration-001
cargo check --message-format=json > ../reports/iteration-001/cargo-check.jsonl
```

From the repository root:

```sh
npm run build
node dist/cli/main.js \
  --input validation/ports/redis/reports/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/redis/reports/iteration-001/ownership-report.json \
  --html-out validation/ports/redis/reports/iteration-001/ownership-report.html
```

## Verify Completed Iteration

```sh
cd validation/ports/redis/rust-port
cargo fmt -- --check
cargo test
```

From the repository root, run the normal project gates after tracked validation files change:

```sh
npm run format:check
npm run lint
npm run type-check
npm run test:run
```

## Record Results

Update `notes/iteration-log.md` with:

- model identity
- prompt summary
- human ownership hints
- command results
- E0382/E0499/E0502 counts
- whether the ownership report changed the next fix
- shortcut pressure, if any
