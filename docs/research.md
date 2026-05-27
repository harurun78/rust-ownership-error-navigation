# Research: Rust Ownership Diagnostic Report MVP

## Decisions

## Post-Redis Validation Product Direction

### Decision

Keep Rust beginners and intermediate users as the primary audience for post-MVP work. Use low-cost agents as a validation consumer, not as the only target user.

### Rationale

The Redis porting validation confirmed that E0382 ownership navigation can help an agent repair code, but it also showed that human learners need explanations, vocabulary, trade-offs, and diagnostic prioritization. Many blockers were not ownership diagnostics: E0308 type mismatches, E0004 non-exhaustive patterns, E0425 unresolved names, E0596 mutability issues, and warning cleanup repeatedly affected progress.

### Alternatives Considered

- Agent-only repair packets: useful but too narrow for the original learning-oriented specification.
- Ownership-only expansion: insufficient for real project work where non-ownership diagnostics often block progress first.
- Automatic fix application: deferred because the current product contract is diagnostic navigation and explanation, not source modification.

### 受け入れ条件

- [ ] Post-MVP tasks include learner summaries and audience modes.
- [ ] Post-MVP tasks include high-frequency non-ownership diagnostics from validation.
- [ ] Repair packet export remains optional and secondary to human-readable reports.

## TypeScript / Node.js CLI

### Decision

Use TypeScript / Node.js for Phase 1.

### Rationale

Future VS Code extension integration can reuse parser, model, and mapper code directly. A Rust CLI would be attractive for ecosystem alignment, but it introduces binary distribution and cross-platform packaging concerns earlier than needed.

### Alternatives Considered

- Rust CLI: rejected for Phase 1 due to VS Code reuse and distribution overhead.
- Browser-only tool: rejected because JSONL parsing and file workflows are simpler in CLI first.

### 受け入れ条件

- [ ] Implementation plan assumes TypeScript modules for parser, mapper, and reporters.
- [ ] Future VS Code reuse is reflected as module boundary, not Phase 1 UI work.

## cargo check JSONL As Primary Input

### Decision

Use `cargo check --message-format json` JSONL files as Phase 1 input.

### Rationale

The fixture corpus confirms that rustc diagnostic code, spans, children, suggestions, and rendered output are available through Cargo JSONL. This input is easy for users to capture without modifying their project.

### Alternatives Considered

- `rustc --error-format json`: useful later, but less representative of normal Cargo project workflows.
- rust-analyzer/LSP diagnostics: useful for VS Code extension, but not necessary for CLI/HTML MVP.
- clippy JSON: separate command family and not Phase 1.

### 受け入れ条件

- [ ] Spec and plan name Cargo JSONL as the only Phase 1 command family.
- [ ] Unsupported command families are listed as out of scope.

## Phase 1 Mapping Scope

### Decision

Map only E0382 / E0499 / E0502 in Phase 1.

### Rationale

These diagnostics cover move after move, multiple mutable borrow, and immutable-vs-mutable borrow conflicts. They are enough to prove the ownership event model without turning the project into a generic Rust diagnostics viewer.

### Alternatives Considered

- Include Priority A diagnostics in Phase 1: rejected because scope would expand before the event contract is proven.
- Include all captured diagnostics: rejected because ownership and non-ownership diagnostics require different explanatory models.

### 受け入れ条件

- [ ] Acceptance criteria distinguish mapping targets from compatibility fixtures.
- [ ] Non-Phase-1 diagnostics are retained as display-only.

## Static HTML First

### Decision

Generate a static HTML report in Phase 1.

### Rationale

Static HTML is easy to inspect, share, and snapshot-test. It avoids a dev server and keeps reporter concerns separate from the event model.

### Alternatives Considered

- Interactive web app: rejected for Phase 1 due to UI scope.
- VS Code extension first: rejected because input/model stability should be proven first.

### 受け入れ条件

- [ ] HTML output does not require a server.
- [ ] HTML-specific grouping does not appear in `OwnershipEvent`.

## Fixture Corpus Role

### Decision

Use captured JSONL as reference corpus for tests and compatibility, but keep normative behavior in spec, data model, and acceptance criteria.

### Rationale

rustc diagnostics may change across versions. Fixtures are evidence, not a permanent compiler contract.

### 受け入れ条件

- [ ] Fixture tests allow intentional updates when rustc output changes.
- [ ] Schema and mapper tests express expected behavior independently of raw fixture file names.
