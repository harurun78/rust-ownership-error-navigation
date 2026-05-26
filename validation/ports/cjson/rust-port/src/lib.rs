//! cJSON porting validation crate for scalar, array, and object values.

pub mod error;
pub mod minify;
pub mod parser;
pub mod value;

pub use error::ParseError;
pub use minify::{minify_json, MinifyError};
pub use parser::parse_scalar;
pub use value::{parse_json_pointer, JsonEditError, JsonPathSegment, JsonPointerError, JsonValue};
