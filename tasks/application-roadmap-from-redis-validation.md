# Application Roadmap From Redis Porting Validation

作成日: 2026-05-27

## 背景

Redis porting validation では、所有権ナビゲーションが E0382 の修正入力として機能することを確認できた。一方で、大きな Rust 実装では E0382/E0499/E0502 だけでなく、型不一致、match 網羅性、未定義名、warning、弱いテストなどが実装者の前進を止めることも分かった。

今後の実装は低コスト agent だけを対象にせず、仕様書の中心ユーザーである Rust 初学者・中級者が「何が起きたか」「なぜ起きたか」「次に何を試せばよいか」を理解できる学習支援アプリとして進める。

## Product Direction

- 初学者には、move / borrow / lifetime / type mismatch を因果順に短く説明する。
- 中級者には、複数診断の優先順位、修正候補のトレードオフ、設計変更が必要な兆候を示す。
- AI agent には、レポートを修正入力として使える構造化 summary を提供する。
- HTML report は静的で開けるまま、学習者が読みやすい drill-down UI に発展させる。
- CLI は既存の `cargo check --message-format=json` 入力に加え、必要に応じて cargo 実行 wrapper として使えるようにする。

## Validation Findings To Preserve

- E0382 report-guided repair は有効だった。iteration-036 で move/use span と `ref` suggestion を低コスト agent が利用して修正できた。
- Redis porting では所有権エラーより、E0308 / E0004 / E0425 / warnings が多く開発ループを止めた。
- Clean final report は信頼感に直結する。unsupported warnings も放置せず、修正または明示的に扱う価値がある。
- `clone()` suggestion は便利だが、学習者には「clone でよい場面」と「設計を見直す場面」の区別が必要。
- 大規模コードでは、単一 diagnostic より「どの診断から直すべきか」の優先順位が重要になる。

## Phase A: Learner-Centered Report UX

### APP-001: Add Learner Summary Cards

Priority: P0

User value: 初学者が最初に読むべき短い説明を得る。

Tasks:

- Add `learnerSummary` to report JSON for supported diagnostics.
- Generate plain-language `what_happened`, `why_it_matters`, and `next_step` fields.
- Render summary cards at the top of HTML supported diagnostic sections.
- Keep Rust terms such as `move`, `borrow`, `mutable borrow`, `lifetime` visible and not over-translated.

Acceptance:

- E0382 report shows moved value, later use, and safest next action in one short card.
- E0499/E0502 report distinguishes first borrow from conflicting borrow/use.
- Existing causality and Borrow Sheet sections remain available.

### APP-002: Add Beginner / Intermediate Display Modes

Priority: P0

User value: 説明量を学習段階に合わせられる。

Tasks:

- Add CLI option `--audience beginner|intermediate|agent`.
- Beginner mode emphasizes vocabulary and small steps.
- Intermediate mode emphasizes scope shortening, ownership redesign, and API boundaries.
- Agent mode emits concise structured repair hints without tutorial prose.

Acceptance:

- Same diagnostic can produce three report variants without changing underlying events.
- Agent mode does not recommend broad `.clone()` as the only path when other strategies exist.

### APP-003: Add Source Slice And Variable Timeline View

Priority: P1

User value: エラーの該当行だけでなく、値や borrow の流れを追える。

Tasks:

- Extract small source excerpts around primary spans when files are available locally.
- Add a variable timeline view grouped by symbol/name where rustc labels permit it.
- Mark cause, conflict, later use, and suggestion spans with stable anchors.

Acceptance:

- HTML report lets a learner jump from summary to source slice to timeline.
- If source files are unavailable, report gracefully falls back to span paths and labels.

## Phase B: Broaden Diagnostic Navigation Beyond Phase 1 Ownership

### APP-004: Prioritize High-Frequency Non-Ownership Errors

Priority: P0

User value: 実際の開発で先に直すべき compiler errors が分かる。

