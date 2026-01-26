# ADR 005: Error Handling Strategy

## Status

Accepted

## Context

The pager can encounter several types of errors: file I/O failures, invalid regex patterns, terminal operation failures, and missing input. These need unified handling for clean error messages and proper exit codes.

## Decision

Define a custom `LessError` enum with four variants:

- `IoError(io::Error)` — file and stdin I/O failures
- `InvalidPattern(String)` — invalid regex search patterns
- `TerminalError(String)` — crossterm operation failures
- `NoInput` — no file argument and empty stdin

Implement `Display`, `Error`, `From<io::Error>`, and `From<regex::Error>` traits to enable the `?` operator throughout the codebase.

## Consequences

- All error paths produce user-friendly messages via `Display`.
- The `?` operator works seamlessly with `io::Error` and `regex::Error`.
- `main.rs` can catch all errors uniformly with a single `match` or `eprintln!`.
- Follows the same pattern as `DuskError` in the sibling project.
