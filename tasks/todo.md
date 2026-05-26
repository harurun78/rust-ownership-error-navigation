# TODO

- [x] Export 後の初期設定を確認
- [x] speckit tasks T001-T011 の実装を開始・完了
- [x] 検証コマンドをローカルで実行

## 2026-05-25 実装開始メモ

- 完了: T001-T011（Phase 1 setup、raw diagnostic/event model、JSONL parser、parser tests）
- 実行: `npm install`, `npm run build`, `npm run cli:verify`, `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm: verify`
- 結果: すべて成功。integration は対象ファイル未作成のため `--passWithNoTests` で成功。
- 次: T012-T016（normalizer、mapper registry、reporter utilities）

## 2026-05-26 foundational slice

- 完了: T012-T016（normalizer、mapper registry、reporter utilities）
- 対応 Issue: #1, #2, #3, #4, #5
- 実行: `npm run lint`, `npm run type-check`, `npm run test:run`, `npm run build`, `npm run format:check`
- 結果: すべて成功。unit tests は 4 files / 14 tests passed。
- 次: T017-T027（US1 E0382 mapper / reporter / CLI）

## 2026-05-26 US1 E0382 slice

- 完了: T017-T027（E0382 mapper、suggestion extraction、JSON/HTML reporters、CLI pipeline）
- 対応 Issue: #6, #7, #8, #9, #10, #11, #12, #13, #14, #15, #16
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 8 files / 18 tests passed、integration 専用は 1 file / 1 test passed。
- 次: T028-T036（US2 E0499/E0502 borrow conflict mapping）

## 2026-05-26 US2 borrow conflicts slice

- 完了: T028-T036（E0499/E0502 mapper、borrow conflict integration、HTML causal ordering、multi-diagnostic summary）
- 対応 Issue: #17, #18, #19, #20, #21, #22, #23, #24, #25
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 12 files / 22 tests passed、integration 専用は 2 files / 2 tests passed。
- 次: T037-T045（US3 unsupported diagnostics compatibility）

## 2026-05-26 US3 unsupported diagnostics slice

- 完了: T037-T045（compatibility fixtures、parser edge cases、malformed JSONL、unsupported fallback、JSON/HTML display-only output）
- 対応 Issue: #26, #27, #28, #29, #30, #31, #32, #33, #35
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 16 files / 30 tests passed、integration 専用は 3 files / 6 tests passed。
- 次: T046-T050（US4 Borrow Sheet）

## 2026-05-26 US4 Borrow Sheet slice

- 完了: T046-T050（Borrow Sheet row model、HTML section、event kind label preservation、exports）
- 対応 Issue: #34, #36, #37, #39, #40
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 18 files / 32 tests passed、integration 専用は 3 files / 6 tests passed。
- 次: T051-T056（contract validation、docs/examples、quickstart validation、final verification）
