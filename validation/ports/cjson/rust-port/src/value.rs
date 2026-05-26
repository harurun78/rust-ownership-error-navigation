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
    EmptyPath,
    MissingPath,
    NotArray,
    NotObject,
}

#[derive(Debug, PartialEq, Eq)]
pub enum JsonPointerError {
    InvalidPrefix,
    InvalidEscape,
    InvalidArrayIndex,
}

#[derive(Debug, PartialEq, Eq)]
pub enum JsonPatchError {
    InvalidPatchDocument,
    InvalidOperation,
    MissingOp,
    MissingPath,
    MissingValue,
    UnsupportedOperation,
    InvalidPointer(JsonPointerError),
    MissingTarget,
    CannotRemoveRoot,
    NotArray,
    NotObject,
    ArrayIndexOutOfBounds,
    InvalidArrayIndex,
}

pub fn parse_json_pointer(pointer: &str) -> Result<Vec<String>, JsonPointerError> {
    if pointer.is_empty() {
        return Ok(Vec::new());
    }

    if !pointer.starts_with('/') {
        return Err(JsonPointerError::InvalidPrefix);
    }

    let mut segments = Vec::new();
    for raw_segment in pointer[1..].split('/') {
        segments.push(decode_pointer_segment(raw_segment)?);
    }

    Ok(segments)
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

    pub fn to_pretty_string(&self) -> String {
        let mut output = String::new();
        self.write_pretty(&mut output, 0);
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

    fn write_pretty(&self, output: &mut String, depth: usize) {
        match self {
            JsonValue::Null => output.push_str("null"),
            JsonValue::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            JsonValue::Number(value) => output.push_str(&value.to_string()),
            JsonValue::String(value) => write_escaped_string(value, output),
            JsonValue::Array(values) => {
                if values.is_empty() {
                    output.push_str("[]");
                    return;
                }

                output.push('[');
                output.push('\n');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                        output.push('\n');
                    }
                    write_indent(output, depth + 1);
                    value.write_pretty(output, depth + 1);
                }
                output.push('\n');
                write_indent(output, depth);
                output.push(']');
            }
            JsonValue::Object(entries) => {
                if entries.is_empty() {
                    output.push_str("{}");
                    return;
                }

                output.push('{');
                output.push('\n');
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                        output.push('\n');
                    }
                    write_indent(output, depth + 1);
                    write_escaped_string(key, output);
                    output.push_str(": ");
                    value.write_pretty(output, depth + 1);
                }
                output.push('\n');
                write_indent(output, depth);
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

    pub fn apply_merge_patch(&mut self, patch: JsonValue) {
        let JsonValue::Object(patch_entries) = patch else {
            *self = patch;
            return;
        };

        if !self.is_object() {
            *self = JsonValue::Object(Vec::new());
        }

        let JsonValue::Object(target_entries) = self else {
            return;
        };

        for (key, patch_value) in patch_entries {
            if patch_value == JsonValue::Null {
                if let Some(index) = target_entries
                    .iter()
                    .position(|(entry_key, _)| entry_key == &key)
                {
                    target_entries.remove(index);
                }
                continue;
            }

            if let Some((_, target_value)) = target_entries
                .iter_mut()
                .find(|(entry_key, _)| entry_key == &key)
            {
                match patch_value {
                    JsonValue::Object(entries) if target_value.is_object() => {
                        target_value.apply_merge_patch(JsonValue::Object(entries));
                    }
                    value => {
                        *target_value = value;
                    }
                }
            } else {
                target_entries.push((key, patch_value));
            }
        }
    }

    pub fn detach_at_path(
        &mut self,
        path: &[JsonPathSegment<'_>],
    ) -> Result<Option<JsonValue>, JsonEditError> {
        let Some((terminal, parent_path)) = path.split_last() else {
            return Err(JsonEditError::EmptyPath);
        };

        let parent = self
            .get_path_mut(parent_path)
            .ok_or(JsonEditError::MissingPath)?;

        match terminal {
            JsonPathSegment::Index(index) => parent.detach_array_item(*index),
            JsonPathSegment::Key(key) => parent.detach_object_member(key),
        }
    }

    pub fn get_pointer(&self, pointer: &str) -> Result<Option<&JsonValue>, JsonPointerError> {
        let segments = parse_json_pointer(pointer)?;
        let mut current = self;

        for segment in &segments {
            current = match current {
                JsonValue::Array(values) => {
                    let index = segment
                        .parse::<usize>()
                        .map_err(|_| JsonPointerError::InvalidArrayIndex)?;
                    let Some(value) = values.get(index) else {
                        return Ok(None);
                    };
                    value
                }
                JsonValue::Object(entries) => {
                    let Some((_, value)) = entries.iter().find(|(key, _)| key == segment) else {
                        return Ok(None);
                    };
                    value
                }
                _ => return Ok(None),
            };
        }

        Ok(Some(current))
    }

    pub fn get_pointer_mut(
        &mut self,
        pointer: &str,
    ) -> Result<Option<&mut JsonValue>, JsonPointerError> {
        let segments = parse_json_pointer(pointer)?;
        let mut current = self;

        for segment in &segments {
            current = match current {
                JsonValue::Array(values) => {
                    let index = segment
                        .parse::<usize>()
                        .map_err(|_| JsonPointerError::InvalidArrayIndex)?;
                    let Some(value) = values.get_mut(index) else {
                        return Ok(None);
                    };
                    value
                }
                JsonValue::Object(entries) => {
                    let Some((_, value)) = entries.iter_mut().find(|(key, _)| key == segment)
                    else {
                        return Ok(None);
                    };
                    value
                }
                _ => return Ok(None),
            };
        }

        Ok(Some(current))
    }

    pub fn apply_json_patch(&mut self, patch: JsonValue) -> Result<(), JsonPatchError> {
        let JsonValue::Array(operations) = patch else {
            return Err(JsonPatchError::InvalidPatchDocument);
        };

        for operation in operations {
            let operation = JsonPatchOperation::from_value(operation)?;
            self.apply_json_patch_operation(operation)?;
        }

        Ok(())
    }

    fn apply_json_patch_operation(
        &mut self,
        operation: JsonPatchOperation,
    ) -> Result<(), JsonPatchError> {
        match operation.kind.as_str() {
            "add" => {
                let value = operation.value.ok_or(JsonPatchError::MissingValue)?;
                self.add_pointer_value(&operation.path, value)
            }
            "remove" => self.remove_pointer_value(&operation.path),
            "replace" => {
                let value = operation.value.ok_or(JsonPatchError::MissingValue)?;
                self.replace_pointer_value(&operation.path, value)
            }
            _ => Err(JsonPatchError::UnsupportedOperation),
        }
    }

    fn add_pointer_value(&mut self, pointer: &str, value: JsonValue) -> Result<(), JsonPatchError> {
        let segments = parse_json_pointer(pointer).map_err(JsonPatchError::InvalidPointer)?;
        let Some((terminal, parent_segments)) = segments.split_last() else {
            *self = value;
            return Ok(());
        };

        let parent = find_pointer_parent_mut(self, parent_segments)?;
        match parent {
            JsonValue::Array(values) => {
                if terminal == "-" {
                    values.push(value);
                    return Ok(());
                }

                let index = parse_patch_array_index(terminal)?;
                if index > values.len() {
                    return Err(JsonPatchError::ArrayIndexOutOfBounds);
                }
                values.insert(index, value);
                Ok(())
            }
            JsonValue::Object(entries) => {
                for (key, entry_value) in entries.iter_mut() {
                    if key == terminal {
                        *entry_value = value;
                        return Ok(());
                    }
                }
                entries.push((terminal.to_owned(), value));
                Ok(())
            }
            _ => Err(JsonPatchError::MissingTarget),
        }
    }

    fn remove_pointer_value(&mut self, pointer: &str) -> Result<(), JsonPatchError> {
        let segments = parse_json_pointer(pointer).map_err(JsonPatchError::InvalidPointer)?;
        let Some((terminal, parent_segments)) = segments.split_last() else {
            return Err(JsonPatchError::CannotRemoveRoot);
        };

        let parent = find_pointer_parent_mut(self, parent_segments)?;
        match parent {
            JsonValue::Array(values) => {
                let index = parse_patch_array_index(terminal)?;
                if index >= values.len() {
                    return Err(JsonPatchError::ArrayIndexOutOfBounds);
                }
                values.remove(index);
                Ok(())
            }
            JsonValue::Object(entries) => {
                let Some(index) = entries.iter().position(|(key, _)| key == terminal) else {
                    return Err(JsonPatchError::MissingTarget);
                };
                entries.remove(index);
                Ok(())
            }
            _ => Err(JsonPatchError::MissingTarget),
        }
    }

    fn replace_pointer_value(
        &mut self,
        pointer: &str,
        value: JsonValue,
    ) -> Result<(), JsonPatchError> {
        let target = self
            .get_pointer_mut(pointer)
            .map_err(JsonPatchError::InvalidPointer)?
            .ok_or(JsonPatchError::MissingTarget)?;
        *target = value;
        Ok(())
    }
}

struct JsonPatchOperation {
    kind: String,
    path: String,
    value: Option<JsonValue>,
}

impl JsonPatchOperation {
    fn from_value(value: JsonValue) -> Result<Self, JsonPatchError> {
        let JsonValue::Object(entries) = value else {
            return Err(JsonPatchError::InvalidOperation);
        };

        let mut kind = None;
        let mut path = None;
        let mut value = None;

        for (key, entry_value) in entries {
            match (key.as_str(), entry_value) {
                ("op", JsonValue::String(op)) => kind = Some(op),
                ("op", _) => return Err(JsonPatchError::MissingOp),
                ("path", JsonValue::String(pointer)) => path = Some(pointer),
                ("path", _) => return Err(JsonPatchError::MissingPath),
                ("value", patch_value) => value = Some(patch_value),
                _ => {}
            }
        }

        Ok(Self {
            kind: kind.ok_or(JsonPatchError::MissingOp)?,
            path: path.ok_or(JsonPatchError::MissingPath)?,
            value,
        })
    }
}

fn find_pointer_parent_mut<'a>(
    value: &'a mut JsonValue,
    segments: &[String],
) -> Result<&'a mut JsonValue, JsonPatchError> {
    let mut current = value;

    for segment in segments {
        current = match current {
            JsonValue::Array(values) => {
                let index = parse_patch_array_index(segment)?;
                values
                    .get_mut(index)
                    .ok_or(JsonPatchError::ArrayIndexOutOfBounds)?
            }
            JsonValue::Object(entries) => entries
                .iter_mut()
                .find(|(key, _)| key == segment)
                .map(|(_, value)| value)
                .ok_or(JsonPatchError::MissingTarget)?,
            _ => return Err(JsonPatchError::MissingTarget),
        };
    }

    Ok(current)
}

fn parse_patch_array_index(segment: &str) -> Result<usize, JsonPatchError> {
    segment
        .parse::<usize>()
        .map_err(|_| JsonPatchError::InvalidArrayIndex)
}

fn decode_pointer_segment(segment: &str) -> Result<String, JsonPointerError> {
    let mut decoded = String::new();
    let mut chars = segment.chars();

    while let Some(ch) = chars.next() {
        if ch != '~' {
            decoded.push(ch);
            continue;
        }

        match chars.next() {
            Some('0') => decoded.push('~'),
            Some('1') => decoded.push('/'),
            _ => return Err(JsonPointerError::InvalidEscape),
        }
    }

    Ok(decoded)
}

fn write_indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("  ");
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
