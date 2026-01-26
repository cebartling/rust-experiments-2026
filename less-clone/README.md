# less-clone

A terminal pager similar to Unix `less`, built in Rust. Reads files or stdin and provides scrollable, searchable viewing in the terminal.

## Features

- Read files from CLI args or stdin
- Scroll by line (j/k), page (Space/b), half-page (d/u)
- Jump to top (g) / bottom (G)
- Forward search (/) and backward search (?) with regex
- Search navigation (n/N) with match highlighting
- Status line with filename, line position, and percentage
- Line number toggle (-N flag or `l` key)
- Help screen (h)
- Terminal resize handling

## Build

```sh
cargo build
```

## Run

```sh
# View a file
cargo run -- README.md

# Pipe from stdin
cat some_file.txt | cargo run

# With line numbers
cargo run -- -N README.md
```

## Test

```sh
cargo test
```

## Documentation

```sh
# Generate API docs (opens in browser)
cargo doc --no-deps --open

# Generate docs including private items
cargo doc --no-deps --document-private-items --open
```

Generated docs are written to `target/doc/less_clone/index.html`.

## Lint & Format

```sh
cargo clippy
cargo fmt -- --check
```

## Key Bindings

| Key | Action |
|-----|--------|
| `j` / Down / Enter | Scroll down one line |
| `k` / Up | Scroll up one line |
| Space / `f` / PgDn | Scroll down one page |
| `b` / PgUp | Scroll up one page |
| `d` | Scroll down half page |
| `u` | Scroll up half page |
| `g` / Home | Go to top |
| `G` / End | Go to bottom |
| `/pattern` | Search forward |
| `?pattern` | Search backward |
| `n` | Next match |
| `N` | Previous match |
| `l` | Toggle line numbers |
| `h` | Toggle help |
| `q` / Ctrl-C | Quit |

## Architecture

See [adr/](adr/) for architecture decision records.

```
src/
  main.rs       Entry point
  lib.rs        Module declarations, run() orchestrator
  cli.rs        CLI argument parsing (clap)
  error.rs      LessError enum
  buffer.rs     TextBuffer: line-indexed content
  input.rs      Key-to-action mapping
  search.rs     Regex search with highlighting
  screen.rs     Terminal trait + crossterm impl
  status.rs     Status line formatting
  pager.rs      Event loop and state management
```
