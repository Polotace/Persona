use std::time::SystemTime;

use crate::{CorrelationId, SchemaVersion};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeState {
    New,
    Starting,
    StorageReady,
    Ready,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeEventKind {
    RuntimeStarting,
    StorageReady,
    RuntimeReady,
    RuntimeStopping,
    RuntimeStopped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeEvent {
    schema_version: SchemaVersion,
    correlation_id: CorrelationId,
    occurred_at: SystemTime,
    kind: RuntimeEventKind,
}

impl RuntimeEvent {
    pub fn new(
        schema_version: SchemaVersion,
        correlation_id: CorrelationId,
        kind: RuntimeEventKind,
    ) -> Self {
        Self {
            schema_version,
            correlation_id,
            occurred_at: SystemTime::now(),
            kind,
        }
    }

    pub fn ready(schema_version: SchemaVersion, correlation_id: CorrelationId) -> Self {
        Self::new(
            schema_version,
            correlation_id,
            RuntimeEventKind::RuntimeReady,
        )
    }

    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    pub const fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }

    pub fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }

    pub const fn kind(&self) -> RuntimeEventKind {
        self.kind
    }
}
