use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub addr: String,
    pub workers: usize,
    pub poll_timeout_ms: u16,
    pub read_chunk: usize,
    pub doc_root: PathBuf,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(
                Environment::with_prefix("SERVER")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize::<Settings>()
    }
}
