use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub struct Attribute<'input> {
    pub name: &'input str,
    pub value: &'input str,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event<'input> {
    StartTag {
        name: &'input str,
        attributes: Vec<Attribute<'input>>,
    },
    EndTag(&'input str),
    Text(&'input str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyTagName,
    InvalidTagName,
    InvalidAttribute,
    IncompleteAttribute,
}

#[derive(Debug, Clone, Copy)]
enum QueuedEventKind {
    StartTag,
    EndTag,
    Text,
}

#[derive(Debug, Clone)]
struct QueuedEvent {
    kind: QueuedEventKind,
    start: usize,
    end: usize,
    attributes: Vec<AttributeSpan>,
}

#[derive(Debug, Clone, Copy)]
struct AttributeSpan {
    name_start: usize,
    name_end: usize,
    value_start: usize,
    value_end: usize,
}

pub struct SaxParser {
    buffer: String,
    cursor: usize,
    queued: VecDeque<QueuedEvent>,
}

impl SaxParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            queued: VecDeque::new(),
        }
    }

    pub fn write(&mut self, input: &str) -> Result<usize, ParseError> {
        self.buffer.push_str(input);

        while self.cursor < self.buffer.len() {
            let remaining = &self.buffer[self.cursor..];

            if remaining.starts_with('<') {
                let Some(tag_end) = remaining.find('>') else {
                    break;
                };

                let content_start = self.cursor + 1;
                let content_end = self.cursor + tag_end;
                let content = &self.buffer[content_start..content_end];
                let (kind, name_start, name_end, attributes) = if content.starts_with('/') {
                    if content[1..].contains(char::is_whitespace) {
                        return Err(ParseError::InvalidTagName);
                    }
                    (
                        QueuedEventKind::EndTag,
                        content_start + 1,
                        content_end,
                        Vec::new(),
                    )
                } else {
                    parse_start_tag_content(&self.buffer, content_start, content_end)?
                };

                validate_name(&self.buffer[name_start..name_end])?;
                self.queued.push_back(QueuedEvent {
                    kind,
                    start: name_start,
                    end: name_end,
                    attributes,
                });
                self.cursor += tag_end + 1;
            } else {
                let text_end = remaining.find('<').unwrap_or(remaining.len());
                if text_end == 0 {
                    break;
                }

                let start = self.cursor;
                let end = self.cursor + text_end;
                self.queued.push_back(QueuedEvent {
                    kind: QueuedEventKind::Text,
                    start,
                    end,
                    attributes: Vec::new(),
                });
                self.cursor = end;
            }
        }

        Ok(input.len())
    }

    pub fn next_event(&mut self) -> Option<Event<'_>> {
        let queued = self.queued.pop_front()?;
        let value = &self.buffer[queued.start..queued.end];

        Some(match queued.kind {
            QueuedEventKind::StartTag => Event::StartTag {
                name: value,
                attributes: queued
                    .attributes
                    .iter()
                    .map(|attribute| Attribute {
                        name: &self.buffer[attribute.name_start..attribute.name_end],
                        value: &self.buffer[attribute.value_start..attribute.value_end],
                    })
                    .collect(),
            },
            QueuedEventKind::EndTag => Event::EndTag(value),
            QueuedEventKind::Text => Event::Text(value),
        })
    }
}

impl Default for SaxParser {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_start_tag_content(
    buffer: &str,
    content_start: usize,
    content_end: usize,
) -> Result<(QueuedEventKind, usize, usize, Vec<AttributeSpan>), ParseError> {
    let content = &buffer[content_start..content_end];
    let name_len = content.find(char::is_whitespace).unwrap_or(content.len());
    let name_start = content_start;
    let name_end = content_start + name_len;

    validate_name(&buffer[name_start..name_end])?;

    let mut attributes = Vec::new();
    let mut cursor = name_end;
    while cursor < content_end {
        while cursor < content_end && buffer.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= content_end {
            break;
        }

        let attribute_name_start = cursor;
        while cursor < content_end {
            let byte = buffer.as_bytes()[cursor];
            if byte == b'=' || byte.is_ascii_whitespace() {
                break;
            }
            cursor += 1;
        }
        let attribute_name_end = cursor;
        validate_name(&buffer[attribute_name_start..attribute_name_end])?;

        while cursor < content_end && buffer.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= content_end || buffer.as_bytes()[cursor] != b'=' {
            return Err(ParseError::InvalidAttribute);
        }
        cursor += 1;

        while cursor < content_end && buffer.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= content_end || buffer.as_bytes()[cursor] != b'"' {
            return Err(ParseError::InvalidAttribute);
        }
        cursor += 1;

        let value_start = cursor;
        while cursor < content_end && buffer.as_bytes()[cursor] != b'"' {
            cursor += 1;
        }
        if cursor >= content_end {
            return Err(ParseError::IncompleteAttribute);
        }
        let value_end = cursor;
        cursor += 1;

        attributes.push(AttributeSpan {
            name_start: attribute_name_start,
            name_end: attribute_name_end,
            value_start,
            value_end,
        });
    }

    Ok((QueuedEventKind::StartTag, name_start, name_end, attributes))
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
    fn compatibility_parser_queues_start_tag() {
        let mut parser = SaxParser::new();

        parser.write("<root>").unwrap();

        assert_eq!(
            parser.next_event(),
            Some(Event::StartTag {
                name: "root",
                attributes: Vec::new(),
            })
        );
    }

    #[test]
    fn compatibility_parser_queues_text_before_next_tag() {
        let mut parser = SaxParser::new();

        parser.write("hello<root>").unwrap();

        assert_eq!(parser.next_event(), Some(Event::Text("hello")));
        assert_eq!(
            parser.next_event(),
            Some(Event::StartTag {
                name: "root",
                attributes: Vec::new(),
            })
        );
    }

    #[test]
    fn compatibility_parser_queues_full_element() {
        let mut parser = SaxParser::new();

        parser.write("<root>hello</root>").unwrap();

        assert_eq!(
            parser.next_event(),
            Some(Event::StartTag {
                name: "root",
                attributes: Vec::new(),
            })
        );
        assert_eq!(parser.next_event(), Some(Event::Text("hello")));
        assert_eq!(parser.next_event(), Some(Event::EndTag("root")));
    }

    #[test]
    fn compatibility_parser_queues_attributes() {
        let mut parser = SaxParser::new();

        parser.write("<root id=\"main\" class=\"top\">").unwrap();

        assert_eq!(
            parser.next_event(),
            Some(Event::StartTag {
                name: "root",
                attributes: vec![
                    Attribute {
                        name: "id",
                        value: "main",
                    },
                    Attribute {
                        name: "class",
                        value: "top",
                    },
                ],
            })
        );
    }

    #[test]
    fn compatibility_parser_accepts_partial_incremental_tag() {
        let mut parser = SaxParser::new();

        parser.write("<ro").unwrap();
        assert_eq!(parser.next_event(), None);
        parser.write("ot id=\"main\">").unwrap();

        assert_eq!(
            parser.next_event(),
            Some(Event::StartTag {
                name: "root",
                attributes: vec![Attribute {
                    name: "id",
                    value: "main",
                }],
            })
        );
    }

    #[test]
    fn compatibility_parser_rejects_invalid_tag_name() {
        let mut parser = SaxParser::new();

        assert_eq!(parser.write("</bad name>"), Err(ParseError::InvalidTagName));
    }

    #[test]
    fn compatibility_parser_rejects_invalid_attribute() {
        let mut parser = SaxParser::new();

        assert_eq!(
            parser.write("<root id=main>"),
            Err(ParseError::InvalidAttribute)
        );
    }
}
