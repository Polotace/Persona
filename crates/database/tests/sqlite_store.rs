use persona_core::{CorrelationId, OwnerId};
use persona_database::{
    AuditActor, AuditReason, AuditRecord, SqliteStorageFactory, StorageError, StorageFactory,
};
use tempfile::tempdir;

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
async fn sqlite_open_of_missing_parent_returns_unavailable() {
    let directory = tempdir().expect("temporary directory must be created");
    let result = SqliteStorageFactory
        .open(&directory.path().join("missing").join("persona.db"))
        .await;

    assert!(matches!(result, Err(StorageError::Unavailable(_))));
}
