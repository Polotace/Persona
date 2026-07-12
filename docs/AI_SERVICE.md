# AI Service Specification

## Boundary

The Python AI service provides intelligence capabilities to the Rust application. It owns prompt construction, model invocation, extraction, style analysis, retrieval orchestration, and embeddings. It does not own application lifecycle, database access, UI state, or message delivery.

## Capabilities

The stable capability surface is:

- `ExtractMemory`: returns candidate memories with source references and rationales.
- `AnalyzeStyle`: returns bounded style observations from user-authored messages.
- `GenerateEmbedding`: returns an embedding with provider and model metadata.
- `RetrieveContext`: ranks supplied candidate memories or embedding matches.
- `GenerateReply`: returns reply candidates and an explanation of the provided context used.

Capabilities exchange structured request and response models defined by [IPC.md](IPC.md). Provider-specific prompts, token counts, and SDK response objects remain internal.

## Provider Model

The service uses provider and embedding interfaces so local models and optional cloud providers are interchangeable. Provider configuration identifies model, capability support, context limits, locality, and user authorization state. A cloud provider cannot be selected unless explicitly enabled.

## Safety and Failure Behavior

The service validates all inputs, returns typed errors, and never invents source identifiers. On model or provider failure, it returns no result or a degraded result with an explicit status; it never sends messages or changes persistent state. The Rust side decides whether to persist validated candidates.

## Observability

Record capability name, provider category, latency, token or compute usage where available, outcome, and correlation identifier. Do not record prompts, conversation text, raw memory values, or credentials in ordinary logs.
