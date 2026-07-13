use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use persona_core::{
    CorrelationId, EventError, RuntimeEvent, RuntimeEventKind, RuntimeState, SchemaVersion,
};
use persona_database::{Storage, StorageError, StorageFactory};
use tokio::sync::Mutex as AsyncMutex;

use crate::{EventDispatcher, LoggerFactory, RuntimeConfig, RuntimeLogger, SafeLogRecord};

/// Composes runtime dependencies and coordinates their lifecycle.
pub struct Runtime {
    config: RuntimeConfig,
    logger_factory: Box<dyn LoggerFactory>,
    storage_factory: Box<dyn StorageFactory>,
    dispatcher: EventDispatcher,
    state: Mutex<RuntimeState>,
    storage: AsyncMutex<Option<Arc<dyn Storage>>>,
    logger: Mutex<Option<Arc<dyn RuntimeLogger>>>,
    correlation_id: Mutex<Option<CorrelationId>>,
}

/// A failure while composing or operating the local runtime.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("logging initialization failed")]
    Logging,
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Event(#[from] EventError),
}

impl Runtime {
    /// Creates a runtime from validated configuration and explicit infrastructure dependencies.
    pub fn new(
        config: RuntimeConfig,
        logger_factory: Box<dyn LoggerFactory>,
        storage_factory: Box<dyn StorageFactory>,
        dispatcher: EventDispatcher,
    ) -> Self {
        Self {
            config,
            logger_factory,
            storage_factory,
            dispatcher,
            state: Mutex::new(RuntimeState::New),
            storage: AsyncMutex::new(None),
            logger: Mutex::new(None),
            correlation_id: Mutex::new(None),
        }
    }

    /// Initializes logging and local storage, then publishes readiness lifecycle events.
    pub async fn start(&self) -> Result<(), RuntimeError> {
        if !self.transition_from_new() {
            return Ok(());
        }

        let logger = match self.logger_factory.initialize(&self.config.log_level) {
            Ok(logger) => logger,
            Err(_) => {
                self.fail_start().await;
                return Err(RuntimeError::Logging);
            }
        };
        *self
            .logger
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(logger);

        let correlation_id = self.correlation_id();
        if let Err(error) =
            self.publish_lifecycle(correlation_id, RuntimeEventKind::RuntimeStarting)
        {
            self.fail_start().await;
            return Err(error);
        }

        if std::fs::create_dir_all(&self.config.data_dir).is_err() {
            self.fail_start().await;
            return Err(RuntimeError::Storage(StorageError::Unavailable(
                "creating local data directory".to_owned(),
            )));
        }

        let storage = match self.storage_factory.open(&self.database_path()).await {
            Ok(storage) => storage,
            Err(error) => {
                self.fail_start().await;
                return Err(RuntimeError::Storage(error));
            }
        };
        if let Err(error) = storage.migrate().await {
            storage.close().await;
            self.fail_start().await;
            return Err(RuntimeError::Storage(error));
        }
        *self.storage.lock().await = Some(storage);

        self.set_state(RuntimeState::StorageReady);
        if let Err(error) = self.publish_lifecycle(correlation_id, RuntimeEventKind::StorageReady) {
            self.fail_start().await;
            return Err(error);
        }

        self.set_state(RuntimeState::Ready);
        if let Err(error) = self.publish_lifecycle(correlation_id, RuntimeEventKind::RuntimeReady) {
            self.fail_start().await;
            return Err(error);
        }

        Ok(())
    }

    /// Releases initialized resources and publishes shutdown lifecycle events once.
    pub async fn stop(&self) -> Result<(), RuntimeError> {
        if !self.transition_from_ready_to_stopping() {
            return Ok(());
        }

        let correlation_id = self.correlation_id();
        if let Err(error) =
            self.publish_lifecycle(correlation_id, RuntimeEventKind::RuntimeStopping)
        {
            self.fail_stop().await;
            return Err(error);
        }

        self.release_storage().await;
        if let Err(error) = self.publish_lifecycle(correlation_id, RuntimeEventKind::RuntimeStopped)
        {
            self.fail_stop().await;
            return Err(error);
        }

        self.cleanup().await;
        self.set_state(RuntimeState::Stopped);
        Ok(())
    }

    fn transition_from_new(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *state != RuntimeState::New {
            return false;
        }

        *state = RuntimeState::Starting;
        true
    }

    fn transition_from_ready_to_stopping(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *state != RuntimeState::Ready {
            return false;
        }

        *state = RuntimeState::Stopping;
        true
    }

    fn set_state(&self, next: RuntimeState) {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = next;
    }

    fn correlation_id(&self) -> CorrelationId {
        let mut correlation_id = self
            .correlation_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *correlation_id.get_or_insert_with(CorrelationId::new)
    }

    fn database_path(&self) -> PathBuf {
        if self.config.database_path.is_absolute() {
            self.config.database_path.clone()
        } else {
            self.config.data_dir.join(&self.config.database_path)
        }
    }

    fn publish_lifecycle(
        &self,
        correlation_id: CorrelationId,
        kind: RuntimeEventKind,
    ) -> Result<(), RuntimeError> {
        let transition = match kind {
            RuntimeEventKind::RuntimeStarting => "starting",
            RuntimeEventKind::StorageReady => "storage_ready",
            RuntimeEventKind::RuntimeReady => "ready",
            RuntimeEventKind::RuntimeStopping => "stopping",
            RuntimeEventKind::RuntimeStopped => "stopped",
        };
        if let Some(logger) = self
            .logger
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            logger.record(SafeLogRecord::transition(
                "runtime",
                transition,
                correlation_id,
            ));
        }

        self.dispatcher
            .try_publish(RuntimeEvent::new(SchemaVersion::V1, correlation_id, kind))?;
        Ok(())
    }

    async fn fail_start(&self) {
        self.cleanup().await;
        self.set_state(RuntimeState::Failed);
    }

    async fn fail_stop(&self) {
        self.cleanup().await;
        self.set_state(RuntimeState::Failed);
    }

    async fn release_storage(&self) {
        if let Some(storage) = self.storage.lock().await.take() {
            storage.close().await;
        }
    }

    async fn cleanup(&self) {
        self.release_storage().await;
        self.logger
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        self.dispatcher.close();
    }
}
