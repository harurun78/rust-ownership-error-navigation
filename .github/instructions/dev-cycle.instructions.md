---
description: "Use when executing the development cycle: picking tasks from Issues, implementing via sub-agents, creating PRs, reviewing, applying suggestions, and merging. Covers spec-unit, chore, and fix issues."
---

# Development Cycle — 自動開発サイクル

## Overview

このドキュメントは、Issue 選択 → 実装 → PR → レビュー → マージ の一連のサイクルを定義する。
ユーザーが「次のタスクを進めて」「サイクルを回して」等と指示した場合はこのフローに従う。

## サイクル全体像

```
Step 1:   タスク選択 (Issue labels)
    ↓
Step 2:   ブランチ作成 & 実装
    ├── 2.1 Issue 取得
    ├── 2.2 ブランチ作成
    └── 2.3 実装
         ├── 調査 (Explore agent)
         ├── 計画 (planner agent)
         ├── 実装 (メインエージェント)
         └── テスト追加 (test-writer agent)
    ↓
Step 3:   検証 (npm run lint && npm run format:check && npm run type-check && npm run test:run)
    ↓
Step 3.5: セキュリティ監査 (security-auditor agent)
    ↓
Step 4:   PR 作成
    ↓
Step 5:   レビュー (reviewer agent)
    ↓
Step 6:   Suggestion 適用 & 再検証
    ↓
Step 7:   CI 確認 & マージ
    ↓
Step 8:   クリーンアップ
```

---

## Step 1: タスク選択

ラベルを使って未着手の Issue をフィルタする。

### 1.1 次タスクの取得

```sh
# spec-unit（仕様実装タスク）
GH_PAGER=cat gh issue list --label spec-unit --state open --limit 10 --json number,title,labels

# spec-epic（進捗確認用）
GH_PAGER=cat gh issue list --label spec-epic --state open --limit 10 --json number,title,labels

# chore（仕様管理不要の変更）
GH_PAGER=cat gh issue list --label chore --state open --limit 10 --json number,title,labels
```

### 1.2 優先度ルール

1. **セキュリティ修正** — `fix(sec)`, セキュリティ関連は最優先
2. **In Progress の完了** — 中途半端な状態を先に解消
3. **ブロッカーの解消** — 依存グラフで後続が多い Issue を優先
4. **Phase 順** — Phase 1 → 2 → 3
5. **Issue 番号順** — 古い Issue を優先

> spec-unit だけでなく `chore` / `fix(...)` Issue も選択対象に含める。Epic は除外。

### 1.3 メインエージェントでの確認

コマンド出力をユーザーに提示し、確認を取る:

```
## 次のタスク候補

| 優先度 | Issue | 種類 | Spec ID | 理由 |
|--------|-------|------|---------|------|
| 1      | #XX   | spec | XXX-001 | ...  |
| 2      | #YY   | chore | —      | ...  |
| 3      | #ZZ   | spec | ZZZ-001 | ...  |

→ #XX を進めます。よろしいですか？
```

ユーザーが別のタスクを指定した場合はそれに従う。

---

## Step 2: ブランチ作成 & 実装

### 2.1 Issue 内容の取得

```sh
GH_PAGER=cat gh issue view <NUMBER> --json title,body,labels
```

### 2.2 ブランチ作成

```sh
git checkout main && git pull origin main
git checkout -b <type>/<issue>-<kebab-case>
```

命名規則:

- spec-unit: `feature/<issue番号>-<spec-id-kebab>` (例: `feature/44-model-orm-001`)
- chore: `chore/<issue番号>-<kebab-case>` (例: `chore/123-frontend-api-client`)
- fix: `fix/<issue番号>-<kebab-case>` (例: `fix/132-oauth-flow`)

### 2.3 実装

実装は以下の手順で行う:

1. **調査**: Explore サブエージェントで関連コードと仕様を把握
2. **計画**: planner サブエージェントで実装計画を策定（3ステップ以上、または設計判断が必要な場合）

    ```
    runSubagent(agentName="planner", prompt="Issue #<NUMBER> の実装計画を作成してください。")
    ```

    - 軽微修正（typo、明白な lint 修正、文言修正）はスキップ可
    - planner が Issue 分割を提案した場合はユーザーに相談

3. **実装**: ファイル編集ツール（例: `apply_patch` / `create_file`）を使用
    - **ターミナルでのファイル編集 (`sed -i` 等) は禁止**
4. **テスト追加**: test-writer サブエージェントでテストを生成

    ```
    runSubagent(agentName="test-writer", prompt="Issue #<NUMBER> の変更に対するテストを作成してください。変更ファイル: <一覧>")
    ```

    - 生成されたテストが検証を通過することを確認
    - 実装と並行して test-writer に委譲してよい

5. **spec ledger 更新** (spec-unit の場合のみ): 実装コミットに以下を含める
    - `docs/spec-index/spec-map.yaml`: status → Done, modules / tests を実装内容で更新
    - `docs/spec-index/spec-index.md`: 該当行の Status / Issue / PR / Modules / Tests を更新
    - PR 番号は PR 作成後に追加コミットで記入する（レビュー suggestion 修正と合わせてよい）

### 2.4 実装の制約

- Issue の Acceptance Criteria / 要件をすべて満たすこと
- 影響範囲を最小化し、無関係な変更を含めない
- 公開 API / 契約を変更する場合は仕様ドキュメントも更新する

---

## Step 3: 検証

PR 作成前に検証を実行し、**すべて**通過させる:

```sh
npm run lint && npm run format:check && npm run type-check && npm run test:run
```

個別に実行する場合:

```sh
npm run lint
npm run format:check
npm run test:run
npm run type-check
```

