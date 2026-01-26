use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DuskError {
    PathNotFound(PathBuf),
    PermissionDenied(PathBuf),
    IoError(io::Error),
    TraversalError(String),
}

impl fmt::Display for DuskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuskError::PathNotFound(path) => {
                write!(f, "path not found: {}", path.display())
            }
            DuskError::PermissionDenied(path) => {
                write!(f, "permission denied: {}", path.display())
            }
            DuskError::IoError(err) => write!(f, "I/O error: {err}"),
            DuskError::TraversalError(msg) => write!(f, "traversal error: {msg}"),
        }
    }
}

impl std::error::Error for DuskError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DuskError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DuskError {
    fn from(err: io::Error) -> Self {
        DuskError::IoError(err)
    }
}

impl From<walkdir::Error> for DuskError {
    fn from(err: walkdir::Error) -> Self {
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
