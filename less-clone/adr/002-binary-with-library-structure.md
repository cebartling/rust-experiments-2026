# ADR 002: Binary with Library Structure

## Status

Accepted

## Context

The project needs both a runnable binary and comprehensive test coverage. Putting all logic in `main.rs` makes it hard to test because the entry point is `fn main()` which returns `()`.

## Decision

Use a binary + library structure:

- **`src/main.rs`** — minimal entry point: parse CLI args, call `less_clone::run()`, handle errors with `eprintln!` and `process::exit(1)`.
- **`src/lib.rs`** — all module declarations and the `run()` orchestrator function that can be unit tested.

## Consequences

- All business logic is testable via library imports.
- Integration tests in `tests/` can use `assert_cmd` to test the binary.
- The `main.rs` stays under 15 lines, following the "thin binary" pattern.
- Follows the same pattern as the sibling `disk-usage-clone` project.
