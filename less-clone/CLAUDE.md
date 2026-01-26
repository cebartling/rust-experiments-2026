# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust CLI tool that clones the Unix `less` pager functionality. Uses Rust 2024 edition. Part of the `rust-experiments-2026` repository.

## Build & Development Commands

- **Build:** `cargo build`
- **Run:** `cargo run`
- **Run with file:** `cargo run -- <file>`
- **Test all:** `cargo test`
- **Test single:** `cargo test <test_name>`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`
- **Format check:** `cargo fmt -- --check`
- **Docs:** `cargo doc --no-deps --open`
- **Docs (with private items):** `cargo doc --no-deps --document-private-items --open`

## Development Workflow

Write test-first, TDD. Write a failing test before implementing functionality, then write the minimum code to make it pass, then refactor.

## Architecture

Binary + library structure. `main.rs` is a thin entry point; all logic lives in the library.

**Module layout:**

```
src/
  main.rs       Entry point: parse args, call run(), handle errors
  lib.rs        Module declarations, run() orchestrator
  cli.rs        CliArgs struct (clap derive)
  error.rs      LessError enum with Display/Error/From impls
  buffer.rs     TextBuffer: file/stdin content as indexed lines
  input.rs      Action enum + key-to-action mapping
  search.rs     Regex forward/backward search, match highlighting
  screen.rs     Terminal trait (testable abstraction) + CrosstermTerminal impl
  status.rs     Status line formatting (filename, position, percentage)
  pager.rs      Event loop, PagerState, action handling
```

**Dependency flow:** `main -> lib -> cli, pager -> buffer, search, input, screen, status -> error`

**Key design:** A `Terminal` trait abstracts all terminal I/O. `CrosstermTerminal` is the production impl; `MockTerminal` captures output for unit tests. See `adr/` for architecture decision records.
