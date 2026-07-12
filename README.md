# Persona

<div align="center">

**A Local-First, Self-Evolving Personal AI Reply Agent**

English | [中文](./docs/README_CN.md)

</div>

Persona is an open-source AI platform that learns how **you** communicate.

Instead of asking users to manually build knowledge bases, write prompts, or configure personalities, Persona continuously observes conversations, extracts meaningful memories, understands communication styles, and generates reply suggestions that feel natural and personal.

The longer you use Persona, the better it understands you.

---

## ✨ Vision

Most AI assistants answer questions.

Persona learns people.

Its goal is not to replace human communication, but to help users communicate more naturally by understanding their long-term preferences, memories, projects, and writing style.

Persona grows with its user over time.

---

## 🚀 Features

### 💬 Intelligent Reply Suggestions

Generate personalized replies based on:

* Current conversation
* Recent chat history
* Long-term memories
* Communication style
* Relationship context

---

### 🧠 Self-Evolving Memory

Persona automatically extracts meaningful information from conversations.

It remembers:

* Interests
* Projects
* Preferences
* Skills
* Relationships
* Long-term facts

Instead of storing every message, Persona builds a structured memory system.

---

### 🎭 Communication Style Learning

Persona gradually learns:

* Writing style
* Vocabulary
* Sentence length
* Emoji usage
* Formality
* Humor
* Punctuation habits

Replies are generated to resemble the user—not a generic AI assistant.

---

### 🔒 Local First

Privacy is a core design principle.

By default:

* Conversations stay on your device.
* Memories stay on your device.
* Models can run locally.
* Cloud AI is optional.

Your data belongs to you.

---

### 🧩 Modular Architecture

Persona is designed as a modular platform.

Every major subsystem is replaceable.

Examples include:

* AI models
* Embedding providers
* Databases
* Collectors
* Plugins
* Desktop UI

---

## 🏗 Architecture

Persona separates application infrastructure from AI intelligence.

```text
                    Persona

           ┌─────────────────────┐
           │   Desktop (Rust)    │
           └──────────┬──────────┘
                      │
           ┌──────────▼──────────┐
           │   Persona Core      │
           │       Rust          │
           └──────────┬──────────┘
                      │
      ┌───────────────┼───────────────┐
      ▼               ▼               ▼

 Collector      Memory Engine     Plugin System

                      │
                      ▼

             Rust ↔ Python IPC

                      │
                      ▼

            Persona AI Service

                      │

      Memory • Style • Retrieval • Reply
```

The architecture is documented in **ARCHITECTURE.md**.

---

## 🛠 Technology Stack

### Core Runtime

* Rust
* Tokio
* Tauri
* SQLx
* Serde
* Reqwest
* Tracing

### AI Service

* Python
* FastAPI
* LiteLLM
* Sentence Transformers
* FAISS

### Storage

* SQLite
* DuckDB
* Vector Index (planned)

---

## 📚 Documentation

Project documentation is organized as follows.

```text
README.md
PLAN.md
AGENT.md
ARCHITECTURE.md

docs/
    MEMORY.md
    USER_MODEL.md
    AI_SERVICE.md
    DATABASE.md
    IPC.md
    PLUGIN.md
    ROADMAP.md
```

Each document focuses on a single aspect of the system.

---

## 📌 Project Status

Current status:

🚧 Planning & Architecture

The project is currently focused on:

* Architecture design
* Documentation
* Core abstractions
* Runtime planning

Implementation will begin after the architecture has stabilized.

---

## 🎯 Long-Term Roadmap

* Personal reply suggestions
* Long-term memory engine
* User modeling
* Communication style learning
* Plugin ecosystem
* Long-term personalization and user control
* Multi-platform support

---

## 🤝 Contributing

Contributions are welcome.

Before implementing new features, please read:

* `PLAN.md`
* `AGENT.md`
* `ARCHITECTURE.md`

Architecture discussions are encouraged before implementation.

Maintaining a clean and consistent architecture is more important than adding features quickly.

---

## 📄 License

License information will be added before the first public release.

---

## 🌟 Philosophy

> AI should not require users to adapt to it.

> AI should gradually adapt to its users.

Persona is an attempt to build an AI that grows together with the person using it.
