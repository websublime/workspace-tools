use std::{io, path::PathBuf};

use super::FileSystemError;

impl FileSystemError {
    /// Creates a new path validation error.
    pub fn validation(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::Validation { path: path.into(), reason: reason.into() }
    }
}

impl FileSystemError {
    /// Creates a `FileSystemError` from an `io::Error` and associated path.
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_io(error: io::Error, path: impl Into<PathBuf>) -> Self {
        let path_buf = path.into();
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path: path_buf },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path: path_buf },
            // Add other kinds if needed, e.g., IsADirectory, NotADirectory
            _ => Self::Io { path: path_buf, message: error.to_string() },
        }
    }
}

impl From<io::Error> for FileSystemError {
    fn from(error: io::Error) -> Self {
        // Create a dummy path or indicate unknown path
        let path = PathBuf::from("<unknown>");
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path },
            _ => Self::Io { path, message: error.to_string() },
        }
    }
}

impl AsRef<str> for FileSystemError {
    fn as_ref(&self) -> &str {
        match self {
            FileSystemError::NotFound { path: _ } => "FileSystemError::NotFound",
            FileSystemError::PermissionDenied { path: _ } => "FileSystemError::PermissionDenied",
            FileSystemError::Io { path: _, message: _ } => "FileSystemError::Io",
            FileSystemError::NotADirectory { path: _ } => "FileSystemError::NotADirectory",
            FileSystemError::NotAFile { path: _ } => "FileSystemError::NotAFile",
            FileSystemError::Utf8Decode { path: _, message: _ } => "FileSystemError::Utf8Decode",
            FileSystemError::Validation { path: _, reason: _ } => "FileSystemError::Validation",
            FileSystemError::Operation(_) => "FileSystemError::Operation",
        }
    }
}
