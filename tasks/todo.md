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
