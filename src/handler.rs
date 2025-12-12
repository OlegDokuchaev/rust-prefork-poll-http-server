use crate::http::{self, Method, Request};
use crate::static_files::{self, StaticError};
use std::io;
use std::path::Path;

pub fn handle_request(req: &Request, doc_root: &Path) -> io::Result<Vec<u8>> {
    match req.method {
        Method::Get => respond_static(req, doc_root, false),
        Method::Head => respond_static(req, doc_root, true),
        Method::Other(ref m) => {
            eprintln!("[worker] unsupported method {m}");
            http::method_not_allowed()
        }
    }
}

fn respond_static(req: &Request, doc_root: &Path, head_only: bool) -> io::Result<Vec<u8>> {
    let result = if head_only {
        static_files::load_head(doc_root, req.target)
    } else {
        static_files::load_get(doc_root, req.target)
    };

    match result {
        Ok(asset) => http::ok(&asset.body, asset.declared_len, asset.content_type),
        Err(StaticError::NotFound) => build_simple(404, "Not Found", head_only),
        Err(StaticError::Forbidden) => build_simple(403, "Forbidden", head_only),
        Err(StaticError::Io(e)) => {
            eprintln!("[worker] io error for {}: {e}", req.target);
            build_simple(500, "Internal Server Error", head_only)
        }
    }
}

fn build_simple(code: u16, reason: &str, head_only: bool) -> io::Result<Vec<u8>> {
    let body = reason.as_bytes();
    let resp_body = if head_only { &[][..] } else { body };
    http::build_response(
        code,
        reason,
        resp_body,
        body.len(),
        "text/plain; charset=utf-8",
        &[],
    )
}
