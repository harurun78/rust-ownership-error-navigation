# Data Model: Rust Ownership Diagnostic Report MVP

## CargoMessage

### スコープ

Represents one line from `cargo check --message-format json`.

### Fields

| field        | type            | required | notes                                 |
| ------------ | --------------- | -------: | ------------------------------------- |
| `reason`     | string          |      yes | Phase 1 processes `compiler-message`. |
| `package_id` | string          |       no | Preserve when present.                |
| `target`     | unknown         |       no | Preserve as metadata if needed.       |
| `message`    | RustcDiagnostic |       no | Present for compiler messages.        |

### 受け入れ条件

- [ ] Non-compiler messages do not crash parser.
- [ ] Compiler messages expose `message` for downstream normalization.

## RustcDiagnostic

### スコープ

Preserves rustc diagnostic payload before project-specific mapping.

### Fields

| field      | type                                  | required | notes                                          |
| ---------- | ------------------------------------- | -------: | ---------------------------------------------- |
| `code`     | `{ code: string; explanation?: string \| null } \| null` |      yes | Some diagnostics have null code.              |
| `level`    | string                                |      yes | Usually `error`, `warning`, `note`, or `help`. |
| `message`  | string                                |      yes | Human diagnostic message.                      |
| `spans`    | RustcSpan[]                           |      yes | Source spans from rustc.                       |
| `children` | RustcDiagnostic[]                     |      yes | Child notes/help/suggestions.                  |
| `rendered` | string                                |       no | Human rendered diagnostic.                     |

### 受け入れ条件

- [ ] Null code diagnostics are preserved.
- [ ] Child diagnostics are recursively preserved.

## RustcSpan

### スコープ

Represents a raw rustc span with source location and suggestions.

### Fields

| field                         | type            | required | notes                                       |
| ----------------------------- | --------------- | -------: | ------------------------------------------- |
| `file_name`                   | string          |      yes | May be relative to cargo working directory. |
| `byte_start` / `byte_end`     | number          |      yes | Byte offsets from rustc.                    |
| `line_start` / `line_end`     | number          |      yes | 1-based rustc lines.                        |
| `column_start` / `column_end` | number          |      yes | 1-based rustc columns.                      |
| `is_primary`                  | boolean         |      yes | Primary span flag.                          |
| `label`                       | string or null  |      yes | Span label.                                 |
| `text`                        | array           |      yes | Source text snippets.                       |
| `suggested_replacement`       | string or null  |       no | Structured suggestion.                      |
| `suggestion_applicability`    | string or null  |       no | Suggestion applicability.                   |
| `expansion`                   | unknown or null |       no | Macro expansion metadata.                   |

### 受け入れ条件

- [ ] Line and column values remain 1-based in core output.
- [ ] Expansion presence can be detected for confidence adjustment.

## DiagnosticSpan

### スコープ

L0 normalized span for UI and mapper use.

### Fields

| field                       | type           | required | notes                                                      |
| --------------------------- | -------------- | -------: | ---------------------------------------------------------- |
| `id`                        | string         |      yes | Stable within report.                                      |
| `diagnosticId`              | string         |      yes | Parent diagnostic.                                         |
| `role`                      | enum           |      yes | `conflict`, `cause`, `context`, `possible_fix`, `unknown`. |
| `file`                      | string         |      yes | Source file name/path.                                     |
| `lineStart` / `lineEnd`     | number         |      yes | 1-based.                                                   |
| `columnStart` / `columnEnd` | number         |      yes | 1-based.                                                   |
| `byteStart` / `byteEnd`     | number         |       no | Optional byte offsets.                                     |
| `isPrimary`                 | boolean        |      yes | From rustc span.                                           |
| `label`                     | string or null |       no | Preserved label.                                           |
| `snippet`                   | string         |       no | Source snippet.                                            |
| `suggestedReplacement`      | string or null |       no | Suggestion.                                                |
| `suggestionApplicability`   | string or null |       no | Applicability.                                             |
| `hasExpansion`              | boolean        |      yes | True when rustc expansion metadata exists.                 |
| `evidence`                  | Evidence[]     |      yes | Why role was assigned.                                     |
| `confidence`                | Confidence     |      yes | `high`, `medium`, or `low`.                                |

### 受け入れ条件

- [ ] Every DiagnosticSpan has role and confidence.
- [ ] Unknown spans are retained.

## OwnershipEvent

### スコープ

L1 event used by JSON and HTML reporters, and future VS Code integration.

### Event Kinds

Phase 1 mapper must produce: `move`, `borrow_shared`, `borrow_mut`, `use`, `conflict`, `possible_fix`, `context`, `unknown`.

Schema may reserve: `declare`, `move_out`, `copy`, `borrow_mut_request`, `assign`, `temporary`, `escape`, `closure_capture`, `partial_move`, `receiver_move`, `implicit_into_iter`, `async_send_boundary`, `static_requirement`, `drop`.

### Fields

| field          | type                | required | notes                       |
| -------------- | ------------------- | -------: | --------------------------- |
| `id`           | string              |      yes | Stable within report.       |
| `diagnosticId` | string              |      yes | Parent diagnostic.          |
| `kind`         | enum                |      yes | Event kind.                 |
| `role`         | DiagnosticSpan role |      yes | Role inherited or derived.  |
| `place`        | string              |       no | Variable/place if inferred. |
| `spanId`       | string              |      yes | Source span.                |
| `message`      | string              |      yes | Event-level label.          |
| `evidence`     | Evidence[]          |      yes | Mapping evidence.           |
| `confidence`   | Confidence          |      yes | Mapping confidence.         |

### 受け入れ条件

- [ ] Events never exist without evidence.
- [ ] Reporter-specific labels are not stored as model-only truth.

## DiagnosticReport

### スコープ

JSON reporter root object.

### Fields

| field           | type   | required | notes                                  |
| --------------- | ------ | -------: | -------------------------------------- |
| `schemaVersion` | string |      yes | Starts at `0.1.0`.                     |
| `input`         | object |      yes | Source file and optional metadata.     |
| `summary`       | object |      yes | Diagnostic counts.                     |
| `diagnostics`   | array  |      yes | Supported and unsupported diagnostics. |

### 受け入れ条件

- [ ] Supported diagnostics contain events.
- [ ] Unsupported diagnostics preserve display-only payload.

## Evidence And Confidence

### Evidence Sources

- `diagnostic_code`
- `rustc_primary_span`
- `rustc_span_label`
- `rustc_child_diagnostic`
- `rustc_suggestion`
- `rustc_span_text`
- `rustc_expansion`
- `heuristic`

### Confidence

- `high`: direct diagnostic code plus explicit span label or suggestion.
- `medium`: code and primary/secondary relationship but weak label.
- `low`: heuristic or expansion-heavy inference.

### 受け入れ条件

- [ ] Confidence is one of `high`, `medium`, `low`.
- [ ] Evidence records include source and field.
