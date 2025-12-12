use crate::http::{self, Method, Request};
use crate::page;
use std::io;

pub fn handle_request(req: &Request) -> io::Result<Vec<u8>> {
    match req.method {
        Method::Get => respond_get(),
        Method::Head => respond_head(),
        Method::Other(ref m) => {
            eprintln!("[worker] unsupported method {m}");
            http::method_not_allowed()
        }
    }
}

fn respond_get() -> io::Result<Vec<u8>> {
    let body = page::html_body();
    let content_type = "text/html; charset=utf-8";

    http::ok(body, content_type)
}

fn respond_head() -> io::Result<Vec<u8>> {
    let content_type = "text/html; charset=utf-8";

    http::ok(&[], content_type)
}