Tasks:

- Add first-class mapper family for E0308 type mismatch.
- Add first-class mapper family for E0004 non-exhaustive patterns.
- Add first-class mapper family for E0425 unresolved name.
- Preserve these as non-ownership navigation events with cause/conflict/fix categories.

Acceptance:

- Reports group ownership and non-ownership diagnostics separately.
- Multi-diagnostic summary recommends a fix order when unsupported errors block ownership diagnostics.
- Redis validation JSONL fixtures can be reused as regression evidence.

### APP-005: Add Borrow/Lifetime Follow-Up Mappers

Priority: P1

User value: 初学者が ownership 学習で次に遭遇する診断も読める。

Tasks:

- Add display/navigation support for E0505 moved out while borrowed.
- Add support for E0596 cannot borrow as mutable.
- Add support for E0597 borrowed value does not live long enough.
- Add support for E0716 temporary value dropped while borrowed.

Acceptance:

- Each new mapper has fixture coverage and plain-language explanation.
- Unsupported fallback remains intact for diagnostics not yet mapped.

### APP-006: Add Warning Quality Lane

Priority: P1

User value: warning を無視すべきか直すべきか判断できる。

Tasks:

- Classify `unused_variables`, `unused_mut`, `dead_code`, and unreachable code warnings.
- Add `reportCleanliness` summary with errors vs warnings.
- Explain when a warning is safe cleanup versus symptom of an incomplete implementation.

Acceptance:

- A clean cargo check can be distinguished from a warning-only report.
- Final report can be configured to fail or pass on warnings.

## Phase C: Fix Strategy Guidance

### APP-007: Explain Fix Strategy Trade-Offs

Priority: P0

User value: `clone()` すればよいのか、borrow すべきか、設計変更すべきか判断できる。

Tasks:

- Add fix strategy taxonomy: borrow, move earlier/later, clone/copy, split scope, extract value, redesign ownership.
- Map rustc suggestions into this taxonomy where possible.
- Add educational notes for common traps such as broad clone shortcuts.

Acceptance:

- E0382 with clone suggestion labels clone as one possible strategy, not always best.
- E0499/E0502 reports include scope-shortening and two-phase operation strategies where applicable.

### APP-008: Add Multi-Diagnostic Fix Order

Priority: P1

User value: 大量の compiler output でどこから直すか分かる。

Tasks:

- Build a deterministic diagnostic priority model.
- Prefer parse/type/name/match errors before downstream borrow errors when spans overlap or counts cascade.
- Mark repeated diagnostics and likely root causes.

Acceptance:

- Report top summary includes `recommendedFirstFixes`.
- Redis-like JSONL with E0308/E0004 before ownership errors yields clear ordering.

### APP-009: Add Agent Repair Packet Export

Priority: P1

User value: AI agent やペアプロ相手に渡しやすい修正指示を生成できる。

Tasks:

- Add `--repair-packet-out` JSON output.
- Include diagnostic code, root spans, source excerpt, suggested fix strategies, and constraints.
- Avoid applying fixes automatically.

Acceptance:

- The packet can reproduce iteration-036 style E0382 repair without manual ownership hints.
- Packet omits secrets and only includes local source excerpts needed for the diagnostic.

## Phase D: Input And Workflow Improvements

### APP-010: Add Cargo Wrapper Mode

Priority: P1

User value: JSONL ファイルを自分で作らなくても使える。

Tasks:

- Add CLI mode that runs `cargo check --message-format=json` in a target project.
- Save raw JSONL, report JSON, report HTML, and optional repair packet under an output directory.
- Keep existing file-input mode unchanged.

Acceptance:

- `ownership-nav check --manifest-path path/to/Cargo.toml --out reports/latest` works.
- Command failures still produce reports when cargo emits usable JSON diagnostics.

### APP-011: Add Project Session Index

Priority: P2

