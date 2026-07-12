# Phase 1 Runtime Foundation Design

**Status:** Approved design pending written-spec review
**Date:** 2026-07-12

## Purpose

Phase 1 establishes Persona's local runtime foundation without creating a user-facing executable. It proves that the future desktop host can compose and stop core infrastructure safely while preserving the architecture's local-first, privacy, and replaceability requirements.

The phase produces Rust libraries and automated integration tests only. It does not create a CLI application, desktop shell, AI service, plugin host, scheduler, collector, or domain features such as conversations, memories, profiles, and replies.

## Scope and Constraints

- Windows is the first supported development platform. Core crates remain platform-neutral and contain no Windows-specific domain logic.
- The only acceptance host is an automated integration test. No package under `apps/` is an executable in this phase.
- Configuration is a versioned local TOML file. Tests may inject in-memory configuration. Environment overrides, secret storage, and hot reload are out of scope.
- SQLite is the concrete local store. Phase 1 persists migration state and privacy-safe audit metadata only.
- Events are in-process, typed, bounded, and non-durable. Schedulers, persistent event logs, plugins, and AI-service calls are deferred.
- Logs must not include configuration secrets, database values, or any user content.

## Architecture

The Cargo workspace enables exactly three crates in Phase 1:

```text
persona-core  <-  database
      ^               ^
      +---- runtime --+
```

### `persona-core`

`persona-core` contains infrastructure-independent contracts and types: owner and correlation identifiers, schema versions, runtime lifecycle states, event envelopes, and structured errors. It depends on no async runtime, database driver, filesystem API, or logging implementation.

### `database`

`database` defines storage-facing ports and their SQLite implementation. It owns database opening, transactional migrations, migration state, and audit metadata. It does not expose database connections or SQL types to consumers. It contains no conversation, memory, profile, collector, or provider data model.

### `runtime`

`runtime` is the composition root. Its constructor receives explicit dependencies and it coordinates configuration validation, privacy-safe logging, database initialization and migration, a bounded event dispatcher, lifecycle publication, and reverse-order shutdown. It must not use global mutable state.

Future crates remain directory placeholders until their phase requires a stable contract. This avoids empty abstractions and preserves the documented architectural direction.

## Lifecycle and Events

The runtime performs initialization in this order:

1. Load and validate the versioned TOML configuration.
2. Initialize metadata-only structured logging.
3. Open the SQLite database and run migrations transactionally.
4. Create the bounded event dispatcher.
5. Publish `RuntimeStarting`, `StorageReady`, and `RuntimeReady` as applicable.

Shutdown stops new event intake, closes the dispatcher, releases database resources, finalizes logging, and publishes `RuntimeStopping` and `RuntimeStopped` where the dispatcher remains available. The shutdown operation is idempotent.

Each event envelope carries schema version, timestamp, correlation identifier, and a typed payload. The Phase 1 event set is limited to runtime and storage lifecycle facts. Event queues apply backpressure by returning an explicit error when full; handlers are not spawned without bounds.

## Storage, Configuration, and Privacy

The TOML configuration contains only a schema version, data directory, database path, log level, and event queue capacity. Invalid values, unsupported versions, missing required fields, and unknown incompatible fields fail validation before storage initialization.

SQLite migrations run within transactions. A failed migration leaves the database in a state that can be retried, and the runtime never reports ready after a migration failure. Audit records include owner scope, actor category, reason category, timestamp, and correlation identifier; they exclude user content and secrets. All audit queries enforce owner scope.

Logging records component names, lifecycle transitions, error categories, correlation identifiers, and elapsed time. It never records TOML values that may become sensitive, database record values, raw event payloads, prompts, credentials, or tokens.

## Failure Behavior

Configuration, logging, and SQLite migration are required dependencies. Failure in any of them returns a structured, actionable error, prevents the ready state, and closes resources initialized earlier in the sequence. Error messages may identify a configuration field or a safe filesystem location, but must not reveal secret values or database content.

No optional dependency is introduced in this phase. The runtime does not attempt cloud fallback, network access, AI processing, or external message delivery.

## Testing and Acceptance

Completion requires the following checks:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

The test suite uses temporary directories and SQLite files only. Fixtures contain no real personal data, credentials, prompts, or embeddings. It demonstrates:

1. Valid TOML configuration loads, while invalid values and unsupported versions return typed errors.
2. The first runtime initialization applies migrations and a second initialization is idempotent.
3. Migration or database-open failures prevent ready state and release initialized resources.
4. Lifecycle events occur in the documented order and include schema versions and correlation identifiers.
5. A full event queue returns a backpressure error, closed dispatchers reject new events, and repeated shutdown is safe.
6. Audit writes and owner-scoped reads are transactional and isolated by owner.
7. Captured logs omit configuration-sensitive values and all database record values.

## Explicit Deferrals

The following work is not part of this plan: application executables, desktop UI, Tauri, Python or FastAPI setup, local HTTP service calls, provider selection, secrets, conversations, memories, profiles, retrieval, reply generation, scheduling, collectors, plugins, DuckDB, vector indexes, persistent event delivery, and multi-platform release support.

## Rationale

This scope is the smallest vertical foundation that demonstrates the Phase 1 completion criterion without borrowing implementation from later milestones. It keeps the domain free of SQLx, Tokio, and host concerns; gives the future desktop application a tested library boundary; and establishes privacy-safe operational behavior before Persona handles personal data.
