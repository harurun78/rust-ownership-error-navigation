# Speckit Export Operations

## 概要

Rust Ownership Error Navigation を speckit 継続可能な実装リポジトリとして export するための操作定義。

通常の `scripts/export-project.ps1` を土台にし、speckit scaffold、AI 設定、開発ツール定義、devcontainer、fixture corpus を追加する。

## 操作 1: 前提確認

### スコープ

export 前に仕様リポジトリが clean であり、speckit scaffold と Phase 1 仕様が存在することを確認する。

### 受け入れ条件

- [ ] `git status --short` が空である、または export 作業だけが未コミットである。
- [ ] `specs/rust-ownership-error-navigation/export.yaml` が存在する。
- [ ] `specs/rust-ownership-error-navigation/.specify/` が存在する。
- [ ] `specs/rust-ownership-error-navigation/specs/001-ownership-report-mvp/spec.md` が存在する。

### スコープ外

- 実装コードの完成確認。

### 検証方法

- `git status --short`
- `Test-Path` による対象ファイル確認。

## 操作 2: 通常 export を実行

### スコープ

既存 export フローで、docs、tasks、AI instructions、generic agents、workflow、Issue / PR templates を生成する。

### 受け入れ条件

- [ ] `export/rust-ownership-error-navigation/` が作成される。
- [ ] `docs/` に仕様 Markdown がコピーされる。
- [ ] `.github/instructions/` と `.github/agents/` が生成される。
- [ ] `.github/workflows/ci.yml` が生成される。

### スコープ外

- speckit scaffold の root 昇格。
- devcontainer と Node.js toolchain 定義。

### 検証方法

```powershell
.\scripts\export-project.ps1 -ProjectName "rust-ownership-error-navigation" -Force
```

## 操作 3: speckit scaffold を root に昇格

### スコープ

仕様ディレクトリ内の speckit scaffold を export 先の実装リポジトリ root にコピーする。

### 受け入れ条件

- [ ] `export/rust-ownership-error-navigation/.specify/` が存在する。
- [ ] `export/rust-ownership-error-navigation/specs/001-ownership-report-mvp/` が存在する。
- [ ] `.github/prompts/speckit.*.prompt.md` が存在する。
- [ ] `.github/agents/speckit.*.agent.md` が存在する。

### スコープ外

- speckit task 生成。
- GitHub Issue 作成。

### 検証方法

- `Get-ChildItem export/rust-ownership-error-navigation/.specify`
- `Get-ChildItem export/rust-ownership-error-navigation/.github/prompts`

## 操作 4: 開発ツール定義を生成

### スコープ

TypeScript / Node.js CLI 実装を開始できる最小 toolchain 定義を export 先に生成する。

### 受け入れ条件

- [ ] `package.json` が存在し、`lint` / `format:check` / `type-check` / `test:run` / `build` scripts を持つ。
- [ ] `tsconfig.json` が存在する。
- [ ] `eslint.config.js` が存在する。
- [ ] `vitest.config.ts` が存在する。
- [ ] `.prettierrc.json` が存在する。
- [ ] `.prettierignore` が存在し、export scaffold / docs / fixture corpus / build output を format check 対象外にする。

### スコープ外

- MVP 機能の実装。

### 検証方法

- `Get-Content export/rust-ownership-error-navigation/package.json | ConvertFrom-Json`

## 操作 5: devcontainer と CI を生成

### スコープ

Node.js、Rust fixture 再生成、GitHub CLI、Docker outside-of-Docker を前提にした開発コンテナと speckit-aware workflow を追加する。

### 受け入れ条件

- [ ] `.devcontainer/devcontainer.json` が存在する。
- [ ] `.github/workflows/speckit-ci.yml` が存在する。
- [ ] workflow が `npm ci`、`npm run lint`、`npm run format:check`、`npm run type-check`、`npm run test:run`、`specify check` を含む。

### スコープ外

- CI secret 設定。

### 検証方法

- JSON parse check for devcontainer.
- YAML file existence check for workflow.

## 操作 6: fixture corpus を配置

### スコープ

Phase 1 mapper と compatibility fallback の検証に使う JSONL corpus を export 先へコピーする。

### 受け入れ条件

- [ ] `test/fixtures/diagnostics/ownership-baseline-2026-05-24.jsonl` が存在する。
- [ ] follow-up / advanced / non-ownership smoke fixtures が存在する。
- [ ] fixture README が同梱される。

### スコープ外

- fixture の再採取。

### 検証方法

- `Get-ChildItem export/rust-ownership-error-navigation/test/fixtures/diagnostics/*.jsonl`

## 操作 7: export 結果を検証

### スコープ

生成物が実装開始可能な構造になっていることを確認する。

### 受け入れ条件

- [ ] `scripts/export-speckit-project.ps1 -ProjectName "rust-ownership-error-navigation" -Force` が成功する。
- [ ] devcontainer JSON が parse できる。
- [ ] package JSON が parse できる。
- [ ] speckit scaffold、AI 設定、workflow、fixtures が export 先に存在する。

### スコープ外

- `npm ci` の実行。
- 実装タスクの開始。

### 検証方法

```powershell
.\scripts\export-speckit-project.ps1 -ProjectName "rust-ownership-error-navigation" -Force
Get-Content export/rust-ownership-error-navigation/package.json -Raw | ConvertFrom-Json | Out-Null
Get-Content export/rust-ownership-error-navigation/.devcontainer/devcontainer.json -Raw | ConvertFrom-Json | Out-Null
```
