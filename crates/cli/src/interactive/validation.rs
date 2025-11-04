//! Validation utilities for interactive prompts.
//!
//! This module provides enhanced validation logic for user input with
//! real-time feedback, helpful error messages, and suggestions.
//!
//! # What
//!
//! Provides:
//! - Input validation functions with detailed error messages
//! - Suggestions for common mistakes
//! - Real-time validation feedback
//! - Reusable validation logic for different prompt types
//!
//! # How
//!
//! Implements validation functions that:
//! - Check input against specific criteria
//! - Return detailed error messages with suggestions
//! - Provide context-aware help
//! - Support custom validation rules
//!
//! The validation functions can be used with dialoguer's `validate_with`
//! method to provide real-time feedback during user input.
//!
//! # Why
//!
//! Centralized validation ensures:
//! - Consistent validation logic across all prompts
//! - Better user experience with helpful error messages
//! - Reduced user frustration through clear guidance
//! - Maintainable validation rules
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::interactive::validation::{validate_non_empty, validate_package_names};
//!
//! // Validate non-empty input
//! let result = validate_non_empty("  ");
//! assert!(result.is_err());
//!
//! let result = validate_non_empty("valid input");
//! assert!(result.is_ok());
//!
//! // Validate package names
//! let packages = vec!["pkg-a".to_string(), "pkg-b".to_string()];
//! let result = validate_package_names(&["pkg-a", "pkg-c"], &packages);
//! assert!(result.is_err());
//! ```

use crate::error::{CliError, Result};

/// Validates that input is not empty or whitespace-only.
///
/// # Arguments
///
/// * `input` - The input string to validate
///
/// # Returns
///
/// * `Result<()>` - Ok if input is non-empty, Err with helpful message otherwise
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::interactive::validation::validate_non_empty;
///
/// assert!(validate_non_empty("valid input").is_ok());
/// assert!(validate_non_empty("   ").is_err());
/// assert!(validate_non_empty("").is_err());
/// ```
pub fn validate_non_empty(input: &str) -> Result<()> {
    if input.trim().is_empty() {
        Err(CliError::validation("Input cannot be empty. Please provide a value."))
    } else {
        Ok(())
    }
}

/// Validates that at least one item is selected.
///
/// # Arguments
///
/// * `selections` - The list of selected items
///
/// # Returns
///
/// * `Result<()>` - Ok if at least one item is selected, Err otherwise
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::interactive::validation::validate_at_least_one_selected;
///
/// assert!(validate_at_least_one_selected(&[0, 1]).is_ok());
/// assert!(validate_at_least_one_selected(&[]).is_err());
/// ```
pub fn validate_at_least_one_selected(selections: &[usize]) -> Result<()> {
    if selections.is_empty() {
        Err(CliError::validation(
            "At least one item must be selected. Use Space to select items, then press Enter.",
        ))
    } else {
        Ok(())
    }
}

/// Validates that package names exist in the available packages list.
///
/// Provides helpful suggestions if a package name is not found.
///
/// # Arguments
///
/// * `selected` - The selected package names
/// * `available` - The list of available package names
///
/// # Returns
///
/// * `Result<()>` - Ok if all packages exist, Err with suggestions otherwise
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::interactive::validation::validate_package_names;
///
/// let available = vec!["pkg-a".to_string(), "pkg-b".to_string()];
/// assert!(validate_package_names(&["pkg-a"], &available).is_ok());
/// assert!(validate_package_names(&["pkg-c"], &available).is_err());
/// ```
pub fn validate_package_names(selected: &[&str], available: &[String]) -> Result<()> {
    let invalid: Vec<&str> = selected
        .iter()
        .filter(|&&name| !available.iter().any(|pkg| pkg == name))
        .copied()
        .collect();

    if invalid.is_empty() {
        Ok(())
    } else {
        let suggestions = suggest_similar_packages(&invalid, available);
        let mut error_msg = format!("Unknown package(s): {}", invalid.join(", "));

        if !suggestions.is_empty() {
            error_msg.push_str("\n\nDid you mean:");
            for (wrong, suggestion) in &suggestions {
                error_msg.push_str("\n  ");
                error_msg.push_str(wrong);
                error_msg.push_str(" -> ");
                error_msg.push_str(suggestion);
            }
        }

        error_msg.push_str("\n\nAvailable packages:");
        for pkg in available {
            error_msg.push_str("\n  - ");
            error_msg.push_str(pkg);
        }

        Err(CliError::validation(error_msg))
    }
}

