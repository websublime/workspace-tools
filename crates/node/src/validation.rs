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
//! - Legacy validators for backward compatibility
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
//! let error = ValidationError::invalid("bumpType", "must be major, minor, or patch", Some("invalid"));
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
//! // Validate bump type
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
///     "must be one of: major, minor, patch",
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

#[allow(dead_code)]
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
/// - `bump_type`: Validates that a bump type is valid (major, minor, patch, etc.)
/// - `timeout`: Validates that a timeout is within specified bounds
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
    use super::{Path, ValidationError};

    /// Validates that a path exists on the file system.
    ///
    /// This validator checks if the given path exists, regardless of whether
    /// it's a file or directory. For directory-specific validation, use
    /// the `validate_root` function instead.
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
    /// Valid bump types are: `major`, `minor`, `patch`, `premajor`, `preminor`,
    /// `prepatch`, and `prerelease`.
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
    /// assert!(validators::bump_type("prerelease").is_ok());
    ///
    /// // Invalid bump type
    /// let result = validators::bump_type("invalid");
    /// assert!(result.is_err());
    /// let error = result.unwrap_err();
    /// assert_eq!(error.value, Some("invalid".to_string()));
    /// ```
    pub fn bump_type(value: &str) -> Result<(), ValidationError> {
        const VALID_BUMP_TYPES: &[&str] =
            &["major", "minor", "patch", "premajor", "preminor", "prepatch", "prerelease"];

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
}

// ============================================================================
// Legacy validators - Maintained for backward compatibility
// These validators return Result<(), ErrorInfo> directly and are used by
// existing code. New code should prefer the validators module above.
// ============================================================================

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

/// Validates that a root path is provided and exists.
///
/// This function checks that the root path:
/// 1. Is not empty
/// 2. Points to an existing directory
///
/// # Arguments
///
/// * `root` - The root path to validate
///
/// # Returns
///
/// * `Ok(())` if the path is valid
/// * `Err(ErrorInfo)` if the path is empty or doesn't exist
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::validation::validate_root;
///
/// // Valid path
/// let result = validate_root("/valid/path");
///
/// // Invalid path (empty)
/// let result = validate_root("");
/// assert!(result.is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_root(root: &str) -> ValidationResult<()> {
    if root.is_empty() {
        return Err(ErrorInfo::validation("root path cannot be empty", Some("root")));
    }

    let path = Path::new(root);
    if !path.exists() {
        return Err(ErrorInfo::not_found(format!("root path does not exist: {root}"), Some(root)));
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
/// use sublime_node_tools::validation::validate_packages_not_empty;
///
/// // Valid packages
/// let result = validate_packages_not_empty(&["@scope/pkg1".to_string()]);
/// assert!(result.is_ok());
///
/// // Invalid packages (empty)
/// let result = validate_packages_not_empty(&[]);
/// assert!(result.is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_packages_not_empty(packages: &[String]) -> ValidationResult<()> {
    if packages.is_empty() {
        return Err(ErrorInfo::validation("packages array cannot be empty", Some("packages")));
    }
    Ok(())
}

/// Validates that a bump type is valid.
///
/// Valid bump types are: "major", "minor", "patch", "premajor", "preminor",
/// "prepatch", "prerelease".
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
/// use sublime_node_tools::validation::validate_bump_type;
///
/// // Valid bump types
/// assert!(validate_bump_type("major").is_ok());
/// assert!(validate_bump_type("minor").is_ok());
/// assert!(validate_bump_type("patch").is_ok());
///
/// // Invalid bump type
/// assert!(validate_bump_type("invalid").is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_bump_type(bump_type: &str) -> ValidationResult<()> {
    const VALID_BUMP_TYPES: &[&str] =
        &["major", "minor", "patch", "premajor", "preminor", "prepatch", "prerelease"];

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
/// use sublime_node_tools::validation::validate_message_not_empty;
///
/// // Valid message
/// let result = validate_message_not_empty("Add new feature", "message");
/// assert!(result.is_ok());
///
/// // Invalid message (empty)
/// let result = validate_message_not_empty("", "message");
/// assert!(result.is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_message_not_empty(message: &str, field_name: &str) -> ValidationResult<()> {
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
/// use sublime_node_tools::validation::validate_semver;
///
/// // Valid versions
/// assert!(validate_semver("1.0.0", "version").is_ok());
/// assert!(validate_semver("2.3.4-beta.1", "version").is_ok());
///
/// // Invalid versions
/// assert!(validate_semver("invalid", "version").is_err());
/// assert!(validate_semver("1.0", "version").is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_semver(version: &str, field_name: &str) -> ValidationResult<()> {
    // Basic semver pattern check (simplified)
    // Full implementation will use proper semver parsing
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
/// use sublime_node_tools::validation::validate_mutual_exclusion;
///
/// // Valid: only one is set
/// let result = validate_mutual_exclusion(&[
///     ("filterPackage", true),
///     ("affected", false),
/// ]);
/// assert!(result.is_ok());
///
/// // Invalid: both are set
/// let result = validate_mutual_exclusion(&[
///     ("filterPackage", true),
///     ("affected", true),
/// ]);
/// assert!(result.is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_mutual_exclusion(params: &[(&str, bool)]) -> ValidationResult<()> {
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

/// Validates that a timeout value is positive.
///
/// # Arguments
///
/// * `timeout` - The timeout value in seconds
/// * `field_name` - The name of the field for error reporting
///
/// # Returns
///
/// * `Ok(())` if the timeout is positive
/// * `Err(ErrorInfo)` if the timeout is zero or negative
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::validation::validate_timeout;
///
/// // Valid timeout
/// assert!(validate_timeout(30, "timeoutSecs").is_ok());
///
/// // Invalid timeout
/// assert!(validate_timeout(0, "timeoutSecs").is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_timeout(timeout: u64, field_name: &str) -> ValidationResult<()> {
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
/// use sublime_node_tools::validation::validate_optional_timeout;
///
/// // Valid: no timeout
/// assert!(validate_optional_timeout(None, "timeoutSecs").is_ok());
///
/// // Valid: positive timeout
/// assert!(validate_optional_timeout(Some(30), "timeoutSecs").is_ok());
///
/// // Invalid: zero timeout
/// assert!(validate_optional_timeout(Some(0), "timeoutSecs").is_err());
/// ```
#[allow(dead_code)]
pub(crate) fn validate_optional_timeout(
    timeout: Option<u64>,
    field_name: &str,
) -> ValidationResult<()> {
    if let Some(t) = timeout {
        validate_timeout(t, field_name)?;
    }
    Ok(())
}
