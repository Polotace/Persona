# Contributing to Persona

Persona prioritizes trustworthy, local, and maintainable personal intelligence over feature velocity.

## Before You Start

Read [the engineering documentation](docs/README.md), [the architecture](docs/ARCHITECTURE.md), and [the agent guidelines](AGENTS.md). Discuss changes that introduce a new dependency, transport, persistent schema, provider integration, or plugin boundary before implementation.

## Contribution Expectations

- Keep changes focused and preserve existing architectural boundaries.
- Include tests appropriate to the changed behavior.
- Do not add real personal data, credentials, prompts, or embeddings to the repository.
- Update relevant documentation and an ADR when changing public contracts or architecture.
- Make privacy, data ownership, and user-control implications explicit in the pull request.

## Review

Reviewers check correctness, interface compatibility, local-first behavior, observability, error handling, and test coverage. Features that send messages, impersonate users, or make social decisions without explicit approval will not be accepted.

## Issue and Pull Request Templates

Use the provided GitHub Issue forms for reproducible bugs and focused feature requests. Pull requests must follow the repository template and include the relevant validation, documentation, architecture, and privacy checks.