/// Validates that environment names exist in the available environments list.
///
/// Provides helpful suggestions if an environment name is not found.
///
/// # Arguments
///
/// * `selected` - The selected environment names
/// * `available` - The list of available environment names
///
/// # Returns
///
/// * `Result<()>` - Ok if all environments exist, Err with suggestions otherwise
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::interactive::validation::validate_environment_names;
///
/// let available = vec!["dev".to_string(), "staging".to_string()];
/// assert!(validate_environment_names(&["dev"], &available).is_ok());
/// assert!(validate_environment_names(&["prod"], &available).is_err());
/// ```
pub fn validate_environment_names(selected: &[&str], available: &[String]) -> Result<()> {
    let invalid: Vec<&str> = selected
        .iter()
        .filter(|&&name| !available.iter().any(|env| env == name))
        .copied()
        .collect();

    if invalid.is_empty() {
        Ok(())
    } else {
        let suggestions = suggest_similar_packages(&invalid, available);
        let mut error_msg = format!("Unknown environment(s): {}", invalid.join(", "));

        if !suggestions.is_empty() {
            error_msg.push_str("\n\nDid you mean:");
            for (wrong, suggestion) in &suggestions {
                error_msg.push_str("\n  ");
                error_msg.push_str(wrong);
                error_msg.push_str(" -> ");
                error_msg.push_str(suggestion);
            }
        }

        error_msg.push_str("\n\nAvailable environments:");
        for env in available {
            error_msg.push_str("\n  - ");
            error_msg.push_str(env);
        }

        Err(CliError::validation(error_msg))
    }
}

/// Validates that a bump type is valid.
///
/// # Arguments
///
/// * `bump_type` - The bump type to validate
///
/// # Returns
///
/// * `Result<()>` - Ok if bump type is valid, Err with suggestions otherwise
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::interactive::validation::validate_bump_type;
///
/// assert!(validate_bump_type("patch").is_ok());
/// assert!(validate_bump_type("minor").is_ok());
/// assert!(validate_bump_type("major").is_ok());
/// assert!(validate_bump_type("invalid").is_err());
/// ```
pub fn validate_bump_type(bump_type: &str) -> Result<()> {
    match bump_type.to_lowercase().as_str() {
        "patch" | "minor" | "major" => Ok(()),
        _ => {
            let mut error_msg = format!("Invalid bump type: '{bump_type}'");

            // Provide suggestions for common typos
            let suggestion = match bump_type.to_lowercase().as_str() {
                "p" | "pat" | "ptch" => Some("patch"),
                "m" | "min" | "mnor" => Some("minor"),
                "maj" | "mjor" => Some("major"),
                _ => None,
            };

            if let Some(sugg) = suggestion {
                error_msg.push_str("\n\nDid you mean '");
                error_msg.push_str(sugg);
                error_msg.push_str("'?");
            }

            error_msg.push_str("\n\nValid bump types:");
            error_msg.push_str("\n  - patch: Bug fixes and small changes (0.0.X)");
            error_msg.push_str("\n  - minor: New features, backwards compatible (0.X.0)");
            error_msg.push_str("\n  - major: Breaking changes (X.0.0)");

            Err(CliError::validation(error_msg))
        }
    }
}

/// Suggests similar package or environment names using fuzzy matching.
///
/// # Arguments
///
/// * `invalid` - The invalid names
/// * `available` - The list of available names
///
/// # Returns
///
/// A vector of tuples containing (invalid_name, suggested_name)
fn suggest_similar_packages(invalid: &[&str], available: &[String]) -> Vec<(String, String)> {
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    let matcher = SkimMatcherV2::default();
    let mut suggestions = Vec::new();

    for &invalid_name in invalid {
        let mut matches: Vec<(i64, &String)> = available
            .iter()
            .filter_map(|pkg| matcher.fuzzy_match(pkg, invalid_name).map(|score| (score, pkg)))
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.0.cmp(&a.0));

        // Take the best match if score is reasonable (> 0)
        if let Some((score, suggestion)) = matches.first()
            && *score > 0
        {
            suggestions.push((invalid_name.to_string(), (*suggestion).clone()));
        }
    }

    suggestions
}
