---
description: "Assess paired compatibility-preserving versus Rust-native porting validation results and summarize navigation repair/prevention value."
---

# Porting Comparison Assessment

Assess a completed paired porting slice.

## Inputs To Provide

- Target directory, for example `validation/ports/<target>`.
- Compatibility iteration ids.
- Rust-native iteration ids.
- Any human intervention notes.

## Required Steps

1. Read `notes/iteration-log.md`, `notes/comparison-matrix.md`, and report JSON files for both tracks.
2. Compare metrics:
   - first cargo-check diagnostics
   - ownership diagnostics
   - non-ownership diagnostics
   - repair iterations to pass tests
   - shortcut pressure events
   - API complexity notes
   - navigation changed next action
3. Separate conclusions into:
   - **repair value**: reports helped fix emitted diagnostics.
   - **prevention value**: Rust-native design avoided diagnostics or shortcut pressure.
4. Update `reports/comparison-summary.md`.
5. Recommend deterministic suggestion rules that should be added to the product, without requiring an LLM API call.

## Output

Return a concise summary of what the comparison proved, what remains unproven, and which suggestion rules are ready for implementation.