# Feature Specification: Rust Ownership Diagnostic Report MVP

**Feature Branch**: `001-ownership-report-mvp`

**Created**: 2026-05-25

**Status**: MVP implemented; post-MVP learner-centered diagnostics planning active

**Input**: User description: "Build a CLI tool that reads Rust cargo check JSONL, maps E0382 E0499 E0502 ownership diagnostics into event JSON, and generates a static HTML report while preserving future diagnostics as display-only compatibility corpus."

## Current State

The Phase 1 MVP has been implemented as a TypeScript / Node.js CLI. It parses Cargo JSONL diagnostics, maps E0382 / E0499 / E0502 into evidence-backed ownership events, preserves unsupported diagnostics, and emits JSON plus static HTML reports.

Validation after implementation used cJSON and Redis porting corpora. The Redis validation confirmed that ownership navigation can guide repair for E0382, but also showed that learner workflows need broader compiler diagnostic triage, especially E0308, E0004, E0425, E0596, and warning quality lanes.

Post-MVP work therefore keeps Rust beginners and intermediate users as the primary audience. Low-cost agents remain a useful validation consumer, but not the product center.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - moved value の原因を見る (Priority: P1)

Rust 初学者として、E0382 が出たときに、どこで値が move され、どこで move 後に使われたのかを 1 つのレポートで確認したい。

**Why this priority**: E0382 は所有権学習で最初にぶつかりやすく、move / use-after-move の因果を event 化できるかがツール価値の最小証明になる。

**Independent Test**: baseline fixture の E0382 diagnostic だけを入力して、move 原因地点、use conflict 地点、possible_fix が JSON と HTML に表示されることを確認する。

**Acceptance Scenarios**:

1. **Given** E0382 を含む cargo JSONL, **When** CLI が report を生成する, **Then** `move` event と `use` conflict event が source location 付きで出力される。
2. **Given** rustc child diagnostic に clone suggestion がある, **When** mapper が diagnostic を処理する, **Then** `possible_fix` として evidence 付きで保持される。

---

### User Story 2 - borrow conflict を因果順に見る (Priority: P1)

Rust 中級者として、E0499 / E0502 が出たときに、最初の borrow と衝突する borrow / use を順に確認したい。

**Why this priority**: mutable borrow conflict と shared-vs-mutable conflict は、所有権診断ナビゲーションの中心ユースケースである。

**Independent Test**: baseline fixture の E0499 / E0502 diagnostics を入力し、cause / conflict / context の span role と event kind が期待どおりに出ることを確認する。

**Acceptance Scenarios**:

1. **Given** E0499 diagnostic, **When** mapper が span labels を処理する, **Then** first mutable borrow が cause、second mutable borrow が conflict、later use が context として出る。
2. **Given** E0502 diagnostic, **When** mapper が span labels を処理する, **Then** immutable borrow cause、mutable borrow conflict、later immutable use context が区別される。

---

### User Story 3 - unsupported diagnostics を落とさず見る (Priority: P2)

実装者として、Phase 1 mapping 対象外の診断が入力に混ざっても、parser / reporter が失敗せず display-only diagnostic として保持することを確認したい。

**Why this priority**: 将来候補まで speckit 入力に含めるため、Phase 1 の scope を広げずに互換性だけを守る必要がある。

**Independent Test**: follow-up / advanced / non-ownership smoke fixtures を入力し、unsupported diagnostics が JSON と HTML summary に保持されることを確認する。

**Acceptance Scenarios**:

1. **Given** E0505 や E0716 を含む fixture, **When** CLI が Phase 1 mapper を実行する, **Then** mapper 未対応 diagnostic は unsupported/display-only として出力される。
2. **Given** no-code async Send diagnostic や E0282 を含む fixture, **When** parser が入力を処理する, **Then** diagnostic code が null または対象外でも message / spans / children が保持される。

---

### User Story 4 - 賃借対照表ビューで眺める (Priority: P3)

学習者として、同じ ownership events を通常の因果ビューとは別に、記憶に残る賃借対照表形式で眺めたい。

**Why this priority**: 学習体験の差別化要素だが、parser / mapper より後に実装できる。

**Independent Test**: E0382 の event JSON を HTML reporter に渡し、Rust 用語を保持したまま balance-sheet section が生成されることを確認する。

**Acceptance Scenarios**:

1. **Given** ownership event JSON, **When** HTML reporter が report を生成する, **Then** causality view と balance-sheet section の両方が含まれる。
2. **Given** balance-sheet section, **When** ユーザーが表示を読む, **Then** `move` / `borrow` / `use` / `conflict` など Rust event kind は置換されずに表示される。

### Post-MVP User Story 5 - 学習者向け summary を読む (Priority: P1)

Rust 初学者として、compiler output を見た直後に、何が起きたか、なぜ問題か、次に何を試すべきかを短い summary で確認したい。

**Why this priority**: Redis validation showed that diagnostic navigation works best when the user can connect spans to a plain-language explanation and a bounded next action.

**Independent Test**: E0382 / E0499 / E0502 fixture を beginner mode で report 生成し、`what_happened` / `why_it_matters` / `next_step` が JSON と HTML の先頭に出ることを確認する。

### Post-MVP User Story 6 - 複数診断の修正順を見る (Priority: P1)

Rust 中級者として、大きな compiler output に複数の error / warning が含まれるとき、どの診断から直すべきかを知りたい。

**Why this priority**: Redis porting では E0308 / E0004 / E0425 / warnings が ownership diagnostics より頻繁に実装ループを止めた。

