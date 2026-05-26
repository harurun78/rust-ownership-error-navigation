//! Scalar-only cJSON porting validation crate.

pub mod error;
pub mod parser;
pub mod value;

pub use error::ParseError;
pub use parser::parse_scalar;
pub use value::JsonValue;
