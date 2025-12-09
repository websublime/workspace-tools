//! Consolidated test module for sublime_node_tools crate.
//!
//! # What
//!
//! This module contains all unit tests for the NAPI bindings crate, organized
//! by logical groupings that correspond to the main modules of the crate.
//!
//! # How
//!
//! Tests are organized into submodules:
//! - **`version_tests`**: Tests for lib.rs version functions and constants
//! - **`error_tests`**: Tests for error handling and ErrorInfo
//! - **`validation_tests`**: Tests for parameter validation functions
//! - **`response_tests`**: Tests for API response utilities
//! - **`types_tests`**: Tests for common type re-exports
//!
//! Each submodule groups related tests together for better organization
//! and discoverability.
//!
//! # Why
//!
//! Centralizing tests in a single file with clear submodules provides:
//! - Consistent test organization across the crate
//! - Easy discovery of all tests in one place
//! - Clear separation between production code and test code
//! - Simplified test maintenance

// Allow unwrap in tests for cleaner assertions
#![allow(clippy::unwrap_used)]

/// Tests for lib.rs version functions and constants.
#[cfg(test)]
mod version_tests {
    use crate::{get_version, VERSION};

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version_constant() {
        // VERSION is a compile-time constant from Cargo.toml
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert_eq!(version, VERSION);
        assert!(!version.is_empty());
    }

    #[test]
    fn test_version_is_semver() {
        let version = get_version();
        let parts: Vec<&str> = version.split('.').collect();

        assert!(parts.len() >= 3, "Version should have at least 3 parts: {version}");

        let major = parts[0];
        assert!(major.parse::<u32>().is_ok(), "Major version should be a number: {major}");

        let minor = parts[1];
        assert!(minor.parse::<u32>().is_ok(), "Minor version should be a number: {minor}");

        let patch_part = parts[2].split('-').next().unwrap_or(parts[2]);
        assert!(
            patch_part.parse::<u32>().is_ok(),
            "Patch version should be a number: {patch_part}"
        );
    }
}

/// Tests for error handling module (error.rs).
#[cfg(test)]
mod error_tests {
    use crate::error::{ErrorCode, ErrorInfo};
    use sublime_cli_tools::error::CliError;

    #[test]
    fn test_error_code_as_str() {
        assert_eq!(ErrorCode::Config.as_str(), "ECONFIG");
        assert_eq!(ErrorCode::Validation.as_str(), "EVALIDATION");
        assert_eq!(ErrorCode::Execution.as_str(), "EEXEC");
        assert_eq!(ErrorCode::Git.as_str(), "EGIT");
        assert_eq!(ErrorCode::Package.as_str(), "EPKG");
        assert_eq!(ErrorCode::NotFound.as_str(), "ENOENT");
        assert_eq!(ErrorCode::Io.as_str(), "EIO");
        assert_eq!(ErrorCode::Network.as_str(), "ENETWORK");
        assert_eq!(ErrorCode::User.as_str(), "EUSER");
        assert_eq!(ErrorCode::Timeout.as_str(), "ETIMEOUT");
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(format!("{}", ErrorCode::Validation), "EVALIDATION");
        assert_eq!(format!("{}", ErrorCode::NotFound), "ENOENT");
    }

    #[test]
    fn test_error_info_new() {
        let error = ErrorInfo::new("ETEST", "Test message", Some("field"), "Test");
        assert_eq!(error.code, "ETEST");
        assert_eq!(error.message, "Test message");
        assert_eq!(error.context, Some("field".to_string()));
        assert_eq!(error.kind, "Test");
    }

    #[test]
    fn test_error_info_validation() {
        let error = ErrorInfo::validation("Invalid value", Some("packages"));
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.message, "Invalid value");
        assert_eq!(error.context, Some("packages".to_string()));
        assert_eq!(error.kind, "Validation");
    }

    #[test]
    fn test_error_info_configuration() {
        let error = ErrorInfo::configuration("Config not found");
        assert_eq!(error.code, "ECONFIG");
        assert_eq!(error.message, "Config not found");
        assert_eq!(error.context, None);
        assert_eq!(error.kind, "Configuration");
    }

    #[test]
    fn test_error_info_not_found() {
        let error = ErrorInfo::not_found("Path not found", Some("/some/path"));
        assert_eq!(error.code, "ENOENT");
        assert_eq!(error.message, "Path not found");
        assert_eq!(error.context, Some("/some/path".to_string()));
        assert_eq!(error.kind, "Io");
    }

    #[test]
    fn test_error_info_git() {
        let error = ErrorInfo::git("Repository not found");
        assert_eq!(error.code, "EGIT");
        assert_eq!(error.message, "Repository not found");
        assert_eq!(error.kind, "Git");
    }

    #[test]
    fn test_error_info_execution() {
        let error = ErrorInfo::execution("Command failed");
        assert_eq!(error.code, "EEXEC");
        assert_eq!(error.kind, "Execution");
    }

    #[test]
    fn test_error_info_package() {
        let error = ErrorInfo::package("Package not found");
        assert_eq!(error.code, "EPKG");
        assert_eq!(error.kind, "Package");
    }

    #[test]
    fn test_error_info_io() {
        let error = ErrorInfo::io("Read failed", Some("/file.txt"));
        assert_eq!(error.code, "EIO");
        assert_eq!(error.context, Some("/file.txt".to_string()));
        assert_eq!(error.kind, "Io");
    }

    #[test]
    fn test_error_info_network() {
        let error = ErrorInfo::network("Connection refused");
        assert_eq!(error.code, "ENETWORK");
        assert_eq!(error.kind, "Network");
    }

    #[test]
    fn test_error_info_user() {
        let error = ErrorInfo::user("Operation cancelled");
        assert_eq!(error.code, "EUSER");
        assert_eq!(error.kind, "User");
    }

    #[test]
    fn test_error_info_timeout() {
        let error = ErrorInfo::timeout("Operation timed out");
        assert_eq!(error.code, "ETIMEOUT");
        assert_eq!(error.kind, "Timeout");
    }

    #[test]
    fn test_from_cli_error_configuration() {
        let cli_error = CliError::configuration("Test config error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "ECONFIG");
        assert_eq!(error_info.kind, "Configuration");
    }

    #[test]
    fn test_from_cli_error_validation() {
        let cli_error = CliError::validation("Test validation error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EVALIDATION");
        assert_eq!(error_info.kind, "Validation");
    }

    #[test]
    fn test_from_cli_error_git() {
        let cli_error = CliError::git("Test git error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EGIT");
        assert_eq!(error_info.kind, "Git");
    }

    #[test]
    fn test_from_cli_error_package() {
        let cli_error = CliError::package("Test package error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EPKG");
        assert_eq!(error_info.kind, "Package");
    }

    #[test]
    fn test_from_cli_error_io() {
        let cli_error = CliError::io("Test io error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EIO");
        assert_eq!(error_info.kind, "Io");
    }

    #[test]
    fn test_from_cli_error_network() {
        let cli_error = CliError::network("Test network error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "ENETWORK");
        assert_eq!(error_info.kind, "Network");
    }

    #[test]
    fn test_from_cli_error_user() {
        let cli_error = CliError::user("Test user error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EUSER");
        assert_eq!(error_info.kind, "User");
    }

    #[test]
    fn test_from_cli_error_execution() {
        let cli_error = CliError::execution("Test execution error");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EEXEC");
        assert_eq!(error_info.kind, "Execution");
    }

    #[test]
    fn test_from_cli_error_owned() {
        let cli_error = CliError::validation("Test owned error");
        let error_info = ErrorInfo::from(cli_error);
        assert_eq!(error_info.code, "EVALIDATION");
    }

    // Tests for I/O error differentiation (ENOENT vs EIO)

    #[test]
    fn test_from_cli_error_io_not_found() {
        let cli_error = CliError::io("File not found: /path/to/file");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "ENOENT");
        assert_eq!(error_info.kind, "Io");
    }

    #[test]
    fn test_from_cli_error_io_no_such_file() {
        let cli_error = CliError::io("No such file or directory");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "ENOENT");
        assert_eq!(error_info.kind, "Io");
    }

    #[test]
    fn test_from_cli_error_io_does_not_exist() {
        let cli_error = CliError::io("The path does not exist");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "ENOENT");
        assert_eq!(error_info.kind, "Io");
    }

    #[test]
    fn test_from_cli_error_io_doesnt_exist() {
        let cli_error = CliError::io("The file doesn't exist");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "ENOENT");
        assert_eq!(error_info.kind, "Io");
    }

    #[test]
    fn test_from_cli_error_io_permission_denied() {
        let cli_error = CliError::io("Permission denied");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EIO");
        assert_eq!(error_info.kind, "Io");
    }

    #[test]
    fn test_from_cli_error_io_disk_full() {
        let cli_error = CliError::io("Disk full");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "EIO");
        assert_eq!(error_info.kind, "Io");
    }

    #[test]
    fn test_from_cli_error_io_case_insensitive() {
        // Test that "NOT FOUND" (uppercase) also matches
        let cli_error = CliError::io("PATH NOT FOUND");
        let error_info = ErrorInfo::from(&cli_error);
        assert_eq!(error_info.code, "ENOENT");
        assert_eq!(error_info.kind, "Io");
    }
}

