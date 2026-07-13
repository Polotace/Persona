use persona_core::{CorrelationId, OwnerId};
use persona_database::{
    AuditActor, AuditReason, AuditRecord, SqliteStorageFactory, StorageError, StorageFactory,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::time::{Duration, UNIX_EPOCH};
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn sqlite_migrates_idempotently_and_scopes_audit_records_to_owner() {
    let directory = tempdir().expect("temporary directory must be created");
    let factory = SqliteStorageFactory;
    let store = factory
        .open(&directory.path().join("persona.db"))
        .await
        .expect("SQLite store must open");

    store.migrate().await.expect("first migration must succeed");
    store
        .migrate()
        .await
        .expect("second migration must succeed");

    let owner_a = OwnerId::try_from("owner-a").expect("owner id must be valid");
    let owner_b = OwnerId::try_from("owner-b").expect("owner id must be valid");
    store
        .write_audit(AuditRecord::new(
            owner_a.clone(),
            AuditActor::System,
            AuditReason::Migration,
            CorrelationId::new(),
        ))
        .await
        .expect("audit record must be written");

    assert_eq!(
        store
            .list_audit(&owner_a)
            .await
            .expect("owner audit records must list")
            .len(),
        1
    );
    assert!(
        store
            .list_audit(&owner_b)
            .await
            .expect("other owner audit records must list")
            .is_empty()
    );
}

#[tokio::test]
async fn sqlite_upgrades_legacy_second_timestamps_without_losing_audit_records() {
    let directory = tempdir().expect("temporary directory must be created");
    let path = directory.path().join("persona.db");
    let owner = OwnerId::try_from("owner-a").expect("owner id must be valid");
    let historical_id = Uuid::from_u128(1);
    let historical_correlation_id = Uuid::from_u128(2);
    let historical_seconds = 1_234_567_890_i64;

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("legacy database must open");
    sqlx::query(
        "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)",
    )
    .execute(&pool)
    .await
    .expect("legacy migration state must be created");
    sqlx::query(
        "CREATE TABLE audit_events (\
            id TEXT PRIMARY KEY, owner_id TEXT NOT NULL, actor TEXT NOT NULL, reason TEXT NOT NULL, \
            correlation_id TEXT NOT NULL, created_at INTEGER NOT NULL\
        )",
    )
    .execute(&pool)
    .await
    .expect("legacy audit table must be created");
    sqlx::query("CREATE INDEX idx_audit_events_owner ON audit_events(owner_id)")
        .execute(&pool)
        .await
        .expect("legacy audit index must be created");
    sqlx::query("INSERT INTO schema_migrations (version, applied_at) VALUES (1, 'phase-1')")
        .execute(&pool)
        .await
        .expect("legacy schema version must be recorded");
    sqlx::query(
        "INSERT INTO audit_events (id, owner_id, actor, reason, correlation_id, created_at) \
         VALUES (?, ?, 'system', 'migration', ?, ?)",
    )
    .bind(historical_id.to_string())
    .bind(owner.as_str())
    .bind(historical_correlation_id.to_string())
    .bind(historical_seconds)
    .execute(&pool)
    .await
    .expect("legacy audit record must be written");
    pool.close().await;

    let store = SqliteStorageFactory
        .open(&path)
        .await
        .expect("SQLite store must open");
    store.migrate().await.expect("legacy database must migrate");

    let mut new_record = AuditRecord::new(
        owner.clone(),
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    );
    new_record.id = Uuid::from_u128(3);
    new_record.occurred_at = UNIX_EPOCH + Duration::new(historical_seconds as u64 + 1, 123);
    store
        .write_audit(new_record.clone())
        .await
        .expect("audit writes must succeed after upgrade");

    let records = store
        .list_audit(&owner)
        .await
        .expect("upgraded audit records must list");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id, historical_id);
    assert_eq!(
        records[0].correlation_id.as_uuid(),
        historical_correlation_id
    );
    assert_eq!(
        records[0].occurred_at,
        UNIX_EPOCH + Duration::from_secs(historical_seconds as u64)
    );
    assert_eq!(records[1].id, new_record.id);
    assert_eq!(records[1].occurred_at, new_record.occurred_at);

    store.close().await;
    let verification_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(false),
        )
        .await
        .expect("upgraded database must reopen");
    let migration_version: i64 =
        sqlx::query_scalar("SELECT version FROM schema_migrations WHERE version = 2")
            .fetch_one(&verification_pool)
            .await
            .expect("nanosecond migration version must be recorded");
    let stored_nanoseconds: i64 =
        sqlx::query_scalar("SELECT occurred_at FROM audit_events WHERE id = ?")
            .bind(historical_id.to_string())
            .fetch_one(&verification_pool)
            .await
            .expect("legacy timestamp must be converted");
    assert_eq!(migration_version, 2);
    assert_eq!(stored_nanoseconds, historical_seconds * 1_000_000_000);
}

