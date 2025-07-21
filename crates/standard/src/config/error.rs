//! Configuration error types.
//!
//! This module defines error types specific to configuration operations,
//! providing detailed error information for debugging configuration issues.

use std::path::PathBuf;
use thiserror::Error;

use crate::error::Error as StandardError;

/// Result type for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Errors that can occur during configuration operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// File not found error.
    #[error("Configuration file not found: {path}")]
    FileNotFound {
        /// The path that was not found.
        path: PathBuf,
    },

    /// File read error.
    #[error("Failed to read configuration file: {path}")]
    FileReadError {
        /// The path that could not be read.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// File write error.
    #[error("Failed to write configuration file: {path}")]
    FileWriteError {
        /// The path that could not be written.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// Parse error for a specific format.
    #[error("Failed to parse {format} configuration: {message}")]
    ParseError {
        /// The format that failed to parse.
        format: String,
        /// Error message.
        message: String,
    },

    /// Serialization error.
    #[error("Failed to serialize configuration as {format}: {message}")]
    SerializeError {
        /// The format that failed to serialize.
        format: String,
        /// Error message.
        message: String,
    },

    /// Validation error.
    #[error("Configuration validation failed: {message}")]
    ValidationError {
        /// Validation error message.
        message: String,
    },

    /// Environment variable error.
    #[error("Environment variable error: {message}")]
    EnvironmentError {
        /// Error message.
        message: String,
    },

    /// Type conversion error.
    #[error("Type conversion error: expected {expected}, got {actual}")]
    TypeError {
        /// The expected type.
        expected: String,
        /// The actual type.
        actual: String,
    },

    /// Key not found error.
    #[error("Configuration key not found: {key}")]
    KeyNotFound {
        /// The key that was not found.
        key: String,
    },

    /// Merge conflict error.
    #[error("Configuration merge conflict: {message}")]
    MergeConflict {
        /// Conflict description.
        message: String,
    },

    /// Provider error.
    #[error("Configuration provider '{provider}' error: {message}")]
    ProviderError {
        /// Provider name.
        provider: String,
        /// Error message.
        message: String,
    },

    /// Generic configuration error.
    #[error("Configuration error: {0}")]
    Other(String),

    /// Standard error wrapper.
    #[error(transparent)]
    Standard(#[from] StandardError),
}

impl ConfigError {
    /// Creates a validation error with the given message.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError { message: message.into() }
    }

    /// Creates a parse error for the given format.
    pub fn parse(format: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ParseError { format: format.into(), message: message.into() }
    }

    /// Creates a serialization error for the given format.
    pub fn serialize(format: impl Into<String>, message: impl Into<String>) -> Self {
        Self::SerializeError { format: format.into(), message: message.into() }
    }

    /// Creates a type error.
    pub fn type_error(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::TypeError { expected: expected.into(), actual: actual.into() }
    }

    /// Creates a provider error.
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProviderError { provider: provider.into(), message: message.into() }
    }

    /// Creates a generic configuration error.
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<String> for ConfigError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for ConfigError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        Self::ParseError { format: "JSON".to_string(), message: err.to_string() }
    }
}
