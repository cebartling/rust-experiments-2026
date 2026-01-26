# ADR 006: Text Buffer Design

## Status

Accepted

## Context

The pager needs to store and provide efficient random access to text content. Content can come from files or stdin. The pager requires line-by-line access for rendering and search.

## Decision

Implement `TextBuffer` as a `Vec<String>` of lines with an optional filename:

- **`from_file(path)`** — reads entire file into memory, splits into lines.
- **`from_reader(reader)`** — reads from any `Read` impl (stdin), splits into lines.
- **`from_string(content)`** — constructs from an in-memory string (useful for tests).
- **`line(index)`** — O(1) access to any line by index.
- **`lines_range(start, end)`** — slice access for rendering a page.

## Consequences

- O(1) line access enables fast scrolling and search.
- Entire file is loaded into memory, which is acceptable for text files (matching `less` behavior for most use cases).
- The `from_reader` and `from_string` constructors make the buffer easy to test without filesystem access.
- Memory usage is proportional to file size; very large files (>available RAM) will cause issues, but this is an acceptable limitation for a learning project.
