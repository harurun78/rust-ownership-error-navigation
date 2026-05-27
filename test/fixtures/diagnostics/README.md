# Diagnostics Reference Outputs

作成日: 2026-05-25

## 概要

`cargo check --message-format json` の実測 JSONL を参考資料として保存する。実装リポジトリへ移す前に、parser / mapper / reporter の fixture として参照する。

## 保存済みファイル

| file                                                                                     | 内容                                                                                    | lines | bytes | sha256                                                             |
| ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ----: | ----: | ------------------------------------------------------------------ |
| [ownership-baseline-2026-05-24.jsonl](ownership-baseline-2026-05-24.jsonl)               | E0382 / E0499 / E0502 baseline fixture                                                  |     6 | 17942 | `6C00E382C2A4B94EA9C11B4E7BD1AB2E7A5393C2837CED48C19F836172C267B9` |
| [ownership-followup-2026-05-25.jsonl](ownership-followup-2026-05-25.jsonl)               | E0373 / E0505 / E0506 / E0507 / E0596 / E0597 / E0716 / E0515 / E0521 follow-up fixture |    13 | 45049 | `1CF5F5802DB893842F33BF12218BDEB33657F326F5E692D273B81DFFF4C73BD1` |
| [ownership-advanced-2026-05-25.jsonl](ownership-advanced-2026-05-25.jsonl)               | no code async Send / E0597 / E0382 / E0277 advanced fixture                             |    11 | 52183 | `14EA1F7A37D7544C1A4200EC0DD2AB8BFA393BB56132A4BCA632D936791D2BD8` |
| [rustc-non-ownership-smoke-2026-05-25.jsonl](rustc-non-ownership-smoke-2026-05-25.jsonl) | E0603 / E0277 / E0282 / E0599 / E0308 / E0004 non-ownership compiler smoke fixture      |     9 | 21643 | `FA35F62F60284AD3DF91123E8463BADFADB2189A6414571A30A227831E5465CC` |
| [porting/non-ownership-navigation-2026-05-25.jsonl](porting/non-ownership-navigation-2026-05-25.jsonl) | Reduced porting navigation fixture for E0308 / E0004 / E0425                            |     3 |  3224 | `FBF25A533610FEF1A57071BFE93DF812D1AC7B7DA20FF8EC17B60937D3A01859` |

## Porting fixture provenance

- `porting/non-ownership-navigation-2026-05-25.jsonl` keeps E0308 and E0004 coverage aligned with the Redis validation notes that identified non-ownership compiler blockers.
- E0425 was not present in the saved Redis JSONL corpus, so the E0425 row is a reduced rustc reproduction fixture for the same high-frequency navigation category.

## 採取条件

- Docker image: `rust:latest` / `rust:1-slim` 相当
- Toolchain observed:
  - `rustc 1.95.0 (59807616e 2026-04-14)`
  - `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Command family: `cargo check --message-format json`
- Note: 実行時に `/usr/local/cargo/bin` を PATH に追加した。

## 使い方

- parser の入力 fixture として使う。
- error code filter のテストに使う。
- `DiagnosticSpan` / `OwnershipEvent` の expected mapping を作る元資料にする。
- compiler version によって JSON 形状や label 文言が変わり得るため、仕様上は「参考出力」として扱う。
