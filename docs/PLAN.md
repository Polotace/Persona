# PLAN.md

# Persona Master Development Plan

**Project:** Persona

**Version:** 1.0

**Status:** Draft

---

# Purpose

This document defines the overall development strategy of Persona.

It serves as the master planning document for both human developers and AI coding agents.

Before implementing any feature, contributors should understand the project vision, development priorities, architecture milestones, and expected deliverables defined in this document.

This document intentionally focuses on **what should be built** and **in what order**, rather than **how it should be implemented**.

Technical implementation details belong to the architecture documentation.

---

# Project Vision

Persona is a Local-First Personal AI Agent that continuously learns from user interactions.

Its goal is not simply to generate AI replies.

Its goal is to gradually understand the user through long-term observation and build a trustworthy digital communication companion.

Persona should become more intelligent through use, not through manual configuration.

---

# Product Objectives

The project has five long-term objectives.

## Objective 1

Provide intelligent reply suggestions that resemble the user's communication style.

---

## Objective 2

Automatically build and maintain a personal memory system from conversation history.

---

## Objective 3

Continuously construct and update a user model representing interests, preferences, projects, skills, and communication habits.

---

## Objective 4

Operate entirely on the user's local machine by default while protecting privacy.

---

## Objective 5

Provide an extensible platform capable of supporting additional communication tools, plugins, and AI capabilities in the future.

---

# Non-Goals

Persona is not intended to become:

* a generic chatbot
* a cloud knowledge platform
* a social AI companion
* a general-purpose agent framework
* an autonomous messaging bot

Every new feature should reinforce the project's core mission.

---

# Development Strategy

Development should proceed in layers.

Higher-level features should never be implemented before lower-level infrastructure becomes stable.

The recommended order is:

1. Documentation
2. Architecture
3. Runtime
4. Storage
5. AI Service
6. Memory System
7. User Modeling
8. Reply Generation
9. Plugin System
10. Desktop Experience

---

# Development Principles

## Documentation First

Architecture should always be documented before implementation begins.

Every major subsystem should have an accompanying design document.

---

## Stable Interfaces

Modules should communicate through clearly defined interfaces.

Internal implementations may change.

Public interfaces should remain stable whenever possible.

---

## Incremental Delivery

Each development milestone should produce a usable and testable result.

Avoid large, monolithic development phases.

---

## Local First

Cloud functionality should always remain optional.

Offline usage should be considered the default experience.

---

## Human Review

AI-generated work should always be reviewed before being accepted.

Architecture decisions should never be accepted solely because an AI proposed them.

---

# Project Phases

## Phase 0 — Foundation

Goal

Establish project standards.

Deliverables

* repository structure
* documentation framework
* coding conventions
* architecture documents

Completion Criteria

The project documentation is complete enough to guide future implementation.

---

## Phase 1 — Core Infrastructure

Goal

Create the runtime foundation.

Deliverables

* runtime
* configuration
* logging
* event system
* storage abstraction
* service interfaces

Completion Criteria

The application can start, load configuration, initialize services, and shut down correctly.

---

## Phase 2 — AI Service

Goal

Provide AI capabilities through an independent service.

Deliverables

* model abstraction
* prompt construction
* reply generation
* embedding interface
* memory extraction interface

Completion Criteria

The Rust runtime can communicate with the AI service through a stable API.

---

## Phase 3 — Memory System

Goal

Build the long-term memory engine.

Deliverables

* memory extraction
* memory storage
* memory retrieval
* confidence management
* forgetting
* conflict resolution

Completion Criteria

Meaningful information can be extracted from conversations and reused later.

---

## Phase 4 — User Modeling

Goal

Build an evolving user profile.

Deliverables

* interests
* preferences
* communication style
* projects
* relationships

Completion Criteria

The system maintains an evolving user representation.

---

## Phase 5 — Reply Generation

Goal

Generate personalized replies.

Deliverables

* context construction
* memory retrieval
* reply ranking
* candidate generation

Completion Criteria

Replies consistently reflect the user's communication style.

---

## Phase 6 — Plugin Platform

Goal

Support external integrations.

Deliverables

* collector plugins
* communication platform adapters
* extension API

Completion Criteria

New platforms can be added without modifying the core application.

---

## Phase 7 — Long-Term Personalization & User Control

Goal

Deepen Persona's long-term personalization while keeping the user in control of every communication outcome.

Deliverables

* advanced memory
* explainable relationship and preference modeling
* cross-conversation context continuity
* privacy controls, memory review, and data export
* optional multi-device continuity

Completion Criteria

Persona provides consistently personalized and explainable reply suggestions across long-term usage, while users retain visibility and control over their data and every external communication.

Boundaries

* Persona must not send messages autonomously.
* Persona must not impersonate the user without explicit approval for the specific communication.
* Persona must not make social or relationship decisions on the user's behalf.

---

# Documentation Plan

The following documents are required.

| Document        | Purpose                   |
| --------------- | ------------------------- |
| README.md       | Project introduction      |
| AGENTS.md        | AI collaboration rules    |
| ARCHITECTURE.md | System architecture       |
| MEMORY.md       | Memory specification      |
| USER_MODEL.md   | User modeling             |
| DATABASE.md     | Storage architecture      |
| AI_SERVICE.md   | Python AI layer           |
| RUNTIME.md      | Rust runtime              |
| IPC.md          | Rust–Python communication |
| PLUGIN.md       | Plugin framework          |
| ROADMAP.md      | Version planning          |
| CONTRIBUTING.md | Contribution guide        |

Every document should remain synchronized with the implementation.

---

# Quality Standards

Every subsystem should satisfy the following requirements.

* Clearly defined responsibility.
* Minimal coupling.
* Stable interface.
* Comprehensive documentation.
* Testability.
* Extensibility.

Implementation quality is measured by maintainability rather than feature count.

---

# Decision Making

When multiple solutions exist:

1. Prefer simplicity.
2. Prefer modularity.
3. Prefer explainability.
4. Prefer local execution.
5. Prefer long-term maintainability.

Avoid introducing dependencies without clear architectural justification.

---

# Success Criteria

Persona should eventually become a system capable of:

* understanding the user's communication habits
* remembering important information
* adapting over time
* protecting user privacy
* generating personalized replies
* remaining maintainable for years

Success is measured by how naturally Persona assists its user rather than how many AI features it contains.

---

# Final Statement

Persona is designed as a long-term software engineering project rather than a short-term AI demonstration.

Every architectural decision should prioritize stability, clarity, maintainability, and user trust.

The objective is not to build another chatbot.

The objective is to build an AI system that grows together with its user.