/// Tests for ValidationError struct and From<ValidationError> for ErrorInfo.
#[cfg(test)]
mod validation_error_tests {
    use crate::error::ErrorInfo;
    use crate::validation::ValidationError;

    #[test]
    fn test_validation_error_required() {
        let error = ValidationError::required("packages");
        assert_eq!(error.field, "packages");
        assert_eq!(error.message, "packages is required");
        assert!(error.value.is_none());
    }

    #[test]
    fn test_validation_error_required_different_fields() {
        let error1 = ValidationError::required("root");
        assert_eq!(error1.field, "root");
        assert_eq!(error1.message, "root is required");

        let error2 = ValidationError::required("message");
        assert_eq!(error2.field, "message");
        assert_eq!(error2.message, "message is required");
    }

    #[test]
    fn test_validation_error_invalid_with_value() {
        let error = ValidationError::invalid(
            "bumpType",
            "must be one of: major, minor, patch",
            Some("invalid"),
        );
        assert_eq!(error.field, "bumpType");
        assert_eq!(error.message, "must be one of: major, minor, patch");
        assert_eq!(error.value, Some("invalid".to_string()));
    }

    #[test]
    fn test_validation_error_invalid_without_value() {
        let error = ValidationError::invalid("packages", "array cannot be empty", None::<String>);
        assert_eq!(error.field, "packages");
        assert_eq!(error.message, "array cannot be empty");
        assert!(error.value.is_none());
    }

    #[test]
    fn test_validation_error_invalid_with_string_value() {
        let error =
            ValidationError::invalid("timeout", "must be positive", Some(String::from("0")));
        assert_eq!(error.value, Some("0".to_string()));
    }

    #[test]
    fn test_validation_error_display_with_value() {
        let error = ValidationError::invalid("timeout", "must be positive", Some("0"));
        let display = format!("{error}");
        assert_eq!(display, "timeout: must be positive (got: 0)");
    }

    #[test]
    fn test_validation_error_display_without_value() {
        let error = ValidationError::required("packages");
        let display = format!("{error}");
        assert_eq!(display, "packages: packages is required");
    }

    #[test]
    fn test_validation_error_clone() {
        let error = ValidationError::invalid("field", "message", Some("value"));
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_validation_error_eq() {
        let error1 = ValidationError::required("packages");
        let error2 = ValidationError::required("packages");
        let error3 = ValidationError::required("root");

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_from_validation_error_for_error_info_required() {
        let validation_error = ValidationError::required("root");
        let error_info: ErrorInfo = validation_error.into();

        assert_eq!(error_info.code, "EVALIDATION");
        assert_eq!(error_info.kind, "Validation");
        assert_eq!(error_info.context, Some("root".to_string()));
        assert!(error_info.message.contains("root is required"));
    }

    #[test]
    fn test_from_validation_error_for_error_info_invalid() {
        let validation_error =
            ValidationError::invalid("bumpType", "must be major, minor, or patch", Some("bad"));
        let error_info: ErrorInfo = validation_error.into();

        assert_eq!(error_info.code, "EVALIDATION");
        assert_eq!(error_info.kind, "Validation");
        assert_eq!(error_info.context, Some("bumpType".to_string()));
        assert!(error_info.message.contains("must be major"));
    }

    #[test]
    fn test_validation_error_is_error_trait() {
        let error = ValidationError::required("field");
        // Verify it implements std::error::Error
        let _: &dyn std::error::Error = &error;
    }
}

/// Tests for validators module.
#[cfg(test)]
mod validators_tests {
    use crate::validation::validators;
    use std::fs;
    use tempfile::TempDir;

    // Tests for path_exists validator
    #[test]
    fn test_path_exists_valid_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = validators::path_exists(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_exists_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = validators::path_exists(file_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_exists_nonexistent() {
        let result = validators::path_exists("/this/path/definitely/does/not/exist");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.field, "path");
        assert!(error.message.contains("does not exist"));
        assert!(error.value.is_some());
    }

    #[test]
    fn test_path_exists_empty_path() {
        // Empty string is a valid path that doesn't exist
        let result = validators::path_exists("");
        assert!(result.is_err());
    }

    // Tests for not_empty validator
    #[test]
    fn test_not_empty_valid() {
        assert!(validators::not_empty("message", "Add feature").is_ok());
        assert!(validators::not_empty("cmd", "npm test").is_ok());
        assert!(validators::not_empty("field", "x").is_ok());
    }

    #[test]
    fn test_not_empty_empty_string() {
        let result = validators::not_empty("message", "");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.field, "message");
        assert!(error.message.contains("cannot be empty"));
        assert!(error.value.is_none());
    }

    #[test]
    fn test_not_empty_whitespace_only() {
        assert!(validators::not_empty("message", "   ").is_err());
        assert!(validators::not_empty("message", "\t").is_err());
        assert!(validators::not_empty("message", "\n").is_err());
        assert!(validators::not_empty("message", "  \t\n  ").is_err());
    }

    #[test]
    fn test_not_empty_preserves_field_name() {
        let result = validators::not_empty("customField", "");
        let error = result.unwrap_err();
        assert_eq!(error.field, "customField");
    }

    // Tests for bump_type validator
    // Valid bump types are: major, minor, patch, none (as per VersionBump enum in sublime_pkg_tools)
    #[test]
    fn test_bump_type_valid_major() {
        assert!(validators::bump_type("major").is_ok());
    }

    #[test]
    fn test_bump_type_valid_minor() {
        assert!(validators::bump_type("minor").is_ok());
    }

    #[test]
    fn test_bump_type_valid_patch() {
        assert!(validators::bump_type("patch").is_ok());
    }

    #[test]
    fn test_bump_type_valid_none() {
        assert!(validators::bump_type("none").is_ok());
    }

    #[test]
    fn test_bump_type_invalid() {
        let result = validators::bump_type("invalid");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.field, "bumpType");
        assert!(error.message.contains("must be one of"));
        assert_eq!(error.value, Some("invalid".to_string()));
    }

    #[test]
    fn test_bump_type_case_sensitive() {
        // Bump types are case-sensitive
        assert!(validators::bump_type("Major").is_err());
        assert!(validators::bump_type("MINOR").is_err());
        assert!(validators::bump_type("Patch").is_err());
    }

    #[test]
    fn test_bump_type_empty() {
        let result = validators::bump_type("");
        assert!(result.is_err());
    }

    // Tests for timeout validator
    #[test]
    fn test_timeout_valid_within_bounds() {
        assert!(validators::timeout("timeoutSecs", 30, 1, 3600).is_ok());
        assert!(validators::timeout("timeoutSecs", 100, 1, 3600).is_ok());
        assert!(validators::timeout("timeoutSecs", 1800, 1, 3600).is_ok());
    }

    #[test]
    fn test_timeout_valid_at_min() {
        assert!(validators::timeout("timeoutSecs", 1, 1, 3600).is_ok());
    }

    #[test]
    fn test_timeout_valid_at_max() {
        assert!(validators::timeout("timeoutSecs", 3600, 1, 3600).is_ok());
    }

    #[test]
    fn test_timeout_below_min() {
        let result = validators::timeout("timeoutSecs", 0, 1, 3600);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.field, "timeoutSecs");
        assert!(error.message.contains("at least 1 seconds"));
        assert_eq!(error.value, Some("0".to_string()));
    }

