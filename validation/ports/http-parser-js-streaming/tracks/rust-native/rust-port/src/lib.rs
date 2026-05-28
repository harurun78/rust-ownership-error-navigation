#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Incomplete,
    MalformedRequestLine,
    MalformedHeader,
    InvalidContentLength,
    MalformedChunk,
    UnsupportedTransferEncoding,
    AlreadyComplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: String,
}

#[derive(Debug, Default)]
pub struct RequestParser {
    buffer: String,
    complete: bool,
}

impl RequestParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &str) -> Result<Option<Request>, ParseError> {
        if self.complete {
            return Err(ParseError::AlreadyComplete);
        }

        self.buffer.push_str(chunk);
        let Some(head_end) = self.buffer.find("\r\n\r\n") else {
            return Ok(None);
        };
        let (request, complete_at) = parse_buffered_request(&self.buffer, head_end)?;
        self.complete = true;
        self.buffer.drain(..complete_at);

        Ok(Some(request))
    }
}

pub fn parse_request(input: &str) -> Result<Request, ParseError> {
    let mut parser = RequestParser::new();
    parser.push(input)?.ok_or(ParseError::Incomplete)
}

fn parse_buffered_request(input: &str, head_end: usize) -> Result<(Request, usize), ParseError> {
    let head = parse_request_head(&input[..head_end])?;
    let body_start = head_end + 4;

    if head.chunked {
        let (body, consumed) = parse_chunked_body(&input[body_start..])?;
        return Ok((head.into_request(body), body_start + consumed));
    }

    let Some(content_length) = head.content_length else {
        return Ok((head.into_request(String::new()), body_start));
    };
    let body_end = body_start + content_length;
    if input.len() < body_end {
        return Err(ParseError::Incomplete);
    }

    Ok((
        head.into_request(input[body_start..body_end].to_owned()),
        body_end,
    ))
}

struct RequestHead {
    method: String,
    path: String,
    version: String,
    headers: Vec<Header>,
    content_length: Option<usize>,
    chunked: bool,
}

impl RequestHead {
    fn into_request(self, body: String) -> Request {
        Request {
            method: self.method,
            path: self.path,
            version: self.version,
            headers: self.headers,
            body,
        }
    }
}

fn parse_request_head(input: &str) -> Result<RequestHead, ParseError> {
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

    Ok(RequestHead {
        method: method.to_owned(),
        path: path.to_owned(),
        version: version.to_owned(),
        headers,
        content_length,
        chunked,
    })
}

fn parse_header(line: &str) -> Result<Header, ParseError> {
    let Some((name, value)) = line.split_once(':') else {
        return Err(ParseError::MalformedHeader);
    };
    let name = name.trim();
    let value = value.trim();

    if name.is_empty() {
        return Err(ParseError::MalformedHeader);
    }

    Ok(Header {
        name: name.to_owned(),
        value: value.to_owned(),
    })
}

fn content_length(headers: &[Header]) -> Result<Option<usize>, ParseError> {
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

fn is_chunked(headers: &[Header]) -> Result<bool, ParseError> {
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

fn parse_chunked_body(input: &str) -> Result<(String, usize), ParseError> {
    let mut offset = 0;
    let mut body = String::new();

    loop {
        let Some(line_end) = input[offset..].find("\r\n") else {
            return Err(ParseError::MalformedChunk);
        };
        let size_text = &input[offset..offset + line_end];
        let size = usize::from_str_radix(size_text, 16).map_err(|_| ParseError::MalformedChunk)?;
        offset += line_end + 2;

        if size == 0 {
            if input[offset..].starts_with("\r\n") {
                return Ok((body, offset + 2));
            }

            return Err(ParseError::MalformedChunk);
        }

        let data_end = offset + size;
        if input.len() < data_end + 2 || &input[data_end..data_end + 2] != "\r\n" {
            return Err(ParseError::MalformedChunk);
        }

        body.push_str(&input[offset..data_end]);
        offset = data_end + 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_native_parser_returns_owned_request() {
        let request = parse_request(
            "GET /chat HTTP/1.1\r\nHost: example.test\r\nConnection: keep-alive\r\n\r\n",
        )
        .unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/chat");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers[0].name, "Host");
        assert_eq!(request.headers[0].value, "example.test");
        assert_eq!(request.body, "");
    }

    #[test]
    fn rust_native_parser_accepts_incremental_input() {
        let mut parser = RequestParser::new();

        assert_eq!(parser.push("GET / HTTP/1.1\r\n"), Ok(None));
        assert_eq!(
            parser
                .push("Host: example.test\r\n\r\n")
                .unwrap()
                .unwrap()
                .path,
            "/"
        );
    }

    #[test]
    fn rust_native_parser_reports_incomplete_request_head() {
        assert_eq!(
            parse_request("GET / HTTP/1.1\r\nHost: example.test"),
            Err(ParseError::Incomplete)
        );
    }

    #[test]
    fn rust_native_parser_rejects_malformed_header() {
        assert_eq!(
            parse_request("GET / HTTP/1.1\r\nHost example.test\r\n\r\n"),
            Err(ParseError::MalformedHeader)
        );
    }

    #[test]
    fn rust_native_parser_returns_content_length_body() {
        let request = parse_request(
            "POST /submit HTTP/1.1\r\nHost: example.test\r\nContent-Length: 11\r\n\r\nhello world",
        )
        .unwrap();

        assert_eq!(request.method, "POST");
        assert_eq!(request.body, "hello world");
    }

    #[test]
    fn rust_native_parser_reports_incomplete_content_length_body() {
        assert_eq!(
            parse_request("POST /submit HTTP/1.1\r\nContent-Length: 11\r\n\r\nhello"),
            Err(ParseError::Incomplete)
        );
    }

    #[test]
    fn rust_native_parser_decodes_chunked_body() {
        let request = parse_request(
            "POST /upload HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n",
        )
        .unwrap();

        assert_eq!(request.body, "Wikipedia");
    }

    #[test]
    fn rust_native_parser_rejects_malformed_chunked_body() {
        assert_eq!(
            parse_request(
                "POST /upload HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\nZ\r\noops\r\n"
            ),
            Err(ParseError::MalformedChunk)
        );
    }
}
