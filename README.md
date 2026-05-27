# rust-ownership-error-navigation

Rust compiler JSON diagnostics into a local ownership-focused JSON and static HTML report.

The Phase 1 MVP maps E0382, E0499, and E0502 into evidence-backed ownership events. Other diagnostics are retained as display-only unsupported records so captured compiler output is not lost.

## Requirements

- Node.js 20+
- npm

## Install

```bash
npm ci
npm run build
```

## CLI Usage

Generate a report from the baseline ownership fixture:

```bash
node dist/cli/main.js \
	--input test/fixtures/diagnostics/ownership-baseline-2026-05-24.jsonl \
	--json-out out/ownership-report.json \
	--html-out out/ownership-report.html
```

Select the learner summary audience when needed:

```bash
node dist/cli/main.js \
	--input test/fixtures/diagnostics/ownership-baseline-2026-05-24.jsonl \
	--json-out out/ownership-report.agent.json \
	--html-out out/ownership-report.agent.html \
	--audience agent
```

`--audience` accepts `beginner`, `intermediate`, or `agent` and defaults to `beginner`. The mode changes the learner summary surface while preserving the underlying diagnostic evidence, spans, and events.

Run a compatibility fixture with unsupported diagnostics preserved:

```bash
node dist/cli/main.js \
	--input test/fixtures/diagnostics/ownership-followup-2026-05-25.jsonl \
	--json-out out/followup-report.json \
	--html-out out/followup-report.html
```

The JSON report contains `schemaVersion`, `input`, `summary`, and `diagnostics`. The HTML report contains Summary, Learner Summaries, Diagnostics, Causality Timeline, Source Spans, Evidence, Borrow Sheet, and Unsupported Diagnostics sections.

## Examples

- [examples/ownership-report.json](examples/ownership-report.json)
- [examples/ownership-report.html](examples/ownership-report.html)

## Verification

```bash
npm run lint
npm run format:check
npm run type-check
npm run test:run
npm run test:integration
npm run build
```

## Documentation

- [docs/quickstart.md](docs/quickstart.md)
- [specs/001-ownership-report-mvp/spec.md](specs/001-ownership-report-mvp/spec.md)
- [docs/specification-discussion-log.md](docs/specification-discussion-log.md)
- [validation/](validation/)
