# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Dictation project.

## What are ADRs?

Architecture Decision Records document important architectural decisions made during the development of this project. Each ADR describes:

- The context and problem being addressed
- The decision that was made
- The consequences (positive and negative) of that decision

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [001](./001-tauri-framework.md) | Use Tauri 2.0 as Application Framework | Accepted |
| [002](./002-dual-stt-backend-strategy.md) | Dual STT Backend Strategy (Local + Cloud) | Accepted |
| [003](./003-audio-pipeline-architecture.md) | Audio Pipeline Architecture (cpal + rubato) | Accepted |
| [004](./004-push-to-talk-interaction.md) | Push-to-Talk Interaction Model | Accepted |
| [005](./005-text-injection-mechanism.md) | Text Injection Mechanism | Accepted |
| [006](./006-settings-persistence.md) | Settings Persistence Strategy | Accepted |
| [007](./007-frontend-framework.md) | Frontend Framework Selection | Accepted |

## Quick Summary

### Core Technology Stack

**Application Framework (ADR-001):**
- Tauri 2.0 for cross-platform desktop application
- Rust backend + Web frontend architecture

**Speech-to-Text (ADR-002):**
- Local: whisper-rs (whisper.cpp bindings)
- Cloud: OpenAI Whisper API
- User-selectable based on privacy/cost/accuracy needs

**Audio Processing (ADR-003):**
- Capture: cpal (cross-platform audio library)
- Resampling: rubato (16kHz mono for Whisper)
- Architecture: Microphone → Buffer → Resample → STT

### User Experience

**Interaction Model (ADR-004):**
- Push-to-talk: Hold key to record, release to transcribe
- Global keyboard shortcut (default: Ctrl+Shift+Space)
- Clear visual feedback for recording state

**Text Injection (ADR-005):**
- Keyboard simulation via enigo
- Types text at cursor position
- Clipboard fallback for long text

### Data & Configuration

**Settings Storage (ADR-006):**
- tauri-plugin-store (JSON-based key-value)
- Platform-specific app data directory
- Type-safe via Rust serde

**Frontend (ADR-007):**
- Vite + TypeScript (minimal framework)
- Vanilla JavaScript with direct Tauri IPC
- Simple state management

## ADR Format

Each ADR follows this structure:

1. **Title**: Clear, concise description of the decision
2. **Status**: Accepted | Proposed | Deprecated | Superseded
3. **Context**: Problem statement and constraints
4. **Decision**: What was decided and why
5. **Consequences**: Trade-offs and implications

## When to Create an ADR

Create an ADR when making decisions about:

- Technology choices (frameworks, libraries, languages)
- Architecture patterns (data flow, module structure)
- User experience flows (interaction models)
- External integrations (APIs, protocols)
- Security or privacy approaches
- Deployment strategies

## Contributing

When proposing changes to architecture:

1. Create a new ADR with status "Proposed"
2. Number sequentially (next available number)
3. Discuss with team before marking "Accepted"
4. If superseding an old ADR, update the old one's status and link to the new one

## References

- [ADR GitHub Organization](https://adr.github.io/)
- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
