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
//! - `ValidationError`: Structured validation error type
//! - `ValidationResult<T>`: Type alias for validation results
//! - `Validator`: Trait for implementing custom validators
//! - Common validators: `validate_root`, `validate_packages`, `validate_bump_type`, etc.
//!
//! Validation follows these principles:
//! 1. Fail fast: Return on first validation error
//! 2. Clear messages: Error messages should indicate the exact problem and field
//! 3. Type safety: Leverage Rust's type system for compile-time validation where possible
//!
//! # Why
//!
//! Validating parameters at the NAPI layer provides several benefits:
//! - Early error detection before CLI execution
//! - Clear, user-friendly error messages with field context
//! - Consistent validation across all NAPI functions
//! - Prevention of unnecessary work when parameters are invalid
//!
//! # Examples
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
