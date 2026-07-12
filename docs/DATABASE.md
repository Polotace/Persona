# Database Specification

## Storage Ownership

Conversation records are the immutable source of truth. Memory, user-model, configuration, and audit records are derived data owned by the local user. Access is mediated by repositories; UI code and AI providers must not access databases directly.

## Storage Responsibilities

| Store | Responsibility |
| --- | --- |
| SQLite | Application state, conversations, contacts, memories, profiles, settings, and audit metadata |
| DuckDB | Local analytical queries and aggregates derived from application data |
| Vector index | Local similarity search over approved embedding records; the provider is replaceable |

## Repository Contracts

Repositories expose domain-oriented operations such as storing an immutable conversation, querying active memories, applying a memory transition, and recording an audit event. They do not expose SQL, database handles, or provider-specific vector types to the domain layer.

All writes that change a memory or profile must record actor (`user` or `system`), reason, source references, and timestamp. Conversation records are append-only; corrections are represented by new metadata or derived records.

## Privacy and Lifecycle

Data remains local by default. Cloud model requests require an explicit enabled provider and must send the minimum context needed for that request. Deleting a conversation or memory must enqueue removal of dependent embeddings and derived profile evidence. Export must produce user-readable data and a documented machine-readable format. Backups are opt-in and must not contain credentials.

## Migration and Integrity

Schema changes use ordered, reversible-where-possible migrations. Each migration has an identifier, a forward operation, validation, and a rollback or recovery note. Repositories enforce owner scoping, foreign-key integrity, and transactional updates for multi-record memory transitions.