#[tokio::test]
async fn sqlite_lists_audit_records_by_id_when_timestamps_are_equal() {
    let directory = tempdir().expect("temporary directory must be created");
    let factory = SqliteStorageFactory;
    let store = factory
        .open(&directory.path().join("persona.db"))
        .await
        .expect("SQLite store must open");
    store.migrate().await.expect("migration must succeed");

    let owner = OwnerId::try_from("owner-a").expect("owner id must be valid");
    let occurred_at = UNIX_EPOCH + Duration::from_secs(1_000);
    let first_id = Uuid::from_u128(1);
    let second_id = Uuid::from_u128(2);
    let mut second_record = AuditRecord::new(
        owner.clone(),
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    );
    second_record.id = second_id;
    second_record.occurred_at = occurred_at;
    let mut first_record = AuditRecord::new(
        owner.clone(),
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    );
    first_record.id = first_id;
    first_record.occurred_at = occurred_at;

    store
        .write_audit(second_record)
        .await
        .expect("second audit record must be written");
    store
        .write_audit(first_record)
        .await
        .expect("first audit record must be written");

    let records = store
        .list_audit(&owner)
        .await
        .expect("audit records must list");

    assert_eq!(
        records
            .into_iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        vec![first_id, second_id]
    );
}

#[tokio::test]
async fn sqlite_preserves_nanosecond_timestamps_and_chronological_order() {
    let directory = tempdir().expect("temporary directory must be created");
    let factory = SqliteStorageFactory;
    let store = factory
        .open(&directory.path().join("persona.db"))
        .await
        .expect("SQLite store must open");
    store.migrate().await.expect("migration must succeed");

    let owner = OwnerId::try_from("owner-a").expect("owner id must be valid");
    let earlier_timestamp = UNIX_EPOCH + Duration::new(1_000, 100);
    let later_timestamp = UNIX_EPOCH + Duration::new(1_000, 900);
    let earlier_id = Uuid::from_u128(1);
    let later_id = Uuid::from_u128(2);
    let mut later_record = AuditRecord::new(
        owner.clone(),
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    );
    later_record.id = later_id;
    later_record.occurred_at = later_timestamp;
    let mut earlier_record = AuditRecord::new(
        owner.clone(),
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    );
    earlier_record.id = earlier_id;
    earlier_record.occurred_at = earlier_timestamp;

    store
        .write_audit(later_record)
        .await
        .expect("later audit record must be written first");
    store
        .write_audit(earlier_record)
        .await
        .expect("earlier audit record must be written second");

    let records = store
        .list_audit(&owner)
        .await
        .expect("audit records must list");

    assert_eq!(
        records.iter().map(|record| record.id).collect::<Vec<_>>(),
        vec![earlier_id, later_id]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.occurred_at)
            .collect::<Vec<_>>(),
        vec![earlier_timestamp, later_timestamp]
    );
}

#[tokio::test]
async fn sqlite_round_trips_pre_epoch_audit_timestamps() {
    let directory = tempdir().expect("temporary directory must be created");
    let factory = SqliteStorageFactory;
    let store = factory
        .open(&directory.path().join("persona.db"))
        .await
        .expect("SQLite store must open");
    store.migrate().await.expect("migration must succeed");

    let owner = OwnerId::try_from("owner-a").expect("owner id must be valid");
    let occurred_at = UNIX_EPOCH
        .checked_sub(Duration::new(1, 123))
        .expect("test timestamp must be representable");
    let mut record = AuditRecord::new(
        owner.clone(),
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    );
    record.occurred_at = occurred_at;

    store
        .write_audit(record)
        .await
        .expect("pre-epoch audit record must be written");

    let records = store
        .list_audit(&owner)
        .await
        .expect("audit records must list");
    assert_eq!(records[0].occurred_at, occurred_at);
}

#[tokio::test]
async fn sqlite_rejects_audit_timestamps_outside_unix_nanosecond_range() {
    let directory = tempdir().expect("temporary directory must be created");
    let factory = SqliteStorageFactory;
    let store = factory
        .open(&directory.path().join("persona.db"))
        .await
        .expect("SQLite store must open");
    store.migrate().await.expect("migration must succeed");

    let owner = OwnerId::try_from("owner-a").expect("owner id must be valid");
    let mut record = AuditRecord::new(
        owner,
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    );
    record.occurred_at = UNIX_EPOCH
        .checked_add(Duration::from_secs(i64::MAX as u64 / 1_000_000_000 + 1))
        .expect("test timestamp must be representable");

    assert!(matches!(
        store.write_audit(record).await,
        Err(StorageError::InvalidAuditTimestamp)
    ));
}

#[tokio::test]
async fn sqlite_open_of_missing_parent_returns_unavailable() {
    let directory = tempdir().expect("temporary directory must be created");
    let result = SqliteStorageFactory
        .open(&directory.path().join("missing").join("persona.db"))
        .await;

    assert!(matches!(result, Err(StorageError::Unavailable(_))));
}
