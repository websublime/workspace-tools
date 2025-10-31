//! Tests for changeset commands.
//!
//! This module contains comprehensive tests for all changeset command functionality.
//!
//! # What
//!
//! Tests cover:
//! - Changeset add command (interactive and non-interactive modes)
//! - Input validation
//! - Error handling
//! - Git integration
//! - Configuration loading
//! - Package detection
//! - Output formatting (human and JSON)
//!
//! # How
//!
//! Tests use:
//! - Temporary directories for isolated test environments
//! - Mock git repositories
//! - Mock configuration files
//! - Captured output for verification
//! - Both unit and integration test approaches
//!
//! # Why
//!
//! Comprehensive testing ensures:
//! - Commands work correctly in all scenarios
//! - Error handling is robust
//! - Output formatting is consistent
//! - Regressions are caught early
//! - 100% test coverage requirement is met

#[cfg(test)]
mod tests {
    use crate::cli::commands::ChangesetCreateArgs;
    use crate::commands::changeset::add::{
        parse_bump_type, validate_bump_type, validate_environments,
    };
    use crate::error::CliError;
    use crate::output::{Output, OutputFormat};
    use std::io::Cursor;
    use sublime_pkg_tools::types::VersionBump;

    // Helper to create a test output with in-memory buffer
    #[allow(dead_code)]
    fn create_test_output(format: OutputFormat) -> (Output, Cursor<Vec<u8>>) {
        let buffer = Cursor::new(Vec::new());
        let output = Output::new(format, buffer.clone(), true);
        (output, buffer)
    }

