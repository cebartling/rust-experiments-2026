//! Error types for the disk usage analyzer.
//!
//! This module defines [`DuskError`], the main error type returned by
//! library functions. It provides type-safe error handling and user-friendly
//! error messages.
//!
//! # Error Conversion
//!
//! The `From` trait implementations enable automatic error conversion using
//! the `?` operator:
//!
//! ```no_run
//! use disk_usage_clone::error::DuskError;
//! use std::fs;
//!
//! fn example() -> Result<(), DuskError> {
//!     // io::Error automatically converts to DuskError
//!     let _metadata = fs::metadata("/tmp")?;
//!     Ok(())
//! }
//! ```
//!
//! # Examples
//!
//! ```
//! use disk_usage_clone::error::DuskError;
//! use std::path::PathBuf;
//!
//! let err = DuskError::PathNotFound(PathBuf::from("/nonexistent"));
//! assert_eq!(err.to_string(), "path not found: /nonexistent");
//! ```

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Error type for disk usage analysis operations.
///
/// Represents all possible errors that can occur during filesystem traversal
/// and analysis. Each variant includes context to help users understand and
/// resolve the issue.
///
/// # Variants
///
/// - `PathNotFound` - The specified path doesn't exist
/// - `PermissionDenied` - Insufficient permissions to access path
/// - `IoError` - Generic I/O error (disk full, read error, etc.)
/// - `TraversalError` - Error during directory traversal
///
/// # Error Messages
///
/// All variants implement [`Display`] for user-friendly error messages:
///
/// ```
/// use disk_usage_clone::error::DuskError;
/// use std::path::PathBuf;
///
/// let err = DuskError::PathNotFound(PathBuf::from("/foo"));
/// assert_eq!(err.to_string(), "path not found: /foo");
/// ```
///
/// # Error Propagation
///
/// Use the `?` operator for clean error propagation:
///
/// ```no_run
/// use disk_usage_clone::error::DuskError;
/// use std::path::Path;
///
/// fn check_path(path: &Path) -> Result<(), DuskError> {
///     if !path.exists() {
///         return Err(DuskError::PathNotFound(path.to_path_buf()));
///     }
///     // io::Error auto-converts to DuskError via From trait
///     let _metadata = path.metadata()?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub enum DuskError {
    /// Path does not exist on the filesystem.
    ///
    /// Returned when attempting to analyze a nonexistent path.
    ///
    /// # User Action
    ///
    /// Check the path spelling and try again.
    PathNotFound(PathBuf),

    /// Permission denied when accessing path.
    ///
    /// Returned when the user lacks read permissions for a file or directory.
    ///
    /// # User Action
    ///
    /// Run with elevated privileges (sudo) or check file permissions.
    PermissionDenied(PathBuf),

    /// Generic I/O error.
    ///
    /// Wraps [`std::io::Error`] for operations like reading metadata,
    /// opening directories, etc.
    ///
    /// Common causes:
    /// - Disk read error
    /// - Filesystem corruption
    /// - Network filesystem timeout
    /// - Disk full (when writing cache, etc.)
    IoError(io::Error),

    /// Error during directory traversal.
    ///
    /// Wraps errors from the `walkdir` crate. Can occur when:
    /// - Symbolic link loop detected
    /// - Filename encoding issues
    /// - Permission denied on subdirectory
    TraversalError(String),
}

impl fmt::Display for DuskError {
    /// Formats the error for display to users.
    ///
    /// Provides helpful, human-readable error messages with context.
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::error::DuskError;
    /// use std::path::PathBuf;
    ///
    /// let err = DuskError::PathNotFound(PathBuf::from("/missing"));
    /// assert_eq!(format!("{}", err), "path not found: /missing");
    ///
    /// let err = DuskError::PermissionDenied(PathBuf::from("/secret"));
    /// assert_eq!(format!("{}", err), "permission denied: /secret");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuskError::PathNotFound(path) => {
                // Use path.display() for cross-platform path formatting
                write!(f, "path not found: {}", path.display())
            }
            DuskError::PermissionDenied(path) => {
                write!(f, "permission denied: {}", path.display())
            }
            DuskError::IoError(err) => {
                // Delegate to io::Error's Display impl
                write!(f, "I/O error: {err}")
            }
            DuskError::TraversalError(msg) => {
                write!(f, "traversal error: {msg}")
            }
        }
    }
}

impl std::error::Error for DuskError {
    /// Returns the underlying error source, if any.
    ///
    /// This enables error chaining and better debugging. Only `IoError`
    /// has an underlying source (the wrapped `std::io::Error`).
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::error::DuskError;
    /// use std::error::Error;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::Other, "disk full");
    /// let err = DuskError::IoError(io_err);
    ///
    /// // Can access the source error
    /// assert!(err.source().is_some());
    /// ```
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DuskError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DuskError {
    /// Converts [`std::io::Error`] to [`DuskError`].
    ///
    /// This enables the `?` operator to work seamlessly with I/O operations:
    ///
    /// ```no_run
    /// use disk_usage_clone::error::DuskError;
    /// use std::fs;
    ///
    /// fn read_metadata(path: &str) -> Result<u64, DuskError> {
    ///     // ? automatically converts io::Error to DuskError
    ///     let metadata = fs::metadata(path)?;
    ///     Ok(metadata.len())
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::error::DuskError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
    /// let dusk_err: DuskError = io_err.into();
    ///
    /// assert!(matches!(dusk_err, DuskError::IoError(_)));
    /// ```
    fn from(err: io::Error) -> Self {
        DuskError::IoError(err)
    }
}

impl From<walkdir::Error> for DuskError {
    /// Converts `walkdir::Error` to [`DuskError`].
    ///
    /// Enables seamless error handling during directory traversal:
    ///
    /// ```no_run
    /// use disk_usage_clone::error::DuskError;
    /// use walkdir::WalkDir;
    ///
    /// fn traverse(path: &str) -> Result<usize, DuskError> {
    ///     let mut count = 0;
    ///     for entry in WalkDir::new(path) {
    ///         let _entry = entry?; // Converts walkdir::Error automatically
    ///         count += 1;
    ///     }
    ///     Ok(count)
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use disk_usage_clone::error::DuskError;
    /// use walkdir::WalkDir;
    ///
    /// let result: Result<(), DuskError> = WalkDir::new("/nonexistent")
    ///     .into_iter()
    ///     .next()
    ///     .unwrap()
    ///     .map(|_| ())
    ///     .map_err(|e| e.into());
    ///
    /// assert!(result.is_err());
    /// ```
    fn from(err: walkdir::Error) -> Self {
        // Convert to string since walkdir::Error is not Send/Sync
        DuskError::TraversalError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_not_found_display() {
        let err = DuskError::PathNotFound(PathBuf::from("/nonexistent"));
        assert_eq!(err.to_string(), "path not found: /nonexistent");
    }

    #[test]
    fn test_permission_denied_display() {
        let err = DuskError::PermissionDenied(PathBuf::from("/secret"));
        assert_eq!(err.to_string(), "permission denied: /secret");
    }

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::Other, "disk full");
        let err = DuskError::IoError(io_err);
        assert_eq!(err.to_string(), "I/O error: disk full");
    }

    #[test]
    fn test_traversal_error_display() {
        let err = DuskError::TraversalError("something broke".to_string());
        assert_eq!(err.to_string(), "traversal error: something broke");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let err: DuskError = io_err.into();
        assert!(matches!(err, DuskError::IoError(_)));
    }

    #[test]
    fn test_error_is_debug() {
        let err = DuskError::PathNotFound(PathBuf::from("/test"));
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("PathNotFound"));
    }
}
