//! Runtime configuration and privacy-safe lifecycle logging contracts.

mod config;
mod dispatcher;
mod logging;

pub use config::{ConfigError, RuntimeConfig};
pub use dispatcher::EventDispatcher;
pub use logging::{LoggerFactory, RuntimeLogger, SafeLogRecord, TracingLoggerFactory};
