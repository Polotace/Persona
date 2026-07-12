# ADR 0001: Local HTTP Service Interface

## Status

Accepted for the initial implementation.

## Context

Persona separates Rust application responsibilities from Python AI capabilities. The boundary must keep both runtimes independently testable, avoid direct language bindings, and remain compatible with local-first operation.

## Decision

Use a local HTTP service bound to loopback with JSON payloads and versioned capability schemas. The runtime launches or connects to the service locally and authenticates each request with a per-launch secret. Detailed protocol rules are defined in [IPC.md](../IPC.md).

## Consequences

Rust and Python can evolve independently and use ordinary HTTP tooling for contract tests and diagnostics. There is transport overhead compared with direct bindings, but the boundary remains observable and replaceable. A future IPC or gRPC transport may reuse the same schemas after an explicit architecture decision.
