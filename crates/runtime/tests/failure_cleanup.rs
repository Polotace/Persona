use std::sync::Arc;

use persona_database::SqliteStorageFactory;
use persona_runtime::{
    EventDispatcher, LogLevel, LoggerFactory, Runtime, RuntimeConfig, RuntimeError, RuntimeLogger,
};
use tempfile::tempdir;

struct FailingLoggerFactory;

impl LoggerFactory for FailingLoggerFactory {
    fn initialize(&self, _log_level: LogLevel) -> Result<Arc<dyn RuntimeLogger>, String> {
        Err("logger initialization failed".to_owned())
    }
}

#[tokio::test]
async fn logger_initialization_failure_never_reports_ready_and_allows_stop() {
    let directory = tempdir().expect("temporary directory");
    let config = RuntimeConfig::from_toml(&format!(
        "schema_version = 1\ndata_dir = '{}'\ndatabase_path = 'persona.db'\nlog_level = 'info'\nevent_queue_capacity = 1",
        directory.path().display()
    ))
    .expect("valid configuration");
    let (dispatcher, mut events) = EventDispatcher::bounded(config.event_queue_capacity);
    let runtime = Runtime::new(
        config,
        Box::new(FailingLoggerFactory),
        Box::new(SqliteStorageFactory),
        dispatcher,
    );

    assert!(matches!(runtime.start().await, Err(RuntimeError::Logging)));
    assert!(events.try_recv().is_err());
    runtime
        .stop()
        .await
        .expect("stopping failed runtime is safe");
}
