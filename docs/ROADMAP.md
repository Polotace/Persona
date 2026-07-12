# Persona Roadmap

The roadmap tracks user-visible outcomes rather than implementation dates. Dates are intentionally not committed until the foundation is validated.

| Milestone | Outcome | Evidence of readiness |
| --- | --- | --- |
| Foundation | Stable boundaries, conventions, and documentation | Phase 0 completion checklist is satisfied |
| Local runtime | Application starts and manages local services safely | Configuration, events, storage abstraction, and shutdown are tested |
| AI capabilities | Local application invokes an independent AI service | Versioned service contract and fake-provider integration tests pass |
| Memory | Meaningful information is retained and controlled by the user | Traceability, conflicts, forgetting, and retrieval are demonstrated |
| User model | Personalization learns without opaque profiling | Profile edits, locks, and explanations work end to end |
| Reply assistance | Users receive reviewable personalized suggestions | Context provenance and style-aware candidates are visible |
| Extensions | New integrations do not modify the core | Permissioned plugin API supports an external example |
| Long-term control | Personalization remains transparent over time | Export, review, privacy controls, and approval boundaries are verified |

Persona does not autonomously send messages or make social decisions for the user at any milestone.
