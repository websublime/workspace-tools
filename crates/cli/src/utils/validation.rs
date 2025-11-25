//! Validation utilities for CLI arguments.
//!
//! This module provides validation functions for common CLI argument types
//! such as URLs, paths, and identifiers.
//!
//! # What
//!
//! Contains validation functions for:
//! - Registry URLs (NPM registry validation)
//! - Other argument validations as needed
//!
//! # How
//!
//! Each validation function takes a string input and returns either:
//! - `Ok(normalized_value)` - The validated and normalized input
//! - `Err(CliError)` - A descriptive validation error
//!
//! # Why
//!
//! Centralizing validation provides:
//! - Consistent error messages across commands
//! - Reusable validation logic
//! - Early failure with clear user feedback
//! - Normalized values for consistent processing
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_cli_tools::utils::validation::validate_registry_url;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let url = validate_registry_url("https://registry.npmjs.org/")?;
//! assert_eq!(url, "https://registry.npmjs.org");
//! # Ok(())
//! # }
//! ```

use url::Url;

use crate::error::CliError;
use crate::error::Result;

/// Validates and normalizes a registry URL.
///
/// Ensures the provided URL is a valid HTTP or HTTPS URL suitable for use
/// as an NPM registry endpoint.
///
/// # What
///
/// Validates that:
/// - URL is syntactically valid
/// - URL uses HTTP or HTTPS scheme
/// - URL has a valid host
///
/// Also normalizes the URL by removing trailing slashes.
///
/// # How
///
/// 1. Parses the URL using the `url` crate
/// 2. Validates the scheme is `http` or `https`
/// 3. Validates the host is present
/// 4. Removes trailing slashes for consistency
/// 5. Returns the normalized URL string
///
/// # Why
///
/// Ensures registry URLs are valid before attempting to use them, preventing
/// obscure HTTP errors and providing clear feedback to users.
///
/// # Arguments
///
/// * `url_str` - Registry URL to validate
///
/// # Returns
///
/// Normalized registry URL string.
///
/// # Errors
///
/// Returns error if:
/// - URL is not a valid HTTP/HTTPS URL
/// - URL contains invalid characters
/// - URL scheme is not http or https
/// - URL does not have a valid host
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::utils::validation::validate_registry_url;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Valid URLs are normalized
/// let url = validate_registry_url("https://custom.com/")?;
/// assert_eq!(url, "https://custom.com");
///
/// let url = validate_registry_url("https://registry.npmjs.org")?;
/// assert_eq!(url, "https://registry.npmjs.org");
///
/// // Local registries are also valid
/// let url = validate_registry_url("http://localhost:4873")?;
/// assert_eq!(url, "http://localhost:4873");
///
/// // Invalid URLs return errors
/// assert!(validate_registry_url("not-a-url").is_err());
/// assert!(validate_registry_url("ftp://invalid.com").is_err());
/// # Ok(())
/// # }
/// ```
pub(crate) fn validate_registry_url(url_str: &str) -> Result<String> {
    // Parse URL
    let parsed = Url::parse(url_str)
        .map_err(|e| CliError::validation(format!("Invalid registry URL '{url_str}': {e}")))?;

    // Validate scheme
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(CliError::validation(format!(
                "Registry URL must use HTTP or HTTPS scheme, found: {scheme}"
            )));
        }
    }

    // Validate host exists
    if parsed.host_str().is_none() {
        return Err(CliError::validation(format!(
            "Registry URL must have a valid host: {url_str}"
        )));
    }

    // Remove trailing slash for consistency
    let normalized = url_str.trim_end_matches('/').to_string();

    Ok(normalized)
}

/// Validates an optional registry URL.
///
/// Convenience wrapper for [`validate_registry_url`] that handles `Option<String>`.
/// If the input is `None`, returns `None`. If `Some`, validates and returns
/// the normalized URL.
///
/// # Arguments
///
/// * `registry` - Optional registry URL to validate
///
/// # Returns
///
/// * `Ok(None)` - If input was `None`
/// * `Ok(Some(normalized_url))` - If input was valid
/// * `Err(CliError)` - If validation failed
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::utils::validation::validate_optional_registry_url;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // None passes through
/// assert_eq!(validate_optional_registry_url(None)?, None);
///
/// // Valid URLs are normalized
/// let url = validate_optional_registry_url(Some("https://custom.com/".to_string()))?;
/// assert_eq!(url, Some("https://custom.com".to_string()));
/// # Ok(())
/// # }
/// ```
pub(crate) fn validate_optional_registry_url(registry: Option<String>) -> Result<Option<String>> {
    registry.map(|url| validate_registry_url(&url)).transpose()
}
