use persona_core::{CorrelationId, OwnerId};
use persona_database::{
    AuditActor, AuditReason, AuditRecord, SqliteStorageFactory, StorageError, StorageFactory,
};
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
async fn sqlite_open_of_missing_parent_returns_unavailable() {
    let directory = tempdir().expect("temporary directory must be created");
    let result = SqliteStorageFactory
        .open(&directory.path().join("missing").join("persona.db"))
        .await;

    assert!(matches!(result, Err(StorageError::Unavailable(_))));
}
