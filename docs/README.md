# Persona Documentation

This directory contains the engineering documentation for Persona. These documents define the contracts that implementation must follow.

## Foundation

- [Architecture](ARCHITECTURE.md): system boundaries and layering.
- [Development plan](PLAN.md): phases and completion criteria.
- [Agent guidelines](../AGENTS.md): repository-wide engineering and coding conventions.
- [Contributing](../CONTRIBUTING.md): contribution and review workflow.

## Domain and Data

- [Memory](MEMORY.md): memory lifecycle and explainability requirements.
- [User model](USER_MODEL.md): profile dimensions and update rules.
- [Database](DATABASE.md): storage ownership and repository contracts.

## Runtime Interfaces

- [Runtime](RUNTIME.md): Rust lifecycle and event model.
- [AI service](AI_SERVICE.md): Python capability boundary.
- [IPC](IPC.md): Rust-Python service contract.
- [Plugin](PLUGIN.md): extension boundary and permissions.

## Planning and Decisions

### Phase 1

- [Phase 1 runtime foundation design](superpowers/specs/2026-07-12-phase-1-runtime-foundation-design.md)
- [Phase 1 implementation plan](superpowers/plans/2026-07-12-phase-1-runtime-foundation.md)

- [Roadmap](ROADMAP.md): outcome-oriented release milestones.
- [ADR 0001](adr/0001-service-interface.md): service-interface decision record.

Architecture documentation is authoritative. Any implementation that requires changing an interface or boundary must update the relevant document and decision record first.
