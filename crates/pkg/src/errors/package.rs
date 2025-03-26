use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Failed to parse package json: {path}")]
    PackageJsonParseFailure {
        path: String,
        #[source]
        error: serde_json::Error,
    },
    #[error("Failed to read/write package json: {path}")]
    PackageJsonIoFailure {
        path: String,
        #[source]
        error: io::Error,
    },
}

impl From<serde_json::Error> for PackageError {
    fn from(error: serde_json::Error) -> Self {
        PackageError::PackageJsonParseFailure { path: error.to_string(), error }
    }
}

impl From<io::Error> for PackageError {
    fn from(error: io::Error) -> Self {
        PackageError::PackageJsonIoFailure { path: error.to_string(), error }
    }
}

impl AsRef<str> for PackageError {
    fn as_ref(&self) -> &str {
        match self {
            PackageError::PackageJsonParseFailure { path: _, error: _ } => {
                "PackageJsonParseFailure"
            }
            PackageError::PackageJsonIoFailure { path: _, error: _ } => "PackageJsonIoFailure",
        }
    }
}

impl PackageError {
    pub fn into_parse_error(error: serde_json::Error, path: String) -> PackageError {
        PackageError::PackageJsonParseFailure { path, error }
    }

    pub fn into_io_error(error: io::Error, path: String) -> PackageError {
        PackageError::PackageJsonIoFailure { path, error }
    }
}
