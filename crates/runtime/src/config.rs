use std::num::NonZeroUsize;
use std::path::PathBuf;

use serde::Deserialize;
use thiserror::Error;

/// The logging severities supported by the runtime's metadata-only logger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// Parses a supported configuration value without retaining invalid input.
    pub fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(ConfigError::Invalid("log level is unsupported".to_owned())),
        }
    }
}

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
    pub log_level: LogLevel,
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

        let log_level = LogLevel::parse(&file_config.log_level)?;

        Ok(Self {
            data_dir: file_config.data_dir,
            database_path: file_config.database_path,
            log_level,
            event_queue_capacity,
        })
    }
}
