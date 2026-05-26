# Validation Workspace

This directory contains porting experiments used to evaluate whether ownership-error navigation can lower the cost of moving C/C++ code to Rust.

The workspace is organized by target project:

```text
validation/
  ports/
    <project>/
      README.md
      upstream/      # optional source snapshots or checkout notes
      rust-port/     # optional Rust port workspace
      reports/       # generated diagnostic reports
      notes/         # experiment logs and model observations
```

## Evaluation Focus

Each porting experiment should capture:

- the original C/C++ ownership model and cleanup responsibilities
- the intended Rust ownership model
- `cargo check --message-format=json` output from failed Rust iterations
- generated JSON and HTML diagnostic reports
- model iteration count, repeated errors, and human intervention points
- places where the model falls back to `unsafe`, cloning, or broad shared mutability

## Candidate Selection

Prefer targets that are:

- widely used in real systems
- small enough to port in slices
- rich in ownership, allocation, buffer, tree, callback, or lifecycle constraints
- likely to produce E0382, E0499, and E0502 during naive Rust migration

The first target is cJSON because it is compact, widely used, and has clear tree ownership and cleanup behavior.