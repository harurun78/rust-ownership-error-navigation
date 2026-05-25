---
applyTo: '**'
---

# rust-ownership-error-navigation Development Instructions (Template)

## 目的
この instruction は、rust-ownership-error-navigation の実装・テスト・レビューを AI coding agent で一貫実行するための運用契約です。

## スコープ
- 対象: `src/**`, `tests/**`, `docs/**`, `.github/**`, `scripts/**`
- 非対象: `vendor/**`, 生成物ディレクトリ（`dist/**`, `build/**` など）

## 参照優先順位
1. `.github/ai/rules/PROJECT_ARCHITECTURE.md`
2. `.github/ai/rules/GIT_WORKFLOW.md`
3. このファイル
4. README / docs（該当セクション）

## 技術前提
- Language: TypeScript/JavaScript
- Runtime: Node.js 20
- Package Manager: npm
- Test Framework: vitest or jest
- Lint/Format: eslint, prettier

## 実装方針
- まず最小変更で要件を満たす（不要なリネーム・大規模リファクタ禁止）
- 公開 API/契約を変更する場合は `docs/adr/` または `docs/specs/` に理由を記録
- 例外処理は握りつぶさず、失敗理由を明示
- 設定値はハードコードせず設定ファイルまたは環境変数を使う

## 計画・実行ルール
- 3ステップ以上、または設計判断が必要なタスクは必ず計画を作成する
- 計画には「実装」と「検証」を必ず含める
- 作業中に前提が崩れた場合は再計画してから再開する
- 軽微修正（typo、明白なlint修正、文言修正）は計画を簡略化してよい

## サブエージェント運用
- リサーチ、コード探索、並列分析はサブエージェントへ委譲してよい
- 1サブエージェント = 1目的を維持する
- 結果統合はメイン担当が行い、重複・矛盾を解消する

## テスト自動化方針
### 必須ゲート（PR前）
1. `install` が成功する
2. `lint` が成功する
3. `test` が成功する
4. 主要機能の統合テスト（最低1本）が成功する

### 推奨ゲート
- `coverage` が 70% 以上
- `typecheck`（型検査）成功
- 変更箇所に対する回帰テスト追加

## コマンド契約（必ず最新化）
- Install: `npm ci`
- Lint: `npm run lint`
- Format Check: `npm run format:check`
- Unit Test: `npm run test:run`
- Integration Test: `npm run test:integration`
- Build: `npm run build`

## AI Agent 実行ルール
- 変更前に対象ファイルと影響範囲を明確化
- 変更後は最小セットで検証（関連テスト → 必要に応じ全体テスト）
- 失敗時はログ根拠付きで原因と次アクションを示す
- 範囲外の失敗は勝手に直さず、影響評価と切り分けを報告する
- タスク完了時は以下を必ず報告:
  - 変更ファイル一覧
  - 実行コマンド一覧
  - テスト結果（成功/失敗）
  - 未解決リスク

## 自己改善ループ
- ユーザーからの修正指摘で再発防止価値があるもののみ `tasks/lessons.md` に記録する
- 記録フォーマットは「再発条件 / 検知方法 / 防止ルール」を短く固定する
- 作業前に関連レッスンを確認する

## DoD（Definition of Done）
- [ ] 要件を満たす実装が存在する
- [ ] 既存契約を壊していない（または仕様更新済み）
- [ ] 必須ゲートがすべて成功
- [ ] ドキュメント更新済み（必要時）
- [ ] ロールバック手順または復旧手順が明記されている（運用機能のみ）

## 変更管理
- 破壊的変更は `BREAKING CHANGE:` をコミット/PR説明に明示
- 仕様変更を伴う場合、`docs/specs/` の該当ページを同時更新

## Git/PR ルール
- ブランチ命名、コミット規約、PRテンプレートは `.github/ai/rules/GIT_WORKFLOW.md` に従う
- 1ブランチ1目的を原則とする
- PR前にセルフレビュー（不要ログ・不要コメント・混入差分）を実施する

## セキュリティ・品質
- 秘密情報をログ出力しない
- 外部入力は必ずバリデーション
- 依存ライブラリ追加時はライセンスと保守状況を確認
- 重大障害につながる処理にはタイムアウト・リトライ方針を定義

## 補足（プロジェクト固有に記入）
- Domain Constraints: {DomainConstraints}
- Performance SLO: {PerformanceSLO}
- Non-Goals: {NonGoals}
- Release Policy: {ReleasePolicy}

## Export 時の必須生成物
- `.github/copilot-instructions.md`
- `.github/ai/rules/PROJECT_ARCHITECTURE.md`
- `.github/ai/rules/GIT_WORKFLOW.md`

`PROJECT_ARCHITECTURE.md` はテンプレート固定コピーではなく、`specs/` の確定内容（構造・技術スタック・実行コマンド・責務分離）を反映して生成すること。

