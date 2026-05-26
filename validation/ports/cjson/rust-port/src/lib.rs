//! cJSON porting validation crate for scalar, array, and object values.

pub mod error;
pub mod parser;
pub mod value;

pub use error::ParseError;
pub use parser::parse_scalar;
pub use value::{JsonEditError, JsonPathSegment, JsonValue};
