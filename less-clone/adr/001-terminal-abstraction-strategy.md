# ADR 001: Terminal Abstraction Strategy

## Status

Accepted

## Context

The pager needs to interact with a terminal to display content, read keyboard input, manage raw mode, and use the alternate screen buffer. Terminal-dependent code is inherently difficult to test because it requires an actual terminal.

## Decision

Introduce a `Terminal` trait that abstracts all terminal I/O operations (enter/leave alternate screen, raw mode, size, print, read_event, cursor control). Provide two implementations:

- **`CrosstermTerminal`** — production implementation backed by the crossterm crate.
- **`MockTerminal`** — test implementation that captures output in a `Vec<String>` and replays scripted events.

All pager logic accepts `&mut dyn Terminal`, making it fully testable without a real terminal.

## Consequences

- Unit tests for the pager and renderer run without a terminal, enabling CI.
- The trait boundary introduces a small amount of dynamic dispatch overhead, which is negligible for a pager.
- Adding a new terminal backend (e.g., termion) requires only a new trait implementation.
