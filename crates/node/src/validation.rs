//! Parameter validation module for Node.js bindings.
//!
//! # What
//!
//! This module provides validation utilities for NAPI function parameters. It ensures
//! that all parameters passed from JavaScript/TypeScript are valid before executing
//! CLI commands, providing clear error messages for invalid inputs.
//!
//! # How
//!
//! The module provides:
//! - `ValidationError`: Structured validation error type with field context and optional value
//! - `ValidationResult<T>`: Type alias for validation results
//! - `validators`: Module containing common parameter validators
//!
//! Validation follows these principles:
//! 1. Fail fast: Return on first validation error
//! 2. Clear messages: Error messages should indicate the exact problem and field
//! 3. Type safety: Leverage Rust's type system for compile-time validation where possible
//! 4. Value context: Include the invalid value in errors when appropriate
//!
//! # Why
//!
//! Validating parameters at the NAPI layer provides several benefits:
//! - Early error detection before CLI execution
//! - Clear, user-friendly error messages with field context
//! - Consistent validation across all NAPI functions
//! - Prevention of unnecessary work when parameters are invalid
//! - The `value` field in `ValidationError` helps debugging by showing what was passed
//!
//! # Examples
//!
//! ## Using ValidationError directly
//!
//! ```rust,ignore
//! use sublime_node_tools::validation::{ValidationError, validators};
//! use sublime_node_tools::error::ErrorInfo;
//!
//! // Create a required field error
//! let error = ValidationError::required("packages");
//! let error_info: ErrorInfo = error.into();
//! assert_eq!(error_info.code, "EVALIDATION");
//!
//! // Create an invalid value error
//! let error = ValidationError::invalid("bumpType", "must be major, minor, patch, or none", Some("invalid"));
//! let error_info: ErrorInfo = error.into();
//! assert!(error_info.message.contains("must be major"));
//! ```
//!
//! ## Using validators module
//!
//! ```rust,ignore
//! use sublime_node_tools::validation::validators;
//!
//! // Validate a path exists
//! validators::path_exists("/path/to/project")?;
//!
//! // Validate a field is not empty
//! validators::not_empty("message", "Add new feature")?;
//!
//! // Validate bump type (major, minor, patch, none)
//! validators::bump_type("minor")?;
//!
//! // Validate timeout within bounds
//! validators::timeout("timeoutSecs", 30, 1, 3600)?;
//! ```
//!
//! ## In NAPI functions
//!
//! ```typescript
//! import { changesetAdd } from '@websublime/workspace-tools';
//!
//! // If packages is empty, validation will fail before CLI execution
//! const result = await changesetAdd({
//!   root: '.',
//!   packages: [], // Empty packages array
//!   bumpType: 'minor',
//!   message: 'Add feature'
//! });
//!
//! if (!result.success && result.error.code === 'EVALIDATION') {
//!   console.error(`Validation failed: ${result.error.message}`);
//!   // Output: Validation failed: packages array cannot be empty
//! }
//! ```

use crate::error::ErrorInfo;
use std::path::Path;

/// Result type for validation operations.
///
/// This type alias provides a convenient way to return validation results
/// that either succeed with a value of type `T` or fail with an `ErrorInfo`.
///
/// # Type Parameters
///
/// * `T` - The type of the successful validation result
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::validation::ValidationResult;
///
/// fn validate_name(name: &str) -> ValidationResult<String> {
///     if name.is_empty() {
///         return Err(ErrorInfo::validation("Name cannot be empty", Some("name")));
///     }
///     Ok(name.to_string())
/// }
/// ```
#[allow(dead_code)]
pub(crate) type ValidationResult<T> = Result<T, ErrorInfo>;

