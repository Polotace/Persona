use std::num::NonZeroUsize;
use std::path::PathBuf;

use serde::Deserialize;
use thiserror::Error;

/// The only configuration schema currently supported by the runtime.
pub const CONFIG_SCHEMA_VERSION: u16 = 1;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    schema_version: u16,
    data_dir: PathBuf,
    database_path: PathBuf,
    log_level: String,
    event_queue_capacity: usize,
}

/// Runtime settings loaded from a versioned TOML document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub log_level: String,
    pub event_queue_capacity: NonZeroUsize,
}

/// A configuration document was structurally invalid or unsupported.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ConfigError {
    #[error("invalid runtime configuration: {0}")]
    Invalid(String),
}

impl RuntimeConfig {
    /// Parses a versioned runtime configuration without retaining source content in errors.
    pub fn from_toml(source: &str) -> Result<Self, ConfigError> {
        let file_config: FileConfig = toml::from_str(source)
            .map_err(|_| ConfigError::Invalid("configuration format is invalid".to_owned()))?;

        if file_config.schema_version != CONFIG_SCHEMA_VERSION {
            return Err(ConfigError::Invalid(
                "configuration schema version is unsupported".to_owned(),
            ));
        }

        if file_config.data_dir.as_os_str().is_empty() {
            return Err(ConfigError::Invalid(
                "data directory must not be empty".to_owned(),
            ));
        }

        if file_config.database_path.as_os_str().is_empty() {
            return Err(ConfigError::Invalid(
                "database path must not be empty".to_owned(),
            ));
        }

        let event_queue_capacity =
            NonZeroUsize::new(file_config.event_queue_capacity).ok_or_else(|| {
                ConfigError::Invalid("event queue capacity must be greater than zero".to_owned())
            })?;

        Ok(Self {
            data_dir: file_config.data_dir,
            database_path: file_config.database_path,
            log_level: file_config.log_level,
            event_queue_capacity,
        })
    }
}
