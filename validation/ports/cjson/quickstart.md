# Quickstart: cJSON Scalar Parser Porting Validation

## 1. Verify Upstream Checkout

```bash
git -C validation/ports/cjson/upstream/cjson rev-parse HEAD
git -C validation/ports/cjson/upstream/cjson describe --tags --always
```

Expected:

```text
c859b25da02955fef659d658b8f324b5cde87be3
v1.7.19
```

The upstream checkout is local validation input and must not be committed.

## 2. Create Or Enter The Rust Port Crate

```bash
cd validation/ports/cjson/rust-port
```

If the crate has not been created yet:

```bash
cargo init --lib .
```

## 3. Run Rust Checks

For repeatable validation runs, use the reusable prompt and agent definitions:

- `.github/agents/porting-lowcost.agent.md`
- `.github/prompts/porting.lowcost-iteration.prompt.md`
- `.github/instructions/porting-validation.instructions.md`

Each iteration should record model identity, prompt summary, saved rustc JSONL, generated report paths, and whether the navigation report changed the next fix.

```bash
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
  --input validation/ports/cjson/reports/iteration-001/cargo-check.jsonl \
  --json-out validation/ports/cjson/reports/iteration-001/ownership-report.json \
  --html-out validation/ports/cjson/reports/iteration-001/ownership-report.html
```

Expected:

- JSON report is created
- static HTML report is created
- E0382, E0499, and E0502 diagnostics, if present, include ownership events
- unsupported diagnostics remain visible as display-only records

## 5. Inspect And Record The Iteration

Open or inspect:

```text
validation/ports/cjson/reports/iteration-001/ownership-report.html
validation/ports/cjson/reports/iteration-001/ownership-report.json
```

Record in `notes/iteration-log.md`:

- model used
- prompt or task slice
- whether ownership hints were given before the attempt
- `cargo check` result
- E0382, E0499, and E0502 counts
- repeated diagnostic patterns
- whether the generated report changed the next fix
- human intervention count

## 6. Final Scalar Validation

```bash
cd validation/ports/cjson/rust-port
cargo check
cargo test
```

Phase 1 is complete when:

- scalar crate compiles
- scalar tests pass
- at least one failed iteration was captured and reported, unless the first generated attempt compiled cleanly
- arrays and objects remain documented as later phases
