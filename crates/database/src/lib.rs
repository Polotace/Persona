//! Replaceable local storage contracts and the SQLite Phase 1 adapter.

mod port;
mod sqlite;

pub use port::{AuditActor, AuditReason, AuditRecord, Storage, StorageError, StorageFactory};
pub use sqlite::SqliteStorageFactory;
