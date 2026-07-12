# Plugin Specification

## Purpose

Plugins extend Persona without modifying the core. They are optional and operate through versioned interfaces; a plugin cannot access internal application state by convention or direct imports.

## Plugin Types

- Collectors normalize messages from an approved local source.
- Platform adapters provide contact and conversation integration.
- AI providers implement a model or embedding adapter behind the AI service boundary.
- Importers and exporters exchange user-authorized data.

## Manifest

Each plugin declares an identifier, version, compatible host API version, type, entry point, requested permissions, and configuration schema. The host validates the manifest before loading the plugin.

## Permissions

Permissions are explicit and least-privilege: conversation read, conversation import, local filesystem scope, network access, model access, and export. A plugin receives no permission by default. Any network permission is disabled unless the user enables it.

## Lifecycle

Plugins progress through discovery, validation, user approval, initialization, active, disabled, and unload states. A plugin failure is isolated, reported through structured events, and must not prevent the runtime from starting. Plugins cannot autonomously send messages.

## Compatibility

The host maintains a versioned extension API. Breaking changes require a new major API version and a migration notice. Plugins exchange typed payloads only and must not depend on database schemas or internal Rust/Python modules.
