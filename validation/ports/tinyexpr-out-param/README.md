# tinyexpr Out-Param Validation

This target validates Rust ownership/navigation behavior on a tinyexpr-style C API.

The compatibility track preserves a C-shaped compile/evaluate API with an error-position out-parameter. The Rust-native track preserves behavior while returning `Result` values and owned expression trees.
