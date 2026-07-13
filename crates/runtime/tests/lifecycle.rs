use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use persona_core::{OwnerId, RuntimeEventKind, RuntimeState};
use persona_database::{AuditRecord, SqliteStorageFactory, Storage, StorageError, StorageFactory};
use persona_runtime::{
    EventDispatcher, LoggerFactory, Runtime, RuntimeConfig, RuntimeLogger, SafeLogRecord,
};
use tempfile::tempdir;
use tokio::sync::{Notify, oneshot};

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

struct BlockingStorageFactory {
    startup_blocked: Mutex<Option<oneshot::Sender<()>>>,
    release_startup: Arc<Notify>,
    storage: Arc<dyn Storage>,
}

#[async_trait::async_trait]
impl StorageFactory for BlockingStorageFactory {
    async fn open(&self, _path: &Path) -> Result<Arc<dyn Storage>, StorageError> {
        if let Some(startup_blocked) = self
            .startup_blocked
            .lock()
            .expect("test storage factory mutex should not be poisoned")
            .take()
        {
            startup_blocked
                .send(())
                .expect("startup observer should remain available");
        }
        self.release_startup.notified().await;
        Ok(Arc::clone(&self.storage))
    }
}

struct InMemoryStorage;

#[async_trait::async_trait]
impl Storage for InMemoryStorage {
    async fn migrate(&self) -> Result<(), StorageError> {
        Ok(())
    }

    async fn write_audit(&self, _record: AuditRecord) -> Result<(), StorageError> {
        Ok(())
    }

    async fn list_audit(&self, _owner_id: &OwnerId) -> Result<Vec<AuditRecord>, StorageError> {
        Ok(Vec::new())
    }

    async fn close(&self) {}
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

    let lifecycle_started_at = SystemTime::now();
    runtime.start().await.expect("runtime starts");
    runtime.stop().await.expect("runtime stops");
    runtime.stop().await.expect("stopping twice is safe");
    let lifecycle_finished_at = SystemTime::now();

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
    assert!(received.iter().all(|event| {
        event.occurred_at() >= lifecycle_started_at && event.occurred_at() <= lifecycle_finished_at
    }));

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

#[tokio::test]
async fn concurrent_stops_publish_one_shutdown_event_pair() {
    let directory = tempdir().expect("temporary directory");
    let config = RuntimeConfig::from_toml(&format!(
        "schema_version = 1\ndata_dir = '{}'\ndatabase_path = 'persona.db'\nlog_level = 'info'\nevent_queue_capacity = 16",
        directory.path().display()
    ))
    .expect("valid configuration");
    let (dispatcher, mut events) = EventDispatcher::bounded(config.event_queue_capacity);
    let runtime = Runtime::new(
        config,
        Box::new(CapturingLoggerFactory::default()),
        Box::new(SqliteStorageFactory),
        dispatcher,
    );

    runtime.start().await.expect("runtime starts");
    let (first_stop, second_stop) = tokio::join!(runtime.stop(), runtime.stop());
    first_stop.expect("first concurrent stop succeeds");
    second_stop.expect("second concurrent stop succeeds");

    let mut received = Vec::new();
    while let Some(event) = events.recv().await {
        received.push(event);
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
}

#[tokio::test]
async fn stop_waits_for_startup_then_completes_shutdown() {
    let directory = tempdir().expect("temporary directory");
    let config = RuntimeConfig::from_toml(&format!(
        "schema_version = 1\ndata_dir = '{}'\ndatabase_path = 'persona.db'\nlog_level = 'info'\nevent_queue_capacity = 16",
        directory.path().display()
    ))
    .expect("valid configuration");
    let (dispatcher, mut events) = EventDispatcher::bounded(config.event_queue_capacity);
    let (startup_blocked, startup_observer) = oneshot::channel();
    let release_startup = Arc::new(Notify::new());
    let runtime = Arc::new(Runtime::new(
        config,
        Box::new(CapturingLoggerFactory::default()),
        Box::new(BlockingStorageFactory {
            startup_blocked: Mutex::new(Some(startup_blocked)),
            release_startup: Arc::clone(&release_startup),
            storage: Arc::new(InMemoryStorage),
        }),
        dispatcher,
    ));

    let starting_runtime = Arc::clone(&runtime);
    let start = tokio::spawn(async move { starting_runtime.start().await });
    startup_observer
        .await
        .expect("startup should block while opening storage");

    let stopping_runtime = Arc::clone(&runtime);
    let stop = tokio::spawn(async move { stopping_runtime.stop().await });
    tokio::task::yield_now().await;
    assert!(
        !stop.is_finished(),
        "stop should wait for the start operation"
    );

    release_startup.notify_waiters();
    start
        .await
        .expect("start task should not panic")
        .expect("runtime starts");
    stop.await
        .expect("stop task should not panic")
        .expect("runtime stops after startup");

    assert_eq!(runtime.state(), RuntimeState::Stopped);

    let mut received = Vec::new();
    while let Some(event) = events.recv().await {
        received.push(event);
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
}
