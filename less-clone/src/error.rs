//! Error types for the less-clone pager.
//!
//! Provides a unified error enum [`LessError`] that covers all error conditions
//! the pager can encounter: I/O failures, invalid search patterns, terminal
//! errors, and missing input.

use std::fmt;
use std::io;

/// Unified error type for the less-clone pager.
#[derive(Debug)]
pub enum LessError {
    /// An I/O error occurred (reading files, stdin, etc.).
    IoError(io::Error),
    /// An invalid regex pattern was provided for search.
    InvalidPattern(String),
    /// A terminal operation failed.
    TerminalError(String),
    /// No input was provided (no file argument and stdin is a TTY).
    NoInput,
}

impl fmt::Display for LessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LessError::IoError(err) => write!(f, "I/O error: {err}"),
            LessError::InvalidPattern(pattern) => {
                write!(f, "Invalid search pattern: {pattern}")
            }
            LessError::TerminalError(msg) => write!(f, "Terminal error: {msg}"),
            LessError::NoInput => write!(f, "No input: specify a file or pipe data via stdin"),
        }
    }
}

impl std::error::Error for LessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LessError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for LessError {
    fn from(err: io::Error) -> Self {
        LessError::IoError(err)
    }
}

impl From<regex::Error> for LessError {
    fn from(err: regex::Error) -> Self {
        LessError::InvalidPattern(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_error() {
        let err = LessError::IoError(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let msg = format!("{err}");
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn display_invalid_pattern() {
        let err = LessError::InvalidPattern("unclosed group".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("Invalid search pattern"));
        assert!(msg.contains("unclosed group"));
    }

    #[test]
    fn display_terminal_error() {
        let err = LessError::TerminalError("cannot enter raw mode".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("Terminal error"));
        assert!(msg.contains("cannot enter raw mode"));
    }

    #[test]
    fn display_no_input() {
        let err = LessError::NoInput;
        let msg = format!("{err}");
        assert!(msg.contains("No input"));
    }

    #[test]
    fn source_io_error_returns_inner() {
        let inner = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let err = LessError::IoError(inner);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn source_non_io_returns_none() {
        let err = LessError::InvalidPattern("bad".to_string());
        assert!(std::error::Error::source(&err).is_none());

        let err = LessError::TerminalError("fail".to_string());
        assert!(std::error::Error::source(&err).is_none());

        let err = LessError::NoInput;
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
        let err: LessError = io_err.into();
        assert!(matches!(err, LessError::IoError(_)));
    }

    #[test]
    fn from_regex_error() {
        let regex_err = regex::Regex::new("[invalid").unwrap_err();
        let err: LessError = regex_err.into();
        assert!(matches!(err, LessError::InvalidPattern(_)));
    }
}
