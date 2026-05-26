# cJSON Porting Validation

## Target

- Upstream repository: `DaveGamble/cJSON`
- Upstream version: `v1.7.19`
- Upstream commit: `c859b25da02955fef659d658b8f324b5cde87be3`
- Language: C
- Domain: JSON parser and serializer
- Initial status: selected as the first validation target

cJSON is a good first target because it is widely used, compact enough to reason about, and still contains ownership patterns that are difficult for Rust beginners and low-cost coding models.

## Why cJSON

The expected Rust migration pressure points are:

- tree ownership for arrays, objects, and child nodes
- parent/child lifetime boundaries
- string ownership versus borrowed string references
- partial parse failure cleanup
- allocation and deallocation responsibility across helper functions
- mutation while walking or constructing nested structures

These patterns are likely to produce ownership and borrowing diagnostics such as E0382, E0499, and E0502 during naive ports.

## Initial Porting Slice

Start with a small slice before attempting API compatibility:

1. Define a Rust JSON value tree model.
2. Port parsing for null, boolean, number, and string values.
3. Add arrays and objects after the scalar parser compiles cleanly.
4. Generate `cargo check --message-format=json` output for each failed iteration.
5. Run this repository's reporter against the JSONL output and save reports under `reports/`.

The detailed porting specification is [spec.md](spec.md). The implementation plan is [plan.md](plan.md), task breakdown is [tasks.md](tasks.md), and execution guide is [quickstart.md](quickstart.md). Upstream acquisition details are tracked in [upstream/UPSTREAM.md](upstream/UPSTREAM.md).

## Proposed Local Layout

```text
validation/ports/cjson/
  README.md
  upstream/      # checkout notes or source snapshot metadata
  rust-port/     # experimental Rust crate
  reports/       # generated JSON and HTML diagnostic reports
  notes/         # iteration logs and model observations
```

## Evaluation Questions

- Can a low-cost model reach a compiling scalar parser without human ownership guidance?
- Do generated reports reduce repeated E0382, E0499, or E0502 mistakes?
- Which C cleanup patterns are translated into `Drop`, `Result`, `Option`, or owned containers?
- Where does the model overuse `clone`, `Rc<RefCell<_>>`, or `unsafe`?
- How many iterations are needed before tests pass?

## Non-Goals For The First Pass

- Full cJSON API compatibility
- FFI compatibility with existing C callers
- Performance parity
- Zero-copy parsing
- Complete serializer support