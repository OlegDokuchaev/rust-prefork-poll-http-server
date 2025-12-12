use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub addr: String,
    pub workers: usize,
    pub poll_timeout_ms: u16,
    pub read_chunk: usize,
    pub html_path: String,
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
