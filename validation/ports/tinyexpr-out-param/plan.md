# Plan: tinyexpr Out-Param Validation

## Phases

1. Create the target documentation and A/B crate layout.
2. Implement compatibility iteration 001 with the C-shaped out-param API and an intentional `Result`/`Option` mismatch to capture E0308.
3. Implement the Rust-native baseline with owned `Result` APIs and passing tests.
4. Repair compatibility iteration 002 by adapting parser `Result` into the C-shaped `Option` plus `error_out` contract.
5. Generate cargo-check JSONL and ownership reports for each iteration.
6. Summarize repair/prevention value and shortcut pressure.

## Verification

- `cargo test` for both final crates.
- `cargo check --message-format=json` captured for each report iteration.
- `node dist/cli/main.js` report generation for all saved JSONL files.
- Repository `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, and `npm run build` before commit.