失敗時は修正してから次のステップに進む。

### 3.5 セキュリティ監査

検証通過後、PR 作成前にセキュリティ監査を実行する:

```
runSubagent(agentName="security-auditor", prompt="PR 作成前のセキュリティ監査を実行してください。変更ファイル: <一覧>")
```

判定に応じた対応:

| 判定     | 対応                                                                                                 |
| -------- | ---------------------------------------------------------------------------------------------------- |
| **PASS** | Step 4 へ進む                                                                                        |
| **WARN** | 警告内容を PR 本文に記載し、フォローアップ Issue を起票（または既存 Issue に追記）して Step 4 へ進む |
| **FAIL** | 指摘事項を修正 → Step 3 から再実行                                                                   |

> セキュリティ修正 Issue（`fix(sec)` 等）では特に念入りに監査する。

---

## Step 4: PR 作成

### 4.1 コミット & プッシュ

```sh
git add -A
git commit -m "<type>(<scope>): <subject>"
git push origin <branch>
```

### 4.2 PR 作成

`.github/PULL_REQUEST_TEMPLATE.md` に従って PR 本文を作成する。

```sh
gh pr create \
  --base main \
  --title "<type>(<scope>): <subject>" \
  --body "<PR本文>" \
  --assignee "@me"
```

PR 本文の必須項目:

- **Related Spec IDs**: 対象の spec-id（chore / fix の場合は「N/A」）
- **Related Issues**: `closes #<番号>`
- **What Changed**: 変更概要
- **Acceptance Criteria**: Issue から転記（チェック済み）
- **Verification**: 検証コマンドの実行結果

---

## Step 5: レビュー

reviewer サブエージェントを起動し、PR のレビューを実行する。
サブエージェントは reviewer.agent.md の指示に従い、レビューを作成して投稿する。

```
runSubagent(agentName="reviewer", prompt="PR #<NUMBER> をレビューしてください。リポジトリは harurun78/rust-ownership-error-navigation です。")
```

レビュー結果の判定:

- **APPROVE (blocker=0)** → Step 6 をスキップして Step 7 へ
- **REQUEST_CHANGES (blocker≥1)** → blocker を修正し、Step 3 から再実行
- **COMMENT** → suggestion を評価して適用

### レビュー実行タイミング（推奨）

レビューは以下の 2 段階で運用する:

1. **早期レビュー**（PR作成直後）
    - `reviewer` サブエージェントを即時実行し、CI 実行と並列化する
    - 早期に blocker/suggestion を回収し、修正を前倒しする
2. **最終レビュー**（CI完了後）
    - CI 全成功を確認したうえで、最終マージ可否を判定する

> これにより待ち時間を削減しつつ、マージゲートを維持できる。

---

## Step 6: Suggestion 適用 & 再検証

### 6.1 レビューコメントの取得と解析

```sh
# レビューコメントを取得
GH_PAGER=cat gh api repos/harurun78/rust-ownership-error-navigation/pulls/<NUMBER>/reviews
GH_PAGER=cat gh api repos/harurun78/rust-ownership-error-navigation/pulls/<NUMBER>/comments
```

> **Extension Point**: プロジェクトにレビューコメントの構造化抽出スクリプトがある場合はそちらを使用する。

### 6.2 Suggestion の適用

レビューの suggestion / blocker を1つずつ適用する:

- コード修正はファイル編集ツールで行う
- 修正ごとにコミット（または一括コミット）

### 6.3 再検証

Step 3 の検証をすべて再実行する。

### 6.4 プッシュ

```sh
git push origin <branch>
```

---

## Step 7: CI 確認 & マージ

### 7.1 CI ステータス確認

非対話・自動運用では以下を優先する:

```sh
GH_PAGER=cat gh pr view <NUMBER> --json statusCheckRollup --jq '.statusCheckRollup[] | [.name, .conclusion] | @tsv'
```

対話的に待機したい場合は以下を使用する:

```sh
gh pr checks <NUMBER> --watch
```

### 7.2 マージ

CI が全て通過したらマージする:

```sh
gh pr merge <NUMBER> --squash --delete-branch
```

実装サイクルを自動実行する場合は auto-merge を使用してよい:

```sh
gh pr merge <NUMBER> --squash --delete-branch --auto
```

マージ方式: **squash merge** を標準とする。

---

## Step 8: クリーンアップ

### 8.1 main 同期 & ブランチクリーンアップ

```sh
git checkout main && git pull origin main
git fetch --prune
```

`--delete-branch` 付きマージでリモートブランチは削除済み。`git fetch --prune` でローカルの追跡ブランチも整理する。

---

## エラーハンドリング

| 障害                     | 対応                                                         |
| ------------------------ | ------------------------------------------------------------ |
| lint 失敗                | エラー箇所を修正して再実行                                   |
| テスト失敗               | 失敗テストの原因を特定・修正。範囲外の失敗は報告のみ         |
| CI 失敗                  | ローカルで再現を試みる。今回の変更に起因しない場合は報告のみ |
| マージコンフリクト       | `git rebase origin/main` で解消。spec-index 系は最新を優先 |
| レビューで設計変更が必要 | ユーザーに判断を仰ぐ。再計画してから再開                     |

## 制約

- 1サイクル = 1 Issue を原則とする（大きすぎる場合は分割を提案）
- 実装サイクルの自動実行では、CI 通過後に自動でマージしてよい
- PR 本文のテンプレート項目は省略不可
- spec-unit と chore / fix で共通フローを使用し、差異は以下のみ:
    - spec ledger 更新: spec-unit のみ
    - ブランチ命名: `feature/` vs `chore/` vs `fix/`
    - PR 本文の Spec ID: chore / fix は「N/A」