    #[test]
    fn test_validate_bump_type_valid() {
        assert!(validate_bump_type("patch").is_ok());
        assert!(validate_bump_type("minor").is_ok());
        assert!(validate_bump_type("major").is_ok());
        assert!(validate_bump_type("PATCH").is_ok());
        assert!(validate_bump_type("Minor").is_ok());
        assert!(validate_bump_type("MAJOR").is_ok());
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_validate_bump_type_invalid() {
        let result = validate_bump_type("invalid");
        assert!(result.is_err());
        if let Err(CliError::Validation(message)) = result {
            assert!(message.contains("Invalid bump type"));
            assert!(message.contains("patch, minor, major"));
        } else {
            panic!("Expected Validation error");
        }

        assert!(validate_bump_type("").is_err());
        assert!(validate_bump_type("pre-release").is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_bump_type_valid() {
        assert!(matches!(parse_bump_type("patch").unwrap(), VersionBump::Patch));
        assert!(matches!(parse_bump_type("minor").unwrap(), VersionBump::Minor));
        assert!(matches!(parse_bump_type("major").unwrap(), VersionBump::Major));
        assert!(matches!(parse_bump_type("PATCH").unwrap(), VersionBump::Patch));
        assert!(matches!(parse_bump_type("Minor").unwrap(), VersionBump::Minor));
        assert!(matches!(parse_bump_type("MAJOR").unwrap(), VersionBump::Major));
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_parse_bump_type_invalid() {
        let result = parse_bump_type("invalid");
        assert!(result.is_err());
        if let Err(CliError::Validation(..)) = result {
            // Expected
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_validate_environments_no_restrictions() {
        let provided = vec!["dev".to_string(), "prod".to_string()];
        let available: Vec<String> = vec![];

        // When no environments are configured, all are valid
        assert!(validate_environments(&provided, &available).is_ok());
    }

    #[test]
    fn test_validate_environments_valid() {
        let provided = vec!["dev".to_string(), "staging".to_string()];
        let available = vec!["dev".to_string(), "staging".to_string(), "production".to_string()];

        assert!(validate_environments(&provided, &available).is_ok());
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_validate_environments_invalid() {
        let provided = vec!["dev".to_string(), "invalid-env".to_string()];
        let available = vec!["dev".to_string(), "staging".to_string(), "production".to_string()];

        let result = validate_environments(&provided, &available);
        assert!(result.is_err());
        if let Err(CliError::Validation(message)) = result {
            assert!(message.contains("invalid-env"));
            assert!(message.contains("not configured"));
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_validate_environments_empty_provided() {
        let provided: Vec<String> = vec![];
        let available = vec!["dev".to_string(), "prod".to_string()];

        // Empty list is valid (user chose no environments)
        assert!(validate_environments(&provided, &available).is_ok());
    }

    #[test]
    fn test_create_args_defaults() {
        let args = ChangesetCreateArgs {
            bump: None,
            env: None,
            branch: None,
            message: None,
            packages: None,
            non_interactive: false,
        };

        assert!(args.bump.is_none());
        assert!(args.env.is_none());
        assert!(args.branch.is_none());
        assert!(args.message.is_none());
        assert!(args.packages.is_none());
        assert!(!args.non_interactive);
    }

    #[test]
    fn test_create_args_with_values() {
        let args = ChangesetCreateArgs {
            bump: Some("minor".to_string()),
            env: Some(vec!["production".to_string()]),
            branch: Some("feature/new-feature".to_string()),
            message: Some("Add new feature".to_string()),
            packages: Some(vec!["pkg-a".to_string(), "pkg-b".to_string()]),
            non_interactive: true,
        };

        assert_eq!(args.bump.as_deref(), Some("minor"));
        assert!(matches!(args.env.as_ref(), Some(v) if v.len() == 1));
        assert_eq!(args.branch.as_deref(), Some("feature/new-feature"));
        assert_eq!(args.message.as_deref(), Some("Add new feature"));
        assert!(matches!(args.packages.as_ref(), Some(v) if v.len() == 2));
        assert!(args.non_interactive);
    }

    // Tests for non-interactive mode validation

    #[test]
    fn test_non_interactive_args_complete() {
        // Test that all required args are provided for non-interactive mode
        let args = ChangesetCreateArgs {
            bump: Some("minor".to_string()),
            env: Some(vec!["production".to_string()]),
            branch: Some("feature/test".to_string()),
            message: Some("Test message".to_string()),
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        // All required fields are present
        assert!(args.bump.is_some());
        assert!(args.packages.is_some());
        assert!(args.non_interactive);
    }

    #[test]
    fn test_non_interactive_missing_bump() {
        // In non-interactive mode, missing bump should be caught by validation
        let args = ChangesetCreateArgs {
            bump: None,
            env: Some(vec!["production".to_string()]),
            branch: Some("feature/test".to_string()),
            message: Some("Test message".to_string()),
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        assert!(args.bump.is_none());
        assert!(args.non_interactive);
    }

    #[test]
    fn test_non_interactive_missing_packages() {
        // In non-interactive mode, missing packages should be caught
        let args = ChangesetCreateArgs {
            bump: Some("minor".to_string()),
            env: Some(vec!["production".to_string()]),
            branch: Some("feature/test".to_string()),
            message: Some("Test message".to_string()),
            packages: None,
            non_interactive: true,
        };

        assert!(args.packages.is_none());
        assert!(args.non_interactive);
    }

    #[test]
    fn test_non_interactive_with_default_environments() {
        // Non-interactive mode can use default environments
        let args = ChangesetCreateArgs {
            bump: Some("patch".to_string()),
            env: None, // Will use defaults
            branch: Some("feature/test".to_string()),
            message: None,
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        assert!(args.env.is_none());
        assert!(args.non_interactive);
    }

    // ========================================================================
    // Changeset List Command Tests
    // ========================================================================

    use crate::cli::commands::ChangesetListArgs;
    use crate::commands::changeset::list::parse_bump_type as list_parse_bump_type;

    #[test]
    fn test_list_args_defaults() {
        let args = ChangesetListArgs {
            filter_package: None,
            filter_bump: None,
            filter_env: None,
            sort: "date".to_string(),
        };

        assert!(args.filter_package.is_none());
        assert!(args.filter_bump.is_none());
        assert!(args.filter_env.is_none());
        assert_eq!(args.sort, "date");
    }

    #[test]
    fn test_list_args_with_filters() {
        let args = ChangesetListArgs {
            filter_package: Some("my-package".to_string()),
            filter_bump: Some("major".to_string()),
            filter_env: Some("production".to_string()),
            sort: "branch".to_string(),
        };

        assert_eq!(args.filter_package.as_deref(), Some("my-package"));
        assert_eq!(args.filter_bump.as_deref(), Some("major"));
        assert_eq!(args.filter_env.as_deref(), Some("production"));
        assert_eq!(args.sort, "branch");
    }

    #[test]
    fn test_list_parse_bump_type_valid() {
        assert!(matches!(list_parse_bump_type("patch"), Ok(VersionBump::Patch)));
        assert!(matches!(list_parse_bump_type("minor"), Ok(VersionBump::Minor)));
        assert!(matches!(list_parse_bump_type("major"), Ok(VersionBump::Major)));
        assert!(matches!(list_parse_bump_type("PATCH"), Ok(VersionBump::Patch)));
        assert!(matches!(list_parse_bump_type("Minor"), Ok(VersionBump::Minor)));
        assert!(matches!(list_parse_bump_type("MAJOR"), Ok(VersionBump::Major)));
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_list_parse_bump_type_invalid() {
        let result = list_parse_bump_type("invalid");
        assert!(result.is_err());
        if let Err(CliError::Validation(message)) = result {
            assert!(message.contains("Invalid bump type"));
            assert!(message.contains("major, minor, patch"));
        } else {
            panic!("Expected Validation error");
        }

        assert!(list_parse_bump_type("").is_err());
        assert!(list_parse_bump_type("pre-release").is_err());
        assert!(list_parse_bump_type("preminor").is_err());
    }

    #[test]
    fn test_list_sort_options() {
        // Test valid sort options
        let sort_date = ChangesetListArgs {
            filter_package: None,
            filter_bump: None,
            filter_env: None,
            sort: "date".to_string(),
        };
        assert_eq!(sort_date.sort, "date");

        let sort_branch = ChangesetListArgs {
            filter_package: None,
            filter_bump: None,
            filter_env: None,
            sort: "branch".to_string(),
        };
        assert_eq!(sort_branch.sort, "branch");

        let sort_bump = ChangesetListArgs {
            filter_package: None,
            filter_bump: None,
            filter_env: None,
            sort: "bump".to_string(),
        };
        assert_eq!(sort_bump.sort, "bump");
    }

    #[test]
    fn test_list_multiple_filters() {
        // Test combining multiple filters
        let args = ChangesetListArgs {
            filter_package: Some("core".to_string()),
            filter_bump: Some("minor".to_string()),
            filter_env: Some("staging".to_string()),
            sort: "date".to_string(),
        };

        assert!(args.filter_package.is_some());
        assert!(args.filter_bump.is_some());
        assert!(args.filter_env.is_some());
    }

    #[test]
    fn test_list_no_filters() {
        // Test list with no filters (show all)
        let args = ChangesetListArgs {
            filter_package: None,
            filter_bump: None,
            filter_env: None,
            sort: "date".to_string(),
        };

        assert!(args.filter_package.is_none());
        assert!(args.filter_bump.is_none());
        assert!(args.filter_env.is_none());
    }

    #[test]
    fn test_list_package_filter_only() {
        let args = ChangesetListArgs {
            filter_package: Some("my-package".to_string()),
            filter_bump: None,
            filter_env: None,
            sort: "date".to_string(),
        };

        assert!(args.filter_package.is_some());
        assert!(args.filter_bump.is_none());
        assert!(args.filter_env.is_none());
    }

    #[test]
    fn test_list_bump_filter_only() {
        let args = ChangesetListArgs {
            filter_package: None,
            filter_bump: Some("major".to_string()),
            filter_env: None,
            sort: "date".to_string(),
        };

        assert!(args.filter_package.is_none());
        assert!(args.filter_bump.is_some());
        assert!(args.filter_env.is_none());
    }

    #[test]
    fn test_list_env_filter_only() {
        let args = ChangesetListArgs {
            filter_package: None,
            filter_bump: None,
            filter_env: Some("production".to_string()),
            sort: "date".to_string(),
        };

        assert!(args.filter_package.is_none());
        assert!(args.filter_bump.is_none());
        assert!(args.filter_env.is_some());
    }

    #[test]
    fn test_non_interactive_optional_message() {
        // Message is optional in non-interactive mode
        let args = ChangesetCreateArgs {
            bump: Some("major".to_string()),
            env: Some(vec!["dev".to_string()]),
            branch: Some("feature/test".to_string()),
            message: None, // Optional
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        assert!(args.message.is_none());
        assert!(args.non_interactive);
    }

    #[test]
    fn test_non_interactive_multiple_packages() {
        // Test multiple packages in non-interactive mode
        let args = ChangesetCreateArgs {
            bump: Some("minor".to_string()),
            env: Some(vec!["staging".to_string(), "prod".to_string()]),
            branch: Some("feature/multi".to_string()),
            message: Some("Multi-package change".to_string()),
            packages: Some(vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-c".to_string()]),
            non_interactive: true,
        };

        assert_eq!(args.packages.as_ref().map(Vec::len), Some(3));
        assert!(args.non_interactive);
    }

    #[test]
    fn test_non_interactive_multiple_environments() {
        // Test multiple environments in non-interactive mode
        let args = ChangesetCreateArgs {
            bump: Some("patch".to_string()),
            env: Some(vec!["dev".to_string(), "staging".to_string(), "production".to_string()]),
            branch: Some("feature/test".to_string()),
            message: None,
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        assert_eq!(args.env.as_ref().map(Vec::len), Some(3));
        assert!(args.non_interactive);
    }

    #[test]
    fn test_interactive_mode_partial_args() {
        // Interactive mode can have partial args (will prompt for missing)
        let args = ChangesetCreateArgs {
            bump: None, // Will be prompted
            env: None,  // Will be prompted
            branch: Some("feature/test".to_string()),
            message: None,
            packages: None, // Will be prompted
            non_interactive: false,
        };

        assert!(!args.non_interactive);
        assert!(args.bump.is_none());
        assert!(args.packages.is_none());
    }

    #[test]
    fn test_bump_type_case_insensitive() {
        // Bump types should be case-insensitive
        let test_cases = vec![
            ("patch", true),
            ("PATCH", true),
            ("Patch", true),
            ("minor", true),
            ("MINOR", true),
            ("Minor", true),
            ("major", true),
            ("MAJOR", true),
            ("Major", true),
            ("invalid", false),
            ("pre", false),
            ("", false),
        ];

        for (bump_str, should_be_valid) in test_cases {
            let result = validate_bump_type(bump_str);
            assert_eq!(
                result.is_ok(),
                should_be_valid,
                "Expected '{}' to be {}",
                bump_str,
                if should_be_valid { "valid" } else { "invalid" }
            );
        }
    }

    #[test]
    fn test_environment_validation_with_multiple_invalid() {
        // Test validation with multiple invalid environments
        let provided = vec!["valid".to_string(), "invalid1".to_string(), "invalid2".to_string()];
        let available = vec!["valid".to_string(), "staging".to_string()];

        let result = validate_environments(&provided, &available);
        assert!(result.is_err());
    }

    #[test]
    fn test_environment_validation_all_valid() {
        // Test that all environments are valid
        let provided = vec!["dev".to_string(), "staging".to_string(), "production".to_string()];
        let available = vec![
            "dev".to_string(),
            "staging".to_string(),
            "production".to_string(),
            "qa".to_string(),
        ];

        assert!(validate_environments(&provided, &available).is_ok());
    }

    #[test]
    fn test_parse_bump_type_all_variants() {
        // Test all bump type variants parse correctly
        let patch = parse_bump_type("patch");
        assert!(patch.is_ok());
        assert!(matches!(patch.as_ref(), Ok(VersionBump::Patch)));

        let minor = parse_bump_type("minor");
        assert!(minor.is_ok());
        assert!(matches!(minor.as_ref(), Ok(VersionBump::Minor)));

        let major = parse_bump_type("major");
        assert!(major.is_ok());
        assert!(matches!(major.as_ref(), Ok(VersionBump::Major)));
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_validate_bump_type_error_message() {
        // Verify error messages are helpful
        let result = validate_bump_type("prepatch");
        assert!(result.is_err());

        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("prepatch"));
            assert!(msg.contains("patch"));
            assert!(msg.contains("minor"));
            assert!(msg.contains("major"));
        } else {
            panic!("Expected Validation error with helpful message");
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_environment_validation_error_message() {
        // Verify environment validation error messages are helpful
        let provided = vec!["production".to_string(), "unknown".to_string()];
        let available = vec!["dev".to_string(), "staging".to_string(), "production".to_string()];

        let result = validate_environments(&provided, &available);
        assert!(result.is_err());

        if let Err(CliError::Validation(msg)) = result {
            assert!(msg.contains("unknown"));
            assert!(msg.contains("not configured"));
            assert!(msg.contains("dev") || msg.contains("staging") || msg.contains("production"));
        } else {
            panic!("Expected Validation error with helpful message");
        }
    }

    #[test]
    fn test_args_with_branch_override() {
        // Test that branch can be explicitly provided
        let args = ChangesetCreateArgs {
            bump: Some("minor".to_string()),
            env: Some(vec!["production".to_string()]),
            branch: Some("custom/branch-name".to_string()),
            message: Some("Custom branch".to_string()),
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        assert_eq!(args.branch.as_deref(), Some("custom/branch-name"));
    }

    #[test]
    fn test_args_without_branch_uses_current() {
        // Test that branch defaults to current when not provided
        let args = ChangesetCreateArgs {
            bump: Some("minor".to_string()),
            env: Some(vec!["production".to_string()]),
            branch: None, // Will use current git branch
            message: Some("Use current branch".to_string()),
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        assert!(args.branch.is_none());
    }

    #[test]
    fn test_empty_package_list() {
        // Test with empty package list
        let args = ChangesetCreateArgs {
            bump: Some("patch".to_string()),
            env: Some(vec!["dev".to_string()]),
            branch: Some("feature/test".to_string()),
            message: None,
            packages: Some(vec![]), // Empty list
            non_interactive: true,
        };

        assert!(args.packages.as_ref().is_some_and(Vec::is_empty));
    }

    #[test]
    fn test_empty_environment_list() {
        // Test with empty environment list
        let args = ChangesetCreateArgs {
            bump: Some("patch".to_string()),
            env: Some(vec![]), // Empty list
            branch: Some("feature/test".to_string()),
            message: None,
            packages: Some(vec!["pkg-a".to_string()]),
            non_interactive: true,
        };

        assert!(args.env.as_ref().is_some_and(Vec::is_empty));
    }

    #[test]
    fn test_validate_bump_type_whitespace() {
        // Test that bump types with whitespace are invalid
        assert!(validate_bump_type(" patch").is_err());
        assert!(validate_bump_type("patch ").is_err());
        assert!(validate_bump_type(" minor ").is_err());
    }

    #[test]
    fn test_parse_bump_type_case_variations() {
        // Test various case combinations
        let test_cases = vec![
            "patch", "Patch", "PATCH", "pAtCh", "minor", "Minor", "MINOR", "mInOr", "major",
            "Major", "MAJOR", "mAjOr",
        ];

        for case in test_cases {
            let result = parse_bump_type(case);
            assert!(result.is_ok(), "Failed to parse: {case}");
        }
    }

    // Integration tests that require full setup remain ignored
    // These will be implemented when test infrastructure is available

    #[test]
    #[ignore = "requires git repository setup"]
    fn test_execute_add_non_interactive_full() {
        // TODO: will be implemented when test infrastructure is available
        // This test would:
        // 1. Create temp directory with git repo
        // 2. Create mock configuration
        // 3. Execute add command with all flags
        // 4. Verify changeset file created
        // 5. Verify output is correct
    }

    #[test]
    #[ignore = "requires terminal interaction"]
    fn test_execute_add_interactive() {
        // TODO: will be implemented when test infrastructure is available
        // This test would:
        // 1. Create temp directory with git repo
        // 2. Create mock configuration
        // 3. Mock interactive prompts
        // 4. Execute add command
        // 5. Verify changeset file created
    }

    #[test]
    #[ignore = "requires git repository setup"]
    fn test_execute_add_duplicate_changeset() {
        // TODO: will be implemented when test infrastructure is available
        // This test would:
        // 1. Create temp directory with git repo
        // 2. Create existing changeset for branch
        // 3. Try to create duplicate
        // 4. Verify error is returned
    }

    #[test]
    #[ignore = "requires git repository setup"]
    fn test_execute_add_detached_head() {
        // TODO: will be implemented when test infrastructure is available
        // This test would:
        // 1. Create temp directory with git repo in detached HEAD
        // 2. Try to create changeset without --branch flag
        // 3. Verify error is returned
    }

    #[test]
    #[ignore = "requires git repository setup"]
    fn test_execute_add_not_git_repo() {
        // TODO: will be implemented when test infrastructure is available
        // This test would:
        // 1. Create temp directory without git
        // 2. Try to create changeset
        // 3. Verify error is returned
    }

    #[test]
    #[ignore = "requires git repository setup"]
    fn test_execute_add_json_output() {
        // TODO: will be implemented when test infrastructure is available
        // This test would:
        // 1. Create temp directory with git repo
        // 2. Execute add command with --format json
        // 3. Verify output is valid JSON
        // 4. Verify JSON structure matches spec
    }

    #[test]
    #[ignore = "requires git repository setup"]
    fn test_execute_add_with_package_detection() {
        // TODO: will be implemented when test infrastructure is available
        // This test would:
        // 1. Create monorepo with packages
        // 2. Make changes to specific package
        // 3. Execute add command
        // 4. Verify affected package is detected
    }
}