/// Structured validation error with field context and optional value.
///
/// `ValidationError` provides detailed information about validation failures,
/// including which field failed validation, a descriptive message, and optionally
/// the value that caused the failure.
///
/// This type is designed to be converted into `ErrorInfo` for use in NAPI responses,
/// ensuring that validation errors are properly formatted with the `EVALIDATION` code.
///
/// # Fields
///
/// * `field` - The name of the field that failed validation
/// * `message` - A descriptive error message explaining the validation failure
/// * `value` - Optional: The actual value that failed validation (useful for debugging)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::validation::ValidationError;
///
/// // Required field error (no value provided)
/// let error = ValidationError::required("username");
/// assert_eq!(error.field, "username");
/// assert!(error.message.contains("required"));
/// assert!(error.value.is_none());
///
/// // Invalid value error (with the problematic value)
/// let error = ValidationError::invalid(
///     "bumpType",
///     "must be one of: major, minor, patch, none",
///     Some("invalid"),
/// );
/// assert_eq!(error.field, "bumpType");
/// assert_eq!(error.value, Some("invalid".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct ValidationError {
    /// The name of the field that failed validation.
    ///
    /// This should match the JavaScript/TypeScript parameter name for
    /// consistency in error messages shown to users.
    pub field: String,

    /// A descriptive error message explaining why validation failed.
    ///
    /// The message should be clear and actionable, explaining what
    /// the valid values or format should be.
    pub message: String,

    /// The optional value that failed validation.
    ///
    /// Including the actual value helps with debugging and provides
    /// better context in error messages. This is `None` for errors
    /// like "required field" where no value was provided.
    pub value: Option<String>,
}

#[allow(dead_code)]
impl ValidationError {
    /// Creates a new `ValidationError` for a required field that was not provided.
    ///
    /// Use this constructor when a mandatory field is missing or empty.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the required field
    ///
    /// # Returns
    ///
    /// A `ValidationError` with a message indicating the field is required.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::ValidationError;
    ///
    /// let error = ValidationError::required("packages");
    /// assert_eq!(error.field, "packages");
    /// assert_eq!(error.message, "packages is required");
    /// assert!(error.value.is_none());
    /// ```
    #[must_use]
    pub fn required(field: &str) -> Self {
        Self { field: field.to_string(), message: format!("{field} is required"), value: None }
    }

