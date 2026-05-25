# Copilot Instructions (Template)

このドキュメントは、実装品質と作業効率を両立するための運用契約です。

## ワークフロー

### 1. 計画フェーズ
- 3ステップ以上、または設計判断を含むタスクは計画を必須とする。
- 計画には「実装」「検証」「完了条件」を含める。
- 想定外の問題が発生した場合は、作業を止めて再計画する。

### 2. サブエージェント戦略
- リサーチ・コード探索・並列分析はサブエージェントへ委譲する。
- 1サブエージェント = 1タスクを原則とする。
- 収集結果は重複排除して統合し、最終判断はメイン担当が行う。

### 3. 完了前の検証（必須）
- 動作を証明せずに完了としない。
- 少なくとも `lint` / `test` / 変更関連の統合テストを実行する。
- 失敗時はログと原因を残し、修正後に再実行して結果を更新する。

### 4. 自己改善
- 再発防止価値のある指摘のみ `tasks/lessons.md` に記録する。
- 記録は「再発条件 / 検知方法 / 防止ルール」で簡潔に残す。

## 実装原則
- 可能な限り単純な設計を優先する。
- 一時しのぎではなく根本原因を修正する。
- 影響範囲を最小化し、無関係な変更を避ける。

## バグ修正ポリシー
- ログ・エラー・失敗テストを根拠に自律的に修正を進める。
- CI失敗対応は「今回変更と因果関係があるもの」に限定する。
- 範囲外の失敗は記録・報告に留め、無断で作業範囲を拡大しない。

## タスク管理
- `tasks/todo.md`: 計画、進捗、レビュー結果を管理する。
- `tasks/lessons.md`: 再発防止ルールを管理する。

## 参照ドキュメント
- プロジェクト構成・技術スタック: `.github/ai/rules/PROJECT_ARCHITECTURE.md`
- Git/PR規約: `.github/ai/rules/GIT_WORKFLOW.md`
- 開発サイクル: `.github/instructions/dev-cycle.instructions.md`
- 初期セットアップ: `.github/instructions/bootstrap.instructions.md`

<!-- SPECKIT START -->
## 実装計画

現在の実装計画: [specs/001-ownership-report-mvp/plan.md](../specs/001-ownership-report-mvp/plan.md)

**Feature**: Rust Ownership Diagnostic Report MVP
**Branch**: `001-ownership-report-mvp` (仕様ブランチ名、実装は main)
**Status**: Phase 0 Research 完了、Phase 1 Design 完了、実装開始前

**重要な参照**:
- 仕様: [specs/001-ownership-report-mvp/spec.md](../specs/001-ownership-report-mvp/spec.md)
- データモデル: [specs/001-ownership-report-mvp/data-model.md](../specs/001-ownership-report-mvp/data-model.md)
- タスク: [specs/001-ownership-report-mvp/tasks.md](../specs/001-ownership-report-mvp/tasks.md)
- 契約: [specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json](../specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json)
- クイックスタート: [specs/001-ownership-report-mvp/quickstart.md](../specs/001-ownership-report-mvp/quickstart.md)

**Phase 1 Scope**: E0382 / E0499 / E0502 所有権診断マッピングのみ
**出力形式**: JSON レポート + 静的 HTML レポート
**非目標**: clippy, rust-analyzer, VS Code 拡張機能, 自動修正適用
<!-- SPECKIT END -->

## サブエージェント一覧

以下のエージェントが `.github/agents/` に定義されている。タスクに応じて使い分けること。

| エージェント | 用途 |
|---|---|
| `planner` | Issue 着手前の実装計画策定。ファイル変更一覧・テスト方針・リスクを出力 |
| `reviewer` | PR レビュー。コード品質・セキュリティ・仕様準拠を検証 |
| `test-writer` | テストコード生成。仕様の受入条件からテストケースを導出 |
| `security-auditor` | セキュリティ監査。OWASP Top 10・依存関係・シークレットを検査 |
| `project-reviewer` | プロジェクト全体の健全性チェック。構成・ドキュメント・CI を検証 |
