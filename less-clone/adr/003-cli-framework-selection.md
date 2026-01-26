# ADR 003: CLI Framework Selection

## Status

Accepted

## Context

The pager needs to accept command-line arguments: an optional file path, flags like `--line-numbers`, and standard `--help`/`--version`.

## Decision

Use **clap v4** with the derive macro feature for argument parsing. Define a `CliArgs` struct with `#[derive(Parser)]` annotations.

## Consequences

- Declarative argument definitions via Rust structs and attributes.
- Automatic `--help` and `--version` generation.
- Compile-time validation of argument definitions.
- Consistent with the sibling `disk-usage-clone` project.
- Adds ~200KB to binary size from clap, acceptable for a CLI tool.
