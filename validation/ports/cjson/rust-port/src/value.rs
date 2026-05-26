#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JsonPathSegment<'a> {
    Key(&'a str),
    Index(usize),
}

#[derive(Debug, PartialEq)]
pub enum JsonEditError {
    NotArray,
    NotObject,
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            JsonValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_number_mut(&mut self) -> Option<&mut f64> {
        match self {
            JsonValue::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        match self {
            JsonValue::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
        match self {
            JsonValue::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        match self {
            JsonValue::Object(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut Vec<(String, JsonValue)>> {
        match self {
            JsonValue::Object(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn array_item(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(values) => values.get(index),
            _ => None,
        }
    }

    pub fn array_item_mut(&mut self, index: usize) -> Option<&mut JsonValue> {
        match self {
            JsonValue::Array(values) => values.get_mut(index),
            _ => None,
        }
    }

    pub fn object_member(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(entries) => entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn object_member_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        match self {
            JsonValue::Object(entries) => entries
                .iter_mut()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn to_compact_string(&self) -> String {
        let mut output = String::new();
        self.write_compact(&mut output);
        output
    }

    fn write_compact(&self, output: &mut String) {
        match self {
            JsonValue::Null => output.push_str("null"),
            JsonValue::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            JsonValue::Number(value) => output.push_str(&value.to_string()),
            JsonValue::String(value) => write_escaped_string(value, output),
            JsonValue::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    value.write_compact(output);
                }
                output.push(']');
            }
            JsonValue::Object(entries) => {
                output.push('{');
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    write_escaped_string(key, output);
                    output.push(':');
                    value.write_compact(output);
                }
                output.push('}');
            }
        }
    }

    pub fn append_array(&mut self, value: JsonValue) -> Result<(), JsonEditError> {
        match self {
            JsonValue::Array(values) => {
                values.push(value);
                Ok(())
            }
            _ => Err(JsonEditError::NotArray),
        }
    }

    pub fn insert_object_member(
        &mut self,
        key: String,
        value: JsonValue,
    ) -> Result<Option<JsonValue>, JsonEditError> {
        match self {
            JsonValue::Object(entries) => {
                for (entry_key, entry_value) in entries.iter_mut() {
                    if entry_key == &key {
                        let old_value = std::mem::replace(entry_value, value);
                        return Ok(Some(old_value));
                    }
                }

                entries.push((key, value));
                Ok(None)
            }
            _ => Err(JsonEditError::NotObject),
        }
    }

    pub fn detach_array_item(&mut self, index: usize) -> Result<Option<JsonValue>, JsonEditError> {
        match self {
            JsonValue::Array(values) => {
                if index < values.len() {
                    Ok(Some(values.remove(index)))
                } else {
                    Ok(None)
                }
            }
            _ => Err(JsonEditError::NotArray),
        }
    }

    pub fn detach_object_member(&mut self, key: &str) -> Result<Option<JsonValue>, JsonEditError> {
        match self {
            JsonValue::Object(entries) => {
                if let Some(index) = entries.iter().position(|(entry_key, _)| entry_key == key) {
                    let (_, value) = entries.remove(index);
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            _ => Err(JsonEditError::NotObject),
        }
    }

    pub fn get_path<'a>(&'a self, path: &[JsonPathSegment<'_>]) -> Option<&'a JsonValue> {
        let mut current = self;

        for segment in path {
            current = match (current, segment) {
                (JsonValue::Array(values), JsonPathSegment::Index(index)) => values.get(*index)?,
                (JsonValue::Object(entries), JsonPathSegment::Key(key)) => entries
                    .iter()
                    .find(|(entry_key, _)| entry_key == key)
                    .map(|(_, value)| value)?,
                _ => return None,
            };
        }

        Some(current)
    }

    pub fn get_path_mut<'a>(
        &'a mut self,
        path: &[JsonPathSegment<'_>],
    ) -> Option<&'a mut JsonValue> {
        let mut current = self;

        for segment in path {
            current = match (current, segment) {
                (JsonValue::Array(values), JsonPathSegment::Index(index)) => {
                    values.get_mut(*index)?
                }
                (JsonValue::Object(entries), JsonPathSegment::Key(key)) => entries
                    .iter_mut()
                    .find(|(entry_key, _)| entry_key == key)
                    .map(|(_, value)| value)?,
                _ => return None,
            };
        }

        Some(current)
    }

    pub fn replace_at_path(
        &mut self,
        path: &[JsonPathSegment<'_>],
        value: JsonValue,
    ) -> Option<JsonValue> {
        self.get_path_mut(path)
            .map(|target| std::mem::replace(target, value))
    }
}

fn write_escaped_string(value: &str, output: &mut String) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{001f}' => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output.push('"');
}
