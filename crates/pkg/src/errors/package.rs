use serde::ser::Error as SerError;
use std::{io, sync::PoisonError};
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
    #[error("Failed to diff package between: {0}")]
    PackageBetweenFailure(String),
    #[error("Failed to found package: {0}")]
    PackageNotFound(String),
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
            PackageError::PackageBetweenFailure(_) => "PackageBetweenFailure",
            PackageError::PackageNotFound(_) => "PackageNotFound",
        }
    }
}

impl Clone for PackageError {
    fn clone(&self) -> Self {
        match self {
            PackageError::PackageJsonParseFailure { path, error } => {
                PackageError::PackageJsonParseFailure {
                    path: path.clone(),
                    // Convert error to string and parse it back
                    error: serde_json::Error::custom(error.to_string()),
                }
            }
            PackageError::PackageJsonIoFailure { path, error } => {
                PackageError::PackageJsonIoFailure {
                    path: path.clone(),
                    // Create a new io::Error with the same kind and message
                    error: io::Error::new(error.kind(), error.to_string()),
                }
            }
            PackageError::PackageBetweenFailure(msg) => {
                PackageError::PackageBetweenFailure(msg.clone())
            }
            PackageError::PackageNotFound(msg) => PackageError::PackageNotFound(msg.clone()),
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

#[derive(Error, Debug)]
pub enum PackageRegistryError {
    #[error("Failed to fetch: {0:?}")]
    FetchFailure(#[source] reqwest::Error),
    #[error("Failed to parse json response: {0:?}")]
    JsonParseFailure(#[source] reqwest::Error),
    #[error("Failed to found package: {package_name}@{version}")]
    NotFound { package_name: String, version: String },
    #[error("Failed to acquire lock on packages")]
    LockFailure,
    #[error("Failed to download package {package_name}@{version}: {source}")]
    DownloadFailure {
        package_name: String,
        version: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("Failed to extract package {package_name}@{version} to {destination}: {source}")]
    ExtractionFailure {
        package_name: String,
        version: String,
        destination: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Invalid package tarball format for {package_name}@{version}: {reason}")]
    InvalidTarball {
        package_name: String,
        version: String,
        reason: String,
    },
    #[error("Failed to create destination directory {path}: {source}")]
    DirectoryCreationFailure {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl AsRef<str> for PackageRegistryError {
    fn as_ref(&self) -> &str {
        match self {
            PackageRegistryError::FetchFailure(_) => "FetchFailure",
            PackageRegistryError::JsonParseFailure(_) => "JsonParseFailure",
            PackageRegistryError::NotFound { package_name: _, version: _ } => "NotFound",
            PackageRegistryError::LockFailure => "LockFailure",
            PackageRegistryError::DownloadFailure { package_name: _, version: _, source: _ } => "DownloadFailure",
            PackageRegistryError::ExtractionFailure { package_name: _, version: _, destination: _, source: _ } => "ExtractionFailure",
            PackageRegistryError::InvalidTarball { package_name: _, version: _, reason: _ } => "InvalidTarball",
            PackageRegistryError::DirectoryCreationFailure { path: _, source: _ } => "DirectoryCreationFailure",
        }
    }
}

impl From<reqwest::Error> for PackageRegistryError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_decode() {
            PackageRegistryError::JsonParseFailure(error)
        } else {
            PackageRegistryError::FetchFailure(error)
        }
    }
}

impl<T> From<PoisonError<T>> for PackageRegistryError {
    fn from(_: PoisonError<T>) -> Self {
        PackageRegistryError::LockFailure
    }
}
