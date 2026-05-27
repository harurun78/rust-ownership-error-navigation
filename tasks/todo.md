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

## 2026-05-26 US1 E0382 slice

- 完了: T017-T027（E0382 mapper、suggestion extraction、JSON/HTML reporters、CLI pipeline）
- 対応 Issue: #6, #7, #8, #9, #10, #11, #12, #13, #14, #15, #16
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 8 files / 18 tests passed、integration 専用は 1 file / 1 test passed。
- 次: T028-T036（US2 E0499/E0502 borrow conflict mapping）

## 2026-05-26 US2 borrow conflicts slice

- 完了: T028-T036（E0499/E0502 mapper、borrow conflict integration、HTML causal ordering、multi-diagnostic summary）
- 対応 Issue: #17, #18, #19, #20, #21, #22, #23, #24, #25
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 12 files / 22 tests passed、integration 専用は 2 files / 2 tests passed。
- 次: T037-T045（US3 unsupported diagnostics compatibility）

## 2026-05-26 US3 unsupported diagnostics slice

- 完了: T037-T045（compatibility fixtures、parser edge cases、malformed JSONL、unsupported fallback、JSON/HTML display-only output）
- 対応 Issue: #26, #27, #28, #29, #30, #31, #32, #33, #35
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 16 files / 30 tests passed、integration 専用は 3 files / 6 tests passed。
- 次: T046-T050（US4 Borrow Sheet）

## 2026-05-26 US4 Borrow Sheet slice

- 完了: T046-T050（Borrow Sheet row model、HTML section、event kind label preservation、exports）
- 対応 Issue: #34, #36, #37, #39, #40
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`
- 結果: すべて成功。unit/integration は 18 files / 32 tests passed、integration 専用は 3 files / 6 tests passed。
- 次: T051-T056（contract validation、docs/examples、quickstart validation、final verification）

## 2026-05-26 Polish and final verification

- 完了: T051-T056（JSON schema validation、README usage、generated examples、quickstart docs/test、final verification）
- 対応 Issue: #38, #41, #42, #43, #44, #45
- 実行: `npm ci`, `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`, `npm run cli:verify`
- 結果: すべて成功。unit/integration は 20 files / 34 tests passed、integration 専用は 4 files / 7 tests passed。
- 補足: malformed JSONL の CLI test は期待通り stderr に parser error を出し、exit code 1 を返す。

## 2026-05-26 Project review response

- 対応: spec quickstart の unsupported count / schema validation 手順を実装済み docs と整合。
- 対応: `passWithNoTests` を test scripts / Vitest config から削除。
- 対応: coverage threshold 70% を Vitest config に追加。
- 対応: Spec Kit CI が `specify` 未導入時に skip せず fail するよう変更。
- 実行: `npm run lint`, `npm run format:check`, `npm run type-check`, `npm run test:run`, `npm run test:integration`, `npm run build`, `npm run test:coverage -- --coverage.reporter=text`
- 結果: すべて成功。coverage は All files statements/lines 92.4%、branches 80.33%、functions 98.46%。

## 2026-05-26 Porting validation scaffold

- 対応: C/C++ から Rust へのポーティング検証用に `validation/ports/` を追加。
- 対応: 最初の検証ターゲットとして `validation/ports/cjson/` を作成。
- 目的: ownership-error navigation が低コスト AI モデルの Rust 移植で有効かを、cJSON の tree ownership / cleanup / string ownership を題材に検証する。
- 追加: cJSON upstream `v1.7.19` (`c859b25da02955fef659d658b8f324b5cde87be3`) をローカル取得し、Git には source snapshot を含めず取得手順と porting spec を記録。
- 追加: Spec Kit 風に `validation/ports/cjson/plan.md`, `tasks.md`, `quickstart.md`, `notes/iteration-log.md` を作成し、scalar parser phase の境界・検証手順・評価指標を明確化。

## 2026-05-26 cJSON low-cost iteration-001

- 対応: `GPT-5 mini (copilot)` を軽量モデルとして cJSON scalar parser の初回実装を実行。
- 追加: `validation/ports/cjson/rust-port/` に Rust library crate、owned `JsonValue`、`ParseError`、scalar parser、7件の parser tests を追加。
- 保存: `reports/iteration-001/cargo-check.jsonl` を自作ナビゲーションツールへ入力し、`ownership-report.json` / `ownership-report.html` を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 7 tests passed。所有権エラー修正ループは Phase 1 では発生せず。

## 2026-05-26 cJSON low-cost iteration-002

- 対応: `GPT-5 mini (copilot)` で cJSON array/object parser slice を実装。
- 追加: empty/mixed/nested arrays、empty/scalar/nested objects、malformed containers、recursion depth guard tests を追加。
- 保存: `reports/iteration-002/cargo-check.jsonl` を自作ナビゲーションツールへ入力し、`ownership-report.json` / `ownership-report.html` を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 15 tests passed。owned `Vec` model ではまだ所有権エラー修正ループは発生せず。

## 2026-05-26 cJSON low-cost iteration-003/004

- iteration-003: mutable tree editing / detach operations を `GPT-5 mini (copilot)` で実装。`cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 23 tests passed。
- iteration-004: path-based immutable/mutable lookup と nested replacement を `GPT-5 mini (copilot)` で実装。`cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 28 tests passed。
- 保存: `reports/iteration-003/` と `reports/iteration-004/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 所見: `unsafe` / shared mutability / broad clone は不要。現状の owned tree model では軽量モデルが所有権エラーなしで進められているため、次にツール効用を強く見るなら borrowed/reference variants や cJSON-like linked node model が候補。

