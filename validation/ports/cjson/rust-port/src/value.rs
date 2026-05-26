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
