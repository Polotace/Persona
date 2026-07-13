use crate::{AuditActor, AuditReason, AuditRecord, Storage, StorageError, StorageFactory};
use async_trait::async_trait;
use persona_core::{CorrelationId, OwnerId};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{
    path::Path,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const INITIAL_SCHEMA_VERSION: i64 = 1;

pub struct SqliteStorageFactory;

pub struct SqliteStorage {
    pool: SqlitePool,
}

#[async_trait]
impl StorageFactory for SqliteStorageFactory {
    async fn open(&self, path: &Path) -> Result<Arc<dyn Storage>, StorageError> {
        if path.parent().is_some_and(|parent| !parent.exists()) {
            return Err(StorageError::Unavailable(
                "database parent directory does not exist".to_owned(),
            ));
        }

        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("the built-in SQLite connection string must be valid")
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|error| {
                StorageError::Unavailable(format!("opening SQLite database: {error}"))
            })?;

        Ok(Arc::new(SqliteStorage { pool }))
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn migrate(&self) -> Result<(), StorageError> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                StorageError::Migration(format!("starting transaction: {error}"))
            })?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (\
                version INTEGER PRIMARY KEY, \
                applied_at TEXT NOT NULL\
            )",
        )
        .execute(&mut *transaction)
        .await
        .map_err(|error| StorageError::Migration(format!("creating migration state: {error}")))?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_events (\
                id TEXT PRIMARY KEY, \
                owner_id TEXT NOT NULL, \
                actor TEXT NOT NULL, \
                reason TEXT NOT NULL, \
                correlation_id TEXT NOT NULL, \
                occurred_at INTEGER NOT NULL\
            )",
        )
        .execute(&mut *transaction)
        .await
        .map_err(|error| StorageError::Migration(format!("creating audit metadata: {error}")))?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_events_owner ON audit_events(owner_id)")
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                StorageError::Migration(format!("creating audit owner index: {error}"))
            })?;
        sqlx::query("INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?, ?)")
            .bind(INITIAL_SCHEMA_VERSION)
            .bind("phase-1")
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                StorageError::Migration(format!("recording schema version: {error}"))
            })?;

        transaction
            .commit()
            .await
            .map_err(|error| StorageError::Migration(format!("committing transaction: {error}")))
    }

    async fn write_audit(&self, record: AuditRecord) -> Result<(), StorageError> {
        let occurred_at = timestamp_to_epoch_nanoseconds(record.occurred_at)?;
        let mut transaction = self.pool.begin().await.map_err(|error| {
            StorageError::Unavailable(format!("starting audit transaction: {error}"))
        })?;

        sqlx::query(
            "INSERT INTO audit_events (id, owner_id, actor, reason, correlation_id, occurred_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(record.id.to_string())
        .bind(record.owner_id.as_str())
        .bind(record.actor.as_str())
        .bind(record.reason.as_str())
        .bind(record.correlation_id.as_uuid().to_string())
        .bind(occurred_at)
        .execute(&mut *transaction)
        .await
        .map_err(|error| StorageError::Unavailable(format!("writing audit metadata: {error}")))?;

        transaction.commit().await.map_err(|error| {
            StorageError::Unavailable(format!("committing audit metadata: {error}"))
        })
    }

    async fn list_audit(&self, owner_id: &OwnerId) -> Result<Vec<AuditRecord>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, actor, reason, correlation_id, occurred_at \
             FROM audit_events WHERE owner_id = ? ORDER BY occurred_at ASC, id ASC",
        )
        .bind(owner_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| StorageError::Unavailable(format!("listing audit metadata: {error}")))?;

        rows.into_iter()
            .map(|row| audit_record_from_row(row, owner_id.clone()))
            .collect()
    }

    async fn close(&self) {
        self.pool.close().await;
    }
}

fn timestamp_to_epoch_nanoseconds(timestamp: SystemTime) -> Result<i64, StorageError> {
    match timestamp.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            i64::try_from(duration.as_nanos()).map_err(|_| StorageError::InvalidAuditTimestamp)
        }
        Err(error) => {
            let nanoseconds = error.duration().as_nanos();
            let minimum = (i64::MAX as u128) + 1;

            if nanoseconds > minimum {
                return Err(StorageError::InvalidAuditTimestamp);
            }
            if nanoseconds == minimum {
                return Ok(i64::MIN);
            }

            i64::try_from(nanoseconds)
                .map(|nanoseconds| -nanoseconds)
                .map_err(|_| StorageError::InvalidAuditTimestamp)
        }
    }
}

fn timestamp_from_epoch_nanoseconds(nanoseconds: i64) -> Result<SystemTime, StorageError> {
    let duration = Duration::from_nanos(nanoseconds.unsigned_abs());
    let timestamp = if nanoseconds.is_negative() {
        UNIX_EPOCH.checked_sub(duration)
    } else {
        UNIX_EPOCH.checked_add(duration)
    };

    timestamp.ok_or(StorageError::InvalidAuditTimestamp)
}

fn audit_record_from_row(
    row: sqlx::sqlite::SqliteRow,
    owner_id: OwnerId,
) -> Result<AuditRecord, StorageError> {
    let id = row
        .try_get::<String, _>("id")
        .map_err(|_| StorageError::InvalidAuditMetadata)
        .and_then(|value| {
            Uuid::parse_str(&value).map_err(|_| StorageError::InvalidAuditMetadata)
        })?;
    let actor = match row
        .try_get::<String, _>("actor")
        .map_err(|_| StorageError::InvalidAuditMetadata)?
        .as_str()
    {
        "system" => AuditActor::System,
        "user" => AuditActor::User,
        _ => return Err(StorageError::InvalidAuditMetadata),
    };
    let reason = match row
        .try_get::<String, _>("reason")
        .map_err(|_| StorageError::InvalidAuditMetadata)?
        .as_str()
    {
        "migration" => AuditReason::Migration,
        _ => return Err(StorageError::InvalidAuditMetadata),
    };
    let correlation_id = row
        .try_get::<String, _>("correlation_id")
        .map_err(|_| StorageError::InvalidAuditMetadata)
        .and_then(|value| Uuid::parse_str(&value).map_err(|_| StorageError::InvalidAuditMetadata))
        .map(CorrelationId::from_uuid)?;
    let nanoseconds = row
        .try_get::<i64, _>("occurred_at")
        .map_err(|_| StorageError::InvalidAuditMetadata)?;
    let occurred_at = timestamp_from_epoch_nanoseconds(nanoseconds)?;

    Ok(AuditRecord {
        id,
        owner_id,
        actor,
        reason,
        correlation_id,
        occurred_at,
    })
}
