use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

pub struct StaticAsset {
    pub declared_len: usize,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub enum StaticError {
    Forbidden,
    NotFound,
    Io(io::Error),
}

impl From<io::Error> for StaticError {
    fn from(e: io::Error) -> Self {
        StaticError::Io(e)
    }
}

pub fn load_get(doc_root: &Path, target: &str) -> Result<StaticAsset, StaticError> {
    load_impl(doc_root, target, false)
}

pub fn load_head(doc_root: &Path, target: &str) -> Result<StaticAsset, StaticError> {
    load_impl(doc_root, target, true)
}

fn load_impl(doc_root: &Path, target: &str, head_only: bool) -> Result<StaticAsset, StaticError> {
    let clean = sanitize_target(target)?;
    let path = if clean.as_os_str().is_empty() {
        doc_root.join("index.html")
    } else {
        doc_root.join(&clean)
    };

    let Some(found) = resolve_file(&path) else {
        return Err(StaticError::NotFound);
    };

    let meta = found.metadata()?;
    if !meta.is_file() {
        return Err(StaticError::Forbidden);
    }

    let declared_len = meta.len() as usize;
    let body = if head_only {
        Vec::new()
    } else {
        let f = fs::File::open(&found)?;
        let mut buf = Vec::with_capacity(declared_len);
        f.take(meta.len()).read_to_end(&mut buf)?;
        buf
    };
    let content_type = guess_content_type(&found);

    Ok(StaticAsset {
        body,
        declared_len,
        content_type,
    })
}

fn resolve_file(path: &Path) -> Option<PathBuf> {
    match path.try_exists() {
        Ok(true) => Some(path.to_path_buf()),
        Ok(false) => None,
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(_) => None,
    }
}

fn sanitize_target(target: &str) -> Result<PathBuf, StaticError> {
    let trimmed = target.trim_start_matches('/');
    let path = Path::new(trimmed);

    let mut clean = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(_) | Component::RootDir => return Err(StaticError::Forbidden),
            Component::CurDir => continue,
            Component::ParentDir => return Err(StaticError::Forbidden),
            Component::Normal(p) => clean.push(p),
        }
    }

    Ok(clean)
}

fn guess_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}
