use once_cell::sync::OnceCell;
use std::fs;
use std::io;

static HTML_BODY: OnceCell<Vec<u8>> = OnceCell::new();

pub fn init_html(path: &str) -> io::Result<()> {
    let data = fs::read(path)?;
    HTML_BODY
        .set(data)
        .map_err(|_| io::Error::new(io::ErrorKind::AlreadyExists, "html already initialized"))?;
    Ok(())
}

pub fn html_body() -> &'static [u8] {
    HTML_BODY
        .get()
        .expect("HTML body not initialized; call init_html first")
        .as_slice()
}
