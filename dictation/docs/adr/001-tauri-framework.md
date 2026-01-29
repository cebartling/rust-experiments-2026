# ADR-001: Use Tauri 2.0 as Application Framework

## Status

Accepted

## Context

We needed to build a cross-platform desktop dictation application with the following requirements:

- Cross-platform support (macOS, Linux, Windows)
- Low resource footprint
- Native system integration (global shortcuts, clipboard, keyboard injection)
- Modern UI with web technologies
- Secure architecture
- Small binary size

Alternative frameworks considered:
- **Electron**: Popular but heavy (~100MB+ bundles), high memory usage
- **Native (Swift/Kotlin/C++)**: Platform-specific, requires 3x development effort
- **Qt**: Large runtime, licensing concerns for commercial use
- **Flutter Desktop**: Immature ecosystem, limited native integration

## Decision

Use Tauri 2.0 as the application framework with:
- Rust backend for system integration and business logic
- Web frontend (Vite + TypeScript) for UI
- Tauri's IPC bridge for frontend-backend communication

## Consequences

### Positive

- **Small binaries**: ~5-10MB vs 100MB+ for Electron
- **Low memory usage**: Uses system webview instead of bundling Chromium
- **Performance**: Rust backend provides excellent performance for audio processing
- **Security**: Tauri's security model with explicit command allowlisting
- **Native integration**: Direct access to OS APIs via Rust
- **Developer experience**: Web technologies for UI, Rust for performance-critical code
- **Cross-platform**: Write once, build for all platforms
- **Active ecosystem**: Growing plugin ecosystem (global-shortcut, store, etc.)

### Negative

- **Smaller community**: Less mature than Electron, fewer resources/examples
- **Platform-specific webview differences**: Behavior varies across OSs
- **Rust learning curve**: Team needs Rust expertise
- **Plugin maturity**: Some plugins are newer and less battle-tested

### Neutral

- **Build complexity**: Requires both Rust and Node.js toolchains
- **Two-language codebase**: Frontend in TypeScript, backend in Rust
