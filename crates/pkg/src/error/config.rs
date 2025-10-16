//! Configuration error types for package tools.
//!
//! **What**: Defines error types specific to configuration loading, parsing, validation,
//! and management operations.
//!
//! **How**: Uses `thiserror` for error definitions with rich context information including
//! file paths, validation errors, and detailed error messages. Implements `AsRef<str>` for
//! string conversion.
//!
//! **Why**: To provide clear, actionable error messages for configuration issues, enabling
//! users to quickly identify and fix configuration problems.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::error::{ConfigError, ConfigResult};
//! use std::path::PathBuf;
//!
//! fn validate_config_file(path: &str) -> ConfigResult<()> {
//!     if path.is_empty() {
//!         return Err(ConfigError::InvalidPath {
//!             path: PathBuf::from(path),
//!             reason: "Configuration path cannot be empty".to_string(),
//!         });
//!     }
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for configuration operations.
///
/// This type alias simplifies error handling in configuration-related functions
/// by defaulting to `ConfigError` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{ConfigError, ConfigResult};
///
/// fn load_config() -> ConfigResult<String> {
///     Ok("config data".to_string())
/// }
/// ```
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Errors that can occur during configuration operations.
///
/// This enum covers all possible error scenarios when working with package tools
/// configuration, including file I/O, parsing, validation, and merging operations.
///
/// # Examples
///
/// ## Handling configuration errors
///
/// ```rust
/// use sublime_pkg_tools::error::ConfigError;
/// use std::path::PathBuf;
///
/// fn handle_config_error(error: ConfigError) {
///     match error {
///         ConfigError::NotFound { path } => {
///             eprintln!("Config file not found: {}", path.display());
///         }
///         ConfigError::ValidationFailed { errors } => {
///             eprintln!("Validation failed with {} errors:", errors.len());
///             for err in errors {
///                 eprintln!("  - {}", err);
///             }
///         }
///         _ => eprintln!("Configuration error: {}", error),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::ConfigError;
/// use std::path::PathBuf;
///
/// let error = ConfigError::NotFound {
///     path: PathBuf::from("/path/to/config.toml"),
/// };
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("not found"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum ConfigError {
    /// Configuration file not found at the specified path.
    ///
    /// This error occurs when attempting to load a configuration file that does not exist.
    #[error("Configuration file not found: {path}")]
    NotFound {
        /// The path to the missing configuration file.
        path: PathBuf,
    },

    /// Failed to parse the configuration file.
    ///
    /// This error occurs when the configuration file exists but contains invalid syntax
    /// or cannot be parsed as the expected format (TOML, JSON, YAML).
    #[error("Failed to parse configuration file '{path}': {reason}")]
    ParseError {
        /// The path to the configuration file that failed to parse.
        path: PathBuf,
        /// The underlying parsing error message.
        reason: String,
    },

    /// Invalid configuration values provided.
    ///
    /// This error occurs when configuration values are syntactically correct but
    /// semantically invalid (e.g., negative numbers where positive expected).
    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        /// Description of why the configuration is invalid.
        message: String,
    },

    /// Configuration validation failed with one or more validation errors.
    ///
    /// This error occurs when the configuration structure is valid but the values
    /// fail business logic validation rules.
    #[error("Configuration validation failed")]
    ValidationFailed {
        /// List of validation error messages.
        errors: Vec<String>,
    },

    /// Unsupported configuration file format.
    ///
    /// This error occurs when attempting to load a configuration file with an
    /// unsupported or unrecognized format.
    #[error("Unsupported configuration format: {format}")]
    UnsupportedFormat {
        /// The format that is not supported.
        format: String,
    },

    /// I/O error during configuration file operations.
    ///
    /// This error occurs when there's a filesystem or I/O error while reading
    /// or writing configuration files.
    #[error("I/O error during configuration operation: {reason}")]
    Io {
        /// The underlying I/O error message.
        reason: String,
    },

    /// Failed to parse or access an environment variable.
    ///
    /// This error occurs when environment variable overrides are enabled but
    /// a variable cannot be parsed or accessed.
    #[error("Environment variable error for '{var_name}': {reason}")]
    EnvVarError {
        /// Name of the environment variable that caused the error.
        var_name: String,
        /// Description of why the variable could not be used.
        reason: String,
    },

    /// Configuration merge conflict between multiple sources.
    ///
    /// This error occurs when merging configurations from multiple sources results
    /// in conflicting values that cannot be automatically resolved.
    #[error("Configuration merge conflict in field '{field}': {reason}")]
    MergeConflict {
        /// The configuration field that has conflicting values.
        field: String,
        /// Description of the merge conflict.
        reason: String,
    },

    /// Invalid file path provided for configuration.
    ///
    /// This error occurs when a configuration path is malformed, contains invalid
    /// characters, or points to an invalid location.
    #[error("Invalid configuration path '{path}': {reason}")]
    InvalidPath {
        /// The invalid path.
        path: PathBuf,
        /// Description of why the path is invalid.
        reason: String,
    },

    /// Required configuration field is missing.
    ///
    /// This error occurs when a mandatory configuration field is not provided
    /// in any configuration source.
    #[error("Missing required configuration field: {field}")]
    MissingField {
        /// Name of the missing required field.
        field: String,
    },

    /// Configuration field has an invalid type.
    ///
    /// This error occurs when a configuration field is present but has the wrong
    /// data type (e.g., string when number expected).
    #[error("Invalid type for configuration field '{field}': expected {expected}, got {actual}")]
    InvalidFieldType {
        /// Name of the field with incorrect type.
        field: String,
        /// Expected type description.
        expected: String,
        /// Actual type found.
        actual: String,
    },

    /// Configuration file permission denied.
    ///
    /// This error occurs when the process lacks necessary permissions to read
    /// or write a configuration file.
    #[error("Permission denied for configuration file: {path}")]
    PermissionDenied {
        /// The path to the configuration file with permission issues.
        path: PathBuf,
    },

    /// Circular dependency detected in configuration includes or references.
    ///
    /// This error occurs when configuration files include each other in a circular
    /// manner, preventing resolution.
    #[error("Circular dependency detected in configuration: {cycle}")]
    CircularDependency {
        /// Description of the circular dependency chain.
        cycle: String,
    },
}

