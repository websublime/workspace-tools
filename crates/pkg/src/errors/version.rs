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
///
/// # Variants
///
/// - `Parse` - Represents errors that occur when parsing a version string fails
/// - `InvalidVersion` - Represents errors from business logic validation
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{Version, VersionError};
///
/// // Handle parse errors
/// let result = Version::parse("not-a-version");
/// if let Err(e) = result {
///     match e {
///         VersionError::Parse { message, .. } => {
///             println!("Failed to parse version: {}", message);
///         }
///         VersionError::InvalidVersion(msg) => {
///             println!("Invalid version: {}", msg);
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum VersionError {
    /// Indicates that a version string couldn't be parsed according to semver rules.
    ///
    /// Contains the original error and a human-readable message.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Version;
    ///
    /// let result = Version::parse("1.x");
    /// assert!(result.is_err());
    /// ```
    #[error("Failed to parse version: {message}")]
    Parse {
        /// The original semver error that occurred during parsing
        #[source]
        error: semver::Error,
        /// A human-readable description of the error
        message: String,
    },

    /// Indicates that a version is invalid for business logic reasons.
    ///
    /// This could be due to:
    /// - Version containing special markers like "workspace:*"
    /// - Version not meeting specific format requirements
    /// - Version being incompatible with other constraints
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Dependency, VersionError};
    ///
    /// let result = Dependency::new("pkg", "workspace:*");
    /// assert!(matches!(result, Err(VersionError::InvalidVersion(_))));
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
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Version, VersionError};
    /// use semver::Version as SemVersion;
    ///
    /// fn get_version(input: &str) -> Result<SemVersion, VersionError> {
    ///     let version = SemVersion::parse(input)?; // semver::Error converts to VersionError
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
                // Create a new semver::Error by constructing a known invalid version
                // This is safe because we're intentionally creating an error
                let error = match semver::Version::parse("invalid-version") {
                    Ok(_) => unreachable!("'invalid-version' should always fail to parse"),
                    Err(e) => e,
                };
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
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Version, VersionError};
    /// use std::convert::AsRef;
    ///
    /// let error = Version::parse("invalid").unwrap_err();
    /// match error.as_ref() {
    ///     "VersionErrorParse" => println!("Got a parse error"),
    ///     "VersionErrorInvalidVersion" => println!("Got an invalid version error"),
    ///     _ => println!("Unknown error type"),
    /// }
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            VersionError::Parse { error: _, message: _ } => "VersionErrorParse",
            VersionError::InvalidVersion(_) => "VersionErrorInvalidVersion",
        }
    }
}
