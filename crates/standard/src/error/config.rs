//! # Configuration error types
//!
//! ## What
//! This module defines errors that can occur during configuration operations,
//! including file I/O, parsing, validation, and environment variable handling.
//!
//! ## How
//! The `ConfigError` enum provides specific variants for different configuration
//! failure modes, with helper methods for common error creation patterns.
//!
//! ## Why
//! Separating configuration errors enables precise error handling for
//! configuration-related operations, supporting multiple configuration sources
//! and formats.

use core::result::Result as CoreResult;
use std::path::PathBuf;
use thiserror::Error as ThisError;

/// Errors that can occur during configuration operations.
#[derive(ThisError, Debug, Clone)]
pub enum ConfigError {
    /// File not found error.
    #[error("Configuration file not found: {path}")]
    FileNotFound {
        /// The path that was not found.
        path: PathBuf,
    },

    /// File read error.
    #[error("Failed to read configuration file '{path}': {message}")]
    FileReadError {
        /// The path that could not be read.
        path: PathBuf,
        /// The error message.
        message: String,
    },

    /// File write error.
    #[error("Failed to write configuration file '{path}': {message}")]
    FileWriteError {
        /// The path that could not be written.
        path: PathBuf,
        /// The error message.
        message: String,
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

/// Result type for configuration operations.
pub type ConfigResult<T> = CoreResult<T, ConfigError>;

impl AsRef<str> for ConfigError {
    fn as_ref(&self) -> &str {
        match self {
            ConfigError::FileNotFound { .. } => "ConfigError::FileNotFound",
            ConfigError::FileReadError { .. } => "ConfigError::FileReadError",
            ConfigError::FileWriteError { .. } => "ConfigError::FileWriteError",
            ConfigError::ParseError { .. } => "ConfigError::ParseError",
            ConfigError::SerializeError { .. } => "ConfigError::SerializeError",
            ConfigError::ValidationError { .. } => "ConfigError::ValidationError",
            ConfigError::EnvironmentError { .. } => "ConfigError::EnvironmentError",
            ConfigError::TypeError { .. } => "ConfigError::TypeError",
            ConfigError::KeyNotFound { .. } => "ConfigError::KeyNotFound",
            ConfigError::MergeConflict { .. } => "ConfigError::MergeConflict",
            ConfigError::ProviderError { .. } => "ConfigError::ProviderError",
            ConfigError::Other(_) => "ConfigError::Other",
        }
    }
}