    #[test]
    fn test_timeout_above_max() {
        let result = validators::timeout("timeoutSecs", 7200, 1, 3600);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.field, "timeoutSecs");
        assert!(error.message.contains("cannot exceed 3600 seconds"));
        assert_eq!(error.value, Some("7200".to_string()));
    }

    #[test]
    fn test_timeout_custom_bounds() {
        // Test with different min/max bounds
        assert!(validators::timeout("delay", 5, 5, 10).is_ok());
        assert!(validators::timeout("delay", 10, 5, 10).is_ok());
        assert!(validators::timeout("delay", 4, 5, 10).is_err());
        assert!(validators::timeout("delay", 11, 5, 10).is_err());
    }

    #[test]
    fn test_timeout_preserves_field_name() {
        let result = validators::timeout("perPackageTimeoutSecs", 0, 1, 3600);
        let error = result.unwrap_err();
        assert_eq!(error.field, "perPackageTimeoutSecs");
    }

    #[test]
    fn test_timeout_zero_min_allowed() {
        // If min is 0, then 0 should be valid
        assert!(validators::timeout("optional", 0, 0, 100).is_ok());
    }

    // Integration tests - converting validator errors to ErrorInfo
    #[test]
    fn test_validator_error_converts_to_error_info() {
        use crate::error::ErrorInfo;

        let result = validators::bump_type("invalid");
        let validation_error = result.unwrap_err();
        let error_info: ErrorInfo = validation_error.into();

        assert_eq!(error_info.code, "EVALIDATION");
        assert_eq!(error_info.kind, "Validation");
        assert_eq!(error_info.context, Some("bumpType".to_string()));
    }
}

/// Tests for validators module - ErrorInfo returning validators.
#[cfg(test)]
mod validation_tests {
    use crate::validation::validators;
    use std::fs;
    use tempfile::TempDir;