    /// Creates a new `ValidationError` for an invalid value.
    ///
    /// Use this constructor when a value is provided but doesn't meet
    /// validation requirements.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field with the invalid value
    /// * `message` - A description of why the value is invalid
    /// * `value` - The invalid value (optional, for debugging context)
    ///
    /// # Returns
    ///
    /// A `ValidationError` with the provided message and value context.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::ValidationError;
    ///
    /// // With value context
    /// let error = ValidationError::invalid(
    ///     "timeout",
    ///     "must be between 1 and 3600 seconds",
    ///     Some("0"),
    /// );
    /// assert_eq!(error.field, "timeout");
    /// assert_eq!(error.value, Some("0".to_string()));
    ///
    /// // Without value context
    /// let error = ValidationError::invalid(
    ///     "packages",
    ///     "array cannot be empty",
    ///     None::<String>,
    /// );
    /// assert!(error.value.is_none());
    /// ```
    #[must_use]
    pub fn invalid<S: Into<String>>(field: &str, message: &str, value: Option<S>) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            value: value.map(Into::into),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(val) => write!(f, "{}: {} (got: {})", self.field, self.message, val),
            None => write!(f, "{}: {}", self.field, self.message),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Converts a `ValidationError` into an `ErrorInfo`.
///
/// This conversion ensures that validation errors are properly formatted
/// with the `EVALIDATION` error code and include the field name in the context.
///
/// # Conversion Details
///
/// - `code`: Always set to `"EVALIDATION"`
/// - `message`: The validation error message
/// - `context`: Set to the field name
/// - `kind`: Always set to `"Validation"`
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::validation::ValidationError;
/// use sublime_node_tools::error::ErrorInfo;
///
/// let validation_error = ValidationError::required("root");
/// let error_info: ErrorInfo = validation_error.into();
///
/// assert_eq!(error_info.code, "EVALIDATION");
/// assert_eq!(error_info.kind, "Validation");
/// assert_eq!(error_info.context, Some("root".to_string()));
/// ```
#[allow(dead_code)]
impl From<ValidationError> for ErrorInfo {
    fn from(err: ValidationError) -> Self {
        ErrorInfo::validation(err.message, Some(&err.field))
    }
}

/// Validators module containing common parameter validation functions.
///
/// This module provides a collection of validators for common parameter patterns
/// used across NAPI functions. Each validator returns `Result<(), ValidationError>`
/// to enable proper error handling and conversion to `ErrorInfo`.
///
/// # Available Validators
///
/// - `path_exists`: Validates that a file system path exists
/// - `not_empty`: Validates that a string is not empty or whitespace-only
/// - `bump_type`: Validates that a bump type is valid (major, minor, patch, none)
/// - `timeout`: Validates that a timeout is within specified bounds
/// - `root`: Validates that a root path exists and is a directory
/// - `packages_not_empty`: Validates that a packages array is not empty
/// - `message_not_empty`: Validates that a message is not empty
/// - `semver`: Validates that a string is valid semver
/// - `mutual_exclusion`: Validates that mutually exclusive params are not both set
/// - `optional_timeout`: Validates an optional timeout value
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::validation::validators;
///
/// // Chain multiple validations
/// fn validate_params(root: &str, message: &str, bump: &str) -> Result<(), ValidationError> {
///     validators::path_exists(root)?;
///     validators::not_empty("message", message)?;
///     validators::bump_type(bump)?;
///     Ok(())
/// }
/// ```
#[allow(dead_code)]
pub(crate) mod validators {
    use super::{ErrorInfo, Path, ValidationError, ValidationResult};

    /// Valid bump types as defined in sublime_pkg_tools::types::VersionBump.
    ///
    /// These match the VersionBump enum variants: Major, Minor, Patch, None.
    pub const VALID_BUMP_TYPES: &[&str] = &["major", "minor", "patch", "none"];

    /// Validates that a path exists on the file system.
    ///
    /// This validator checks if the given path exists, regardless of whether
    /// it's a file or directory. For directory-specific validation, use
    /// the `root` validator instead.
    ///
    /// # Arguments
    ///
    /// * `path` - The file system path to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the path exists
    /// * `Err(ValidationError)` if the path does not exist
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid path (assuming /tmp exists)
    /// assert!(validators::path_exists("/tmp").is_ok());
    ///
    /// // Invalid path
    /// let result = validators::path_exists("/nonexistent/path");
    /// assert!(result.is_err());
    /// let error = result.unwrap_err();
    /// assert_eq!(error.field, "path");
    /// ```
    pub fn path_exists(path: &str) -> Result<(), ValidationError> {
        if !Path::new(path).exists() {
            return Err(ValidationError::invalid(
                "path",
                &format!("path does not exist: {path}"),
                Some(path),
            ));
        }
        Ok(())
    }

