//! Tests for the interactive module.
//!
//! This module contains comprehensive tests for all interactive prompt functionality
//! including theme, validation, selection, and confirmation prompts.
//!
//! # What
//!
//! Tests cover:
//! - Theme creation and styling
//! - Input validation functions
//! - Fuzzy search and filtering
//! - Selection logic (empty lists, defaults, etc.)
//! - Confirmation dialog logic
//! - Interactive prompt functions
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
//! - Validation logic with various inputs
//! - Fuzzy matching algorithms
//! - Edge case handling
//! - Theme configuration
//!
//! # Why
//!
//! While interactive prompts can't be fully tested automatically, we can
//! test the validation and error handling logic, fuzzy matching, and
//! provide manual tests for interactive flows.

#[cfg(test)]
mod theme_tests {
    use crate::interactive::theme::WntTheme;

    #[test]
    fn test_theme_creation_with_color() {
        let theme = WntTheme::new(false);
        assert!(!theme.is_no_color());
    }

    #[test]
    fn test_theme_creation_no_color() {
        let theme = WntTheme::new(true);
        assert!(theme.is_no_color());
    }

    #[test]
    fn test_theme_default() {
        let theme = WntTheme::default();
        // Should respect NO_COLOR environment variable
        let expected_no_color = std::env::var("NO_COLOR").is_ok();
        assert_eq!(theme.is_no_color(), expected_no_color);
    }

    #[test]
    fn test_theme_cloneable() {
        let theme = WntTheme::new(false);
        let _cloned = theme.clone();
        // If this compiles, Clone is implemented correctly
    }
}

#[cfg(test)]
mod validation_tests {
    use crate::error::CliError;
    use crate::interactive::validation::*;

    #[test]
    fn test_validate_non_empty_valid() {
        assert!(validate_non_empty("valid input").is_ok());
        assert!(validate_non_empty("  valid with spaces  ").is_ok());
        assert!(validate_non_empty("a").is_ok());
    }

    #[test]
    fn test_validate_non_empty_invalid() {
        assert!(validate_non_empty("").is_err());
        assert!(validate_non_empty("   ").is_err());
        assert!(validate_non_empty("\t\n").is_err());
        assert!(validate_non_empty("\t  \n  ").is_err());
    }

    #[test]
    fn test_validate_at_least_one_selected_valid() {
        assert!(validate_at_least_one_selected(&[0]).is_ok());
        assert!(validate_at_least_one_selected(&[0, 1, 2]).is_ok());
        assert!(validate_at_least_one_selected(&[5, 10, 15]).is_ok());
    }

    #[test]
    fn test_validate_at_least_one_selected_invalid() {
        assert!(validate_at_least_one_selected(&[]).is_err());
    }

    #[test]
    fn test_validate_package_names_valid() {
        let available = vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-c".to_string()];

        assert!(validate_package_names(&["pkg-a"], &available).is_ok());
        assert!(validate_package_names(&["pkg-a", "pkg-b"], &available).is_ok());
        assert!(validate_package_names(&["pkg-a", "pkg-b", "pkg-c"], &available).is_ok());
    }

    #[test]
    fn test_validate_package_names_invalid() {
        let available = vec!["pkg-a".to_string(), "pkg-b".to_string()];

        let result = validate_package_names(&["pkg-c"], &available);
        assert!(result.is_err());

        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("Unknown package(s): pkg-c"));
            assert!(msg.contains("Available packages:"));
        }
    }

    #[test]
    fn test_validate_package_names_with_suggestions() {
        let available =
            vec!["package-a".to_string(), "package-b".to_string(), "package-core".to_string()];

        let result = validate_package_names(&["pakage-a"], &available);
        assert!(result.is_err());

        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("Unknown package"));
            // Should suggest similar package
            assert!(msg.contains("Did you mean:") || msg.contains("Available packages:"));
        }
    }

    #[test]
    fn test_validate_environment_names_valid() {
        let available = vec!["dev".to_string(), "staging".to_string(), "production".to_string()];

        assert!(validate_environment_names(&["dev"], &available).is_ok());
        assert!(validate_environment_names(&["dev", "staging"], &available).is_ok());
    }

    #[test]
    fn test_validate_environment_names_invalid() {
        let available = vec!["dev".to_string(), "staging".to_string()];

        let result = validate_environment_names(&["prod"], &available);
        assert!(result.is_err());

        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("Unknown environment(s): prod"));
            assert!(msg.contains("Available environments:"));
        }
    }

    #[test]
    fn test_validate_bump_type_valid() {
        assert!(validate_bump_type("patch").is_ok());
        assert!(validate_bump_type("minor").is_ok());
        assert!(validate_bump_type("major").is_ok());

        // Case insensitive
        assert!(validate_bump_type("PATCH").is_ok());
        assert!(validate_bump_type("Minor").is_ok());
        assert!(validate_bump_type("MAJOR").is_ok());
    }

    #[test]
    fn test_validate_bump_type_invalid() {
        assert!(validate_bump_type("invalid").is_err());
        assert!(validate_bump_type("fix").is_err());
        assert!(validate_bump_type("feature").is_err());
    }

    #[test]
    fn test_validate_bump_type_with_suggestions() {
        // Test patch suggestions
        let result = validate_bump_type("p");
        assert!(result.is_err());
        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("Did you mean 'patch'?"));
        }

        // Test minor suggestions
        let result = validate_bump_type("m");
        assert!(result.is_err());
        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("Did you mean 'minor'?"));
        }

        // Test major suggestions
        let result = validate_bump_type("maj");
        assert!(result.is_err());
        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("Did you mean 'major'?"));
        }
    }
}

