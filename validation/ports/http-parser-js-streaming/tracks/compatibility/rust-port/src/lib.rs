#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserType {
    Request,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    Complete,
    Incomplete,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnsupportedParserType,
    MalformedRequestLine,
    MalformedHeader,
    InvalidContentLength,
    MalformedChunk,
    UnsupportedTransferEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header<'input> {
    pub name: &'input str,
    pub value: &'input str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadersComplete<'input> {
    pub method: &'input str,
    pub path: &'input str,
    pub version: &'input str,
    pub headers: Vec<Header<'input>>,
    pub content_length: Option<usize>,
    pub chunked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyChunk<'input> {
    pub data: &'input str,
    pub is_final: bool,
}

pub type HeadersCompleteCallback<'callback> =
    Box<dyn for<'input> FnMut(&HeadersComplete<'input>) -> ParseStatus + 'callback>;
pub type BodyCallback<'callback> =
    Box<dyn for<'input> FnMut(&BodyChunk<'input>) -> ParseStatus + 'callback>;

pub struct HttpParser<'callback> {
    parser_type: ParserType,
    on_headers_complete: Option<HeadersCompleteCallback<'callback>>,
    on_body: Option<BodyCallback<'callback>>,
    buffer: String,
    paused: bool,
    completed: bool,
    bytes_consumed: usize,
}

impl<'callback> HttpParser<'callback> {
    pub fn new(parser_type: ParserType) -> Self {
        Self {
            parser_type,
            on_headers_complete: None,
            on_body: None,
            buffer: String::new(),
            paused: false,
            completed: false,
            bytes_consumed: 0,
        }
    }

    pub fn on_headers_complete(&mut self, callback: HeadersCompleteCallback<'callback>) {
        self.on_headers_complete = Some(callback);
    }

    pub fn on_body(&mut self, callback: BodyCallback<'callback>) {
        self.on_body = Some(callback);
    }

    pub fn execute(&mut self, input: &str) -> Result<ParseStatus, ParseError> {
        if self.parser_type != ParserType::Request {
            return Err(ParseError::UnsupportedParserType);
        }

        self.buffer.push_str(input);
        let Some(head_end) = self.buffer.find("\r\n\r\n") else {
            return Ok(ParseStatus::Incomplete);
        };
        let body_start = head_end + 4;
        let event = parse_request_head(&self.buffer[..head_end])?;

        if let Some(callback) = self.on_headers_complete.as_mut() {
            let status = callback(&event);
            self.paused = status == ParseStatus::Paused;
            if self.paused {
                self.bytes_consumed = body_start;
                return Ok(status);
            }
        }

        let complete_at = if event.chunked {
            let (chunks, consumed) = parse_chunked_body(&self.buffer[body_start..])?;
            deliver_body_chunks(&mut self.on_body, &chunks)?;
            body_start + consumed
        } else if let Some(content_length) = event.content_length {
            let body_end = body_start + content_length;
            if self.buffer.len() < body_end {
                return Ok(ParseStatus::Incomplete);
            }
            let chunk = BodyChunk {
                data: &self.buffer[body_start..body_end],
                is_final: true,
            };
            deliver_body_chunks(&mut self.on_body, &[chunk])?;
            body_end
        } else {
            body_start
        };

        self.bytes_consumed = complete_at;
        self.completed = true;
        Ok(ParseStatus::Complete)
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_complete(&self) -> bool {
        self.completed
    }

    pub fn bytes_consumed(&self) -> usize {
        self.bytes_consumed
    }
}

fn parse_request_head(input: &str) -> Result<HeadersComplete<'_>, ParseError> {
    let mut lines = input.split("\r\n");
    let request_line = lines.next().ok_or(ParseError::MalformedRequestLine)?;
    let mut parts = request_line.split_ascii_whitespace();
    let method = parts.next().ok_or(ParseError::MalformedRequestLine)?;
    let path = parts.next().ok_or(ParseError::MalformedRequestLine)?;
    let version = parts.next().ok_or(ParseError::MalformedRequestLine)?;

    if parts.next().is_some() || !version.starts_with("HTTP/") {
        return Err(ParseError::MalformedRequestLine);
    }

    let headers = lines
        .filter(|line| !line.is_empty())
        .map(parse_header)
        .collect::<Result<Vec<_>, _>>()?;
    let content_length = content_length(&headers)?;
    let chunked = is_chunked(&headers)?;

    Ok(HeadersComplete {
        method,
        path,
        version,
        headers,
        content_length,
        chunked,
    })
}

fn parse_header(line: &str) -> Result<Header<'_>, ParseError> {
    let Some((name, value)) = line.split_once(':') else {
        return Err(ParseError::MalformedHeader);
    };
    let name = name.trim();
    let value = value.trim();

    if name.is_empty() {
        return Err(ParseError::MalformedHeader);
    }

    Ok(Header { name, value })
}

