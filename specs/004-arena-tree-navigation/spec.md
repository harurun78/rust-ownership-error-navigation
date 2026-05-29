# Feature Specification: Arena And Tree Navigation Suggestions

**Feature Branch**: `004-arena-tree-navigation`  
**Created**: 2026-05-29  
**Status**: Draft  
**Input**: Validation results from `sax-js-queued-events` and `domhandler-tree-builder`

## User Scenarios & Testing

### User Story 1 - Explain tree aliasing as design pressure (Priority: P1)

As a Rust learner porting object-graph code, I want borrow conflicts around parent/child tree mutation to suggest arena or stable ID storage, so I can avoid fighting direct `&mut` graph references.

**Independent Test**: Feed a synthetic or saved E0499 diagnostic where spans mention `parent.children`, `self.stack`, `Node`, or `root`; verify JSON and HTML include an `arena-backed-tree` or `stable-node-id` design suggestion with evidence.

### User Story 2 - Identify self-referential struct pressure (Priority: P1)

As a Rust learner encountering self-referential construction diagnostics, I want E0505/E0515 reports to explain why storing a reference to a field or local inside the returned object is not a normal Rust ownership shape.

**Independent Test**: Feed domhandler iteration-001 E0505/E0515 diagnostics; verify the report preserves the diagnostics and emits `avoid-self-referential-struct` guidance.

### User Story 3 - Render design translation in HTML (Priority: P2)

As a learner or porting agent, I want the HTML report to show a concise translation from source-language object graph concepts to Rust-native identity mechanisms.

**Independent Test**: Render a report containing arena/tree suggestions and verify HTML includes design translation language such as direct references, arena, stable IDs, parent links, and child lists.

## Requirements

- **FR-001**: The mapper SHALL emit deterministic tree-design suggestions for E0499/E0502 diagnostics when evidence mentions tree/object-graph pressure such as parent, child, children, node, root, stack, or arena-like mutation.
- **FR-002**: The mapper SHALL support suggestion kinds for `arena-backed-tree`, `stable-node-id`, and `avoid-self-referential-struct` without external LLM API calls.
- **FR-003**: The report SHALL include evidence for each emitted tree-design suggestion, including diagnostic code, rule ID, trigger term, and primary span when available.
- **FR-004**: E0505/E0515 diagnostics SHALL receive conservative design guidance when evidence indicates self-referential construction or returning values that borrow local state.
- **FR-005**: JSON output SHALL remain backward compatible by keeping `designSuggestions` optional.
- **FR-006**: HTML output SHALL render the new suggestions in a user-readable way and continue to work when no suggestions are present.
- **FR-007**: The implementation SHALL not generate patches or call external services.

## Non-Goals

- Full semantic tree analysis.
- Automated code transformation.
- rust-analyzer or MIR integration.
- Broad support for all lifetime diagnostics.
- UI redesign beyond report rendering refinements.

## Success Criteria

- E0499 diagnostics from DOM-like tree construction produce a design suggestion that mentions arena or stable node IDs.
- E0505/E0515 diagnostics from self-referential construction produce conservative guidance rather than only an unsupported message.
- Existing E0382/E0499/E0502/E0308 suggestion behavior remains unchanged.
- Full repository verification passes.
