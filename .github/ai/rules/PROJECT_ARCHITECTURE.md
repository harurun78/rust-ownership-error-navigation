# PROJECT_ARCHITECTURE (Template)

このドキュメントは、エクスポート時点の仕様確定結果から自動生成する。
固定テンプレートのコピペではなく、`specs/` の合意済み内容を反映すること。

---

## 1. プロジェクト概要
- 名称: `rust-ownership-error-navigation`
- 目的: `仕様書に基づく実装プロジェクト`
- 主要ユースケース: `仕様書を参照`
- 想定ユーザー: `仕様書を参照`

## 2. 技術スタック
- 言語: `{Languages}`
- ランタイム: `{Runtime}`
- フレームワーク: `- cli: nodejs -> src (tests: test)`
- ストレージ/ミドルウェア: `仕様書を参照`
- テスト/品質: `vitest or jest, eslint, prettier`

## 3. 実行コマンド
- Install: `npm ci`
- Dev: `npm run dev`
- Build: `npm run build`
- Lint: `npm run lint`
- TypeCheck: `npm run type-check`
- Unit Test: `npm run test:run`
- Integration Test: `npm run test:integration`

## 4. ディレクトリ構成
```text
.
├── .github/
│   ├── instructions/
│   └── ai/rules/
├── docs/
├── src/
├── tests/
├── examples/
└── tasks/

Components:
- cli: nodejs -> src (tests: test)
```

## 5. 責務分離
- `src/`: `アプリケーション実装`
- `tests/`: `検証コード`
- `docs/`: `仕様・運用ドキュメント`

## 6. アーキテクチャ制約
- 変更禁止領域: `vendor/, build生成物`
- 互換性方針: `仕様書の決定事項を優先`
- 外部公開契約: `docs/contracts/ または仕様書定義`

## 7. テスト戦略要約
- 最低限実行する検証: `npm run lint, npm run test:run, npm run test:integration`
- 重要な非機能要件: `性能・可観測性・保守性を仕様準拠で担保`
- 失敗時の復旧方針: `失敗時はロールバック手順に従う`

## 8. 参照仕様
- 生成元仕様: `docs/speckit-export-operations.md, docs/copilot-instructions.md, docs/speckit.analyze.agent.md, docs/speckit.checklist.agent.md, docs/speckit.clarify.agent.md, docs/speckit.constitution.agent.md, docs/speckit.implement.agent.md, docs/speckit.plan.agent.md, docs/speckit.specify.agent.md, docs/speckit.tasks.agent.md, docs/speckit.taskstoissues.agent.md, docs/speckit.analyze.prompt.md, docs/speckit.checklist.prompt.md, docs/speckit.clarify.prompt.md, docs/speckit.constitution.prompt.md, docs/speckit.implement.prompt.md, docs/speckit.plan.prompt.md, docs/speckit.specify.prompt.md, docs/speckit.tasks.prompt.md, docs/speckit.taskstoissues.prompt.md, docs/constitution.md, docs/checklist-template.md, docs/constitution-template.md, docs/plan-template.md, docs/spec-template.md, docs/tasks-template.md, docs/data-model.md, docs/plan.md, docs/quickstart.md, docs/research.md, docs/spec.md`
- ADR/補足: `docs/adr/`

---

## Exporter 生成ルール（必須）
1. `specs/` から以下を抽出して埋める:
   - 構造（モジュール、責務、境界）
   - 技術スタック（言語、FW、永続化、主要ライブラリ）
   - 実行コマンド（install/build/test/lint）
2. プレースホルダ `{...}` を残さない。
3. 抽出不能な項目は `TBD` ではなく「未定（理由）」を明記する。
4. 生成後、`copilot-instructions.md` と矛盾がないことを確認する。