impl AsRef<str> for ConfigError {
    /// Returns a string representation of the error.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ConfigError;
    /// use std::path::PathBuf;
    ///
    /// let error = ConfigError::NotFound {
    ///     path: PathBuf::from("config.toml"),
    /// };
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("not found"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::NotFound { .. } => "configuration file not found",
            Self::ParseError { .. } => "configuration parse error",
            Self::InvalidConfig { .. } => "invalid configuration",
            Self::ValidationFailed { .. } => "configuration validation failed",
            Self::UnsupportedFormat { .. } => "unsupported configuration format",
            Self::Io { .. } => "configuration I/O error",
            Self::EnvVarError { .. } => "environment variable error",
            Self::MergeConflict { .. } => "configuration merge conflict",
            Self::InvalidPath { .. } => "invalid configuration path",
            Self::MissingField { .. } => "missing required configuration field",
            Self::InvalidFieldType { .. } => "invalid configuration field type",
            Self::PermissionDenied { .. } => "configuration permission denied",
            Self::CircularDependency { .. } => "circular configuration dependency",
        }
    }
}

impl ConfigError {
    /// Returns the number of errors for `ValidationFailed` variant.
    ///
    /// This helper method provides a convenient way to get the count of validation
    /// errors without pattern matching.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::ValidationFailed {
    ///     errors: vec![
    ///         "Field 'version' cannot be empty".to_string(),
    ///         "Field 'strategy' must be 'independent' or 'unified'".to_string(),
    ///     ],
    /// };
    ///
    /// assert_eq!(error.count(), 2);
    /// ```
    #[must_use]
    pub fn count(&self) -> usize {
        match self {
            Self::ValidationFailed { errors } => errors.len(),
            _ => 1,
        }
    }

    /// Returns the formatted error list as a single string.
    ///
    /// This helper method formats all validation errors as a bulleted list,
    /// useful for displaying multiple errors in a user-friendly format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::ValidationFailed {
    ///     errors: vec![
    ///         "Invalid bump type".to_string(),
    ///         "Missing environment".to_string(),
    ///     ],
    /// };
    ///
    /// let formatted = error.errors();
    /// assert!(formatted.contains("Invalid bump type"));
    /// assert!(formatted.contains("Missing environment"));
    /// ```
    #[must_use]
    pub fn errors(&self) -> String {
        match self {
            Self::ValidationFailed { errors } => {
                errors.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n")
            }
            _ => self.to_string(),
        }
    }
}