fn content_length(headers: &[Header<'_>]) -> Result<Option<usize>, ParseError> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("content-length"))
        .map(|header| {
            header
                .value
                .parse::<usize>()
                .map_err(|_| ParseError::InvalidContentLength)
        })
        .transpose()
}

fn is_chunked(headers: &[Header<'_>]) -> Result<bool, ParseError> {
    let Some(header) = headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("transfer-encoding"))
    else {
        return Ok(false);
    };

    if header.value.eq_ignore_ascii_case("chunked") {
        return Ok(true);
    }

    Err(ParseError::UnsupportedTransferEncoding)
}

fn parse_chunked_body(input: &str) -> Result<(Vec<BodyChunk<'_>>, usize), ParseError> {
    let mut offset = 0;
    let mut chunks = Vec::new();

    loop {
        let Some(line_end) = input[offset..].find("\r\n") else {
            return Err(ParseError::MalformedChunk);
        };
        let size_text = &input[offset..offset + line_end];
        let size = usize::from_str_radix(size_text, 16).map_err(|_| ParseError::MalformedChunk)?;
        offset += line_end + 2;

        if size == 0 {
            if input[offset..].starts_with("\r\n") {
                return Ok((chunks, offset + 2));
            }

            return Err(ParseError::MalformedChunk);
        }

        let data_end = offset + size;
        if input.len() < data_end + 2 || &input[data_end..data_end + 2] != "\r\n" {
            return Err(ParseError::MalformedChunk);
        }

        chunks.push(BodyChunk {
            data: &input[offset..data_end],
            is_final: false,
        });
        offset = data_end + 2;
    }
}

fn deliver_body_chunks(
    callback: &mut Option<BodyCallback<'_>>,
    chunks: &[BodyChunk<'_>],
) -> Result<(), ParseError> {
    let Some(callback) = callback.as_mut() else {
        return Ok(());
    };

    for (index, chunk) in chunks.iter().enumerate() {
        let event = BodyChunk {
            data: chunk.data,
            is_final: index + 1 == chunks.len(),
        };
        if callback(&event) == ParseStatus::Paused {
            return Ok(());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_parser_invokes_headers_complete_callback() {
        let input = "GET /chat HTTP/1.1\r\nHost: example.test\r\nConnection: keep-alive\r\n\r\n";
        let mut seen = Vec::new();
        let mut parser = HttpParser::new(ParserType::Request);
        parser.on_headers_complete(Box::new(|event| {
            seen.push(format!(
                "{} {} {} {}",
                event.method,
                event.path,
                event.version,
                event.headers.len()
            ));
            ParseStatus::Complete
        }));

        assert_eq!(parser.execute(input), Ok(ParseStatus::Complete));
        assert_eq!(parser.bytes_consumed(), input.len());
        assert!(parser.is_complete());
        drop(parser);
        assert_eq!(seen, vec!["GET /chat HTTP/1.1 2"]);
    }

    #[test]
    fn compatibility_parser_can_pause_from_callback() {
        let input = "GET / HTTP/1.1\r\nHost: example.test\r\n\r\n";
        let mut parser = HttpParser::new(ParserType::Request);
        parser.on_headers_complete(Box::new(|_| ParseStatus::Paused));

        assert_eq!(parser.execute(input), Ok(ParseStatus::Paused));
        assert!(parser.is_paused());
    }

    #[test]
    fn compatibility_parser_reports_incomplete_request_head() {
        let mut parser = HttpParser::new(ParserType::Request);

        assert_eq!(
            parser.execute("GET / HTTP/1.1\r\nHost: example.test"),
            Ok(ParseStatus::Incomplete)
        );
        assert!(!parser.is_complete());
    }

    #[test]
    fn compatibility_parser_rejects_malformed_header() {
        let mut parser = HttpParser::new(ParserType::Request);

        assert_eq!(
            parser.execute("GET / HTTP/1.1\r\nHost example.test\r\n\r\n"),
            Err(ParseError::MalformedHeader)
        );
    }

    #[test]
    fn compatibility_parser_delivers_content_length_body() {
        let input =
            "POST /submit HTTP/1.1\r\nHost: example.test\r\nContent-Length: 11\r\n\r\nhello world";
        let mut chunks = Vec::new();
        let mut parser = HttpParser::new(ParserType::Request);
        parser.on_body(Box::new(|chunk| {
            chunks.push(format!("{}:{}", chunk.data, chunk.is_final));
            ParseStatus::Complete
        }));

        assert_eq!(parser.execute(input), Ok(ParseStatus::Complete));
        assert_eq!(parser.bytes_consumed(), input.len());
        drop(parser);
        assert_eq!(chunks, vec!["hello world:true"]);
    }

    #[test]
    fn compatibility_parser_reports_incomplete_content_length_body() {
        let mut parser = HttpParser::new(ParserType::Request);

        assert_eq!(
            parser.execute("POST /submit HTTP/1.1\r\nContent-Length: 11\r\n\r\nhello"),
            Ok(ParseStatus::Incomplete)
        );
    }

    #[test]
    fn compatibility_parser_delivers_chunked_body() {
        let input = "POST /upload HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
        let mut chunks = Vec::new();
        let mut parser = HttpParser::new(ParserType::Request);
        parser.on_body(Box::new(|chunk| {
            chunks.push(format!("{}:{}", chunk.data, chunk.is_final));
            ParseStatus::Complete
        }));

        assert_eq!(parser.execute(input), Ok(ParseStatus::Complete));
        drop(parser);
        assert_eq!(chunks, vec!["Wiki:false", "pedia:true"]);
    }

    #[test]
    fn compatibility_parser_rejects_malformed_chunked_body() {
        let mut parser = HttpParser::new(ParserType::Request);

        assert_eq!(
            parser.execute(
                "POST /upload HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\nZ\r\noops\r\n"
            ),
            Err(ParseError::MalformedChunk)
        );
    }
}
