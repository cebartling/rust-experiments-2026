# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A multi-threaded Rust CLI tool that clones disk usage analysis functionality (similar to the Unix `du` command). Uses Rust 2024 edition with walkdir for directory traversal, rayon for parallel metadata collection, clap for CLI parsing, and colored for terminal output.

## Build & Development Commands

- **Build:** `cargo build`
- **Run:** `cargo run`
- **Run with args:** `cargo run -- <path>`
- **Test all:** `cargo test`
- **Test single:** `cargo test <test_name>`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`
- **Format check:** `cargo fmt -- --check`

## Development Workflow

Write test-first, TDD. Write a failing test before implementing functionality, then write the minimum code to make it pass, then refactor.

## Architecture

Binary crate with library structure. Entry point at `src/main.rs`, orchestration in `src/lib.rs`.

### Module layout

```
src/
  main.rs          Entry point: parse args, call run(), handle errors
  lib.rs           Module declarations, run() and run_to_string() orchestrator
  cli.rs           CliArgs struct (clap derive) with all CLI flags
  entry.rs         DiskEntry, EntryType, SortOrder data structures
  error.rs         DuskError enum with Display/From impls
  formatter.rs     Size formatting (human-readable, padding)
  traversal.rs     walkdir + rayon parallel filesystem traversal
  output.rs        Colorized rendering (size colors, path colors)
tests/
  integration.rs   End-to-end CLI tests with assert_cmd
```

### Dependency flow

`main -> lib -> cli, traversal, output -> entry, error, formatter`

### Multi-threading strategy

1. `walkdir` enumerates directory entries single-threaded (fast readdir)
2. `rayon::par_iter()` parallelizes `metadata()` stat syscalls in parallel
3. Single-threaded tree building from the flat sized-entry list
4. `--threads` / `-j` flag configures rayon's ThreadPoolBuilder

### CLI flags

`-H` human-readable, `-s` summarize, `-d N` max-depth, `-a` all files, `--sort` (size/size-asc/name/none), `-j N` threads, `--no-color`