    // Tests for root validator
    #[test]
    fn test_validate_root_empty() {
        let result = validators::root("");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("root".to_string()));
    }

    #[test]
    fn test_validate_root_not_exists() {
        let result = validators::root("/this/path/does/not/exist/at/all");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "ENOENT");
    }

    #[test]
    fn test_validate_root_valid() {
        let temp_dir = TempDir::new().unwrap();
        let result = validators::root(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_root_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let result = validators::root(file_path.to_str().unwrap());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
    }

    // Tests for packages_not_empty validator
    #[test]
    fn test_validate_packages_not_empty_valid() {
        let packages = vec!["@scope/pkg1".to_string()];
        let result = validators::packages_not_empty(&packages);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_packages_not_empty_empty() {
        let packages: Vec<String> = vec![];
        let result = validators::packages_not_empty(&packages);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("packages".to_string()));
    }

    // Tests for bump_type_info validator (returns ErrorInfo)
    #[test]
    fn test_validate_bump_type_valid() {
        // Valid bump types as per VersionBump enum: major, minor, patch, none
        assert!(validators::bump_type_info("major").is_ok());
        assert!(validators::bump_type_info("minor").is_ok());
        assert!(validators::bump_type_info("patch").is_ok());
        assert!(validators::bump_type_info("none").is_ok());
    }

    #[test]
    fn test_validate_bump_type_invalid() {
        let result = validators::bump_type_info("invalid");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("bumpType".to_string()));
    }

    #[test]
    fn test_validate_bump_type_prerelease_invalid() {
        // prerelease is NOT a valid bump type in sublime_pkg_tools
        let result = validators::bump_type_info("prerelease");
        assert!(result.is_err());
    }

    // Tests for message_not_empty validator
    #[test]
    fn test_validate_message_not_empty_valid() {
        let result = validators::message_not_empty("Add feature", "message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_message_not_empty_empty() {
        let result = validators::message_not_empty("", "message");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
    }

    #[test]
    fn test_validate_message_not_empty_whitespace() {
        let result = validators::message_not_empty("   ", "message");
        assert!(result.is_err());
    }

    // Tests for semver validator
    #[test]
    fn test_validate_semver_valid() {
        assert!(validators::semver("1.0.0", "version").is_ok());
        assert!(validators::semver("2.3.4", "version").is_ok());
        assert!(validators::semver("10.20.30", "version").is_ok());
        assert!(validators::semver("1.0.0-beta.1", "version").is_ok());
    }

    #[test]
    fn test_validate_semver_invalid() {
        let result = validators::semver("invalid", "version");
        assert!(result.is_err());

        let result = validators::semver("1.0", "version");
        assert!(result.is_err());

        let result = validators::semver("a.b.c", "version");
        assert!(result.is_err());
    }

    // Tests for mutual_exclusion validator
    #[test]
    fn test_validate_mutual_exclusion_none_set() {
        let result = validators::mutual_exclusion(&[("a", false), ("b", false)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_mutual_exclusion_one_set() {
        let result = validators::mutual_exclusion(&[("a", true), ("b", false)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_mutual_exclusion_both_set() {
        let result = validators::mutual_exclusion(&[("filterPackage", true), ("affected", true)]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
    }

    // Tests for timeout_positive validator
    #[test]
    fn test_validate_timeout_valid() {
        assert!(validators::timeout_positive(1, "timeout").is_ok());
        assert!(validators::timeout_positive(30, "timeout").is_ok());
        assert!(validators::timeout_positive(3600, "timeout").is_ok());
    }

    #[test]
    fn test_validate_timeout_zero() {
        let result = validators::timeout_positive(0, "timeoutSecs");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("timeoutSecs".to_string()));
    }

    // Tests for optional_timeout validator
    #[test]
    fn test_validate_optional_timeout_none() {
        let result = validators::optional_timeout(None, "timeout");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_optional_timeout_valid() {
        let result = validators::optional_timeout(Some(30), "timeout");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_optional_timeout_zero() {
        let result = validators::optional_timeout(Some(0), "timeout");
        assert!(result.is_err());
    }
}

/// Tests for response module (response.rs).
#[cfg(test)]
mod response_tests {
    use crate::error::ErrorInfo;
    use crate::response::{result_to_response, ApiResponse, ApiResponseExt, JsonResponse};
    use serde::Serialize;
    use std::io::{Error as IoError, ErrorKind};
    use sublime_cli_tools::error::CliError;

    // =====================================
    // JsonResponse Tests (existing)
    // =====================================

    #[test]
    fn test_json_response_success() {
        let response = JsonResponse::success("test data".to_string());
        assert!(response.is_success());
        assert!(!response.is_error());
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_response_error() {
        let response: JsonResponse<String> = JsonResponse::error("test error".to_string());
        assert!(!response.is_success());
        assert!(response.is_error());
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }

    #[test]
    fn test_api_response_ext_from_error_info() {
        let error = ErrorInfo::validation("Invalid input", Some("field"));
        let response: JsonResponse<String> = JsonResponse::from_error_info(error);

        assert!(response.is_error());
        assert!(response.error.is_some());
        let error_msg = response.error.unwrap();
        assert!(error_msg.contains("EVALIDATION"));
        assert!(error_msg.contains("Invalid input"));
    }

    #[test]
    fn test_api_response_ext_validation_error_with_field() {
        let response: JsonResponse<String> =
            JsonResponse::validation_error("Cannot be empty", Some("packages"));

        assert!(response.is_error());
        let error_msg = response.error.unwrap();
        assert!(error_msg.contains("EVALIDATION"));
        assert!(error_msg.contains("packages"));
        assert!(error_msg.contains("Cannot be empty"));
    }

    #[test]
    fn test_api_response_ext_validation_error_without_field() {
        let response: JsonResponse<String> =
            JsonResponse::validation_error("General validation error", None);

        assert!(response.is_error());
        let error_msg = response.error.unwrap();
        assert!(error_msg.contains("EVALIDATION"));
        assert!(error_msg.contains("General validation error"));
    }

    #[test]
    fn test_result_to_response_ok() {
        let result: Result<String, ErrorInfo> = Ok("success".to_string());
        let response = result_to_response(result);

        assert!(response.is_success());
        assert_eq!(response.data, Some("success".to_string()));
    }

    #[test]
    fn test_result_to_response_err() {
        let result: Result<String, ErrorInfo> =
            Err(ErrorInfo::validation("test error", None::<String>));
        let response = result_to_response(result);

        assert!(response.is_error());
        assert!(response.error.is_some());
    }

    // =====================================
    // ApiResponse Tests (Story 2.3)
    // =====================================

    /// Test data structure for ApiResponse tests.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct TestData {
        value: String,
        count: u32,
    }

    #[test]
    fn test_api_response_success() {
        let data = TestData { value: "test".to_string(), count: 42 };
        let response = ApiResponse::success(data.clone());

        assert!(response.success);
        assert!(response.is_success());
        assert!(!response.is_failure());
        assert_eq!(response.data, Some(data));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_success_with_string() {
        let response = ApiResponse::success("simple string".to_string());

        assert!(response.success);
        assert!(response.is_success());
        assert_eq!(response.data, Some("simple string".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_success_with_unit() {
        let response: ApiResponse<()> = ApiResponse::success(());

        assert!(response.success);
        assert!(response.is_success());
        assert_eq!(response.data, Some(()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_failure() {
        let error = ErrorInfo::validation("Invalid input", Some("field_name"));
        let response: ApiResponse<TestData> = ApiResponse::failure(error.clone());

        assert!(!response.success);
        assert!(!response.is_success());
        assert!(response.is_failure());
        assert!(response.data.is_none());
        assert!(response.error.is_some());

        let err = response.error.unwrap();
        assert_eq!(err.code, "EVALIDATION");
        assert_eq!(err.message, "Invalid input");
        assert_eq!(err.context, Some("field_name".to_string()));
        assert_eq!(err.kind, "Validation");
    }

    #[test]
    fn test_api_response_failure_different_error_types() {
        // Test various error types
        let test_cases = vec![
            (ErrorInfo::validation("msg", None::<String>), "EVALIDATION"),
            (ErrorInfo::not_found("msg", None::<String>), "ENOENT"),
            (ErrorInfo::io("msg", None::<String>), "EIO"),
            (ErrorInfo::network("msg"), "ENETWORK"),
            (ErrorInfo::user("msg"), "EUSER"),
            (ErrorInfo::timeout("msg"), "ETIMEOUT"),
            (ErrorInfo::execution("msg"), "EEXEC"),
            (ErrorInfo::package("msg"), "EPKG"),
            (ErrorInfo::configuration("msg"), "ECONFIG"),
            (ErrorInfo::git("msg"), "EGIT"),
        ];

        for (error, expected_code) in test_cases {
            let response: ApiResponse<String> = ApiResponse::failure(error);
            assert!(response.is_failure());
            assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some(expected_code));
        }
    }

    #[test]
    fn test_api_response_failure_from_io_not_found() {
        let io_error = IoError::new(ErrorKind::NotFound, "File not found: config.json");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        assert!(response.error.is_some());

        let err = response.error.unwrap();
        assert_eq!(err.code, "ENOENT");
        assert!(err.message.contains("File not found"));
    }

    #[test]
    fn test_api_response_failure_from_io_permission_denied() {
        let io_error = IoError::new(ErrorKind::PermissionDenied, "Access denied");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EIO");
        assert!(err.message.contains("Permission denied"));
    }

    #[test]
    fn test_api_response_failure_from_io_already_exists() {
        let io_error = IoError::new(ErrorKind::AlreadyExists, "File already exists");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EIO");
        assert!(err.message.contains("Already exists"));
    }

    #[test]
    fn test_api_response_failure_from_io_invalid_input() {
        let io_error = IoError::new(ErrorKind::InvalidInput, "Invalid argument");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EVALIDATION");
    }

    #[test]
    fn test_api_response_failure_from_io_invalid_data() {
        let io_error = IoError::new(ErrorKind::InvalidData, "Corrupted data");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EVALIDATION");
        assert!(err.message.contains("Invalid data"));
    }

    #[test]
    fn test_api_response_failure_from_io_timed_out() {
        let io_error = IoError::new(ErrorKind::TimedOut, "Operation timed out");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ETIMEOUT");
    }

    #[test]
    fn test_api_response_failure_from_io_connection_refused() {
        let io_error = IoError::new(ErrorKind::ConnectionRefused, "Connection refused");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ENETWORK");
    }

    #[test]
    fn test_api_response_failure_from_io_connection_reset() {
        let io_error = IoError::new(ErrorKind::ConnectionReset, "Connection reset");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ENETWORK");
    }

    #[test]
    fn test_api_response_failure_from_io_connection_aborted() {
        let io_error = IoError::new(ErrorKind::ConnectionAborted, "Connection aborted");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ENETWORK");
    }

    #[test]
    fn test_api_response_failure_from_io_not_connected() {
        let io_error = IoError::new(ErrorKind::NotConnected, "Not connected");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ENETWORK");
    }

    #[test]
    fn test_api_response_failure_from_io_other() {
        let io_error = IoError::other("Unknown error");
        let response: ApiResponse<String> = ApiResponse::failure_from_io(io_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EIO");
    }

    #[test]
    fn test_api_response_failure_from_cli_validation() {
        let cli_error = CliError::validation("Invalid package name");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EVALIDATION");
        assert_eq!(err.kind, "Validation");
    }

    #[test]
    fn test_api_response_failure_from_cli_configuration() {
        let cli_error = CliError::configuration("Config file not found");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ECONFIG");
        assert_eq!(err.kind, "Configuration");
    }

    #[test]
    fn test_api_response_failure_from_cli_git() {
        let cli_error = CliError::git("Repository not found");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EGIT");
        assert_eq!(err.kind, "Git");
    }

    #[test]
    fn test_api_response_failure_from_cli_package() {
        let cli_error = CliError::package("Package not in workspace");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EPKG");
        assert_eq!(err.kind, "Package");
    }

    #[test]
    fn test_api_response_failure_from_cli_io_not_found() {
        let cli_error = CliError::io("File not found: /path/to/file");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ENOENT");
        assert_eq!(err.kind, "Io");
    }

    #[test]
    fn test_api_response_failure_from_cli_io_other() {
        let cli_error = CliError::io("Permission denied");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EIO");
        assert_eq!(err.kind, "Io");
    }

    #[test]
    fn test_api_response_failure_from_cli_network() {
        let cli_error = CliError::network("Registry unreachable");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "ENETWORK");
        assert_eq!(err.kind, "Network");
    }

    #[test]
    fn test_api_response_failure_from_cli_user() {
        let cli_error = CliError::user("Operation cancelled");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EUSER");
        assert_eq!(err.kind, "User");
    }

    #[test]
    fn test_api_response_failure_from_cli_execution() {
        let cli_error = CliError::execution("Command failed");
        let response: ApiResponse<String> = ApiResponse::failure_from_cli(cli_error);

        assert!(response.is_failure());
        let err = response.error.unwrap();
        assert_eq!(err.code, "EEXEC");
        assert_eq!(err.kind, "Execution");
    }

    #[test]
    fn test_api_response_map_success() {
        let response = ApiResponse::success(42i32);
        let mapped = response.map(|n| n.to_string());

        assert!(mapped.is_success());
        assert_eq!(mapped.data, Some("42".to_string()));
        assert!(mapped.error.is_none());
    }

    #[test]
    fn test_api_response_map_failure() {
        let error = ErrorInfo::validation("error", None::<String>);
        let response: ApiResponse<i32> = ApiResponse::failure(error);
        let mapped = response.map(|n| n.to_string());

        assert!(mapped.is_failure());
        assert!(mapped.data.is_none());
        assert!(mapped.error.is_some());
        assert_eq!(mapped.error.as_ref().map(|e| e.code.as_str()), Some("EVALIDATION"));
    }

    #[test]
    fn test_api_response_map_complex_transformation() {
        let data = TestData { value: "test".to_string(), count: 10 };
        let response = ApiResponse::success(data);
        let mapped = response.map(|d| d.count * 2);

        assert!(mapped.is_success());
        assert_eq!(mapped.data, Some(20));
    }

    #[test]
    fn test_api_response_into_result_success() {
        let response = ApiResponse::success("data".to_string());
        let result = response.into_result();

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some("data".to_string()));
    }

    #[test]
    fn test_api_response_into_result_failure() {
        let error = ErrorInfo::validation("error message", Some("field"));
        let response: ApiResponse<String> = ApiResponse::failure(error);
        let result = response.into_result();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "EVALIDATION");
        assert_eq!(err.message, "error message");
    }

    #[test]
    fn test_api_response_into_result_malformed_success() {
        // Manually create a malformed response (success=true but data=None)
        let response: ApiResponse<String> = ApiResponse { success: true, data: None, error: None };
        let result = response.into_result();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "EEXEC");
        assert!(err.message.contains("Malformed response"));
    }

    #[test]
    fn test_api_response_into_result_malformed_failure() {
        // Manually create a malformed response (success=false but error=None)
        let response: ApiResponse<String> = ApiResponse { success: false, data: None, error: None };
        let result = response.into_result();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "EEXEC");
        assert!(err.message.contains("Malformed response"));
    }

    #[test]
    fn test_api_response_from_result_ok_error_info() {
        let result: Result<String, ErrorInfo> = Ok("success".to_string());
        let response: ApiResponse<String> = result.into();

        assert!(response.is_success());
        assert_eq!(response.data, Some("success".to_string()));
    }

    #[test]
    fn test_api_response_from_result_err_error_info() {
        let result: Result<String, ErrorInfo> = Err(ErrorInfo::validation("error", None::<String>));
        let response: ApiResponse<String> = result.into();

        assert!(response.is_failure());
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EVALIDATION"));
    }

    #[test]
    fn test_api_response_from_result_ok_cli_error() {
        let result: Result<String, CliError> = Ok("success".to_string());
        let response: ApiResponse<String> = result.into();

        assert!(response.is_success());
        assert_eq!(response.data, Some("success".to_string()));
    }

    #[test]
    fn test_api_response_from_result_err_cli_error() {
        let result: Result<String, CliError> = Err(CliError::git("Git error"));
        let response: ApiResponse<String> = result.into();

        assert!(response.is_failure());
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EGIT"));
    }

    #[test]
    fn test_api_response_from_result_ok_io_error() {
        let result: Result<String, IoError> = Ok("success".to_string());
        let response: ApiResponse<String> = result.into();

        assert!(response.is_success());
        assert_eq!(response.data, Some("success".to_string()));
    }

    #[test]
    fn test_api_response_from_result_err_io_error() {
        let result: Result<String, IoError> = Err(IoError::new(ErrorKind::NotFound, "not found"));
        let response: ApiResponse<String> = result.into();

        assert!(response.is_failure());
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("ENOENT"));
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::success(TestData { value: "test".to_string(), count: 42 });

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("\"success\":true"));
        assert!(json_str.contains("\"value\":\"test\""));
        assert!(json_str.contains("\"count\":42"));
        // error should be skipped when None
        assert!(!json_str.contains("\"error\""));
    }

    #[test]
    fn test_api_response_serialization_failure() {
        let error = ErrorInfo::validation("Invalid input", Some("field"));
        let response: ApiResponse<String> = ApiResponse::failure(error);

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("\"success\":false"));
        assert!(json_str.contains("\"code\":\"EVALIDATION\""));
        assert!(json_str.contains("\"message\":\"Invalid input\""));
        // data should be skipped when None
        assert!(!json_str.contains("\"data\""));
    }

    #[test]
    fn test_api_response_clone() {
        let response = ApiResponse::success("data".to_string());
        let cloned = response.clone();

        assert_eq!(response.success, cloned.success);
        assert_eq!(response.data, cloned.data);
    }

    #[test]
    fn test_api_response_debug() {
        let response = ApiResponse::success("data".to_string());
        let debug_str = format!("{response:?}");

        assert!(debug_str.contains("ApiResponse"));
        assert!(debug_str.contains("success: true"));
        assert!(debug_str.contains("data"));
    }
}

/// Tests for common types module (types/common.rs).
#[cfg(test)]
mod types_tests {
    use crate::types::common::{
        JsonResponse, MonorepoKind, PackageManagerKind, RepoKind, VersionBump,
    };

    #[test]
    fn test_version_bump_reexport() {
        let bump = VersionBump::Minor;
        assert_eq!(bump.as_str(), "minor");
    }

    #[test]
    fn test_package_manager_kind_reexport() {
        let kind = PackageManagerKind::Pnpm;
        assert_eq!(kind.name(), "pnpm");
    }

    #[test]
    fn test_repo_kind_reexport() {
        let kind = RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces);
        assert!(kind.is_monorepo());
    }

    #[test]
    fn test_json_response_reexport() {
        let response: JsonResponse<String> = JsonResponse::success("test".to_string());
        assert!(response.is_success());
        assert!(!response.is_error());
    }
}

/// Tests for status types (Story 3.1).
///
/// These tests verify that the status command types are correctly defined
/// and can be used for parameter validation and response construction.
mod status_api_response_tests {
    use crate::error::ErrorInfo;
    use crate::types::status::{PackageManagerInfo, RepositoryInfo, StatusApiResponse, StatusData};

    #[test]
    fn test_status_api_response_success() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::npm());
        let response = StatusApiResponse::success(data);

        assert!(response.success);
        assert!(response.is_success());
        assert!(!response.is_failure());
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_status_api_response_failure() {
        let error = ErrorInfo::validation("Invalid root path", Some("root"));
        let response = StatusApiResponse::failure(error);

        assert!(!response.success);
        assert!(!response.is_success());
        assert!(response.is_failure());
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EVALIDATION"));
    }

    #[test]
    fn test_status_api_response_failure_with_different_error_codes() {
        // Test with ENOENT
        let error = ErrorInfo::not_found("Path not found", Some("/invalid/path"));
        let response = StatusApiResponse::failure(error);
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("ENOENT"));

        // Test with EEXEC
        let error = ErrorInfo::execution("Command failed");
        let response = StatusApiResponse::failure(error);
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EEXEC"));

        // Test with EGIT
        let error = ErrorInfo::git("Git operation failed");
        let response = StatusApiResponse::failure(error);
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EGIT"));
    }

    #[test]
    fn test_status_api_response_clone() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::npm());
        let response = StatusApiResponse::success(data);
        let cloned = response.clone();

        assert_eq!(response.success, cloned.success);
        assert!(cloned.data.is_some());
    }

    #[test]
    fn test_status_api_response_debug() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::npm());
        let response = StatusApiResponse::success(data);
        let debug_str = format!("{response:?}");

        assert!(debug_str.contains("StatusApiResponse"));
        assert!(debug_str.contains("success: true"));
    }

    #[test]
    fn test_status_api_response_serialize_success() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::npm());
        let response = StatusApiResponse::success(data);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_status_api_response_serialize_failure() {
        let error = ErrorInfo::validation("Invalid", Some("field"));
        let response = StatusApiResponse::failure(error);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\""));
        assert!(json.contains("\"code\":\"EVALIDATION\""));
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn test_status_api_response_with_full_data() {
        use crate::types::status::{BranchInfo, ChangesetInfo, PackageInfo};

        let data = StatusData::new(RepositoryInfo::monorepo("pnpm"), PackageManagerInfo::pnpm())
            .with_branch(BranchInfo::new("main"))
            .with_changesets(vec![ChangesetInfo::new("feature-1")])
            .with_packages(vec![PackageInfo::new("@org/pkg", "1.0.0", "packages/pkg")]);

        let response = StatusApiResponse::success(data);

        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert_eq!(data.repository.kind, "monorepo");
        assert_eq!(data.repository.monorepo_type, Some("pnpm".to_string()));
        assert!(data.branch.is_some());
        assert_eq!(data.changesets.len(), 1);
        assert_eq!(data.packages.len(), 1);
    }
}