## 2026-05-26 cJSON low-cost iteration-005

- 対応: compact JSON printing を `GPT-5 mini (copilot)` で実装。
- 追加: scalar printing、string escaping、array/object printing、parse -> compact print round-trip tests を追加。
- 保存: `reports/iteration-005/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 32 tests passed。

## 2026-05-26 cJSON low-cost iteration-006

- 対応: typed predicates / accessors / object member and array item helper APIs を `GPT-5 mini (copilot)` で実装。
- 追加: type predicate、typed immutable/mutable accessor、object member、array item tests を追加。
- 保存: `reports/iteration-006/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 38 tests passed。

## 2026-05-26 cJSON low-cost iteration-007

- 対応: pretty JSON printing を `GPT-5 mini (copilot)` で実装。
- 追加: scalar / array / nested object pretty printing、string escaping preservation tests を追加。
- 保存: `reports/iteration-007/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 42 tests passed。

## 2026-05-26 cJSON low-cost iteration-008

- 対応: cJSON_Minify 相当の JSON minify utility を `GPT-5 mini (copilot)` で実装。
- 追加: insignificant whitespace removal、string preservation、line/block comments、unterminated string/comment tests を追加。
- 保存: `reports/iteration-008/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 46 tests passed。

## 2026-05-26 cJSON low-cost iteration-009

- 対応: path-based detach/delete helper を `GPT-5 mini (copilot)` で実装。
- 追加: nested array/object detach、missing terminal、missing parent、empty path、container mismatch tests を追加。
- 保存: `reports/iteration-009/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 51 tests passed。

## 2026-05-26 cJSON low-cost iteration-010

- 対応: JSON merge patch utility を `GPT-5 mini (copilot)` で実装。
- 追加: object merge、null deletion、nested merge、non-object replacement、non-object target handling tests を追加。
- 保存: `reports/iteration-010/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 56 tests passed。

## 2026-05-26 cJSON low-cost iteration-011

- 対応: JSON Pointer parsing and lookup helpers を `GPT-5 mini (copilot)` で実装。
- 追加: empty/root pointer、nested lookup、array index、escaped keys、invalid prefix/escape/index tests を追加。
- 保存: `reports/iteration-011/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 62 tests passed。

## 2026-05-26 cJSON low-cost iteration-012

- 対応: JSON Patch add/remove/replace utility を `GPT-5 mini (copilot)` で実装。
- 追加: object/array add、remove、replace、invalid document、unsupported op、missing path/value、array bounds tests を追加。
- 保存: `reports/iteration-012/` に cargo JSONL、ownership JSON/HTML、notes を保存。
- 結果: `cargo check --message-format=json` 成功、navigation diagnostics は 0 件、`cargo test` は 73 tests passed。

## 2026-05-27 Post Redis validation roadmap

- 作成: [application-roadmap-from-redis-validation.md](application-roadmap-from-redis-validation.md)
- 方針: Redis porting 検証結果を、低コスト agent 専用ではなく Rust 初学者・中級者に有用な学習支援アプリの実装タスクへ変換。
- 重点: learner summary、beginner/intermediate 表示、E0308/E0004/E0425 など高頻度 non-ownership diagnostics、fix strategy trade-off、cargo wrapper、porting fixture corpus。

## 2026-05-27 Speckit issue creation

- 作成: `specs/002-learner-centered-diagnostics/` に post-MVP speckit spec / plan / tasks を追加。
- Issue 化: #48 learner model foundation、#49 learner summary cards、#50 audience modes、#51 non-ownership diagnostics、#52 first-fix order / fix strategy guidance。
- 着手順: #48 を最初の実装ブランチとして開始する。

## 2026-05-27 Issue #48 implementation start

- ブランチ: `feature/48-learner-diagnostic-model-foundation`
- 実装: learner diagnostic shared model types、optional report schema fields、audience-aware fixture helper、model/schema tests を追加。
- 対象 speckit tasks: T001-T004。
- 検証: `npm: verify` 成功（lint / format:check / type-check / test:run / test:integration）、`npm: build` 成功。