    /// Validates that a string value is not empty or whitespace-only.
    ///
    /// This validator checks that the provided string contains meaningful content,
    /// rejecting empty strings and strings containing only whitespace.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field being validated (for error messages)
    /// * `value` - The string value to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the value is not empty
    /// * `Err(ValidationError)` if the value is empty or whitespace-only
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid values
    /// assert!(validators::not_empty("message", "Add feature").is_ok());
    /// assert!(validators::not_empty("cmd", "npm test").is_ok());
    ///
    /// // Invalid values
    /// assert!(validators::not_empty("message", "").is_err());
    /// assert!(validators::not_empty("message", "   ").is_err());
    /// ```
    pub fn not_empty(field: &str, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::invalid(field, "cannot be empty", None::<String>));
        }
        Ok(())
    }

    /// Validates that a bump type is valid.
    ///
    /// Valid bump types are: `major`, `minor`, `patch`, `none`.
    /// These correspond to the `VersionBump` enum in `sublime_pkg_tools`.
    ///
    /// # Arguments
    ///
    /// * `value` - The bump type string to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the bump type is valid
    /// * `Err(ValidationError)` if the bump type is not recognized
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid bump types
    /// assert!(validators::bump_type("major").is_ok());
    /// assert!(validators::bump_type("minor").is_ok());
    /// assert!(validators::bump_type("patch").is_ok());
    /// assert!(validators::bump_type("none").is_ok());
    ///
    /// // Invalid bump type
    /// let result = validators::bump_type("invalid");
    /// assert!(result.is_err());
    /// let error = result.unwrap_err();
    /// assert_eq!(error.value, Some("invalid".to_string()));
    /// ```
    pub fn bump_type(value: &str) -> Result<(), ValidationError> {
        if !VALID_BUMP_TYPES.contains(&value) {
            return Err(ValidationError::invalid(
                "bumpType",
                &format!("must be one of: {}", VALID_BUMP_TYPES.join(", ")),
                Some(value),
            ));
        }
        Ok(())
    }

    /// Validates that a timeout value is within specified bounds.
    ///
    /// This validator ensures that a timeout value falls within an acceptable
    /// range, providing clear error messages for values outside the bounds.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field being validated (for error messages)
    /// * `value` - The timeout value in seconds
    /// * `min` - The minimum allowed timeout (inclusive)
    /// * `max` - The maximum allowed timeout (inclusive)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the timeout is within bounds
    /// * `Err(ValidationError)` if the timeout is outside bounds
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid timeout (within 1-3600 range)
    /// assert!(validators::timeout("timeoutSecs", 30, 1, 3600).is_ok());
    /// assert!(validators::timeout("timeoutSecs", 1, 1, 3600).is_ok());
    /// assert!(validators::timeout("timeoutSecs", 3600, 1, 3600).is_ok());
    ///
    /// // Invalid timeout (below minimum)
    /// let result = validators::timeout("timeoutSecs", 0, 1, 3600);
    /// assert!(result.is_err());
    ///
    /// // Invalid timeout (above maximum)
    /// let result = validators::timeout("timeoutSecs", 7200, 1, 3600);
    /// assert!(result.is_err());
    /// ```
    pub fn timeout(field: &str, value: u64, min: u64, max: u64) -> Result<(), ValidationError> {
        if value < min {
            return Err(ValidationError::invalid(
                field,
                &format!("must be at least {min} seconds"),
                Some(value.to_string()),
            ));
        }
        if value > max {
            return Err(ValidationError::invalid(
                field,
                &format!("cannot exceed {max} seconds"),
                Some(value.to_string()),
            ));
        }
        Ok(())
    }

    /// Validates that a root path is provided and exists as a directory.
    ///
    /// This function checks that the root path:
    /// 1. Is not empty
    /// 2. Points to an existing path
    /// 3. Is a directory (not a file)
    ///
    /// # Arguments
    ///
    /// * `root` - The root path to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the path is valid
    /// * `Err(ErrorInfo)` if the path is empty, doesn't exist, or is not a directory
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid path
    /// let result = validators::root("/valid/directory");
    ///
    /// // Invalid path (empty)
    /// let result = validators::root("");
    /// assert!(result.is_err());
    /// ```
    pub fn root(root: &str) -> ValidationResult<()> {
        if root.is_empty() {
            return Err(ErrorInfo::validation("root path cannot be empty", Some("root")));
        }

        let path = Path::new(root);
        if !path.exists() {
            return Err(ErrorInfo::not_found(
                format!("root path does not exist: {root}"),
                Some(root),
            ));
        }

        if !path.is_dir() {
            return Err(ErrorInfo::validation(
                format!("root path must be a directory: {root}"),
                Some("root"),
            ));
        }

        Ok(())
    }

    /// Validates that a packages array is not empty.
    ///
    /// This function checks that the packages array contains at least one element.
    ///
    /// # Arguments
    ///
    /// * `packages` - The packages array to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the array contains at least one package
    /// * `Err(ErrorInfo)` if the array is empty
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid packages
    /// let result = validators::packages_not_empty(&["@scope/pkg1".to_string()]);
    /// assert!(result.is_ok());
    ///
    /// // Invalid packages (empty)
    /// let result = validators::packages_not_empty(&[]);
    /// assert!(result.is_err());
    /// ```
    pub fn packages_not_empty(packages: &[String]) -> ValidationResult<()> {
        if packages.is_empty() {
            return Err(ErrorInfo::validation("packages array cannot be empty", Some("packages")));
        }
        Ok(())
    }

    /// Validates that a bump type is valid (returns ErrorInfo).
    ///
    /// Valid bump types are: "major", "minor", "patch", "none".
    /// These correspond to the `VersionBump` enum in `sublime_pkg_tools`.
    ///
    /// # Arguments
    ///
    /// * `bump_type` - The bump type to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the bump type is valid
    /// * `Err(ErrorInfo)` if the bump type is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid bump types
    /// assert!(validators::bump_type_info("major").is_ok());
    /// assert!(validators::bump_type_info("minor").is_ok());
    /// assert!(validators::bump_type_info("patch").is_ok());
    /// assert!(validators::bump_type_info("none").is_ok());
    ///
    /// // Invalid bump type
    /// assert!(validators::bump_type_info("invalid").is_err());
    /// ```
    pub fn bump_type_info(bump_type: &str) -> ValidationResult<()> {
        if !VALID_BUMP_TYPES.contains(&bump_type) {
            return Err(ErrorInfo::validation(
                format!(
                    "invalid bump type '{}'. Valid types are: {}",
                    bump_type,
                    VALID_BUMP_TYPES.join(", ")
                ),
                Some("bumpType"),
            ));
        }
        Ok(())
    }

    /// Validates that a message is not empty.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to validate
    /// * `field_name` - The name of the field for error reporting
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the message is not empty
    /// * `Err(ErrorInfo)` if the message is empty
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid message
    /// let result = validators::message_not_empty("Add new feature", "message");
    /// assert!(result.is_ok());
    ///
    /// // Invalid message (empty)
    /// let result = validators::message_not_empty("", "message");
    /// assert!(result.is_err());
    /// ```
    pub fn message_not_empty(message: &str, field_name: &str) -> ValidationResult<()> {
        if message.trim().is_empty() {
            return Err(ErrorInfo::validation(
                format!("{field_name} cannot be empty"),
                Some(field_name),
            ));
        }
        Ok(())
    }

    /// Validates that a string is a valid semver version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to validate
    /// * `field_name` - The name of the field for error reporting
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the version is valid semver
    /// * `Err(ErrorInfo)` if the version is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid versions
    /// assert!(validators::semver("1.0.0", "version").is_ok());
    /// assert!(validators::semver("2.3.4-beta.1", "version").is_ok());
    ///
    /// // Invalid versions
    /// assert!(validators::semver("invalid", "version").is_err());
    /// assert!(validators::semver("1.0", "version").is_err());
    /// ```
    pub fn semver(version: &str, field_name: &str) -> ValidationResult<()> {
        let parts: Vec<&str> = version.split('.').collect();

        if parts.len() < 3 {
            return Err(ErrorInfo::validation(
                format!("invalid semver version '{version}'. Expected format: major.minor.patch"),
                Some(field_name),
            ));
        }

        // Check that major and minor are valid numbers
        for (i, part) in parts.iter().take(2).enumerate() {
            if part.parse::<u64>().is_err() {
                let component = match i {
                    0 => "major",
                    1 => "minor",
                    _ => "component",
                };
                return Err(ErrorInfo::validation(
                    format!(
                        "invalid semver version '{version}'. {component} component must be a number"
                    ),
                    Some(field_name),
                ));
            }
        }

        // Check patch (may contain prerelease identifier)
        let patch_part = parts[2].split('-').next().unwrap_or(parts[2]);
        if patch_part.parse::<u64>().is_err() {
            return Err(ErrorInfo::validation(
                format!("invalid semver version '{version}'. patch component must be a number"),
                Some(field_name),
            ));
        }

        Ok(())
    }

    /// Validates mutual exclusion of parameters.
    ///
    /// This function ensures that only one of the mutually exclusive parameters
    /// is set at a time.
    ///
    /// # Arguments
    ///
    /// * `params` - A slice of tuples containing (field_name, is_set) pairs
    ///
    /// # Returns
    ///
    /// * `Ok(())` if at most one parameter is set
    /// * `Err(ErrorInfo)` if more than one parameter is set
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid: only one is set
    /// let result = validators::mutual_exclusion(&[
    ///     ("filterPackage", true),
    ///     ("affected", false),
    /// ]);
    /// assert!(result.is_ok());
    ///
    /// // Invalid: both are set
    /// let result = validators::mutual_exclusion(&[
    ///     ("filterPackage", true),
    ///     ("affected", true),
    /// ]);
    /// assert!(result.is_err());
    /// ```
    pub fn mutual_exclusion(params: &[(&str, bool)]) -> ValidationResult<()> {
        let set_params: Vec<&str> =
            params.iter().filter(|(_, is_set)| *is_set).map(|(name, _)| *name).collect();

        if set_params.len() > 1 {
            return Err(ErrorInfo::validation(
                format!(
                    "parameters {} are mutually exclusive. Only one can be specified at a time",
                    set_params.join(", ")
                ),
                None::<String>,
            ));
        }

        Ok(())
    }

    /// Validates that a timeout value is positive (returns ErrorInfo).
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout value in seconds
    /// * `field_name` - The name of the field for error reporting
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the timeout is positive
    /// * `Err(ErrorInfo)` if the timeout is zero
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid timeout
    /// assert!(validators::timeout_positive(30, "timeoutSecs").is_ok());
    ///
    /// // Invalid timeout
    /// assert!(validators::timeout_positive(0, "timeoutSecs").is_err());
    /// ```
    pub fn timeout_positive(timeout: u64, field_name: &str) -> ValidationResult<()> {
        if timeout == 0 {
            return Err(ErrorInfo::validation(
                format!("{field_name} must be greater than 0"),
                Some(field_name),
            ));
        }
        Ok(())
    }

    /// Validates an optional timeout value if provided.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Optional timeout value in seconds
    /// * `field_name` - The name of the field for error reporting
    ///
    /// # Returns
    ///
    /// * `Ok(())` if no timeout is provided or if it's positive
    /// * `Err(ErrorInfo)` if the timeout is zero
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid: no timeout
    /// assert!(validators::optional_timeout(None, "timeoutSecs").is_ok());
    ///
    /// // Valid: positive timeout
    /// assert!(validators::optional_timeout(Some(30), "timeoutSecs").is_ok());
    ///
    /// // Invalid: zero timeout
    /// assert!(validators::optional_timeout(Some(0), "timeoutSecs").is_err());
    /// ```
    pub fn optional_timeout(timeout: Option<u64>, field_name: &str) -> ValidationResult<()> {
        if let Some(t) = timeout {
            timeout_positive(t, field_name)?;
        }
        Ok(())
    }

    /// Validates a prerelease tag.
    ///
    /// Valid tags contain only ASCII alphanumerics and hyphens `[0-9A-Za-z-]`,
    /// as per SemVer 2.0.0 specification.
    ///
    /// # Arguments
    ///
    /// * `tag` - The prerelease tag to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the tag is valid
    /// * `Err(ErrorInfo)` if the tag is empty or contains invalid characters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid prerelease tags
    /// assert!(validators::prerelease_tag("alpha").is_ok());
    /// assert!(validators::prerelease_tag("beta").is_ok());
    /// assert!(validators::prerelease_tag("rc").is_ok());
    /// assert!(validators::prerelease_tag("beta-1").is_ok());
    /// assert!(validators::prerelease_tag("RC1").is_ok());
    ///
    /// // Invalid prerelease tags
    /// assert!(validators::prerelease_tag("").is_err());
    /// assert!(validators::prerelease_tag("alpha.1").is_err());  // Contains period
    /// assert!(validators::prerelease_tag("beta_1").is_err());   // Contains underscore
    /// ```
    pub fn prerelease_tag(tag: &str) -> ValidationResult<()> {
        if tag.is_empty() {
            return Err(ErrorInfo::validation(
                "prerelease tag cannot be empty",
                Some("prerelease"),
            ));
        }

        // Check for valid characters (SemVer 2.0.0 spec: [0-9A-Za-z-])
        if !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ErrorInfo::validation(
                format!(
                    "prerelease tag '{tag}' contains invalid characters. \
                     Must contain only ASCII alphanumerics and hyphens [0-9A-Za-z-]"
                ),
                Some("prerelease"),
            ));
        }

        Ok(())
    }

    /// Valid snapshot format template variables.
    ///
    /// These are the variables that can be used in snapshot format templates:
    /// - `{version}`: Current package version
    /// - `{branch}`: Current Git branch name (sanitized)
    /// - `{short_commit}`: Short Git commit hash (7 characters)
    /// - `{commit}`: Full Git commit hash
    /// - `{timestamp}`: Unix timestamp
    pub const VALID_SNAPSHOT_VARIABLES: &[&str] =
        &["{version}", "{branch}", "{short_commit}", "{commit}", "{timestamp}"];

    /// Validates a snapshot format template.
    ///
    /// A valid snapshot format must contain at least one valid template variable.
    ///
    /// # Valid Variables
    ///
    /// - `{version}`: Current package version
    /// - `{branch}`: Current Git branch name (sanitized)
    /// - `{short_commit}`: Short Git commit hash (7 characters)
    /// - `{commit}`: Full Git commit hash
    /// - `{timestamp}`: Unix timestamp
    ///
    /// # Arguments
    ///
    /// * `format` - The snapshot format template to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the format is valid
    /// * `Err(ErrorInfo)` if the format is empty or contains no valid variables
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::validation::validators;
    ///
    /// // Valid snapshot formats
    /// assert!(validators::snapshot_format("{version}-snapshot.{short_commit}").is_ok());
    /// assert!(validators::snapshot_format("{version}-{branch}.{commit}").is_ok());
    /// assert!(validators::snapshot_format("{version}-dev.{timestamp}").is_ok());
    ///
    /// // Invalid snapshot formats
    /// assert!(validators::snapshot_format("").is_err());
    /// assert!(validators::snapshot_format("no-variables-here").is_err());
    /// assert!(validators::snapshot_format("{invalid}").is_err());
    /// ```
    pub fn snapshot_format(format: &str) -> ValidationResult<()> {
        if format.is_empty() {
            return Err(ErrorInfo::validation("snapshot format cannot be empty", Some("format")));
        }

        // Check for at least one valid variable
        let has_valid_var = VALID_SNAPSHOT_VARIABLES.iter().any(|v| format.contains(v));

        if !has_valid_var {
            return Err(ErrorInfo::validation(
                format!(
                    "snapshot format '{}' must contain at least one valid variable: {}",
                    format,
                    VALID_SNAPSHOT_VARIABLES.join(", ")
                ),
                Some("format"),
            ));
        }

        Ok(())
    }
}
