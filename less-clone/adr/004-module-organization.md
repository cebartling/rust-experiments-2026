# ADR 004: Module Organization

## Status

Accepted

## Context

The pager has several distinct concerns: CLI parsing, error handling, text storage, keyboard input, search, terminal rendering, status formatting, and the main event loop. These need to be organized into focused, testable modules.

## Decision

Organize the codebase into 8 modules following a layered dependency flow:

```
main -> lib -> cli, pager -> buffer, search, input, screen, status -> error
```

| Module   | Responsibility |
|----------|----------------|
| `cli`    | Command-line argument parsing |
| `error`  | Unified error type |
| `buffer` | Text content storage with line-indexed access |
| `input`  | Keyboard event to action mapping |
| `search` | Regex-based search with highlighting |
| `screen` | Terminal trait and rendering |
| `status` | Status line formatting |
| `pager`  | Event loop and state management |

## Consequences

- Each module has a single responsibility and can be tested independently.
- Dependencies flow in one direction (no circular dependencies).
- New features can be added by extending the appropriate module.