mod status_types_tests {
    use crate::types::status::{
        BranchInfo, ChangesetInfo, PackageInfo, PackageManagerInfo, RepositoryInfo, StatusData,
        StatusParams,
    };

    // ========================================================================
    // StatusParams Tests
    // ========================================================================

    #[test]
    fn test_status_params_new() {
        let params = StatusParams::new("/path/to/workspace");
        assert_eq!(params.root, "/path/to/workspace");
        assert!(params.config_path.is_none());
    }

    #[test]
    fn test_status_params_with_config() {
        let params = StatusParams::with_config("/path/to/workspace", "/path/to/config.json");
        assert_eq!(params.root, "/path/to/workspace");
        assert_eq!(params.config_path, Some("/path/to/config.json".to_string()));
    }

    #[test]
    fn test_status_params_clone() {
        let params = StatusParams::with_config("/workspace", "/config.json");
        let cloned = params.clone();
        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.config_path, params.config_path);
    }

    #[test]
    fn test_status_params_debug() {
        let params = StatusParams::new("/workspace");
        let debug_str = format!("{params:?}");
        assert!(debug_str.contains("StatusParams"));
        assert!(debug_str.contains("/workspace"));
    }

    #[test]
    fn test_status_params_serialize() {
        let params = StatusParams::with_config("/workspace", "/config.json");
        let json = serde_json::to_string(&params).unwrap_or_default();
        assert!(json.contains("\"root\":\"/workspace\""));
        assert!(json.contains("\"config_path\":\"/config.json\""));
    }

    // ========================================================================
    // RepositoryInfo Tests
    // ========================================================================

    #[test]
    fn test_repository_info_simple() {
        let info = RepositoryInfo::simple();
        assert_eq!(info.kind, "simple");
        assert!(info.monorepo_type.is_none());
        assert!(info.is_simple());
        assert!(!info.is_monorepo());
    }

    #[test]
    fn test_repository_info_monorepo() {
        let info = RepositoryInfo::monorepo("pnpm");
        assert_eq!(info.kind, "monorepo");
        assert_eq!(info.monorepo_type, Some("pnpm".to_string()));
        assert!(!info.is_simple());
        assert!(info.is_monorepo());
    }

    #[test]
    fn test_repository_info_monorepo_types() {
        let npm = RepositoryInfo::monorepo("npm");
        assert_eq!(npm.monorepo_type, Some("npm".to_string()));

        let yarn = RepositoryInfo::monorepo("yarn");
        assert_eq!(yarn.monorepo_type, Some("yarn".to_string()));

        let bun = RepositoryInfo::monorepo("bun");
        assert_eq!(bun.monorepo_type, Some("bun".to_string()));

        let deno = RepositoryInfo::monorepo("deno");
        assert_eq!(deno.monorepo_type, Some("deno".to_string()));

        let custom = RepositoryInfo::monorepo("custom");
        assert_eq!(custom.monorepo_type, Some("custom".to_string()));
    }

    #[test]
    fn test_repository_info_unknown() {
        let info = RepositoryInfo::unknown();
        assert_eq!(info.kind, "unknown");
        assert!(info.monorepo_type.is_none());
        assert!(!info.is_simple());
        assert!(!info.is_monorepo());
    }

    #[test]
    fn test_repository_info_clone() {
        let info = RepositoryInfo::monorepo("pnpm");
        let cloned = info.clone();
        assert_eq!(cloned.kind, info.kind);
        assert_eq!(cloned.monorepo_type, info.monorepo_type);
    }

    #[test]
    fn test_repository_info_serialize() {
        let info = RepositoryInfo::monorepo("pnpm");
        let json = serde_json::to_string(&info).unwrap_or_default();
        assert!(json.contains("\"kind\":\"monorepo\""));
        assert!(json.contains("\"monorepo_type\":\"pnpm\""));
    }

    #[test]
    fn test_repository_info_serialize_simple() {
        let info = RepositoryInfo::simple();
        let json = serde_json::to_string(&info).unwrap_or_default();
        assert!(json.contains("\"kind\":\"simple\""));
        // monorepo_type should not be present when None
        assert!(!json.contains("monorepo_type"));
    }

    // ========================================================================
    // PackageManagerInfo Tests
    // ========================================================================

    #[test]
    fn test_package_manager_info_new() {
        let info = PackageManagerInfo::new("pnpm", "pnpm-lock.yaml");
        assert_eq!(info.name, "pnpm");
        assert_eq!(info.lock_file, "pnpm-lock.yaml");
    }

    #[test]
    fn test_package_manager_info_npm() {
        let info = PackageManagerInfo::npm();
        assert_eq!(info.name, "npm");
        assert_eq!(info.lock_file, "package-lock.json");
    }

    #[test]
    fn test_package_manager_info_yarn() {
        let info = PackageManagerInfo::yarn();
        assert_eq!(info.name, "yarn");
        assert_eq!(info.lock_file, "yarn.lock");
    }

    #[test]
    fn test_package_manager_info_pnpm() {
        let info = PackageManagerInfo::pnpm();
        assert_eq!(info.name, "pnpm");
        assert_eq!(info.lock_file, "pnpm-lock.yaml");
    }

    #[test]
    fn test_package_manager_info_bun() {
        let info = PackageManagerInfo::bun();
        assert_eq!(info.name, "bun");
        assert_eq!(info.lock_file, "bun.lockb");
    }

    #[test]
    fn test_package_manager_info_unknown() {
        let info = PackageManagerInfo::unknown();
        assert_eq!(info.name, "unknown");
        assert_eq!(info.lock_file, "");
    }

    #[test]
    fn test_package_manager_info_clone() {
        let info = PackageManagerInfo::pnpm();
        let cloned = info.clone();
        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.lock_file, info.lock_file);
    }

    #[test]
    fn test_package_manager_info_serialize() {
        let info = PackageManagerInfo::pnpm();
        let json = serde_json::to_string(&info).unwrap_or_default();
        assert!(json.contains("\"name\":\"pnpm\""));
        assert!(json.contains("\"lock_file\":\"pnpm-lock.yaml\""));
    }

    // ========================================================================
    // BranchInfo Tests
    // ========================================================================

    #[test]
    fn test_branch_info_new() {
        let branch = BranchInfo::new("main");
        assert_eq!(branch.name, "main");
    }

    #[test]
    fn test_branch_info_feature_branch() {
        let branch = BranchInfo::new("feature/add-login");
        assert_eq!(branch.name, "feature/add-login");
    }

    #[test]
    fn test_branch_info_clone() {
        let branch = BranchInfo::new("develop");
        let cloned = branch.clone();
        assert_eq!(cloned.name, branch.name);
    }

    #[test]
    fn test_branch_info_serialize() {
        let branch = BranchInfo::new("main");
        let json = serde_json::to_string(&branch).unwrap_or_default();
        assert!(json.contains("\"name\":\"main\""));
    }

    // ========================================================================
    // ChangesetInfo Tests
    // ========================================================================

    #[test]
    fn test_changeset_info_new() {
        let changeset = ChangesetInfo::new("feature-login");
        assert_eq!(changeset.id, "feature-login");
    }

    #[test]
    fn test_changeset_info_with_slashes() {
        let changeset = ChangesetInfo::new("feature/add-new-api");
        assert_eq!(changeset.id, "feature/add-new-api");
    }

    #[test]
    fn test_changeset_info_clone() {
        let changeset = ChangesetInfo::new("fix-bug");
        let cloned = changeset.clone();
        assert_eq!(cloned.id, changeset.id);
    }

    #[test]
    fn test_changeset_info_serialize() {
        let changeset = ChangesetInfo::new("feature-login");
        let json = serde_json::to_string(&changeset).unwrap_or_default();
        assert!(json.contains("\"id\":\"feature-login\""));
    }

    // ========================================================================
    // PackageInfo Tests
    // ========================================================================

    #[test]
    fn test_package_info_new() {
        let pkg = PackageInfo::new("@org/core", "1.2.3", "packages/core");
        assert_eq!(pkg.name, "@org/core");
        assert_eq!(pkg.version, "1.2.3");
        assert_eq!(pkg.path, "packages/core");
    }

    #[test]
    fn test_package_info_unscoped() {
        let pkg = PackageInfo::new("utils", "0.1.0", "packages/utils");
        assert_eq!(pkg.name, "utils");
        assert_eq!(pkg.version, "0.1.0");
        assert_eq!(pkg.path, "packages/utils");
    }

    #[test]
    fn test_package_info_root_path() {
        let pkg = PackageInfo::new("my-package", "1.0.0", ".");
        assert_eq!(pkg.path, ".");
    }

    #[test]
    fn test_package_info_prerelease_version() {
        let pkg = PackageInfo::new("@org/core", "1.0.0-beta.1", "packages/core");
        assert_eq!(pkg.version, "1.0.0-beta.1");
    }

    #[test]
    fn test_package_info_clone() {
        let pkg = PackageInfo::new("@org/core", "1.2.3", "packages/core");
        let cloned = pkg.clone();
        assert_eq!(cloned.name, pkg.name);
        assert_eq!(cloned.version, pkg.version);
        assert_eq!(cloned.path, pkg.path);
    }

    #[test]
    fn test_package_info_serialize() {
        let pkg = PackageInfo::new("@org/core", "1.2.3", "packages/core");
        let json = serde_json::to_string(&pkg).unwrap_or_default();
        assert!(json.contains("\"name\":\"@org/core\""));
        assert!(json.contains("\"version\":\"1.2.3\""));
        assert!(json.contains("\"path\":\"packages/core\""));
    }

    // ========================================================================
    // StatusData Tests
    // ========================================================================

    #[test]
    fn test_status_data_new() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::pnpm());
        assert!(data.repository.is_simple());
        assert_eq!(data.package_manager.name, "pnpm");
        assert!(data.branch.is_none());
        assert!(data.changesets.is_empty());
        assert!(data.packages.is_empty());
    }

    #[test]
    fn test_status_data_with_branch() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::pnpm())
            .with_branch(BranchInfo::new("main"));
        assert!(data.branch.is_some());
        assert_eq!(data.branch.as_ref().map(|b| b.name.as_str()), Some("main"));
    }

    #[test]
    fn test_status_data_with_changesets() {
        let changesets = vec![ChangesetInfo::new("feature-a"), ChangesetInfo::new("feature-b")];
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::pnpm())
            .with_changesets(changesets);
        assert_eq!(data.changesets.len(), 2);
        assert_eq!(data.changesets[0].id, "feature-a");
        assert_eq!(data.changesets[1].id, "feature-b");
    }

    #[test]
    fn test_status_data_with_packages() {
        let packages = vec![
            PackageInfo::new("@org/core", "1.0.0", "packages/core"),
            PackageInfo::new("@org/utils", "0.5.0", "packages/utils"),
        ];
        let data = StatusData::new(RepositoryInfo::monorepo("pnpm"), PackageManagerInfo::pnpm())
            .with_packages(packages);
        assert_eq!(data.packages.len(), 2);
        assert_eq!(data.packages[0].name, "@org/core");
        assert_eq!(data.packages[1].name, "@org/utils");
    }

    #[test]
    fn test_status_data_builder_chain() {
        let data = StatusData::new(RepositoryInfo::monorepo("pnpm"), PackageManagerInfo::pnpm())
            .with_branch(BranchInfo::new("main"))
            .with_changesets(vec![ChangesetInfo::new("feature-x")])
            .with_packages(vec![PackageInfo::new("@org/core", "1.0.0", "packages/core")]);

        assert!(data.repository.is_monorepo());
        assert_eq!(data.repository.monorepo_type, Some("pnpm".to_string()));
        assert_eq!(data.package_manager.name, "pnpm");
        assert!(data.branch.is_some());
        assert_eq!(data.changesets.len(), 1);
        assert_eq!(data.packages.len(), 1);
    }

    #[test]
    fn test_status_data_clone() {
        let data = StatusData::new(RepositoryInfo::monorepo("pnpm"), PackageManagerInfo::pnpm())
            .with_branch(BranchInfo::new("main"))
            .with_packages(vec![PackageInfo::new("@org/core", "1.0.0", "packages/core")]);

        let cloned = data.clone();
        assert_eq!(cloned.repository.kind, data.repository.kind);
        assert_eq!(cloned.package_manager.name, data.package_manager.name);
        assert_eq!(
            cloned.branch.as_ref().map(|b| b.name.as_str()),
            data.branch.as_ref().map(|b| b.name.as_str())
        );
        assert_eq!(cloned.packages.len(), data.packages.len());
    }

    #[test]
    fn test_status_data_serialize() {
        let data = StatusData::new(RepositoryInfo::monorepo("pnpm"), PackageManagerInfo::pnpm())
            .with_branch(BranchInfo::new("main"))
            .with_changesets(vec![ChangesetInfo::new("feature-x")])
            .with_packages(vec![PackageInfo::new("@org/core", "1.0.0", "packages/core")]);

        let json = serde_json::to_string(&data).unwrap_or_default();
        assert!(json.contains("\"kind\":\"monorepo\""));
        assert!(json.contains("\"monorepo_type\":\"pnpm\""));
        assert!(json.contains("\"name\":\"pnpm\""));
        assert!(json.contains("\"name\":\"main\""));
        assert!(json.contains("\"id\":\"feature-x\""));
        assert!(json.contains("\"name\":\"@org/core\""));
    }

    #[test]
    fn test_status_data_serialize_without_optional_fields() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::npm());

        let json = serde_json::to_string(&data).unwrap_or_default();
        assert!(json.contains("\"kind\":\"simple\""));
        // branch should not be present when None
        assert!(!json.contains("\"branch\":null"));
        // changesets and packages should be empty arrays
        assert!(json.contains("\"changesets\":[]"));
        assert!(json.contains("\"packages\":[]"));
    }

    // ========================================================================
    // Integration-style Tests
    // ========================================================================

    #[test]
    fn test_complete_status_response_simple_repo() {
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::npm())
            .with_branch(BranchInfo::new("main"))
            .with_packages(vec![PackageInfo::new("my-app", "1.0.0", ".")]);

        assert!(data.repository.is_simple());
        assert!(!data.repository.is_monorepo());
        assert_eq!(data.package_manager.name, "npm");
        assert_eq!(data.package_manager.lock_file, "package-lock.json");
        assert_eq!(data.branch.as_ref().map(|b| b.name.as_str()), Some("main"));
        assert!(data.changesets.is_empty());
        assert_eq!(data.packages.len(), 1);
        assert_eq!(data.packages[0].path, ".");
    }

    #[test]
    fn test_complete_status_response_monorepo() {
        let data = StatusData::new(RepositoryInfo::monorepo("pnpm"), PackageManagerInfo::pnpm())
            .with_branch(BranchInfo::new("develop"))
            .with_changesets(vec![
                ChangesetInfo::new("feature/add-auth"),
                ChangesetInfo::new("fix/memory-leak"),
            ])
            .with_packages(vec![
                PackageInfo::new("@myorg/core", "2.0.0", "packages/core"),
                PackageInfo::new("@myorg/utils", "1.5.0", "packages/utils"),
                PackageInfo::new("@myorg/cli", "1.0.0-beta.1", "packages/cli"),
            ]);

        assert!(data.repository.is_monorepo());
        assert_eq!(data.repository.monorepo_type, Some("pnpm".to_string()));
        assert_eq!(data.package_manager.name, "pnpm");
        assert_eq!(data.package_manager.lock_file, "pnpm-lock.yaml");
        assert_eq!(data.branch.as_ref().map(|b| b.name.as_str()), Some("develop"));
        assert_eq!(data.changesets.len(), 2);
        assert_eq!(data.packages.len(), 3);
    }

    #[test]
    fn test_status_response_without_git() {
        // Simulate a response when Git is not available
        let data = StatusData::new(RepositoryInfo::simple(), PackageManagerInfo::npm())
            .with_packages(vec![PackageInfo::new("my-package", "1.0.0", ".")]);

        assert!(data.branch.is_none());
        assert!(data.changesets.is_empty());
    }
}
