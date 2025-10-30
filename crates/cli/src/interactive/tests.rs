//! Tests for the interactive module.
//!
//! This module contains tests for interactive prompt functionality.
//!
//! # What
//!
//! Tests cover:
//! - Interactive prompt functions
//! - Input validation
//! - Error handling for cancelled prompts
//! - Edge cases (empty lists, invalid inputs)
//!
//! # How
//!
//! Most interactive tests are marked as `#[ignore]` because they require
//! terminal interaction and cannot be run automatically. They serve as
//! documentation and can be run manually for testing.
//!
//! Unit tests verify:
//! - Error handling for invalid inputs
//! - Validation logic
//! - Edge case handling
//!
//! # Why
//!
//! While interactive prompts can't be fully tested automatically, we can
//! test the validation and error handling logic, and provide manual tests
//! for interactive flows.

#[cfg(test)]
mod tests {
    use super::super::prompts::{
        prompt_bump_type, prompt_confirm, prompt_environments, prompt_packages, prompt_summary,
    };
    use crate::error::CliError;

    // Note: Most interactive prompt tests cannot be run automatically as they require
    // terminal interaction. They are provided as documentation and for manual testing.

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_prompt_bump_type_manual() {
        let result = prompt_bump_type(true);
        assert!(result.is_ok());
        let bump = result.unwrap();
        assert!(["patch", "minor", "major"].contains(&bump.as_str()));
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_prompt_packages_manual() {
        let packages =
            vec!["package-a".to_string(), "package-b".to_string(), "package-c".to_string()];
        let detected = vec!["package-a".to_string()];
        let result = prompt_packages(&packages, &detected, true);
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert!(!selected.is_empty());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_prompt_environments_manual() {
        let environments = vec!["dev".to_string(), "staging".to_string(), "production".to_string()];
        let defaults = vec!["staging".to_string(), "production".to_string()];
        let result = prompt_environments(&environments, &defaults, true);
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert!(!selected.is_empty());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_prompt_summary_manual() {
        let result = prompt_summary(None, true);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert!(!summary.trim().is_empty());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_prompt_confirm_manual() {
        let result = prompt_confirm("Proceed?", true, true);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_prompt_packages_empty_list() {
        let packages: Vec<String> = vec![];
        let detected: Vec<String> = vec![];
        let result = prompt_packages(&packages, &detected, true);
        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected error type
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_prompt_environments_empty_list() {
        let environments: Vec<String> = vec![];
        let defaults: Vec<String> = vec![];
        let result = prompt_environments(&environments, &defaults, true);
        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected error type
        } else {
            panic!("Expected Validation error");
        }
    }
}
