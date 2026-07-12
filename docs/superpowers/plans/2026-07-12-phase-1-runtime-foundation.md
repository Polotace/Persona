# Phase 1 Runtime Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Persona's tested Rust runtime foundation as libraries only: validated local configuration, metadata-safe logging, SQLite migrations and audit metadata, a bounded lifecycle event dispatcher, and deterministic shutdown.

**Architecture:** The workspace contains `persona-core` for infrastructure-free contracts, `database` for a replaceable storage port and SQLite adapter, and `runtime` for explicit dependency composition and lifecycle orchestration. No CLI, desktop shell, network service, AI capability, plugin, scheduler, or user-content schema is introduced; integration tests are the only host that starts the runtime.

**Tech Stack:** Rust 2024 edition, Tokio, Serde, TOML, SQLx with SQLite, thiserror, tracing, uuid, tempfile, Cargo fmt, and Clippy.

---

## File Structure

| File | Responsibility |
| --- | --- |
| `Cargo.toml` | Workspace membership, shared package metadata, and shared dependency versions. |
| `rust-toolchain.toml` | Reproducible Windows-first Rust toolchain components. |
| `crates/persona-core/src/identity.rs` | Validated owner and correlation identifiers plus schema version constants. |
| `crates/persona-core/src/lifecycle.rs` | Runtime states, lifecycle facts, and versioned event envelopes. |
| `crates/persona-core/src/error.rs` | Infrastructure-free event and lifecycle errors. |
| `crates/database/src/port.rs` | Storage-facing traits and safe audit record DTOs. |
| `crates/database/src/sqlite.rs` | SQLx SQLite connection, transactional migrations, and owner-scoped audit operations. |
| `crates/runtime/src/config.rs` | Versioned TOML parsing and validation. |
| `crates/runtime/src/logging.rs` | Metadata-only logging port and tracing adapter. |
| `crates/runtime/src/dispatcher.rs` | Bounded in-process event dispatcher with explicit backpressure and closure. |
| `crates/runtime/src/runtime.rs` | Composition root, lifecycle transitions, failure cleanup, and idempotent shutdown. |
| `crates/*/tests/*.rs` | Unit-level contracts and integration tests using temporary files only. |
| `docs/README.md` | Index the approved Phase 1 specification and implementation plan. |

## Task 1: Establish the Cargo Workspace and Core Contracts

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `crates/persona-core/Cargo.toml`
- Create: `crates/persona-core/src/lib.rs`
- Create: `crates/persona-core/src/identity.rs`
- Create: `crates/persona-core/src/lifecycle.rs`
- Create: `crates/persona-core/src/error.rs`
- Create: `crates/persona-core/tests/contracts.rs`

- [ ] **Step 1: Write failing tests for validated identifiers and lifecycle event envelopes.**

```rust
use persona_core::{CorrelationId, OwnerId, RuntimeEvent, SchemaVersion};
use std::time::SystemTime;

#[test]
fn owner_id_rejects_blank_values() {
    assert!(OwnerId::try_from("   ").is_err());
}

#[test]
fn lifecycle_event_has_version_and_correlation_id() {
    let correlation_id = CorrelationId::new();
    let event = RuntimeEvent::ready(SchemaVersion::V1, correlation_id);

    assert_eq!(event.schema_version(), SchemaVersion::V1);
    assert_eq!(event.correlation_id(), correlation_id);
    assert!(event.occurred_at() <= SystemTime::now());
}
```

- [ ] **Step 2: Run the test to verify it fails because the workspace and crate do not exist.**

Run: `cargo test -p persona-core --test contracts`

Expected: FAIL because Cargo cannot find package `persona-core`.

- [ ] **Step 3: Create the workspace and the core contracts.**

Create root `Cargo.toml`:

```toml
[workspace]
members = ["crates/persona-core"]
resolver = "2"

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT"

[workspace.dependencies]
thiserror = "2"
uuid = { version = "1", features = ["v4"] }
```

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.85.0"
profile = "minimal"
components = ["clippy", "rustfmt"]
```

Create `crates/persona-core/Cargo.toml`:

```toml
[package]
name = "persona-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
thiserror.workspace = true
uuid.workspace = true
```

Create `crates/persona-core/src/lib.rs`:

```rust
mod error;
mod identity;
mod lifecycle;

