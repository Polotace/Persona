# User Model Specification

## Purpose

The user model is a slow-changing, explainable view of the user built from approved or high-confidence memories. It supports personalization but is not a hidden personality score and must remain editable by the user.

## Dimensions

The model maintains independent dimensions for interests, skills, projects, preferences, communication style, and relationships. Facts and style are separate: a style observation cannot become a factual claim, and a factual memory cannot alter style without relevant evidence.

## Profile Entry Contract

Each entry has an identifier, owner, dimension, normalized value, optional scope, confidence, supporting memory identifiers, source count, timestamps, and state. Relationship entries are scoped to a contact identifier. Style entries may include aggregate metrics, but never require storing raw messages.

## Update Rules

- Update a profile entry only through linked memories or an explicit user edit.
- Prefer merging evidence into an existing entry over creating duplicates.
- Preserve historical evidence when a value changes; mark the old entry superseded.
- Do not infer sensitive attributes unless the user explicitly supplies or confirms them.
- A user edit or lock takes precedence over automated updates.

## Use in Reply Generation

The context builder selects only entries relevant to the current conversation and relationship. It records which entries influenced a reply and exposes that explanation to the user. Missing profile information must result in less personalization, never fabricated assumptions.