**Independent Test**: Redis 由来の mixed diagnostic fixture を入力し、型・名前・match 網羅性エラーが downstream borrow diagnostics より先に recommended first fixes として表示されることを確認する。

### Edge Cases

- JSONL に `reason != compiler-message` の cargo messages が含まれる。
- `message.code` が null の diagnostic が含まれる。
- `children[]` が空でも spans だけで mapping する必要がある。
- primary span がない、または label が null の diagnostic が含まれる。
- macro expansion を含む spans がある。
- 入力 JSONL の一部行が壊れている。
- Windows path と POSIX path が混在する。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST read a local `cargo check --message-format json` JSONL file without modifying the Rust project that produced it.
- **FR-002**: System MUST extract `reason = compiler-message` cargo messages and preserve rustc diagnostic `code`, `level`, `message`, `spans`, `children`, and `rendered` when present.
- **FR-003**: System MUST classify Phase 1 mapping targets E0382, E0499, and E0502.
- **FR-004**: System MUST create `DiagnosticSpan` records from rustc spans with role, source location, label, suggestion, expansion flag, evidence, and confidence.
- **FR-005**: System MUST map E0382 into ownership events for move cause, use conflict, context, and possible_fix when supported by diagnostic payload.
- **FR-006**: System MUST map E0499 into ownership events for first mutable borrow, second mutable borrow conflict, and later use context.
- **FR-007**: System MUST map E0502 into ownership events for immutable borrow cause, mutable borrow conflict, and later immutable use context.
- **FR-008**: System MUST attach evidence and `high` / `medium` / `low` confidence to every mapped ownership event.
- **FR-009**: System MUST preserve unsupported diagnostics as display-only records instead of dropping them.
- **FR-010**: System MUST output event JSON conforming to the diagnostic report contract.
- **FR-011**: System MUST output a static HTML report containing summary, causality view, source spans, evidence, and balance-sheet section.
- **FR-012**: System MUST treat clippy, rust-analyzer/LSP, VS Code extension UI, and automatic fix application as out of scope for Phase 1.

### Post-MVP Functional Requirements

- **FR-013**: System SHOULD emit learner summaries for supported diagnostics with `what_happened`, `why_it_matters`, and `next_step` fields.
- **FR-014**: System SHOULD support `beginner`, `intermediate`, and `agent` audience modes without changing the underlying diagnostic evidence.
- **FR-015**: System SHOULD classify high-frequency non-ownership diagnostics from validation corpora, starting with E0308, E0004, and E0425.
- **FR-016**: System SHOULD explain fix strategy trade-offs such as borrow, clone/copy, scope split, move timing, extraction, and ownership redesign.
- **FR-017**: System SHOULD recommend a deterministic first-fix order for multi-diagnostic reports.
- **FR-018**: System SHOULD provide a cargo wrapper workflow that captures raw JSONL and generated reports while preserving the non-mutating behavior of analysis.

### Key Entities *(include if feature involves data)*

- **CargoMessage**: One JSONL line emitted by Cargo. Used to locate rustc compiler diagnostics.
- **RustcDiagnostic**: Diagnostic payload emitted by rustc, including code, message, spans, children, suggestions, and rendered output.
- **DiagnosticSpan**: L0 normalized span with source location, role, evidence, and confidence.
- **OwnershipEvent**: L1 navigation event such as `move`, `borrow_mut`, `borrow_shared`, `use`, `conflict`, `possible_fix`, `context`, or `unknown`.
- **DiagnosticReport**: Reporter output that groups supported ownership events and unsupported display-only diagnostics.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Baseline fixture with E0382 / E0499 / E0502 produces at least two meaningful ownership events for each supported diagnostic.
- **SC-002**: Compatibility fixtures parse without crashing and preserve unsupported diagnostics in JSON output.
- **SC-003**: HTML output contains Summary, Causality Timeline, Source Spans, Evidence, and Borrow Sheet sections for supported diagnostics.
- **SC-004**: Every mapped event includes at least one evidence item and a confidence value.
- **SC-005**: The report preserves rustc line and column data as 1-based values and documents adapter conversion needs for future VS Code usage.

## Assumptions

- Initial implementation is TypeScript / Node.js with `npm`.
- Input JSONL is produced by `cargo check --message-format json`.
- Fixture corpus is stored in this repository under `test/fixtures/diagnostics/` and remains compatibility evidence, not the only source of normative behavior.
- Phase 1 is a CLI and static HTML report, not a VS Code extension.
- HTML can be opened as a static file and does not require a dev server.

## Scope Boundaries

### 受け入れ条件

- [x] Phase 1 mapping targets are limited to E0382 / E0499 / E0502.
- [x] Follow-up, advanced, and non-ownership smoke fixtures are treated as parse/display compatibility inputs.
- [x] Unsupported diagnostics are visible in output instead of silently ignored.

### スコープ外

- clippy diagnostics via `cargo clippy --message-format json`.
- rust-analyzer or LSP diagnostics.
- VS Code extension implementation.
- Automatic fix application.
- Borrow checker reimplementation.

### 依存関係

- [Data Model](data-model.md) defines the normative output contracts.
- [Diagnostic Report Contract](../specs/001-ownership-report-mvp/contracts/diagnostic-report.schema.json) defines the initial JSON reporter shape.
- [Research](research.md) records technology and phase decisions.
- [Post Redis validation roadmap](../tasks/application-roadmap-from-redis-validation.md) records the prioritized post-MVP task candidates.
