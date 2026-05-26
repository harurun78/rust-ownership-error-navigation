use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::{ParseOutcome, RedisMiniSession, RespCommandParser, RespError};

#[derive(Debug, Default)]
pub struct RedisMiniClientSession {
    parser: RespCommandParser,
    session: RedisMiniSession,
}

impl RedisMiniClientSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_input(&mut self, bytes: &[u8]) -> Result<Vec<u8>, RespError> {
        self.parser.append(bytes);
        let mut output = Vec::new();

        loop {
            match self.parser.parse_available()? {
                ParseOutcome::Complete(command) => {
                    output.extend(self.session.execute_encoded(command));
                }
                ParseOutcome::Incomplete => return Ok(output),
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct RedisMiniServer;

impl RedisMiniServer {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        TcpListener::bind(addr)
    }

    pub fn serve<A: ToSocketAddrs>(addr: A) -> io::Result<()> {
        let listener = Self::bind(addr)?;
        Self::serve_forever(listener)
    }

    pub fn serve_forever(listener: TcpListener) -> io::Result<()> {
        for stream in listener.incoming() {
            Self::handle_client(stream?)?;
        }
        Ok(())
    }

    pub fn serve_listener(listener: TcpListener, max_clients: usize) -> io::Result<()> {
        for _ in 0..max_clients {
            let (stream, _) = listener.accept()?;
            Self::handle_client(stream)?;
        }
        Ok(())
    }

    pub fn handle_client(mut stream: TcpStream) -> io::Result<()> {
        let mut session = RedisMiniClientSession::new();
        let mut buffer = [0_u8; 4096];

        loop {
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read == 0 {
                return Ok(());
            }

            let replies = session
                .process_input(&buffer[..bytes_read])
                .map_err(|error| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}"))
                })?;
            if !replies.is_empty() {
                stream.write_all(&replies)?;
            }
        }
    }
}
