---
description: "Use after exporting a project to initialize spec-driven development tracking. Runs spec extraction, generates spec-index/spec-map, and creates GitHub Issues."
---

# Post-Export Bootstrap — 仕様駆動開発の初期化

## 目的

エクスポート直後のプロジェクトで、仕様書 → spec-index → GitHub Issue の一連を構築し、
dev-cycle による実装サイクルをすぐに開始できる状態にする。

ユーザーが「初期化して」「セットアップして」「Issue を作って」等と指示した場合はこのフローに従う。

## 前提

- `export/` でエクスポート済みのプロジェクトが存在する
- `docs/` に仕様書がコピー済み
- `.github/agents/`, `.github/instructions/dev-cycle.instructions.md` が生成済み
- GitHub リポジトリが作成済みで `gh` CLI が認証済み

## 初期化フロー

```
Phase 1: spec-index 生成
    1.1 spec-extraction.yaml の生成（specnote 側で実施済みの場合はスキップ）
    1.2 spec-id 割り当て & spec-index.md / spec-map.yaml 生成
    ↓
Phase 2: GitHub Issue 作成
    2.1 Epic Issue 作成
    2.2 spec-unit Issue 一括作成
    ↓
Phase 3: 検証
    3.1 Issue とラベルの確認
    3.2 初回 dev-cycle の準備確認
```

---

## Phase 1: spec-index 生成

### 1.1 spec-extraction.yaml の確認

specnote 側で `extract-spec-structure.ps1` を実行済みの場合は、
`docs/` にコピーされた仕様書を元に spec-extraction.yaml が存在する可能性がある。

存在しない場合は、エクスポート先の `docs/` に対して手動で構造分析を行う:

```sh
# docs/ 配下の .md ファイル一覧
find docs -name "*.md" -not -name "spec-index.md" -not -name "specification-discussion-log.md" | sort
```

各ファイルの H1/H2/H3 見出し構造を読み取り、spec-id 割り当ての材料にする。

### 1.2 spec-id 割り当て

以下のルールで spec-id を割り当てる:

- **形式**: `{PREFIX}-{TOPIC}-{NNN}`
- **PREFIX**: プロジェクト略称（2〜4文字、大文字）
- **TOPIC**: 仕様領域（英単語、ハイフン区切り）
- **NNN**: 3桁連番

**粒度の目安**: 1 spec-id ≒ 1 GitHub Issue ≒ 1〜3日の実装タスク

分割判断:
- H2 が 1〜3 個 → ファイル全体で 1 spec-id
- H2 が 4〜8 個 → H2 単位で分割
- H2 が 9 個以上 → 論理グループで分割

### 出力ファイル

#### docs/spec-index/spec-index.md

```markdown
# rust-ownership-error-navigation Spec Index

> Last updated: {date}

| Spec ID | Title | Status | Spec Link | Issue | PR | Modules | Tests |
|---|---|---|---|---|---|---|---|
| XX-YYY-001 | ... | Not Started | [link](../docs/xxx.md) | | | | |
```

#### docs/spec-index/spec-map.yaml

```yaml
version: "1.0"
project: {project_name}
generated: "{date}"

specs:
  - id: XX-YYY-001
    title: "..."
    status: Not Started
    spec_links:
      - xxx.md
    issue: null
    prs: []
    modules: []
    tests: []
    dependencies: []
    epic: null
```

spec-index.md と spec-map.yaml を生成したら、ユーザーに確認を取る:

```
## spec-index 生成結果

- spec-unit: N 件
- Epic 候補: N グループ
- 要確認: N 件（粒度が大きすぎる / 分割境界が曖昧）

確認の上、Phase 2 に進みますか？
```

---

## Phase 2: GitHub Issue 作成

### 2.1 ラベルの作成

```sh
# 必須ラベル（存在しない場合のみ作成）
gh label create spec-unit --description "仕様実装タスク" --color 0E8A16 2>/dev/null || true
gh label create spec-epic --description "仕様グループ (Epic)" --color 5319E7 2>/dev/null || true
gh label create chore --description "仕様管理不要の変更" --color FBCA04 2>/dev/null || true

# Phase ラベル（オプション）
gh label create "phase:1" --description "Phase 1" --color C2E0C6 2>/dev/null || true
gh label create "phase:2" --description "Phase 2" --color BFD4F2 2>/dev/null || true
gh label create "phase:3" --description "Phase 3" --color D4C5F9 2>/dev/null || true
```

