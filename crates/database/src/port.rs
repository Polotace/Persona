use async_trait::async_trait;
use persona_core::{CorrelationId, OwnerId};
use std::{path::Path, sync::Arc, time::SystemTime};
use uuid::Uuid;

/// Privacy-safe metadata describing a storage operation.
#[derive(Clone, Debug)]
pub struct AuditRecord {
    pub id: Uuid,
    pub owner_id: OwnerId,
    pub actor: AuditActor,
    pub reason: AuditReason,
    pub correlation_id: CorrelationId,
    pub occurred_at: SystemTime,
}

impl AuditRecord {
    pub fn new(
        owner_id: OwnerId,
        actor: AuditActor,
        reason: AuditReason,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            actor,
            reason,
            correlation_id,
            occurred_at: SystemTime::now(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditActor {
    System,
    User,
}

impl AuditActor {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditReason {
    Migration,
}

impl AuditReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Migration => "migration",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("local storage is unavailable: {0}")]
    Unavailable(String),
    #[error("local storage migration failed: {0}")]
    Migration(String),
    #[error("local storage contains invalid audit metadata")]
    InvalidAuditMetadata,
    #[error("audit timestamp cannot be represented as Unix nanoseconds")]
    InvalidAuditTimestamp,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn migrate(&self) -> Result<(), StorageError>;
    async fn write_audit(&self, record: AuditRecord) -> Result<(), StorageError>;
    async fn list_audit(&self, owner_id: &OwnerId) -> Result<Vec<AuditRecord>, StorageError>;
    async fn close(&self);
}

#[async_trait]
pub trait StorageFactory: Send + Sync {
    async fn open(&self, path: &Path) -> Result<Arc<dyn Storage>, StorageError>;
}
