#[derive(Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    StartTag {
        name: String,
        attributes: Vec<Attribute>,
    },
    EndTag(String),
    Text(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyTagName,
    InvalidTagName,
    InvalidAttribute,
    IncompleteAttribute,
    IncompleteTag,
}

pub fn parse_events(input: &str) -> Result<Vec<Event>, ParseError> {
    let mut events = Vec::new();
    let mut cursor = 0;

    while cursor < input.len() {
        let remaining = &input[cursor..];

        if remaining.starts_with('<') {
            let tag_end = remaining.find('>').ok_or(ParseError::IncompleteTag)?;
            let raw_name = &remaining[1..tag_end];
            let (is_end, name, attributes) = if let Some(name) = raw_name.strip_prefix('/') {
                if name.contains(char::is_whitespace) {
                    return Err(ParseError::InvalidTagName);
                }
                (true, name, Vec::new())
            } else {
                let (name, attributes) = parse_start_tag_content(raw_name)?;
                (false, name, attributes)
            };

            validate_name(name)?;
            if is_end {
                events.push(Event::EndTag(name.to_owned()));
            } else {
                events.push(Event::StartTag {
                    name: name.to_owned(),
                    attributes,
                });
            }

            cursor += tag_end + 1;
        } else {
            let text_end = remaining.find('<').unwrap_or(remaining.len());
            let text = &remaining[..text_end];
            if !text.is_empty() {
                events.push(Event::Text(text.to_owned()));
            }
            cursor += text_end;
        }
    }

    Ok(events)
}

fn parse_start_tag_content(content: &str) -> Result<(&str, Vec<Attribute>), ParseError> {
    let name_len = content.find(char::is_whitespace).unwrap_or(content.len());
    let name = &content[..name_len];
    validate_name(name)?;

    let mut attributes = Vec::new();
    let mut cursor = name_len;
    while cursor < content.len() {
        while cursor < content.len() && content.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= content.len() {
            break;
        }

        let attribute_name_start = cursor;
        while cursor < content.len() {
            let byte = content.as_bytes()[cursor];
            if byte == b'=' || byte.is_ascii_whitespace() {
                break;
            }
            cursor += 1;
        }
        let attribute_name = &content[attribute_name_start..cursor];
        validate_name(attribute_name)?;

        while cursor < content.len() && content.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= content.len() || content.as_bytes()[cursor] != b'=' {
            return Err(ParseError::InvalidAttribute);
        }
        cursor += 1;

        while cursor < content.len() && content.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= content.len() || content.as_bytes()[cursor] != b'"' {
            return Err(ParseError::InvalidAttribute);
        }
        cursor += 1;

        let value_start = cursor;
        while cursor < content.len() && content.as_bytes()[cursor] != b'"' {
            cursor += 1;
        }
        if cursor >= content.len() {
            return Err(ParseError::IncompleteAttribute);
        }
        let value = &content[value_start..cursor];
        cursor += 1;

        attributes.push(Attribute {
            name: attribute_name.to_owned(),
            value: value.to_owned(),
        });
    }

    Ok((name, attributes))
}

fn validate_name(name: &str) -> Result<(), ParseError> {
    if name.is_empty() {
        return Err(ParseError::EmptyTagName);
    }

    if name.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return Err(ParseError::InvalidTagName);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_native_parser_returns_owned_events() {
        let events = parse_events("<root>hello</root>").unwrap();

        assert_eq!(
            events,
            vec![
                Event::StartTag {
                    name: "root".to_owned(),
                    attributes: Vec::new(),
                },
                Event::Text("hello".to_owned()),
                Event::EndTag("root".to_owned()),
            ]
        );
    }

    #[test]
    fn rust_native_parser_rejects_empty_tag_name() {
        assert_eq!(parse_events("<>").unwrap_err(), ParseError::EmptyTagName);
    }

    #[test]
    fn rust_native_parser_rejects_whitespace_tag_name() {
        assert_eq!(
            parse_events("</bad name>").unwrap_err(),
            ParseError::InvalidTagName
        );
    }

    #[test]
    fn rust_native_parser_returns_owned_attributes() {
        let events = parse_events("<root id=\"main\" class=\"top\">").unwrap();

        assert_eq!(
            events,
            vec![Event::StartTag {
                name: "root".to_owned(),
                attributes: vec![
                    Attribute {
                        name: "id".to_owned(),
                        value: "main".to_owned(),
                    },
                    Attribute {
                        name: "class".to_owned(),
                        value: "top".to_owned(),
                    },
                ],
            }]
        );
    }

    #[test]
    fn rust_native_parser_reports_incomplete_partial_tag() {
        assert_eq!(
            parse_events("<root id=\"main\"").unwrap_err(),
            ParseError::IncompleteTag
        );
    }

    #[test]
    fn rust_native_parser_rejects_invalid_attribute() {
        assert_eq!(
            parse_events("<root id=main>").unwrap_err(),
            ParseError::InvalidAttribute
        );
    }
}