### 2.2 Epic Issue の作成

spec-map.yaml で `epic` が定義されているグループごとに Epic Issue を作成する。
Epic がない場合はこのステップをスキップする。

```sh
gh issue create \
  --title "[Epic] <Epic タイトル>" \
  --body "## 概要
<Epic の説明>

## 含まれる Spec Units
- [ ] XX-YYY-001: <title>
- [ ] XX-YYY-002: <title>

## 完了条件
配下の spec-unit がすべて Done になること" \
  --label spec-epic \
  --assignee "@me"
```

作成後、Issue 番号を記録する。

### 2.3 spec-unit Issue の一括作成

spec-map.yaml の各 spec に対して Issue を作成する:

```sh
gh issue create \
  --title "[XX-YYY-001] <title>" \
  --body "## Spec ID
XX-YYY-001

## 仕様書リンク
- [xxx.md](docs/xxx.md)

## Requirements
<spec-map.yaml の spec_links を参照して要件を列挙>

## Acceptance Criteria
- [ ] <仕様書の受け入れ条件を転記>
- [ ] テストが追加されている
- [ ] spec-index / spec-map が更新されている

## Dependencies
<依存する spec-id があれば記載>

## Epic
<Epic Issue の番号、または N/A>" \
  --label spec-unit \
  --assignee "@me"
```

**一括作成の注意事項**:
- Issue は 1 件ずつ作成し、番号を逐次 spec-map.yaml に記録する
- 一度に大量作成する場合は API レート制限に注意（1秒間隔を推奨）
- 依存関係がある場合は、依存先を先に作成する

### 2.4 spec-map.yaml の更新

作成した Issue 番号を spec-map.yaml に反映する:

```yaml
specs:
  - id: XX-YYY-001
    issue: 1        # ← 作成された Issue 番号
    epic: 10         # ← Epic Issue 番号（該当する場合）
```

spec-index.md の Issue 列も更新する。

### 2.5 コミット

```sh
git add docs/spec-index/
git commit -m "spec: initialize spec-index and create tracking issues"
git push origin main
```

---

## Phase 3: 検証

### 3.1 Issue の確認

```sh
# spec-unit Issue の一覧
GH_PAGER=cat gh issue list --label spec-unit --state open --json number,title --jq '.[] | "#\(.number) \(.title)"'

# spec-epic Issue の一覧
GH_PAGER=cat gh issue list --label spec-epic --state open --json number,title --jq '.[] | "#\(.number) \(.title)"'

# ラベル確認
GH_PAGER=cat gh label list --json name,description --jq '.[] | "\(.name): \(.description)"'
```

### 3.2 初回サイクルの準備確認

以下をチェックし、結果をユーザーに報告する:

```
## Bootstrap 完了チェック

### ファイル
- [ ] docs/spec-index/spec-index.md が存在する
- [ ] docs/spec-index/spec-map.yaml が存在する
- [ ] .github/agents/ にエージェントファイルがある
- [ ] .github/instructions/dev-cycle.instructions.md がある
- [ ] .github/ISSUE_TEMPLATE/ にテンプレートがある
- [ ] .github/PULL_REQUEST_TEMPLATE.md がある

### GitHub
- [ ] spec-unit ラベルが存在する
- [ ] spec-epic ラベルが存在する
- [ ] chore ラベルが存在する
- [ ] spec-unit Issue が N 件作成済み
- [ ] CI ワークフローが設定済み（.github/workflows/ci.yml）

### 準備完了
「サイクルを回して」で dev-cycle.instructions.md に従って実装を開始できます。
```

---

## エラーハンドリング

| 問題 | 対応 |
|---|---|
| `gh` が認証されていない | `gh auth login` を案内 |
| リポジトリが存在しない | `gh repo create` を案内 |
| ラベル作成が権限不足で失敗 | リポジトリの権限設定を確認 |
| spec-id の粒度がユーザーの想定と異なる | Phase 1 で確認を取り、修正してから Phase 2 に進む |

## 制約

- Phase 1 → 2 の間で必ずユーザー確認を挟む
- Issue の本文は仕様書の内容を正確に転記する（解釈や要約は最小限）
- 仕様書に記載のない要件を勝手に Issue に追加しない

