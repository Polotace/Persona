# AGENT.md

> **Persona Development Agent Specification**
>
> This document defines how AI coding agents should participate in the development of Persona.
>
> It is the highest-level engineering guideline for all AI-assisted development.
>
> **Status:** Draft v1.0

---

# 1. Project Overview

## Project Name

**Persona**

## Mission

Persona is a **Local-First, Self-Evolving Personal AI Reply Agent**.

Its purpose is to help users communicate naturally by continuously learning from their conversations.

Persona should gradually understand:

* who the user is
* how the user communicates
* what the user knows
* what the user prefers

The system should evolve over time rather than requiring manual configuration.

---

# 2. Development Philosophy

Persona is **NOT** a chatbot.

Persona is **NOT** a traditional RAG project.

Persona is **NOT** a wrapper around LLM APIs.

Persona is a long-running local AI platform whose core capability is **understanding people through long-term interaction**.

Every design decision should support this vision.

---

# 3. AI Agent Responsibilities

When contributing to Persona, AI agents should prioritize:

1. Architecture consistency.
2. Modular design.
3. Maintainability.
4. Local-first execution.
5. Privacy protection.
6. Explainable behavior.
7. Replaceable implementations.

AI agents should avoid introducing unnecessary complexity.

---

# 4. Core Engineering Principles

## Local First

Everything should work locally whenever possible.

Cloud services must always be optional.

---

## User Owns Data

The user owns:

* conversations
* memories
* embeddings
* profiles
* settings

The application never owns user data.

---

## Explainability

Every memory should be traceable.

Every AI decision should have a reason.

Every generated reply should be explainable.

---

## Long-Term Evolution

Persona should continuously evolve.

It should improve through observation instead of manual configuration.

---

## Separation of Concerns

Every subsystem must have a single responsibility.

Business logic should never depend on implementation details.

---

# 5. Architecture Principles

Persona is divided into two independent systems.

## Rust Layer

Responsibilities:

* Desktop application
* Runtime
* Scheduler
* Memory Engine
* Storage
* Plugin System
* Event Bus
* IPC
* Configuration

Rust owns the application lifecycle.

---

## Python Layer

Responsibilities:

* Memory extraction
* Style analysis
* Prompt construction
* Semantic retrieval
* Reply generation
* LLM integration
* Embedding

Python provides AI capabilities only.

Python should not own business logic.

---

# 6. Communication Rules

Rust and Python must communicate through a stable service interface.

Allowed:

* HTTP
* gRPC
* IPC

Forbidden:

* Direct language bindings
* Shared business logic
* Tight coupling

The Rust layer must never depend on Python internals.

The Python layer must never depend on Rust implementation details.

---

# 7. Modularity Rules

Every module should be independently replaceable.

Examples include:

* LLM providers
* Embedding providers
* Database implementations
* Collectors
* Plugins
* UI frameworks

Changing one module should not require rewriting another.

---

# 8. Memory Philosophy

Memory is the core of Persona.

Conversation history is not memory.

Memory is extracted from conversations.

Only meaningful information should become memory.

Every memory should include:

* source
* confidence
* timestamp
* category
* lifecycle

Persona must support forgetting.

Remembering everything is considered a design failure.

---

# 9. User Modeling

Persona builds a user model over time.

The model includes:

* interests
* projects
* skills
* preferences
* communication habits
* relationships

Knowledge and communication style are independent.

Never mix them.

---

# 10. Reply Generation

Replies should be generated using:

* current conversation
* recent context
* user profile
* long-term memory
* communication style
* relationship context

The objective is to sound like the user.

Not like an AI assistant.

---

# 11. Coding Principles

AI agents should always:

* prefer interfaces over concrete implementations
* minimize coupling
* maximize readability
* document architectural decisions
* avoid premature optimization

Never hardcode:

* model providers
* database engines
* plugin implementations

Everything should be abstracted.

## General Standards

These rules apply to all code, scripts, tests, and configuration.

* Keep domain logic independent of UI, storage, transport, and model providers.
* Prefer small modules with one responsibility and explicit input/output types.
* Do not log conversation content, memory values, prompts, credentials, or access tokens at normal log levels.
* Return structured errors with actionable context. Do not swallow failures or use empty catch-all handlers.
* Add tests for behavior changes and regression tests for defects.
* Document public interfaces, persistent schemas, event payloads, and non-obvious privacy decisions.

## Rust Standards

* Format with `cargo fmt`; lint with `cargo clippy -- -D warnings`.
* Use `thiserror` for library errors and `anyhow` only at application boundaries.
* Pass dependencies through traits or constructors; do not access global mutable state.
* Use `tracing` with structured fields and redact user data by default.
* Keep crates directional: domain crates must not depend on Tauri, SQLx, HTTP clients, or provider SDKs.

## Python Standards

* Format with Ruff; type-check public APIs with Pyright or an equivalent configured checker.
* Use Pydantic models for HTTP and service-boundary payloads.
* Keep FastAPI routes thin; capability logic belongs in service modules.
* Use dependency injection for model providers, embedding providers, and repositories.
* Never expose provider-specific response objects outside the AI service boundary.

## Testing and Review

* Unit tests cover domain rules and pure transformations.
* Contract tests validate Rust-Python request and response schemas.
* Integration tests use temporary local databases and fake model providers.
* No test fixture may contain real personal conversations, credentials, or private embeddings.
* Keep commits focused on one coherent change.
* Update documentation when a public interface, schema, event, or architectural boundary changes.
* A reviewer must be able to identify data ownership, failure behavior, and user-control implications from the change.

---

# 12. Documentation Rules

Architecture must be documented before implementation.

Every major subsystem should include:

* purpose
* responsibilities
* boundaries
* dependencies
* extension points

Diagrams should be used whenever they improve clarity.

---

# 13. Repository Structure

Recommended layout:

```text
persona/

├── apps/
│   ├── desktop/
│   └── ai-service/
│
├── crates/
│   ├── persona-core/
│   ├── collector/
│   ├── database/
│   ├── memory/
│   ├── profile/
│   ├── runtime/
│   ├── plugin/
│   └── common/
│
├── agents/
│   ├── api/
│   ├── llm/
│   ├── memory/
│   ├── retrieval/
│   ├── reply/
│   ├── style/
│   └── prompt/
│
├── docs/
├── scripts/
├── tests/
└── examples/
```

This structure may evolve, but architectural boundaries should remain stable.

---

# 14. Development Roadmap

Development should follow this order:

1. Documentation
2. Core Architecture
3. Runtime
4. Storage
5. AI Service
6. IPC
7. Memory System
8. Reply Engine
9. Plugin System
10. User Interface

Implementation should never begin before architecture documentation is completed.

---

# 15. AI Agent Workflow

When assigned a task:

1. Understand the architectural context.
2. Identify affected modules.
3. Minimize cross-module impact.
4. Update documentation if architecture changes.
5. Keep interfaces stable.
6. Preserve backward compatibility where possible.

If implementation conflicts with this document, the AI agent should stop and propose an architectural discussion before proceeding.

---

# 16. Non-Goals

Persona is not intended to:

* replace human communication
* impersonate users without consent
* send messages or make social decisions on behalf of users without explicit approval
* collect cloud-based personal data
* become a general-purpose AI framework

Persona is focused on building a trustworthy, privacy-preserving, local communication assistant.

---

# 17. Guiding Principles

Every contribution should make Persona:

* more understandable
* more modular
* more private
* more explainable
* more maintainable
* more user-centric

Architecture is valued over implementation.

Long-term maintainability is valued over short-term convenience.

The project should grow through thoughtful engineering rather than rapid accumulation of features.
