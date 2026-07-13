use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use persona_core::RuntimeEventKind;
use persona_database::SqliteStorageFactory;
use persona_runtime::{
    EventDispatcher, LoggerFactory, Runtime, RuntimeConfig, RuntimeLogger, SafeLogRecord,
};
use tempfile::tempdir;

#[derive(Clone, Default)]
struct CapturingLoggerFactory {
    records: Arc<Mutex<Vec<SafeLogRecord>>>,
}

impl LoggerFactory for CapturingLoggerFactory {
    fn initialize(&self, _log_level: &str) -> Result<Arc<dyn RuntimeLogger>, String> {
        Ok(Arc::new(CapturingLogger {
            records: Arc::clone(&self.records),
        }))
    }
}

struct CapturingLogger {
    records: Arc<Mutex<Vec<SafeLogRecord>>>,
}

impl RuntimeLogger for CapturingLogger {
    fn record(&self, record: SafeLogRecord) {
        self.records
            .lock()
            .expect("test logger mutex should not be poisoned")
            .push(record);
    }
}

#[tokio::test]
async fn runtime_publishes_ordered_lifecycle_events_with_one_correlation_id() {
    let directory = tempdir().expect("temporary directory");
    let config = RuntimeConfig::from_toml(&format!(
        "schema_version = 1\ndata_dir = '{}'\ndatabase_path = 'persona.db'\nlog_level = 'info'\nevent_queue_capacity = 16",
        directory.path().display()
    ))
    .expect("valid configuration");
    let (dispatcher, mut events) = EventDispatcher::bounded(config.event_queue_capacity);
    let logger_factory = CapturingLoggerFactory::default();
    let log_records = Arc::clone(&logger_factory.records);
    let runtime = Runtime::new(
        config,
        Box::new(logger_factory),
        Box::new(SqliteStorageFactory),
        dispatcher,
    );

    runtime.start().await.expect("runtime starts");
    runtime.stop().await.expect("runtime stops");
    runtime.stop().await.expect("stopping twice is safe");

    let mut received = Vec::new();
    for _ in 0..5 {
        received.push(events.recv().await.expect("lifecycle event"));
    }

    assert_eq!(
        received
            .iter()
            .map(|event| event.kind())
            .collect::<Vec<_>>(),
        vec![
            RuntimeEventKind::RuntimeStarting,
            RuntimeEventKind::StorageReady,
            RuntimeEventKind::RuntimeReady,
            RuntimeEventKind::RuntimeStopping,
            RuntimeEventKind::RuntimeStopped,
        ]
    );

    let correlation_id = received[0].correlation_id();
    assert!(
        received
            .iter()
            .all(|event| event.correlation_id() == correlation_id)
    );
    assert!(
        received
            .iter()
            .all(|event| event.occurred_at() <= SystemTime::now())
    );

    let records = log_records
        .lock()
        .expect("test logger mutex should not be poisoned");
    assert_eq!(records.len(), 5);
    assert!(
        records
            .iter()
            .all(|record| record.correlation_id == correlation_id)
    );
}