pub use error::{EventError, IdentityError, LifecycleError};
pub use identity::{CorrelationId, OwnerId, SchemaVersion};
pub use lifecycle::{RuntimeEvent, RuntimeEventKind, RuntimeState};
```

Create `crates/persona-core/src/identity.rs`:

```rust
use crate::IdentityError;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaVersion {
    V1,
}

impl SchemaVersion {
    pub const fn as_u16(self) -> u16 {
        match self {
            Self::V1 => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub const fn from_uuid(value: Uuid) -> Self { Self(value) }
    pub const fn as_uuid(self) -> Uuid { self.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OwnerId(String);

impl TryFrom<&str> for OwnerId {
    type Error = IdentityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(IdentityError::BlankOwnerId);
        }
        Ok(Self(value.to_owned()))
    }
}

impl OwnerId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Create `crates/persona-core/src/lifecycle.rs`:

```rust
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
    occurred_at: std::time::SystemTime,
    kind: RuntimeEventKind,
}

impl RuntimeEvent {
    pub fn new(
        schema_version: SchemaVersion,
        correlation_id: CorrelationId,
        kind: RuntimeEventKind,
    ) -> Self {
        Self { schema_version, correlation_id, occurred_at: std::time::SystemTime::now(), kind }
    }

    pub fn ready(schema_version: SchemaVersion, correlation_id: CorrelationId) -> Self {
        Self::new(schema_version, correlation_id, RuntimeEventKind::RuntimeReady)
    }

    pub const fn schema_version(&self) -> SchemaVersion { self.schema_version }
    pub const fn correlation_id(&self) -> CorrelationId { self.correlation_id }
    pub fn occurred_at(&self) -> std::time::SystemTime { self.occurred_at }
    pub const fn kind(&self) -> RuntimeEventKind { self.kind }
}
```

Create `crates/persona-core/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("owner identifier must not be blank")]
    BlankOwnerId,
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("event queue is full")]
    QueueFull,
    #[error("event dispatcher is closed")]
    Closed,
}

#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("invalid lifecycle transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: crate::RuntimeState,
        to: crate::RuntimeState,
    },
}
```

- [ ] **Step 4: Run the core tests and formatter.**

Run: `cargo fmt --check; cargo test -p persona-core --test contracts`

Expected: PASS.

- [ ] **Step 5: Commit the workspace and core contracts.**

```bash
git add Cargo.toml rust-toolchain.toml crates/persona-core
git commit -m "feat: add runtime core contracts"
```

## Task 2: Add Versioned TOML Configuration and Metadata-Safe Logging Ports

**Files:**
- Create: `crates/runtime/Cargo.toml`
- Create: `crates/runtime/src/lib.rs`
- Create: `crates/runtime/src/config.rs`
- Create: `crates/runtime/src/logging.rs`
- Create: `crates/runtime/tests/config.rs`
- Create: `crates/runtime/tests/logging.rs`

- [ ] **Step 1: Write failing configuration and logging tests.**

```rust
use persona_core::CorrelationId;
use persona_runtime::{ConfigError, RuntimeConfig, SafeLogRecord};

#[test]
fn config_rejects_unknown_fields_and_zero_queue_capacity() {
    let input = "schema_version = 1\ndata_dir = 'data'\ndatabase_path = 'persona.db'\nlog_level = 'info'\nevent_queue_capacity = 0\nunknown = true";
    assert!(matches!(RuntimeConfig::from_toml(input), Err(ConfigError::Invalid(_))));
}

#[test]
fn safe_log_record_has_no_message_or_payload_field() {
    let correlation_id = CorrelationId::new();
    let record = SafeLogRecord::transition("runtime", "ready", correlation_id);
    assert_eq!(record.component(), "runtime");
    assert_eq!(record.transition(), "ready");
    assert_eq!(record.correlation_id(), correlation_id);
}
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p persona-runtime --test config --test logging`

Expected: FAIL because package `persona-runtime` does not exist.

- [ ] **Step 3: Add the runtime package, configuration contract, and logging port.**

Create `crates/runtime/Cargo.toml`:

```toml
[package]
name = "persona-runtime"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
persona-core = { path = "../persona-core" }
serde = { version = "1", features = ["derive"] }
thiserror.workspace = true
toml = "0.8"
tracing = "0.1"
```

Before running the runtime tests, modify the root `Cargo.toml` workspace member list to:

```toml
members = ["crates/persona-core", "crates/runtime"]
```

Create `crates/runtime/src/lib.rs`:

```rust
mod config;
mod logging;

pub use config::{ConfigError, RuntimeConfig};
pub use logging::{LoggerFactory, RuntimeLogger, SafeLogRecord, TracingLoggerFactory};
```

Create `crates/runtime/src/config.rs`:

```rust
use serde::Deserialize;
use std::{num::NonZeroUsize, path::PathBuf};

const CONFIG_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    schema_version: u16,
    data_dir: PathBuf,
    database_path: PathBuf,
    log_level: String,
    event_queue_capacity: usize,
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub log_level: String,
    pub event_queue_capacity: NonZeroUsize,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration is invalid: {0}")]
    Invalid(String),
}

impl RuntimeConfig {
    pub fn from_toml(input: &str) -> Result<Self, ConfigError> {
        let file: FileConfig = toml::from_str(input)
            .map_err(|error| ConfigError::Invalid(error.to_string()))?;
        if file.schema_version != CONFIG_SCHEMA_VERSION {
            return Err(ConfigError::Invalid("unsupported schema version".to_owned()));
        }
        if file.data_dir.as_os_str().is_empty() || file.database_path.as_os_str().is_empty() {
            return Err(ConfigError::Invalid("data and database paths must not be empty".to_owned()));
        }
        let event_queue_capacity = NonZeroUsize::new(file.event_queue_capacity)
            .ok_or_else(|| ConfigError::Invalid("event queue capacity must be positive".to_owned()))?;
        Ok(Self {
            data_dir: file.data_dir,
            database_path: file.database_path,
            log_level: file.log_level,
            event_queue_capacity,
        })
    }
}
```

Create `crates/runtime/src/logging.rs`:

```rust
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeLogRecord {
    component: &'static str,
    transition: &'static str,
    correlation_id: persona_core::CorrelationId,
}

impl SafeLogRecord {
    pub const fn transition(
        component: &'static str,
        transition: &'static str,
        correlation_id: persona_core::CorrelationId,
    ) -> Self {
        Self { component, transition, correlation_id }
    }
    pub const fn component(&self) -> &'static str { self.component }
    pub const fn transition(&self) -> &'static str { self.transition }
    pub const fn correlation_id(&self) -> persona_core::CorrelationId { self.correlation_id }
}

pub trait RuntimeLogger: Send + Sync {
    fn record(&self, record: SafeLogRecord);
}

pub trait LoggerFactory: Send + Sync {
    fn initialize(&self, log_level: &str) -> Result<Arc<dyn RuntimeLogger>, String>;
}

pub struct TracingLoggerFactory;
pub struct TracingRuntimeLogger;

impl LoggerFactory for TracingLoggerFactory {
    fn initialize(&self, _log_level: &str) -> Result<Arc<dyn RuntimeLogger>, String> {
        Ok(Arc::new(TracingRuntimeLogger))
    }
}

impl RuntimeLogger for TracingRuntimeLogger {
    fn record(&self, record: SafeLogRecord) {
        tracing::info!(
            component = record.component(),
            transition = record.transition(),
            correlation_id = ?record.correlation_id(),
            "runtime lifecycle"
        );
    }
}
```

- [ ] **Step 4: Run the configuration and logging tests.**

Run: `cargo test -p persona-runtime --test config --test logging`

Expected: PASS.

- [ ] **Step 5: Commit configuration and safe logging ports.**

```bash
git add crates/runtime
git commit -m "feat: add runtime configuration and safe logging"
```

## Task 3: Implement the Bounded Event Dispatcher

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/runtime/Cargo.toml`
- Modify: `crates/runtime/src/lib.rs`
- Create: `crates/runtime/src/dispatcher.rs`
- Create: `crates/runtime/tests/dispatcher.rs`

- [ ] **Step 1: Write failing tests for ordering, backpressure, and closure.**

```rust
use persona_core::{CorrelationId, RuntimeEvent, SchemaVersion};
use persona_runtime::EventDispatcher;

#[tokio::test]
async fn dispatcher_returns_backpressure_and_rejects_events_after_close() {
    let (dispatcher, mut receiver) = EventDispatcher::bounded(1);
    let event = RuntimeEvent::ready(SchemaVersion::V1, CorrelationId::new());

    dispatcher.try_publish(event).unwrap();
    assert!(dispatcher.try_publish(event).is_err());
    assert_eq!(receiver.recv().await, Some(event));

    dispatcher.close();
    assert!(dispatcher.try_publish(event).is_err());
}
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `cargo test -p persona-runtime --test dispatcher`

Expected: FAIL because `EventDispatcher` is not exported.

- [ ] **Step 3: Add Tokio and implement the dispatcher without unbounded task creation.**

Add to root `Cargo.toml` under `[workspace.dependencies]`:

```toml
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync"] }
```

Add to `crates/runtime/Cargo.toml` dependencies:

```toml
tokio.workspace = true
```

Create `crates/runtime/src/dispatcher.rs`:

```rust
use persona_core::{EventError, RuntimeEvent};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct EventDispatcher {
    sender: Arc<Mutex<Option<mpsc::Sender<RuntimeEvent>>>>,
}

impl EventDispatcher {
    pub fn bounded(capacity: usize) -> (Self, mpsc::Receiver<RuntimeEvent>) {
        let (sender, receiver) = mpsc::channel(capacity);
        (Self { sender: Arc::new(Mutex::new(Some(sender))) }, receiver)
    }

    pub fn try_publish(&self, event: RuntimeEvent) -> Result<(), EventError> {
        let sender = self.sender.lock().expect("dispatcher mutex must not be poisoned").clone();
        let sender = sender.ok_or(EventError::Closed)?;
        sender.try_send(event).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => EventError::QueueFull,
            mpsc::error::TrySendError::Closed(_) => EventError::Closed,
        })
    }

    pub fn close(&self) {
        self.sender.lock().expect("dispatcher mutex must not be poisoned").take();
    }
}
```

Add to `crates/runtime/src/lib.rs`:

```rust
mod dispatcher;
pub use dispatcher::EventDispatcher;
```

- [ ] **Step 4: Run dispatcher tests and Clippy.**

Run: `cargo test -p persona-runtime --test dispatcher; cargo clippy -p persona-runtime -- -D warnings`

Expected: PASS.

- [ ] **Step 5: Commit the bounded dispatcher.**

```bash
git add Cargo.toml crates/runtime
git commit -m "feat: add bounded runtime event dispatcher"
```

## Task 4: Add SQLite Storage Ports, Migrations, and Owner-Scoped Audit Metadata

**Files:**
- Create: `crates/database/Cargo.toml`
- Create: `crates/database/src/lib.rs`
- Create: `crates/database/src/port.rs`
- Create: `crates/database/src/sqlite.rs`
- Create: `crates/database/tests/sqlite_store.rs`

- [ ] **Step 1: Write failing storage tests for idempotent migrations, transactional audit writes, and owner isolation.**

```rust
use persona_core::{CorrelationId, OwnerId};
use persona_database::{AuditActor, AuditReason, AuditRecord, SqliteStorageFactory, Storage, StorageFactory};
use tempfile::tempdir;

#[tokio::test]
async fn sqlite_migrates_idempotently_and_scopes_audit_records_to_owner() {
    let directory = tempdir().unwrap();
    let factory = SqliteStorageFactory;
    let store = factory.open(&directory.path().join("persona.db")).await.unwrap();
    store.migrate().await.unwrap();
    store.migrate().await.unwrap();

    let owner_a = OwnerId::try_from("owner-a").unwrap();
    let owner_b = OwnerId::try_from("owner-b").unwrap();
    store.write_audit(AuditRecord::new(
        owner_a.clone(),
        AuditActor::System,
        AuditReason::Migration,
        CorrelationId::new(),
    )).await.unwrap();

    assert_eq!(store.list_audit(&owner_a).await.unwrap().len(), 1);
    assert!(store.list_audit(&owner_b).await.unwrap().is_empty());
}
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `cargo test -p persona-database --test sqlite_store`

Expected: FAIL because package `persona-database` does not exist.

- [ ] **Step 3: Add the database crate, storage interfaces, and SQLite implementation.**

Add to root `Cargo.toml` under `[workspace.dependencies]`:

```toml
async-trait = "0.1"
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio-rustls", "sqlite"] }
tempfile = "3"
```

Create `crates/database/Cargo.toml`:

```toml
[package]
name = "persona-database"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
async-trait.workspace = true
persona-core = { path = "../persona-core" }
sqlx.workspace = true
thiserror.workspace = true
tokio.workspace = true
uuid.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

Before running database tests, modify the root `Cargo.toml` workspace member list to:

```toml
members = ["crates/persona-core", "crates/database", "crates/runtime"]
```

Create `crates/database/src/lib.rs`:

```rust
mod port;
mod sqlite;

pub use port::{AuditActor, AuditReason, AuditRecord, Storage, StorageError, StorageFactory};
pub use sqlite::SqliteStorageFactory;
```

Create `crates/database/src/port.rs`:

```rust
use async_trait::async_trait;
use persona_core::{CorrelationId, OwnerId};
use std::{path::Path, sync::Arc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuditRecord {
    pub id: Uuid,
    pub owner_id: OwnerId,
    pub actor: AuditActor,
    pub reason: AuditReason,
    pub correlation_id: CorrelationId,
    pub occurred_at: std::time::SystemTime,
}

impl AuditRecord {
    pub fn new(owner_id: OwnerId, actor: AuditActor, reason: AuditReason, correlation_id: CorrelationId) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            actor,
            reason,
            correlation_id,
            occurred_at: std::time::SystemTime::now(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditActor { System, User }

impl AuditActor {
    pub const fn as_str(self) -> &'static str {
        match self { Self::System => "system", Self::User => "user" }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditReason { Migration }

impl AuditReason {
    pub const fn as_str(self) -> &'static str {
        match self { Self::Migration => "migration" }
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
```

Create `crates/database/src/sqlite.rs`. Implement a `SqliteStorage { pool: sqlx::SqlitePool }` and `SqliteStorageFactory`. `open` must use `SqliteConnectOptions::create_if_missing(true)` and `SqlitePoolOptions::max_connections(1)`. `migrate` must transactionally create these tables and insert version `1` only when absent:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    actor TEXT NOT NULL,
    reason TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_events_owner ON audit_events(owner_id);
```

`write_audit` must use a parameterized `INSERT` in a transaction, using `AuditActor::as_str`, `AuditReason::as_str`, and seconds since `UNIX_EPOCH` for the timestamp. `list_audit` must use `WHERE owner_id = ? ORDER BY created_at ASC`, parse `id` as `Uuid`, rebuild `CorrelationId` with `CorrelationId::from_uuid`, map only `system`/`user` and `migration` back to their enum variants, and rebuild the timestamp from `UNIX_EPOCH`; invalid persisted identifiers, categories, or timestamps map to `StorageError::InvalidAuditMetadata` without echoing a record value. Map all SQLx errors to `StorageError` with operation context but no SQL statement or record value. `close` calls `SqlitePool::close().await`.

- [ ] **Step 4: Run storage tests.**

Run: `cargo test -p persona-database --test sqlite_store`

Expected: PASS. Add a second test that opens a path whose parent does not exist and asserts `StorageError::Unavailable` without requiring any user data.

- [ ] **Step 5: Commit the SQLite adapter.**

```bash
git add Cargo.toml crates/database
git commit -m "feat: add sqlite runtime storage"
```

## Task 5: Compose the Runtime and Verify the Full Lifecycle

**Files:**
- Modify: `crates/runtime/Cargo.toml`
- Modify: `crates/runtime/src/lib.rs`
- Create: `crates/runtime/src/runtime.rs`
- Create: `crates/runtime/tests/lifecycle.rs`
- Create: `crates/runtime/tests/failure_cleanup.rs`

- [ ] **Step 1: Write failing integration tests for ready-state ordering, idempotent shutdown, and required-dependency failure.**

```rust
use persona_database::SqliteStorageFactory;
use persona_runtime::{EventDispatcher, Runtime, RuntimeConfig, TracingLoggerFactory};
use tempfile::tempdir;

#[tokio::test]
async fn runtime_publishes_lifecycle_events_and_stops_idempotently() {
    let directory = tempdir().unwrap();
    let config = RuntimeConfig::from_toml(&format!(
        "schema_version = 1\ndata_dir = '{}'\ndatabase_path = 'persona.db'\nlog_level = 'info'\nevent_queue_capacity = 16",
        directory.path().display()
    )).unwrap();
    let (dispatcher, mut events) = EventDispatcher::bounded(16);
    let runtime = Runtime::new(config, Box::new(TracingLoggerFactory), Box::new(SqliteStorageFactory), dispatcher);

    runtime.start().await.unwrap();
    runtime.stop().await.unwrap();
    runtime.stop().await.unwrap();

    let mut kinds = Vec::new();
    for _ in 0..5 {
        kinds.push(events.recv().await.unwrap().kind());
    }
    assert_eq!(kinds.len(), 5);
}
```

- [ ] **Step 2: Run integration tests to verify they fail.**

Run: `cargo test -p persona-runtime --test lifecycle --test failure_cleanup`

Expected: FAIL because `Runtime` is not defined.

- [ ] **Step 3: Implement explicit runtime composition and cleanup.**

Add to `crates/runtime/Cargo.toml`:

```toml
persona-database = { path = "../database" }
tokio.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

Create `crates/runtime/src/runtime.rs` with these public types and behavior:

```rust
pub struct Runtime {
    config: RuntimeConfig,
    logger_factory: Box<dyn LoggerFactory>,
    storage_factory: Box<dyn StorageFactory>,
    dispatcher: EventDispatcher,
    state: std::sync::Mutex<RuntimeState>,
    storage: tokio::sync::Mutex<Option<std::sync::Arc<dyn Storage>>>,
    logger: std::sync::Mutex<Option<std::sync::Arc<dyn RuntimeLogger>>>,
    correlation_id: std::sync::Mutex<Option<persona_core::CorrelationId>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("logging initialization failed")]
    Logging,
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Event(#[from] persona_core::EventError),
}
```

`Runtime::new` stores only explicit dependencies. `start` must:

1. change `New` to `Starting`;
2. initialize the logger, mapping the factory error to `RuntimeError::Logging`;
3. create one `CorrelationId` for the runtime run, retain it in `correlation_id`, and publish `RuntimeStarting` through the existing dispatcher with that identifier;
4. resolve `database_path` relative to `data_dir`, create the data directory, open SQLite, and call `migrate`;
5. retain the opened `Storage` and initialized logger, transition to `StorageReady`, publish `StorageReady`, then transition to `Ready` and publish `RuntimeReady`; every lifecycle event and `SafeLogRecord` for the run uses the same retained correlation identifier;
6. if any required action after logging fails, call the same internal cleanup method used by `stop`, set `Failed`, and return the original structured error.

`stop` must return `Ok(())` when already stopped or failed. From `Ready`, it publishes `RuntimeStopping`, transitions to `Stopping`, closes the stored database, publishes `RuntimeStopped`, closes the dispatcher, and transitions to `Stopped`. It must never log or publish configuration values, database values, or event payloads. Add the module and `Runtime`/`RuntimeError` exports to `lib.rs`.

- [ ] **Step 4: Strengthen the integration test with exact event ordering and a correlation-ID invariant.**

Replace the collection in the lifecycle test with:

```rust
use persona_core::RuntimeEventKind;

let mut kinds = Vec::new();
for _ in 0..5 {
    kinds.push(events.recv().await.unwrap().kind());
}
assert_eq!(kinds, vec![
    RuntimeEventKind::RuntimeStarting,
    RuntimeEventKind::StorageReady,
    RuntimeEventKind::RuntimeReady,
    RuntimeEventKind::RuntimeStopping,
    RuntimeEventKind::RuntimeStopped,
]);
```

Also assert all five received events have the same `correlation_id()` and a timestamp no later than `SystemTime::now()`.

Add a `FailingLoggerFactory` to `failure_cleanup.rs` that returns `Err("not logged")` and assert that `start` returns `RuntimeError::Logging`, no ready event is emitted, and a subsequent `stop` succeeds.

- [ ] **Step 5: Run runtime integration tests and the full workspace suite.**

Run: `cargo test -p persona-runtime --test lifecycle --test failure_cleanup; cargo test --workspace`

Expected: PASS.

- [ ] **Step 6: Commit runtime composition.**

```bash
git add crates/runtime
git commit -m "feat: add runtime lifecycle composition"
```

## Task 6: Document the Delivered Boundary and Run the Release-Quality Checks

**Files:**
- Modify: `docs/README.md`
- Modify: `docs/RUNTIME.md`
- Modify: `README.md`

- [ ] **Step 1: Add a documentation test checklist before editing implementation-facing docs.**

Add this checklist to the pull request description or local review notes and require every item to be true before commit:

```text
- The documentation says Phase 1 has no CLI, desktop shell, AI service, plugin, scheduler, or user-content schema.
- The documented crate dependency direction is persona-core <- database and runtime -> both.
- The documented acceptance commands match the workspace commands that pass locally.
- No document claims the runtime sends messages or accesses cloud services.
```

- [ ] **Step 2: Update the documentation index and runtime document.**

In `docs/README.md`, add a `Phase 1` subsection under `Planning and Decisions` linking to:

```markdown
- [Phase 1 runtime foundation design](superpowers/specs/2026-07-12-phase-1-runtime-foundation-design.md)
- [Phase 1 implementation plan](superpowers/plans/2026-07-12-phase-1-runtime-foundation.md)
```

In `docs/RUNTIME.md`, add a `Phase 1 implementation slice` subsection immediately before `## 12. Acceptance Criteria`. State that Phase 1 implements configuration, metadata-safe logging, SQLite migration/audit metadata, bounded in-process lifecycle events, and idempotent shutdown as libraries only; its executable host, AI integration, plugins, scheduler, and domain storage are explicitly deferred.

In `README.md`, add one sentence after the architecture description: `Phase 1 currently establishes the local Rust runtime foundation; user-facing applications and AI capabilities follow in later milestones.`

- [ ] **Step 3: Run the complete quality gate.**

Run: `cargo fmt --check; cargo clippy --workspace -- -D warnings; cargo test --workspace; git diff --check`

Expected: every command exits with code 0 and `git diff --check` produces no output.

- [ ] **Step 4: Review logs and fixtures for private data.**

Run: `rg -n "prompt|token|conversation|memory value|credential|secret|api[_-]?key" crates tests`

Expected: no fixture, test assertion, or normal log statement contains personal content, credentials, or provider tokens. Any allowed mention must be a policy assertion only.

- [ ] **Step 5: Commit the documentation and verification evidence.**

```bash
git add README.md docs/README.md docs/RUNTIME.md
git commit -m "docs: record phase 1 runtime boundary"
```

## Final Verification

- [ ] Run `git status --short` and confirm only intentionally uncommitted changes remain.
- [ ] Run `git log --oneline -6` and confirm the six focused commits describe the workspace, configuration/logging, dispatcher, SQLite storage, runtime composition, and documentation.
- [ ] Re-read `docs/superpowers/specs/2026-07-12-phase-1-runtime-foundation-design.md` and confirm every scoped requirement is covered by Tasks 1 through 6.
