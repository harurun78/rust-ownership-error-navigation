# Feature Specification: Learner-Centered Diagnostic Navigation

**Feature Branch**: `002-learner-centered-diagnostics`

**Created**: 2026-05-27

**Status**: Ready for task issue creation

**Input**: Redis porting validation showed that the MVP ownership report is useful, but Rust beginners and intermediate users need learner summaries, diagnostic prioritization, broader non-ownership coverage, and fix strategy trade-offs.

## User Scenarios & Testing

### User Story 1 - 初学者が最初に読む summary を得る (Priority: P1)

Rust 初学者として、E0382 / E0499 / E0502 の report を開いたときに、何が起きたか、なぜ問題か、次に何を試すべきかを短い説明で確認したい。

**Why this priority**: MVP は evidence-backed events を持つが、学習者が最初に読む導線が弱い。Redis validation でも report-guided repair は有効だったため、人間向けにも同じ evidence を短く説明する価値が高い。

**Independent Test**: Baseline ownership fixture を beginner mode で report 生成し、JSON と HTML に `whatHappened`, `whyItMatters`, `nextStep` が含まれることを確認する。

**Acceptance Scenarios**:

1. **Given** E0382 diagnostic, **When** report is generated for beginner audience, **Then** moved value, later use, and safest next action are shown in a learner summary.
2. **Given** E0499 or E0502 diagnostic, **When** report is generated for beginner audience, **Then** first borrow and conflicting borrow/use are named in causal order.

---

### User Story 2 - 学習段階に合わせて説明量を変える (Priority: P1)

Rust 中級者として、同じ diagnostic でも beginner / intermediate / agent の表示モードを切り替え、必要な説明量と修正視点を選びたい。

**Why this priority**: 初学者には語彙と小さな一歩が必要だが、中級者には scope shortening や ownership redesign のような設計判断が必要になる。Agent には tutorial prose より構造化 repair hints が役立つ。

**Independent Test**: 同じ fixture を `beginner`, `intermediate`, `agent` の 3 mode で生成し、underlying events は同一で summary 表現だけが audience に応じて変わることを確認する。

**Acceptance Scenarios**:

1. **Given** `--audience beginner`, **When** report is generated, **Then** glossary-friendly explanation and next step are prioritized.
2. **Given** `--audience intermediate`, **When** report is generated, **Then** scope shortening, API boundary, and ownership redesign options are emphasized.
3. **Given** `--audience agent`, **When** report is generated, **Then** concise structured repair hints are emitted without tutorial prose.

---

### User Story 3 - non-ownership diagnostics の優先順位を見る (Priority: P1)

Rust 中級者として、大きな compiler output に E0308 / E0004 / E0425 などが混在するとき、どの diagnostic から直すべきかを知りたい。

**Why this priority**: Redis validation では ownership diagnostics より type mismatch, non-exhaustive match, unresolved name が開発ループを止める場面が多かった。

**Independent Test**: Redis validation 由来の mixed diagnostic fixture を入力し、high-frequency non-ownership diagnostics が ownership diagnostics と分けて表示され、recommended first fixes が出ることを確認する。

**Acceptance Scenarios**:

1. **Given** E0308 diagnostic, **When** report is generated, **Then** type mismatch is shown as a first-class navigation record.
2. **Given** E0004 diagnostic, **When** report is generated, **Then** non-exhaustive pattern is shown with likely missing branch context.
3. **Given** E0425 diagnostic, **When** report is generated, **Then** unresolved name is shown as a likely upstream blocker.
4. **Given** multiple diagnostics, **When** summary is generated, **Then** deterministic recommended first fixes are listed.

---

### User Story 4 - 修正方針の trade-off を理解する (Priority: P2)

Rust 学習者として、`clone()`、borrow、scope 分割、move timing、ownership redesign のどれを試すべきか、利点と注意点を理解したい。

**Why this priority**: rustc suggestion は有用だが、学習者にとっては clone が常に正解に見えやすい。修正方針の意味とリスクを説明する必要がある。

**Independent Test**: E0382 / E0499 / E0502 fixture を入力し、possible fix strategy が taxonomy と trade-off note を持つことを確認する。

**Acceptance Scenarios**:

1. **Given** clone suggestion, **When** report is generated, **Then** clone is labeled as one strategy with allocation/semantics trade-off.
2. **Given** borrow conflict, **When** report is generated, **Then** scope split or operation ordering strategy is shown when applicable.

## Requirements

### Functional Requirements

- **FR-001**: System MUST preserve existing MVP JSON and HTML report behavior for E0382 / E0499 / E0502.
- **FR-002**: System MUST add learner summary fields without removing existing evidence, spans, or ownership events.
- **FR-003**: System MUST support audience selection for `beginner`, `intermediate`, and `agent`.
- **FR-004**: System MUST classify E0308, E0004, and E0425 as first-class non-ownership navigation records.
- **FR-005**: System MUST keep unsupported fallback behavior for diagnostics not yet mapped.
- **FR-006**: System MUST provide deterministic recommended first fixes for multi-diagnostic reports.
- **FR-007**: System SHOULD classify fix strategy trade-offs for supported ownership diagnostics.
- **FR-008**: System MUST remain non-mutating; automatic source code modification remains out of scope.

### Key Entities

- **LearnerSummary**: Audience-specific explanation with `whatHappened`, `whyItMatters`, `nextStep`, and optional concept terms.
- **AudienceMode**: `beginner`, `intermediate`, or `agent` report generation mode.
- **DiagnosticNavigationRecord**: Generalized navigation record for ownership and non-ownership diagnostics.
- **FixStrategy**: Explanation of a possible repair approach and trade-off.
- **RecommendedFirstFix**: Deterministic prioritized diagnostic/action pair for multi-diagnostic reports.

## Success Criteria

- **SC-001**: Beginner reports for E0382 / E0499 / E0502 include learner summary cards in JSON and HTML.
- **SC-002**: Audience mode changes explanatory surface while preserving mapped evidence and events.
- **SC-003**: Redis-derived fixtures containing E0308 / E0004 / E0425 produce first-class non-ownership records.
- **SC-004**: Multi-diagnostic report summary includes recommended first fixes.
- **SC-005**: Existing MVP tests continue to pass.

## Scope Boundaries

### In Scope

- CLI options and report model additions.
- Static HTML report updates.
- Fixture promotion from validation corpus.
- Unit, reporter, and integration tests.

### Out of Scope

- Automatic fix application.
- VS Code extension UI.
- rust-analyzer or clippy integration.
- Full generic Rust diagnostic explanation for every error code.
