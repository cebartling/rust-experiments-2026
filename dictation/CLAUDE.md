# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dictation is a cross-platform dictation tool built with Tauri 2.0 and Rust (edition 2024), part of the `rust-experiments-2026` monorepo. The frontend is Vite + TypeScript; the backend is Rust in `src-tauri/`.

## Build Commands

Rust commands run from `src-tauri/`:

```bash
cd src-tauri
cargo build            # Build the Rust backend
cargo test             # Run all tests
cargo test <test_name> # Run a single test
cargo clippy           # Lint
cargo fmt              # Format code
cargo fmt -- --check   # Check formatting without modifying
```

Tauri app commands run from the project root:

```bash
npm install            # Install frontend dependencies
cargo tauri dev        # Launch the full Tauri app (frontend + backend)
cargo tauri build      # Build release bundle
```

## Development Approach

Follow test-driven development (TDD): write a failing test first, then write the minimal code to make it pass, then refactor. Every new function or module should begin with a test that defines the expected behavior before any implementation is written.

## Architecture

Tauri 2.0 project with a Vite + TypeScript frontend and a Rust backend:

- `src/` — Frontend (TypeScript): status indicator, settings panel, transcription log
- `src-tauri/src/` — Rust backend modules:
  - `lib.rs` — Tauri builder, plugin setup, command registration
  - `main.rs` — Entry point (calls `lib::run`)
  - `error.rs` — `DictationError` enum (thiserror)
  - `state.rs` — `AppState` (Tauri managed state)
  - `events.rs` — Event names and payload types
  - `commands.rs` — Tauri command functions
  - `audio/` — Microphone capture (cpal) and resampling (rubato)
  - `stt/` — `SttEngine` trait, local (whisper-rs) and cloud (reqwest) backends
  - `injection/` — Text injection via enigo
  - `hotkey/` — Push-to-talk state machine
  - `config/` — `AppSettings` with defaults and persistence

## System Dependencies (Linux)

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libasound2-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev cmake
```

## Architecture Plan

A detailed architecture plan for building this into a cross-platform dictation tool (Tauri 2.0, whisper-rs, cloud API support, push-to-talk) is at:

`~/.claude/plans/purrfect-meandering-dove.md`
