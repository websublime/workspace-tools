//! # Version Error Handling
//!
//! This module provides error types for handling version-related errors in package management.
//!
//! ## Usage
//!
//! ```rust
//! use your_crate::VersionError;
//! use semver::Version;
//!
//! fn parse_version(version_str: &str) -> Result<Version, VersionError> {
//!     let version = Version::parse(version_str)?;
//!     Ok(version)
//! }
//!
//! // Handle errors
//! fn example() {
//!     match parse_version("invalid") {
//!         Ok(v) => println!("Valid version: {}", v),
//!         Err(e) => match e {
//!             VersionError::Parse { message, .. } => println!("Parse error: {}", message),
//!             VersionError::InvalidVersion(msg) => println!("Invalid version: {}", msg),
//!         }
//!     }
//! }
//! ```

use thiserror::Error;

/// Errors that can occur when working with semantic versions.
///
/// This enum represents the various error conditions that can arise when
/// parsing, validating, or comparing semantic versions.
#[derive(Error, Debug)]
pub enum VersionError {
    /// Indicates that a version string couldn't be parsed according to semver rules.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use your_crate::VersionError;
    /// # use semver::Version;
    /// let result = Version::parse("1.x");
    /// assert!(matches!(result.map_err(VersionError::from),
    ///                  Err(VersionError::Parse { .. })));
    /// ```
    #[error("Failed to parse version: {message}")]
    Parse {
        /// The original semver error that occurred during parsing
        #[source]
        error: semver::Error,
        /// A human-readable description of the error
        message: String,
    },

    /// Indicates that a version is invalid for reasons other than semver parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use your_crate::VersionError;
    /// fn validate_version(version: &str) -> Result<(), VersionError> {
    ///     if version.starts_with("0.0.") {
    ///         return Err(VersionError::InvalidVersion(
    ///             "Zero-prefixed versions are not allowed".to_string()));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("Invalid version: {0}")]
    InvalidVersion(String),
}

impl From<semver::Error> for VersionError {
    /// Converts a semver::Error into a VersionError.
    ///
    /// This allows semver parsing errors to be easily converted into
    /// the application's error type using the `?` operator.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use your_crate::VersionError;
    /// # use semver::Version;
    /// fn get_version(input: &str) -> Result<Version, VersionError> {
    ///     let version = Version::parse(input)?; // semver::Error converts to VersionError
    ///     Ok(version)
    /// }
    /// ```
    fn from(error: semver::Error) -> Self {
        VersionError::Parse { message: error.to_string(), error }
    }
}

impl Clone for VersionError {
    /// Creates a clone of the VersionError.
    ///
    /// Note: For `Parse` variants, this creates a new semver::Error since
    /// semver::Error doesn't implement Clone.
    fn clone(&self) -> Self {
        match self {
            VersionError::Parse { message, .. } => {
                // Create a new semver::Error
                let error = semver::Version::parse("invalid-version").unwrap_err();
                VersionError::Parse { error, message: message.clone() }
            }
            VersionError::InvalidVersion(message) => VersionError::InvalidVersion(message.clone()),
        }
    }
}

impl AsRef<str> for VersionError {
    /// Returns a string slice representation of the error type.
    ///
    /// This is useful for error categorization and reporting.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use your_crate::VersionError;
    /// # use std::convert::AsRef;
    /// fn report_error_type(err: &VersionError) {
    ///     let type_name = err.as_ref();
    ///     println!("Error type: {}", type_name);
    /// }
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            VersionError::Parse { error: _, message: _ } => "VersionErrorParse",
            VersionError::InvalidVersion(_) => "VersionErrorInvalidVersion",
        }
    }
}
