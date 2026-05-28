# Deterministic design suggestions minimal slice

GitHub Issue: #58

## Summary

Implement local-only deterministic design suggestions in JSON/HTML diagnostic reports. This turns the miniz A/B validation result into product behavior: keep local repair guidance and add prevention guidance for Rust-native ownership boundaries.

## Scope

- Add optional `designSuggestions` to diagnostic records.
- Implement `avoid-long-lived-buffer-borrow`, `split-mutation-phase`, and `owned-result` suggestion kinds.
- Derive suggestions from local rustc diagnostics, spans, labels, snippets, events, and audience mode.
- Render suggestions in JSON and static HTML.
- Add mapper/reporter tests and docs.

## Acceptance Criteria

- [x] E0499/E0502 reports can emit `split-mutation-phase` with evidence and confidence.
- [x] E0499/E0502 reports can emit `avoid-long-lived-buffer-borrow` when buffer/parser/stream evidence is present.
- [x] E0382/E0308 reports can emit `owned-result` when moved-value reuse or type-boundary pressure is present.
- [x] JSON report preserves `designSuggestions`.
- [x] HTML report contains a `Design Direction` section.
- [x] No external LLM API calls or source patch generation are introduced.
- [x] Verification passes: lint, format check, type-check, unit tests, integration tests, build.

## Notes

This issue tracks the local implementation slice. The implementation is present in the working tree and should be committed with the issue number once reviewed.

## Implementation Result

- Implemented optional `designSuggestions` records.
- Implemented `split-mutation-phase`, `avoid-long-lived-buffer-borrow`, and `owned-result` rules.
- Added JSON and HTML reporter coverage.
- Verification passed: `npm run format:check`, `npm run lint`, `npm run type-check`, `npm run test:run` (37 files / 70 tests), `npm run test:integration` (5 files / 8 tests), `npm run build`.