use crate::{AuditActor, AuditReason, AuditRecord, Storage, StorageError, StorageFactory};
use async_trait::async_trait;
use persona_core::{CorrelationId, OwnerId};
use sqlx::{
    Row, Sqlite, SqlitePool, Transaction,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const INITIAL_SCHEMA_VERSION: i64 = 1;
const AUDIT_NANOSECOND_SCHEMA_VERSION: i64 = 2;

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

        let options = SqliteConnectOptions::new()
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

        create_migration_state(&mut transaction).await?;
        if !migration_is_applied(&mut transaction, INITIAL_SCHEMA_VERSION).await? {
            ensure_audit_events_table(&mut transaction).await?;
            record_migration(&mut transaction, INITIAL_SCHEMA_VERSION).await?;
        }

        if !migration_is_applied(&mut transaction, AUDIT_NANOSECOND_SCHEMA_VERSION).await? {
            upgrade_audit_events_to_nanoseconds(&mut transaction).await?;
            record_migration(&mut transaction, AUDIT_NANOSECOND_SCHEMA_VERSION).await?;
        } else if !audit_events_use_nanoseconds(&mut transaction).await? {
            return Err(StorageError::Migration(
                "schema version 2 is recorded but audit events do not use occurred_at".to_owned(),
            ));
        }

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_events_owner ON audit_events(owner_id)")
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                StorageError::Migration(format!("creating audit owner index: {error}"))
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

async fn create_migration_state(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<(), StorageError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\
            version INTEGER PRIMARY KEY, \
            applied_at TEXT NOT NULL\
        )",
    )
    .execute(&mut **transaction)
    .await
    .map_err(|error| StorageError::Migration(format!("creating migration state: {error}")))?;

    Ok(())
}

async fn migration_is_applied(
    transaction: &mut Transaction<'_, Sqlite>,
    version: i64,
) -> Result<bool, StorageError> {
    sqlx::query("SELECT 1 FROM schema_migrations WHERE version = ?")
        .bind(version)
        .fetch_optional(&mut **transaction)
        .await
        .map(|row| row.is_some())
        .map_err(|error| StorageError::Migration(format!("reading schema version: {error}")))
}

async fn record_migration(
    transaction: &mut Transaction<'_, Sqlite>,
    version: i64,
) -> Result<(), StorageError> {
    sqlx::query("INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)")
        .bind(version)
        .bind("phase-1")
        .execute(&mut **transaction)
        .await
        .map_err(|error| StorageError::Migration(format!("recording schema version: {error}")))?;

    Ok(())
}

async fn audit_event_columns(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<Vec<String>, StorageError> {
    sqlx::query("PRAGMA table_info(audit_events)")
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| StorageError::Migration(format!("reading audit schema: {error}")))?
        .into_iter()
        .map(|row| {
            row.try_get("name").map_err(|error| {
                StorageError::Migration(format!("reading audit column name: {error}"))
            })
        })
        .collect()
}

async fn ensure_audit_events_table(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<(), StorageError> {
    if audit_event_columns(transaction).await?.is_empty() {
        create_nanosecond_audit_events_table(transaction).await?;
    }

    Ok(())
}

async fn audit_events_use_nanoseconds(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<bool, StorageError> {
    let columns = audit_event_columns(transaction).await?;
    Ok(columns.iter().any(|column| column == "occurred_at")
        && !columns.iter().any(|column| column == "created_at"))
}

async fn upgrade_audit_events_to_nanoseconds(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<(), StorageError> {
    let columns = audit_event_columns(transaction).await?;
    if columns.is_empty() {
        return create_nanosecond_audit_events_table(transaction).await;
    }
    if audit_events_use_nanoseconds(transaction).await? {
        return Ok(());
    }
    if !columns.iter().any(|column| column == "created_at")
        || columns.iter().any(|column| column == "occurred_at")
    {
        return Err(StorageError::Migration(
            "audit events schema is not compatible with the phase-1 timestamp migration".to_owned(),
        ));
    }

    let invalid_timestamp_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events \
         WHERE created_at > 9223372036 OR created_at < -9223372036",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| {
        StorageError::Migration(format!("validating legacy audit timestamps: {error}"))
    })?;
    if invalid_timestamp_count != 0 {
        return Err(StorageError::Migration(
            "legacy audit timestamp is outside the signed nanosecond range".to_owned(),
        ));
    }

    sqlx::query("DROP INDEX IF EXISTS idx_audit_events_owner")
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            StorageError::Migration(format!("replacing audit owner index: {error}"))
        })?;
    sqlx::query("ALTER TABLE audit_events RENAME TO audit_events_v1")
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            StorageError::Migration(format!("renaming legacy audit events: {error}"))
        })?;
    create_nanosecond_audit_events_table(transaction).await?;
    sqlx::query(
        "INSERT INTO audit_events (id, owner_id, actor, reason, correlation_id, occurred_at) \
         SELECT id, owner_id, actor, reason, correlation_id, created_at * 1000000000 \
         FROM audit_events_v1",
    )
    .execute(&mut **transaction)
    .await
    .map_err(|error| StorageError::Migration(format!("copying legacy audit events: {error}")))?;
    sqlx::query("DROP TABLE audit_events_v1")
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            StorageError::Migration(format!("removing legacy audit events: {error}"))
        })?;

    Ok(())
}

async fn create_nanosecond_audit_events_table(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<(), StorageError> {
    sqlx::query(
        "CREATE TABLE audit_events (\
            id TEXT PRIMARY KEY, \
            owner_id TEXT NOT NULL, \
            actor TEXT NOT NULL, \
            reason TEXT NOT NULL, \
            correlation_id TEXT NOT NULL, \
            occurred_at INTEGER NOT NULL\
        )",
    )
    .execute(&mut **transaction)
    .await
    .map_err(|error| StorageError::Migration(format!("creating audit metadata: {error}")))?;

    Ok(())
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
