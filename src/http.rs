use std::io::{self, Error, ErrorKind, Write};

#[derive(Debug, Clone)]
pub enum Method<'a> {
    Get,
    Head,
    Other(&'a str),
}

#[derive(Debug, Clone)]
pub struct Request<'a> {
    pub method: Method<'a>,
    pub target: &'a str,
}

pub fn parse_request(buf: &[u8]) -> io::Result<Request<'_>> {
    let line_end = buf
        .windows(2)
        .position(|w| w == b"\r\n")
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing request line"))?;

    let line = &buf[..line_end];
    let mut parts = line.split(|&b| b == b' ');

    let method_bytes = parts
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing method"))?;
    let target_bytes = parts
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing target"))?;

    let method = match method_bytes {
        b"GET" => Method::Get,
        b"HEAD" => Method::Head,
        other => Method::Other(std::str::from_utf8(other).unwrap_or("UNKNOWN")),
    };

    let raw_target = std::str::from_utf8(target_bytes)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "target is not utf-8"))?;
    let target = raw_target.split('?').next().unwrap_or(raw_target);

    if !target.starts_with('/') {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "target must start with /",
        ));
    }

    Ok(Request { method, target })
}

pub fn build_response(
    code: u16,
    reason: &str,
    body: &[u8],
    declared_len: usize,
    content_type: &str,
    extra_hdrs: &[(&str, &str)],
) -> io::Result<Vec<u8>> {
    let mut resp = Vec::with_capacity(256 + declared_len);

    write!(
        &mut resp,
        "HTTP/1.1 {code} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {declared_len}\r\n\
         Connection: close\r\n"
    )?;

    for (name, value) in extra_hdrs {
        write!(&mut resp, "{name}: {value}\r\n")?;
    }
    write!(&mut resp, "\r\n")?;

    resp.extend_from_slice(body);

    Ok(resp)
}

pub fn ok(body: &[u8], declared_len: usize, content_type: &str) -> io::Result<Vec<u8>> {
    build_response(200, "OK", body, declared_len, content_type, &[])
}

pub fn bad_request(msg: &str) -> io::Result<Vec<u8>> {
    build_response(
        400,
        "Bad Request",
        msg.as_bytes(),
        msg.len(),
        "text/plain; charset=utf-8",
        &[],
    )
}

pub fn method_not_allowed() -> io::Result<Vec<u8>> {
    build_response(
        405,
        "Method Not Allowed",
        b"Method Not Allowed",
        "Method Not Allowed".len(),
        "text/plain; charset=utf-8",
        &[("Allow", "GET, HEAD")],
    )
}
