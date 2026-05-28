use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub enum Event<'input> {
    StartTag(&'input str),
    EndTag(&'input str),
    Text(&'input str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyTagName,
    InvalidTagName,
}

#[derive(Debug, Clone, Copy)]
enum QueuedEventKind {
    StartTag,
    EndTag,
    Text,
}

#[derive(Debug, Clone, Copy)]
struct QueuedEvent {
    kind: QueuedEventKind,
    start: usize,
    end: usize,
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
                let raw_name = &self.buffer[content_start..content_end];
                let (kind, name_start, name_end) = if raw_name.starts_with('/') {
                    (QueuedEventKind::EndTag, content_start + 1, content_end)
                } else {
                    (QueuedEventKind::StartTag, content_start, content_end)
                };

                validate_name(&self.buffer[name_start..name_end])?;
                self.queued.push_back(QueuedEvent {
                    kind,
                    start: name_start,
                    end: name_end,
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
            QueuedEventKind::StartTag => Event::StartTag(value),
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

        assert_eq!(parser.next_event(), Some(Event::StartTag("root")));
    }

    #[test]
    fn compatibility_parser_queues_text_before_next_tag() {
        let mut parser = SaxParser::new();

        parser.write("hello<root>").unwrap();

        assert_eq!(parser.next_event(), Some(Event::Text("hello")));
        assert_eq!(parser.next_event(), Some(Event::StartTag("root")));
    }

    #[test]
    fn compatibility_parser_queues_full_element() {
        let mut parser = SaxParser::new();

        parser.write("<root>hello</root>").unwrap();

        assert_eq!(parser.next_event(), Some(Event::StartTag("root")));
        assert_eq!(parser.next_event(), Some(Event::Text("hello")));
        assert_eq!(parser.next_event(), Some(Event::EndTag("root")));
    }

    #[test]
    fn compatibility_parser_rejects_invalid_tag_name() {
        let mut parser = SaxParser::new();

        assert_eq!(parser.write("<bad name>"), Err(ParseError::InvalidTagName));
    }
}
