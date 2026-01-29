# Dictation

A cross-platform dictation tool built with Tauri 2.0 and Rust (edition 2024).

## Overview

Dictation provides push-to-talk speech-to-text capabilities with support for both local (whisper-rs) and cloud-based (OpenAI) transcription backends. The application features a modern TypeScript/Vite frontend and a performant Rust backend.

## Prerequisites

### macOS

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install required dependencies
brew install cmake node rust
```

### Linux

```bash
# Ubuntu/Debian
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libasound2-dev \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  cmake \
  nodejs \
  npm

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd dictation
```

2. Install frontend dependencies:
```bash
npm install
```

3. Build the Rust backend:
```bash
cd src-tauri
cargo build
```

## Build Commands

### Rust Backend

Run these commands from the `src-tauri/` directory:

```bash
cd src-tauri

# Build the backend
cargo build

# Build for release
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test <test_name>

# Lint
cargo clippy

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Frontend

Run these commands from the project root:

```bash
# Build frontend only
npm run build

# Type check
npm run type-check

# Lint
npm run lint
```

### Full Application

Run these commands from the project root:

```bash
# Development mode (hot reload)
cargo tauri dev

# Build release bundle
cargo tauri build
```

## Development

### Running the Application

The easiest way to run the application during development:

```bash
cargo tauri dev
```

This launches both the Vite dev server (with hot reload) and the Rust backend.

### Test-Driven Development

Follow TDD practices:
1. Write a failing test first
2. Implement the minimal code to make it pass
3. Refactor as needed

Every new function or module should begin with a test that defines expected behavior.

### Project Structure

```
dictation/
├── src/                    # Frontend (TypeScript + Vite)
│   ├── components/         # UI components
│   ├── styles/            # CSS/styling
│   └── main.ts            # Entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Tauri builder and setup
│   │   ├── main.rs        # Entry point
│   │   ├── error.rs       # Error types
│   │   ├── state.rs       # Application state
│   │   ├── events.rs      # Event definitions
│   │   ├── commands.rs    # Tauri commands
│   │   ├── audio/         # Audio capture (cpal + rubato)
│   │   ├── stt/           # Speech-to-text engines
│   │   ├── injection/     # Text injection (enigo)
│   │   ├── hotkey/        # Global shortcuts
│   │   └── config/        # Settings persistence
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Frontend dependencies
└── README.md              # This file
```

## Architecture

### Frontend
- **Framework**: Vite + TypeScript
- **Components**: Status indicator, settings panel, transcription log

### Backend
- **Core**: Tauri 2.0 with Rust 2024 edition
- **Audio**: Microphone capture via `cpal`, resampling via `rubato`
- **STT Backends**:
  - Local: `whisper-rs` for offline transcription
  - Cloud: OpenAI API via `reqwest`
- **Text Injection**: Keyboard simulation via `enigo`
- **Global Shortcuts**: Push-to-talk via `tauri-plugin-global-shortcut`
- **Settings**: Persistent configuration via `tauri-plugin-store`

## Troubleshooting

### Build fails with "cmake not found"

Install cmake:
```bash
# macOS
brew install cmake

# Linux
sudo apt-get install cmake
```

### Frontend dependencies missing

```bash
npm install
```

### Rust compilation errors

Ensure you have the latest Rust toolchain:
```bash
rustup update
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