#[cfg(test)]
mod select_tests {
    use crate::error::CliError;
    use crate::interactive::select::*;

    #[test]
    fn test_fuzzy_filter_empty_query() {
        let items = vec!["package-a", "package-b", "package-c"];
        let results = fuzzy_filter(&items, "");

        // Empty query returns all items
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_fuzzy_filter_with_match() {
        let items = vec!["package-a", "package-b", "package-core", "utils"];
        let results = fuzzy_filter(&items, "pkg");

        assert!(!results.is_empty());

        // Results should be sorted by score (highest first)
        if results.len() > 1 {
            assert!(results[0].1 >= results[1].1);
        }
    }

    #[test]
    fn test_fuzzy_filter_exact_match() {
        let items = vec!["package-a", "package-b", "package-c"];
        let results = fuzzy_filter(&items, "package-b");

        // Exact match should be first
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 1); // Index of "package-b"
    }

    #[test]
    fn test_fuzzy_filter_partial_match() {
        let items = vec!["my-awesome-package", "my-other-package", "different"];
        let results = fuzzy_filter(&items, "awe");

        // Should match "my-awesome-package"
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0);
    }

    #[test]
    fn test_fuzzy_filter_case_insensitive() {
        let items = vec!["Package-A", "Package-B", "PACKAGE-C"];
        let results = fuzzy_filter(&items, "package");

        // Should match all regardless of case
        assert_eq!(results.len(), 3);
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_simple_select_empty_list() {
        let items: Vec<String> = vec![];
        let result = simple_select("Choose", &items, None, true);

        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_fuzzy_select_empty_list() {
        let items: Vec<String> = vec![];
        let result = fuzzy_select("Choose", &items, None, true);

        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_fuzzy_multi_select_empty_list() {
        let items: Vec<String> = vec![];
        let defaults: Vec<usize> = vec![];
        let result = fuzzy_multi_select("Choose", &items, &defaults, true);

        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_select_packages_empty_list() {
        let items: Vec<String> = vec![];
        let detected: Vec<String> = vec![];
        let result = select_packages("Choose", &items, &detected, true);

        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_select_environments_empty_list() {
        let items: Vec<String> = vec![];
        let defaults: Vec<String> = vec![];
        let result = select_environments("Choose", &items, &defaults, true);

        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected
        } else {
            panic!("Expected Validation error");
        }
    }
}

#[cfg(test)]
mod prompts_tests {
    use crate::error::CliError;
    use crate::interactive::prompts::*;

    // Manual interaction tests - marked as ignored
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
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_prompt_confirm_with_context_manual() {
        let result =
            prompt_confirm_with_context("Apply changes?", "This will modify 3 files", false, true);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_prompt_confirm_dangerous_manual() {
        let result =
            prompt_confirm_dangerous("Delete all data?", "This action cannot be undone", true);
        assert!(result.is_ok());
    }

    // Unit tests for error cases
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

#[cfg(test)]
mod confirm_tests {
    use crate::interactive::confirm::*;

    // Note: Most tests require terminal interaction and are marked as ignored.
    // They serve as documentation and can be run manually.

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_confirm_manual() {
        let result = confirm("Proceed?", true, true);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_confirm_with_context_manual() {
        let result =
            confirm_with_context("Apply changes?", "This will modify 3 files", false, true);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_confirm_dangerous_manual() {
        let result = confirm_dangerous("Delete all data?", "This action cannot be undone", true);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_confirm_with_items_manual() {
        let items = vec!["item1", "item2", "item3"];
        let result = confirm_with_items("process", &items, true, true);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_confirm_with_details_manual() {
        let details = vec!["Detail 1".to_string(), "Detail 2".to_string(), "Detail 3".to_string()];
        let result = confirm_with_details("Operation", &details, false, true);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    #[allow(clippy::unwrap_used)]
    fn test_confirm_with_many_items_manual() {
        let items: Vec<String> = (1..=15).map(|i| format!("item-{i}")).collect();
        let result = confirm_with_items("process", &items, true, true);
        assert!(result.is_ok());
        // Should only show first 10 items
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::interactive::*;

    #[test]
    fn test_module_exports() {
        // Verify that all expected exports are available

        // Theme
        let _theme = WntTheme::new(true);

        // Validation functions
        let _result = validate_non_empty("test");
        let _result = validate_at_least_one_selected(&[0, 1]);
        let _result = validate_bump_type("patch");

        // Select functions
        let items = vec!["a", "b", "c"];
        let _result = fuzzy_filter(&items, "a");

        // All function exports are accessible
    }

    #[test]
    fn test_fuzzy_search_workflow() {
        // Test a complete fuzzy search workflow
        let packages = vec!["workspace-tools", "workspace-config", "build-system", "test-utils"];

        // Search for "work"
        let results = fuzzy_filter(&packages, "work");
        assert!(!results.is_empty());

        // First two results should be workspace-related
        assert!(results.len() >= 2);
        assert!(packages[results[0].0].contains("workspace"));
    }

    #[test]
    fn test_validation_workflow() {
        // Test validation workflow
        let packages = vec!["pkg-a".to_string(), "pkg-b".to_string()];

        // Valid package name
        assert!(validate_package_names(&["pkg-a"], &packages).is_ok());

        // Invalid package name with suggestion
        let result = validate_package_names(&["pkga"], &packages);
        assert!(result.is_err());
    }

    #[test]
    fn test_theme_consistency() {
        // Test that theme respects no_color setting
        let theme_color = WntTheme::new(false);
        let theme_no_color = WntTheme::new(true);

        assert!(!theme_color.is_no_color());
        assert!(theme_no_color.is_no_color());
    }
}