User value: 学習者が修正の推移を振り返れる。

Tasks:

- Add a small report index page for repeated runs.
- Track diagnostic counts by code over time.
- Highlight newly introduced, fixed, and repeated diagnostics.

Acceptance:

- Repeated cargo check runs produce a static index with trend summary.
- No database or server is required.

### APP-012: Add Fixture Corpus From Porting Validation

Priority: P1

User value: 実際の大きめ Rust 移植から得た例で機能を鍛える。

Tasks:

- Promote selected cJSON and Redis JSONL diagnostics into `test/fixtures/diagnostics/porting/`.
- Include E0382 repair case from iteration-036.
- Include non-ownership blocking cases such as E0308, E0004, E0425.

Acceptance:

- Regression tests cover both learner examples and porting-scale diagnostics.
- Fixture README explains why each case is included.

## Phase E: HTML Report As Learning Surface

### APP-013: Add Glossary And Concept Links

Priority: P2

User value: 初学者が知らない語をその場で確認できる。

Tasks:

- Add compact glossary entries for move, borrow, mutable borrow, lifetime, ownership, scope, clone.
- Link glossary terms from learner summary cards.
- Keep glossary local/static and avoid external network dependency.

Acceptance:

- HTML report remains self-contained.
- Terms are concise and Rust-specific.

### APP-014: Add Exercise Mode From Diagnostics

Priority: P2

User value: エラーを読んで終わりではなく、理解確認ができる。

Tasks:

- Generate optional exercise prompts from mapped events.
- Ask learners to identify moved value, active borrow, conflicting use, or likely first fix.
- Provide answer reveal section.

Acceptance:

- Exercise mode can be toggled in HTML generation.
- No source modification or automatic fix is performed.

### APP-015: Improve Accessibility And Printability

Priority: P2

User value: レポートを読みやすく共有しやすくする。

Tasks:

- Add keyboard-friendly navigation between diagnostics and spans.
- Improve color contrast for cause/conflict/context labels.
- Add print stylesheet for classroom or review use.

Acceptance:

- HTML remains static and passes basic accessibility checks for headings, labels, and contrast.

## Phase F: Validation And Release Readiness

### APP-016: Define Learning-Oriented Acceptance Fixtures

Priority: P0

User value: 機能追加が学習者への説明品質を壊さない。

Tasks:

- Add golden reports for beginner and intermediate modes.
- Add snapshot tests for HTML sections that learners see first.
- Keep raw rustc evidence available for every explanation.

Acceptance:

- Tests fail if learner summary disappears or loses evidence links.
- Golden updates require explicit review.

### APP-017: Add Report Quality Metrics

Priority: P1

User value: どの診断が「分かりやすく説明できたか」を測れる。

Tasks:

- Add report metrics: supported coverage, evidence completeness, source availability, fix strategy availability.
- Display quality warnings when confidence is low.

Acceptance:

- JSON report includes quality metrics.
- HTML summary tells the user when guidance is incomplete.

### APP-018: Publish Next Spec Slice

Priority: P0

User value: MVP 後の開発を Issue 化しやすくする。

Tasks:

- Convert APP-001 through APP-006 into speckit tasks or GitHub issues.
- Update spec docs with learner-centered post-MVP scope.
- Keep automatic fix application explicitly out of scope unless a future spec changes it.

Acceptance:

- Next implementation cycle can start from the prioritized tasks without reinterpreting Redis validation notes.

## Recommended First Implementation Order

1. APP-001 Learner Summary Cards
2. APP-004 High-Frequency Non-Ownership Errors
3. APP-007 Fix Strategy Trade-Offs
4. APP-016 Learning-Oriented Acceptance Fixtures
5. APP-010 Cargo Wrapper Mode
6. APP-008 Multi-Diagnostic Fix Order

This order turns the Redis validation result into direct product value: better explanations for humans first, broader diagnostic coverage second, and smoother workflow third.
