use crate::handler::handle_request;
use crate::http;
use std::io::{self, Read, Write as IoWrite};
use std::net::TcpStream;

pub struct Conn {
    stream: TcpStream,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
    write_pos: usize,
    wrote_response: bool,
    read_chunk: usize,
}

impl Conn {
    pub fn new(stream: TcpStream, read_chunk: usize) -> Self {
        Self {
            stream,
            read_buf: Vec::with_capacity(read_chunk),
            write_buf: Vec::new(),
            write_pos: 0,
            wrote_response: false,
            read_chunk,
        }
    }

    pub fn has_pending_write(&self) -> bool {
        self.write_pos < self.write_buf.len()
    }

    pub fn on_read(&mut self) -> io::Result<bool> {
        let mut tmp = vec![0u8; self.read_chunk];

        loop {
            match self.stream.read(&mut tmp) {
                Ok(0) => return Ok(false),
                Ok(n) => {
                    self.read_buf.extend_from_slice(&tmp[..n]);

                    if !self.wrote_response && self.read_buf.windows(4).any(|w| w == b"\r\n\r\n") {
                        let response = match http::parse_request(&self.read_buf) {
                            Ok(req) => handle_request(&req),
                            Err(e) => http::bad_request(&e.to_string()),
                        }?;

                        self.write_buf = response;
                        self.wrote_response = true;
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }

        Ok(true)
    }

    pub fn on_write(&mut self) -> io::Result<bool> {
        while self.write_pos < self.write_buf.len() {
            match self.stream.write(&self.write_buf[self.write_pos..]) {
                Ok(0) => break,
                Ok(n) => self.write_pos += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }

        if self.wrote_response && self.write_pos >= self.write_buf.len() {
            return Ok(false);
        }

        Ok(true)
    }
}
