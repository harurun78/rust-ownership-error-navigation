#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    StartTag(String),
    EndTag(String),
    Text(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyTagName,
    InvalidTagName,
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
            let (is_end, name) = if let Some(name) = raw_name.strip_prefix('/') {
                (true, name)
            } else {
                (false, raw_name)
            };

            validate_name(name)?;
            if is_end {
                events.push(Event::EndTag(name.to_owned()));
            } else {
                events.push(Event::StartTag(name.to_owned()));
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
                Event::StartTag("root".to_owned()),
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
            parse_events("<bad name>").unwrap_err(),
            ParseError::InvalidTagName
        );
    }
}
