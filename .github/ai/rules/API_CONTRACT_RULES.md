# API_CONTRACT_RULES (Template)

このドキュメントは、プロジェクト種別を問わず API/契約変更時の運用ルールを定義する。

## 1. 対象
- HTTP API / RPC API
- JSON Schema / OpenAPI / protobuf 等の契約定義
- 外部公開されるデータフォーマット

## 2. 変更原則
- 互換性を壊す変更は明示し、移行手順を同時提供する
- 仕様変更と実装変更を分離せず、同一PRで整合させる
- 命名・必須項目・型の変更は影響範囲を明記する

## 3. バージョニング
- 破壊的変更: major または新version endpoint
- 後方互換追加: minor
- パッチ修正: patch

## 4. 必須チェック
- 契約ファイルの差分レビュー
- サーバ/クライアント双方の互換テスト
- 代表的サンプル（request/response）更新
- 変更履歴（changelog または migration doc）更新

## 5. 破壊的変更の扱い
- `BREAKING CHANGE:` をPR/コミットに明記する
- 影響対象、移行期限、移行手順を明示する
- 段階的切替が必要な場合は deprecate 期間を定義する

## 6. AI Agent ルール
- 契約更新時は必ず対応するテスト/サンプルを同時更新する
- 契約だけ更新して実装が追随しない状態を作らない
- 不明な互換性は推測せず、前提を明示する

## 7. 記録先
- 契約定義: `docs/contracts/`
- 変更履歴: `docs/contracts/CHANGELOG.md`
- 移行ガイド: `docs/contracts/MIGRATION.md`

