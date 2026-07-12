# Rust-Python Service Interface

## Decision

The initial transport is local HTTP over loopback using JSON request and response bodies. This keeps Rust and Python independently runnable, inspectable, and testable. The capability schemas are transport-neutral so a future local IPC or gRPC transport can be introduced without changing domain contracts.

## Endpoint Shape

The AI service exposes a versioned capability endpoint:

```text
POST /v1/capabilities/{capability}
```

Every request contains `request_id`, `schema_version`, `owner_id`, `capability`, `payload`, and `context_policy`. Every response contains `request_id`, `schema_version`, `status`, `result`, `warnings`, and optional typed `error`.

`context_policy` identifies which local data classes the caller has authorized for the request. The AI service may not fetch additional application data itself.

## Compatibility and Errors

The service rejects unsupported major schema versions before processing. Additive fields are optional and backward compatible. Errors use a stable code, message safe for logs, retryability flag, and correlation identifier. Initial error codes are `INVALID_REQUEST`, `UNAUTHORIZED_CONTEXT`, `UNAVAILABLE`, `TIMEOUT`, `PROVIDER_FAILURE`, and `UNSUPPORTED_VERSION`.

## Reliability

Requests have caller-defined deadlines. The runtime retries only idempotent capabilities such as embedding generation, with bounded backoff. Reply generation and extraction requests include idempotency keys so duplicate delivery does not create duplicate work. No request permits the AI service to persist application records or send external messages.

## Security

The service binds to loopback only. A per-launch secret is passed through a local protected channel and required on each request. Request and response logs are metadata-only. Remote transport is out of scope for the initial interface.
