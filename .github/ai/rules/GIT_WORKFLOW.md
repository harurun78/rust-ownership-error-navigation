# Branch / Commit / PR Guidelines (Template)

このガイドラインは、AI coding agent と開発者が同じ規約で変更を進めるための基準です。

## 1. ブランチ命名規則
- フォーマット: `<type>/<issue番号>-<kebab-case-description>`
- type: `feature`, `fix`, `refactor`, `docs`, `test`, `chore`, `hotfix`
- 1ブランチ1目的を原則とする。
- デフォルトブランチ（`main`）最新から分岐する。

## 2. コミット規約
- Conventional Commits を使用する。
- フォーマット: `<type>(<scope>): <subject>`
- subject は簡潔に（目安50文字以内）、命令形で書く。
- 破壊的変更は `!` と `BREAKING CHANGE:` を明記する。

## 3. Pull Request 規約
- PRタイトルは Conventional Commits 準拠。
- PR本文に必須項目:
  - 概要
  - 背景/目的（Issueリンク）
  - 変更内容
  - スコープ外
  - 動作確認方法
  - チェックリスト

## 4. 品質ゲート
- 必須: lint / test / typecheck（該当時）
- 変更範囲が大きい場合は Draft PR で早期共有する。
- レビュー依頼前にセルフレビューを実施する。

## 5. AI Agent 追加ルール
- 無関係な差分を含めない。
- 失敗テストは原因を記録し、修正後に再実行する。
- 範囲外のCI失敗は修正対象に含めず、報告のみ行う。
