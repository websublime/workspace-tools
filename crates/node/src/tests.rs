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
    use crate::{VERSION, get_version};

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
    use crate::response::{ApiResponse, ApiResponseExt, JsonResponse, result_to_response};
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

// ============================================================================
// Init Types Tests (Story 3.3)
// ============================================================================

mod init_api_response_tests {
    use crate::error::ErrorInfo;
    use crate::types::init::{InitApiResponse, InitData};

    #[test]
    fn test_init_api_response_success() {
        let data = InitData::with_defaults("repo.config.json", "independent");
        let response = InitApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
        assert!(response.is_success());
        assert!(!response.is_failure());
    }

    #[test]
    fn test_init_api_response_failure() {
        let error = ErrorInfo::validation("Invalid strategy", Some("strategy"));
        let response = InitApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert!(!response.is_success());
        assert!(response.is_failure());
    }

    #[test]
    fn test_init_api_response_failure_with_different_error_codes() {
        let config_error = ErrorInfo::configuration("Config already exists");
        let response1 = InitApiResponse::failure(config_error);
        assert_eq!(response1.error.as_ref().map(|e| e.code.as_str()), Some("ECONFIG"));

        let not_found_error = ErrorInfo::not_found("Path not found", Some("/invalid"));
        let response2 = InitApiResponse::failure(not_found_error);
        assert_eq!(response2.error.as_ref().map(|e| e.code.as_str()), Some("ENOENT"));

        let validation_error = ErrorInfo::validation("Invalid format", Some("configFormat"));
        let response3 = InitApiResponse::failure(validation_error);
        assert_eq!(response3.error.as_ref().map(|e| e.code.as_str()), Some("EVALIDATION"));
    }

    #[test]
    fn test_init_api_response_clone() {
        let data = InitData::with_defaults("repo.config.toml", "unified");
        let response = InitApiResponse::success(data);
        let cloned = response.clone();

        assert_eq!(cloned.success, response.success);
        assert!(cloned.data.is_some());
    }

    #[test]
    fn test_init_api_response_debug() {
        let data = InitData::with_defaults("repo.config.json", "independent");
        let response = InitApiResponse::success(data);
        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("InitApiResponse"));
        assert!(debug_str.contains("success: true"));
    }

    #[test]
    fn test_init_api_response_serialize_success() {
        let data = InitData::with_defaults("repo.config.json", "independent");
        let response = InitApiResponse::success(data);
        let json = serde_json::to_string(&response).unwrap_or_default();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"config_file\":\"repo.config.json\""));
    }

    #[test]
    fn test_init_api_response_serialize_failure() {
        let error = ErrorInfo::validation("Invalid strategy", Some("strategy"));
        let response = InitApiResponse::failure(error);
        let json = serde_json::to_string(&response).unwrap_or_default();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"code\":\"EVALIDATION\""));
    }
}

mod init_types_tests {
    use crate::types::init::{InitData, InitParams, VALID_CONFIG_FORMATS, VALID_STRATEGIES};

    // ========================================================================
    // InitParams Tests
    // ========================================================================

    #[test]
    fn test_init_params_new() {
        let params = InitParams::new("/path/to/workspace");
        assert_eq!(params.root, "/path/to/workspace");
        assert!(params.changeset_path.is_none());
        assert!(params.environments.is_none());
        assert!(params.default_env.is_none());
        assert!(params.strategy.is_none());
        assert!(params.registry.is_none());
        assert!(params.config_format.is_none());
        assert!(params.force.is_none());
    }

    #[test]
    fn test_init_params_with_changeset_path() {
        let params = InitParams::new("/workspace").with_changeset_path(".changesets");
        assert_eq!(params.changeset_path, Some(".changesets".to_string()));
    }

    #[test]
    fn test_init_params_with_environments() {
        let params =
            InitParams::new("/workspace").with_environments(vec!["dev", "staging", "prod"]);
        assert_eq!(
            params.environments,
            Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()])
        );
    }

    #[test]
    fn test_init_params_with_default_env() {
        let params = InitParams::new("/workspace").with_default_env(vec!["prod"]);
        assert_eq!(params.default_env, Some(vec!["prod".to_string()]));
    }

    #[test]
    fn test_init_params_with_strategy() {
        let params = InitParams::new("/workspace").with_strategy("independent");
        assert_eq!(params.strategy, Some("independent".to_string()));
    }

    #[test]
    fn test_init_params_with_registry() {
        let params = InitParams::new("/workspace").with_registry("https://npm.pkg.github.com");
        assert_eq!(params.registry, Some("https://npm.pkg.github.com".to_string()));
    }

    #[test]
    fn test_init_params_with_config_format() {
        let params = InitParams::new("/workspace").with_config_format("toml");
        assert_eq!(params.config_format, Some("toml".to_string()));
    }

    #[test]
    fn test_init_params_with_force() {
        let params = InitParams::new("/workspace").with_force(true);
        assert_eq!(params.force, Some(true));
    }

    #[test]
    fn test_init_params_builder_chain() {
        let params = InitParams::new("/workspace")
            .with_changeset_path(".changesets")
            .with_environments(vec!["dev", "prod"])
            .with_default_env(vec!["prod"])
            .with_strategy("independent")
            .with_registry("https://registry.npmjs.org")
            .with_config_format("toml")
            .with_force(false);

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.changeset_path, Some(".changesets".to_string()));
        assert_eq!(params.environments, Some(vec!["dev".to_string(), "prod".to_string()]));
        assert_eq!(params.default_env, Some(vec!["prod".to_string()]));
        assert_eq!(params.strategy, Some("independent".to_string()));
        assert_eq!(params.registry, Some("https://registry.npmjs.org".to_string()));
        assert_eq!(params.config_format, Some("toml".to_string()));
        assert_eq!(params.force, Some(false));
    }

    #[test]
    fn test_init_params_clone() {
        let params = InitParams::new("/workspace").with_strategy("unified").with_force(true);
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.strategy, params.strategy);
        assert_eq!(cloned.force, params.force);
    }

    #[test]
    fn test_init_params_debug() {
        let params = InitParams::new("/workspace");
        let debug_str = format!("{params:?}");
        assert!(debug_str.contains("InitParams"));
        assert!(debug_str.contains("/workspace"));
    }

    #[test]
    fn test_init_params_serialize() {
        let params = InitParams::new("/workspace").with_strategy("independent").with_force(true);
        let json = serde_json::to_string(&params).unwrap_or_default();
        assert!(json.contains("\"root\":\"/workspace\""));
        assert!(json.contains("\"strategy\":\"independent\""));
        assert!(json.contains("\"force\":true"));
    }

    #[test]
    fn test_init_params_serialize_skips_none_fields() {
        let params = InitParams::new("/workspace");
        let json = serde_json::to_string(&params).unwrap_or_default();
        assert!(json.contains("\"root\":\"/workspace\""));
        // None fields should be skipped
        assert!(!json.contains("changeset_path"));
        assert!(!json.contains("environments"));
        assert!(!json.contains("strategy"));
    }

    // ========================================================================
    // InitData Tests
    // ========================================================================

    #[test]
    fn test_init_data_new() {
        let data = InitData::new(
            "repo.config.toml",
            "toml",
            "independent",
            ".changesets",
            vec!["dev".to_string(), "prod".to_string()],
            vec!["prod".to_string()],
            "https://registry.npmjs.org",
        );

        assert_eq!(data.config_file, "repo.config.toml");
        assert_eq!(data.config_format, "toml");
        assert_eq!(data.strategy, "independent");
        assert_eq!(data.changeset_path, ".changesets");
        assert_eq!(data.environments, vec!["dev", "prod"]);
        assert_eq!(data.default_environments, vec!["prod"]);
        assert_eq!(data.registry, "https://registry.npmjs.org");
    }

    #[test]
    fn test_init_data_with_defaults_json() {
        let data = InitData::with_defaults("repo.config.json", "unified");

        assert_eq!(data.config_file, "repo.config.json");
        assert_eq!(data.config_format, "json");
        assert_eq!(data.strategy, "unified");
        assert_eq!(data.changeset_path, ".changesets");
        assert!(data.environments.is_empty());
        assert!(data.default_environments.is_empty());
        assert_eq!(data.registry, "https://registry.npmjs.org");
    }

    #[test]
    fn test_init_data_with_defaults_toml() {
        let data = InitData::with_defaults("repo.config.toml", "independent");

        assert_eq!(data.config_file, "repo.config.toml");
        assert_eq!(data.config_format, "toml");
        assert_eq!(data.strategy, "independent");
    }

    #[test]
    fn test_init_data_with_defaults_yaml() {
        let data = InitData::with_defaults("repo.config.yaml", "unified");

        assert_eq!(data.config_file, "repo.config.yaml");
        assert_eq!(data.config_format, "yaml");
    }

    #[test]
    fn test_init_data_with_defaults_yml() {
        let data = InitData::with_defaults("repo.config.yml", "unified");

        assert_eq!(data.config_file, "repo.config.yml");
        assert_eq!(data.config_format, "yaml");
    }

    #[test]
    fn test_init_data_with_defaults_unknown_extension() {
        let data = InitData::with_defaults("repo.config", "independent");

        assert_eq!(data.config_file, "repo.config");
        assert_eq!(data.config_format, "json"); // Defaults to json
    }

    #[test]
    fn test_init_data_with_defaults_case_insensitive() {
        let data1 = InitData::with_defaults("repo.config.TOML", "independent");
        assert_eq!(data1.config_format, "toml");

        let data2 = InitData::with_defaults("repo.config.YAML", "unified");
        assert_eq!(data2.config_format, "yaml");

        let data3 = InitData::with_defaults("repo.config.JSON", "independent");
        assert_eq!(data3.config_format, "json");
    }

    #[test]
    fn test_init_data_clone() {
        let data = InitData::new(
            "repo.config.toml",
            "toml",
            "independent",
            ".changesets",
            vec!["dev".to_string()],
            vec!["dev".to_string()],
            "https://registry.npmjs.org",
        );
        let cloned = data.clone();

        assert_eq!(cloned.config_file, data.config_file);
        assert_eq!(cloned.config_format, data.config_format);
        assert_eq!(cloned.strategy, data.strategy);
        assert_eq!(cloned.environments, data.environments);
    }

    #[test]
    fn test_init_data_debug() {
        let data = InitData::with_defaults("repo.config.json", "independent");
        let debug_str = format!("{data:?}");
        assert!(debug_str.contains("InitData"));
        assert!(debug_str.contains("repo.config.json"));
    }

    #[test]
    fn test_init_data_serialize() {
        let data = InitData::new(
            "repo.config.toml",
            "toml",
            "independent",
            ".changesets",
            vec!["dev".to_string(), "prod".to_string()],
            vec!["prod".to_string()],
            "https://registry.npmjs.org",
        );
        let json = serde_json::to_string(&data).unwrap_or_default();
        assert!(json.contains("\"config_file\":\"repo.config.toml\""));
        assert!(json.contains("\"config_format\":\"toml\""));
        assert!(json.contains("\"strategy\":\"independent\""));
        assert!(json.contains("\"changeset_path\":\".changesets\""));
        assert!(json.contains("\"environments\":[\"dev\",\"prod\"]"));
        assert!(json.contains("\"default_environments\":[\"prod\"]"));
        assert!(json.contains("\"registry\":\"https://registry.npmjs.org\""));
    }

    // ========================================================================
    // Validation Constants Tests
    // ========================================================================

    #[test]
    fn test_valid_strategies() {
        assert!(VALID_STRATEGIES.contains(&"independent"));
        assert!(VALID_STRATEGIES.contains(&"unified"));
        assert_eq!(VALID_STRATEGIES.len(), 2);
    }

    #[test]
    fn test_valid_config_formats() {
        assert!(VALID_CONFIG_FORMATS.contains(&"json"));
        assert!(VALID_CONFIG_FORMATS.contains(&"yaml"));
        assert!(VALID_CONFIG_FORMATS.contains(&"toml"));
        assert_eq!(VALID_CONFIG_FORMATS.len(), 3);
    }

    #[test]
    fn test_invalid_strategy_not_in_list() {
        assert!(!VALID_STRATEGIES.contains(&"fixed"));
        assert!(!VALID_STRATEGIES.contains(&""));
        assert!(!VALID_STRATEGIES.contains(&"INDEPENDENT")); // Case sensitive
    }

    #[test]
    fn test_invalid_config_format_not_in_list() {
        assert!(!VALID_CONFIG_FORMATS.contains(&"xml"));
        assert!(!VALID_CONFIG_FORMATS.contains(&""));
        assert!(!VALID_CONFIG_FORMATS.contains(&"JSON")); // Case sensitive
    }

    // ========================================================================
    // Complete Initialization Scenario Tests
    // ========================================================================

    #[test]
    fn test_complete_init_scenario_monorepo() {
        // Simulate a complete init for a monorepo with all options
        let params = InitParams::new("/projects/my-monorepo")
            .with_changeset_path(".changesets")
            .with_environments(vec!["dev", "staging", "prod"])
            .with_default_env(vec!["prod"])
            .with_strategy("independent")
            .with_registry("https://registry.npmjs.org")
            .with_config_format("toml")
            .with_force(false);

        // Verify all params are correctly set
        assert_eq!(params.root, "/projects/my-monorepo");
        assert_eq!(params.changeset_path, Some(".changesets".to_string()));
        assert_eq!(params.environments.as_ref().map(Vec::len), Some(3));
        assert_eq!(params.strategy, Some("independent".to_string()));
        assert_eq!(params.config_format, Some("toml".to_string()));
        assert_eq!(params.force, Some(false));
    }

    #[test]
    fn test_complete_init_scenario_simple_project() {
        // Simulate a minimal init for a simple project
        let params = InitParams::new(".").with_strategy("unified");

        assert_eq!(params.root, ".");
        assert!(params.changeset_path.is_none()); // Use default
        assert!(params.environments.is_none()); // No custom environments
        assert_eq!(params.strategy, Some("unified".to_string()));
    }

    #[test]
    fn test_complete_init_response_scenario() {
        // Simulate a complete init response
        let data = InitData::new(
            "repo.config.toml",
            "toml",
            "independent",
            ".changesets",
            vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
            vec!["prod".to_string()],
            "https://registry.npmjs.org",
        );

        assert_eq!(data.config_file, "repo.config.toml");
        assert_eq!(data.config_format, "toml");
        assert_eq!(data.strategy, "independent");
        assert_eq!(data.changeset_path, ".changesets");
        assert_eq!(data.environments.len(), 3);
        assert_eq!(data.default_environments.len(), 1);
        assert_eq!(data.registry, "https://registry.npmjs.org");
    }

    #[test]
    fn test_init_data_empty_environments() {
        // Test that empty environments are valid
        let data = InitData::new(
            "repo.config.json",
            "json",
            "unified",
            ".changesets",
            vec![],
            vec![],
            "https://registry.npmjs.org",
        );

        assert!(data.environments.is_empty());
        assert!(data.default_environments.is_empty());
    }

    #[test]
    fn test_init_params_github_packages_registry() {
        // Test with GitHub Packages registry
        let params =
            InitParams::new("/workspace").with_registry("https://npm.pkg.github.com/@myorg");

        assert_eq!(params.registry, Some("https://npm.pkg.github.com/@myorg".to_string()));
    }
}

// ============================================================================
// Changeset Types Tests (Story 4.1)
// ============================================================================

mod changeset_api_response_tests {
    use crate::error::ErrorInfo;
    use crate::types::changeset::{
        ChangesetAddApiResponse, ChangesetAddData, ChangesetCheckApiResponse, ChangesetCheckData,
        ChangesetDetailInfo, ChangesetHistoryApiResponse, ChangesetHistoryData,
        ChangesetListApiResponse, ChangesetListData, ChangesetListItemInfo,
        ChangesetRemoveApiResponse, ChangesetRemoveData, ChangesetShowApiResponse,
        ChangesetShowData, ChangesetUpdateApiResponse, ChangesetUpdateData, UpdateSummaryInfo,
    };

    // ========================================================================
    // ChangesetAddApiResponse Tests
    // ========================================================================

    #[test]
    fn test_changeset_add_api_response_success() {
        let data =
            ChangesetAddData::new("feature-api", "feature/api", "minor", "2024-01-15T10:30:00Z")
                .with_packages(vec!["@scope/pkg1".to_string()])
                .with_environments(vec!["staging".to_string()]);
        let response = ChangesetAddApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
        assert!(response.is_success());
        assert!(!response.is_failure());
    }

    #[test]
    fn test_changeset_add_api_response_failure() {
        let error = ErrorInfo::validation("Invalid bump type", Some("bump"));
        let response = ChangesetAddApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert!(!response.is_success());
        assert!(response.is_failure());
    }

    #[test]
    fn test_changeset_add_api_response_clone() {
        let data = ChangesetAddData::new("test-id", "test-branch", "patch", "2024-01-15T10:30:00Z");
        let response = ChangesetAddApiResponse::success(data);
        let cloned = response.clone();

        assert_eq!(cloned.success, response.success);
        assert!(cloned.data.is_some());
    }

    #[test]
    fn test_changeset_add_api_response_serialize() {
        let data = ChangesetAddData::new("test-id", "test-branch", "minor", "2024-01-15T10:30:00Z");
        let response = ChangesetAddApiResponse::success(data);
        let json = serde_json::to_string(&response).unwrap_or_default();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"id\":\"test-id\""));
    }

    // ========================================================================
    // ChangesetUpdateApiResponse Tests
    // ========================================================================

    #[test]
    fn test_changeset_update_api_response_success() {
        let summary = UpdateSummaryInfo::new(2, 1, true, 1);
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "major",
            "2024-01-15T10:00:00Z",
            "2024-01-15T12:00:00Z",
        );
        let data = ChangesetUpdateData::success(summary, changeset);
        let response = ChangesetUpdateApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.is_success());
    }

    #[test]
    fn test_changeset_update_api_response_no_changes() {
        let summary = UpdateSummaryInfo::empty();
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "minor",
            "2024-01-15T10:00:00Z",
            "2024-01-15T10:00:00Z",
        );
        let data = ChangesetUpdateData::new(false, summary, changeset);
        let response = ChangesetUpdateApiResponse::success(data);

        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert!(!data.updated);
        assert_eq!(data.summary.commits_added, 0);
    }

    #[test]
    fn test_changeset_update_api_response_failure() {
        let error = ErrorInfo::not_found("Changeset not found", Some("feature/old"));
        let response = ChangesetUpdateApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("ENOENT"));
    }

    // ========================================================================
    // ChangesetListApiResponse Tests
    // ========================================================================

    #[test]
    fn test_changeset_list_api_response_success_with_items() {
        let changeset = ChangesetListItemInfo::new(
            "cs-1",
            "feature/one",
            "minor",
            3, // commit_count
            "2024-01-15T10:00:00Z",
            "2024-01-15T10:00:00Z",
        );
        let data = ChangesetListData::new(vec![changeset]);
        let response = ChangesetListApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.as_ref().unwrap().count, 1);
        assert_eq!(response.data.as_ref().unwrap().changesets[0].commit_count, 3);
    }

    #[test]
    fn test_changeset_list_api_response_success_empty() {
        let data = ChangesetListData::empty();
        let response = ChangesetListApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.as_ref().unwrap().count, 0);
        assert!(response.data.as_ref().unwrap().changesets.is_empty());
    }

    #[test]
    fn test_changeset_list_api_response_failure() {
        let error = ErrorInfo::configuration("Invalid config path");
        let response = ChangesetListApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.is_failure());
    }

    // ========================================================================
    // ChangesetShowApiResponse Tests
    // ========================================================================

    #[test]
    fn test_changeset_show_api_response_success() {
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "major",
            "2024-01-15T10:00:00Z",
            "2024-01-16T14:30:00Z",
        )
        .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
        .with_environments(vec!["production".to_string()])
        .with_commits(vec!["abc123".to_string(), "def456".to_string()])
        .with_message("Breaking API change");

        let data = ChangesetShowData::new(changeset);
        let response = ChangesetShowApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        let show_data = response.data.as_ref().unwrap();
        assert_eq!(show_data.changeset.id, "feature-api");
        assert_eq!(show_data.changeset.packages.len(), 2);
        assert_eq!(show_data.changeset.message, Some("Breaking API change".to_string()));
    }

    #[test]
    fn test_changeset_show_api_response_failure() {
        let error = ErrorInfo::not_found("Changeset not found", Some("feature/nonexistent"));
        let response = ChangesetShowApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    // ========================================================================
    // ChangesetRemoveApiResponse Tests
    // ========================================================================

    #[test]
    fn test_changeset_remove_api_response_success() {
        let data = ChangesetRemoveData::success("feature/old");
        let response = ChangesetRemoveApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.data.as_ref().unwrap().removed);
        assert_eq!(response.data.as_ref().unwrap().branch, "feature/old");
    }

    #[test]
    fn test_changeset_remove_api_response_failure() {
        let error = ErrorInfo::validation("Cannot remove changeset in production", None::<String>);
        let response = ChangesetRemoveApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.is_failure());
    }

    // ========================================================================
    // ChangesetHistoryApiResponse Tests
    // ========================================================================

    #[test]
    fn test_changeset_history_api_response_success_empty() {
        let data = ChangesetHistoryData::empty();
        let response = ChangesetHistoryApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.as_ref().unwrap().count, 0);
    }

    #[test]
    fn test_changeset_history_api_response_failure() {
        let error = ErrorInfo::io("Cannot read history file", None::<String>);
        let response = ChangesetHistoryApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    // ========================================================================
    // ChangesetCheckApiResponse Tests
    // ========================================================================

    #[test]
    fn test_changeset_check_api_response_exists() {
        let data = ChangesetCheckData::exists("feature/api");
        let response = ChangesetCheckApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        let check_data = response.data.as_ref().unwrap();
        assert!(check_data.has_changeset);
        assert_eq!(check_data.branch, Some("feature/api".to_string()));
    }

    #[test]
    fn test_changeset_check_api_response_not_found() {
        let data = ChangesetCheckData::not_found();
        let response = ChangesetCheckApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        let check_data = response.data.as_ref().unwrap();
        assert!(!check_data.has_changeset);
        assert!(check_data.branch.is_none());
    }

    #[test]
    fn test_changeset_check_api_response_failure() {
        let error = ErrorInfo::git("Cannot determine current branch");
        let response = ChangesetCheckApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EGIT"));
    }
}

mod changeset_params_tests {
    use crate::types::changeset::{
        ChangesetAddParams, ChangesetCheckParams, ChangesetHistoryParams, ChangesetListParams,
        ChangesetRemoveParams, ChangesetShowParams, ChangesetUpdateParams, VALID_SORT_OPTIONS,
    };

    // ========================================================================
    // ChangesetAddParams Tests
    // ========================================================================

    #[test]
    fn test_changeset_add_params_new() {
        let params = ChangesetAddParams::new(".");

        assert_eq!(params.root, ".");
        assert!(params.config_path.is_none());
        assert!(params.bump.is_none());
        assert!(params.environments.is_none());
        assert!(params.branch.is_none());
        assert!(params.message.is_none());
        assert!(params.packages.is_none());
        assert!(params.force.is_none());
    }

    #[test]
    fn test_changeset_add_params_builder_chain() {
        let params = ChangesetAddParams::new("/workspace")
            .with_config_path("/workspace/repo.config.json")
            .with_bump("minor")
            .with_environments(vec!["staging".to_string(), "production".to_string()])
            .with_branch("feature/new-api")
            .with_message("Add new REST API endpoints")
            .with_packages(vec!["@scope/api".to_string(), "@scope/client".to_string()])
            .with_force(true);

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.config_path, Some("/workspace/repo.config.json".to_string()));
        assert_eq!(params.bump, Some("minor".to_string()));
        assert_eq!(
            params.environments,
            Some(vec!["staging".to_string(), "production".to_string()])
        );
        assert_eq!(params.branch, Some("feature/new-api".to_string()));
        assert_eq!(params.message, Some("Add new REST API endpoints".to_string()));
        assert_eq!(
            params.packages,
            Some(vec!["@scope/api".to_string(), "@scope/client".to_string()])
        );
        assert_eq!(params.force, Some(true));
    }

    #[test]
    fn test_changeset_add_params_clone() {
        let params = ChangesetAddParams::new("/workspace")
            .with_bump("major")
            .with_packages(vec!["@scope/core".to_string()]);
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.bump, params.bump);
        assert_eq!(cloned.packages, params.packages);
    }

    #[test]
    fn test_changeset_add_params_serialize() {
        let params = ChangesetAddParams::new("/workspace").with_bump("patch");
        let json = serde_json::to_string(&params).unwrap_or_default();

        assert!(json.contains("\"root\":\"/workspace\""));
        assert!(json.contains("\"bump\":\"patch\""));
        // Optional fields that are None should not be present
        assert!(!json.contains("\"config_path\""));
    }

    // ========================================================================
    // ChangesetUpdateParams Tests
    // ========================================================================

    #[test]
    fn test_changeset_update_params_new() {
        let params = ChangesetUpdateParams::new(".");

        assert_eq!(params.root, ".");
        assert!(params.id.is_none());
        assert!(params.commit.is_none());
        assert!(params.packages.is_none());
        assert!(params.bump.is_none());
        assert!(params.environments.is_none());
    }

    #[test]
    fn test_changeset_update_params_builder_chain() {
        let params = ChangesetUpdateParams::new("/workspace")
            .with_id("feature/api")
            .with_commit("abc123def456")
            .with_packages(vec!["@scope/new".to_string()])
            .with_bump("major")
            .with_environments(vec!["production".to_string()]);

        assert_eq!(params.id, Some("feature/api".to_string()));
        assert_eq!(params.commit, Some("abc123def456".to_string()));
        assert_eq!(params.packages, Some(vec!["@scope/new".to_string()]));
        assert_eq!(params.bump, Some("major".to_string()));
        assert_eq!(params.environments, Some(vec!["production".to_string()]));
    }

    // ========================================================================
    // ChangesetListParams Tests
    // ========================================================================

    #[test]
    fn test_changeset_list_params_new() {
        let params = ChangesetListParams::new(".");

        assert_eq!(params.root, ".");
        assert!(params.filter_package.is_none());
        assert!(params.filter_bump.is_none());
        assert!(params.filter_env.is_none());
        assert!(params.sort.is_none());
    }

    #[test]
    fn test_changeset_list_params_with_filters() {
        let params = ChangesetListParams::new("/workspace")
            .with_filter_package("@scope/core")
            .with_filter_bump("major")
            .with_filter_env("production")
            .with_sort("date");

        assert_eq!(params.filter_package, Some("@scope/core".to_string()));
        assert_eq!(params.filter_bump, Some("major".to_string()));
        assert_eq!(params.filter_env, Some("production".to_string()));
        assert_eq!(params.sort, Some("date".to_string()));
    }

    #[test]
    fn test_valid_sort_options() {
        assert!(VALID_SORT_OPTIONS.contains(&"date"));
        assert!(VALID_SORT_OPTIONS.contains(&"bump"));
        assert!(VALID_SORT_OPTIONS.contains(&"branch"));
        assert!(!VALID_SORT_OPTIONS.contains(&"invalid"));
    }

    // ========================================================================
    // ChangesetShowParams Tests
    // ========================================================================

    #[test]
    fn test_changeset_show_params_new() {
        let params = ChangesetShowParams::new(".", "feature/api");

        assert_eq!(params.root, ".");
        assert_eq!(params.branch, "feature/api");
        assert!(params.config_path.is_none());
    }

    #[test]
    fn test_changeset_show_params_with_config() {
        let params = ChangesetShowParams::new("/workspace", "feature/api")
            .with_config_path("/workspace/repo.config.json");

        assert_eq!(params.config_path, Some("/workspace/repo.config.json".to_string()));
    }

    // ========================================================================
    // ChangesetRemoveParams Tests
    // ========================================================================

    #[test]
    fn test_changeset_remove_params_new() {
        let params = ChangesetRemoveParams::new(".", "feature/old");

        assert_eq!(params.root, ".");
        assert_eq!(params.branch, "feature/old");
        assert!(params.force.is_none());
    }

    #[test]
    fn test_changeset_remove_params_with_force() {
        let params = ChangesetRemoveParams::new("/workspace", "feature/old").with_force(true);

        assert_eq!(params.force, Some(true));
    }

    // ========================================================================
    // ChangesetHistoryParams Tests
    // ========================================================================

    #[test]
    fn test_changeset_history_params_new() {
        let params = ChangesetHistoryParams::new(".");

        assert_eq!(params.root, ".");
        assert!(params.filter_package.is_none());
        assert!(params.filter_env.is_none());
        assert!(params.filter_bump.is_none());
        assert!(params.since.is_none());
        assert!(params.until.is_none());
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_changeset_history_params_builder_chain() {
        let params = ChangesetHistoryParams::new("/workspace")
            .with_filter_package("@scope/core")
            .with_filter_env("production")
            .with_filter_bump("major")
            .with_since("2024-01-01")
            .with_until("2024-12-31")
            .with_limit(50);

        assert_eq!(params.filter_package, Some("@scope/core".to_string()));
        assert_eq!(params.filter_env, Some("production".to_string()));
        assert_eq!(params.filter_bump, Some("major".to_string()));
        assert_eq!(params.since, Some("2024-01-01".to_string()));
        assert_eq!(params.until, Some("2024-12-31".to_string()));
        assert_eq!(params.limit, Some(50));
    }

    // ========================================================================
    // ChangesetCheckParams Tests
    // ========================================================================

    #[test]
    fn test_changeset_check_params_new() {
        let params = ChangesetCheckParams::new(".");

        assert_eq!(params.root, ".");
        assert!(params.branch.is_none());
    }

    #[test]
    fn test_changeset_check_params_with_branch() {
        let params = ChangesetCheckParams::new("/workspace").with_branch("feature/api");

        assert_eq!(params.branch, Some("feature/api".to_string()));
    }
}

mod changeset_data_tests {
    use crate::types::changeset::{
        ArchivedChangesetInfo, ChangesetAddData, ChangesetCheckData, ChangesetDetailInfo,
        ChangesetHistoryData, ChangesetListData, ChangesetListItemInfo, ChangesetRemoveData,
        ChangesetShowData, ChangesetUpdateData, ReleaseInfoData, ReleasedVersionEntry,
        UpdateSummaryInfo,
    };

    // ========================================================================
    // ChangesetDetailInfo Tests
    // ========================================================================

    #[test]
    fn test_changeset_detail_info_new() {
        let info = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "minor",
            "2024-01-15T10:00:00Z",
            "2024-01-15T10:00:00Z",
        );

        assert_eq!(info.id, "feature-api");
        assert_eq!(info.branch, "feature/api");
        assert_eq!(info.bump, "minor");
        assert!(info.packages.is_empty());
        assert!(info.environments.is_empty());
        assert!(info.commits.is_empty());
        assert!(info.message.is_none());
        assert_eq!(info.created_at, "2024-01-15T10:00:00Z");
        assert_eq!(info.updated_at, "2024-01-15T10:00:00Z");
    }

    #[test]
    fn test_changeset_detail_info_builder_chain() {
        let info = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "major",
            "2024-01-15T10:00:00Z",
            "2024-01-16T14:30:00Z",
        )
        .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
        .with_environments(vec!["staging".to_string(), "production".to_string()])
        .with_commits(vec!["abc123".to_string(), "def456".to_string()])
        .with_message("Breaking API change");

        assert_eq!(info.packages.len(), 2);
        assert_eq!(info.environments.len(), 2);
        assert_eq!(info.commits.len(), 2);
        assert_eq!(info.message, Some("Breaking API change".to_string()));
    }

    #[test]
    fn test_changeset_detail_info_clone() {
        let info = ChangesetDetailInfo::new(
            "test-id",
            "test-branch",
            "patch",
            "2024-01-15T10:00:00Z",
            "2024-01-15T10:00:00Z",
        )
        .with_packages(vec!["@scope/pkg".to_string()]);
        let cloned = info.clone();

        assert_eq!(cloned.id, info.id);
        assert_eq!(cloned.packages, info.packages);
    }

    #[test]
    fn test_changeset_detail_info_serialize() {
        let info = ChangesetDetailInfo::new(
            "test-id",
            "test-branch",
            "minor",
            "2024-01-15T10:00:00Z",
            "2024-01-15T10:00:00Z",
        );
        let json = serde_json::to_string(&info).unwrap_or_default();

        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"branch\":\"test-branch\""));
        assert!(json.contains("\"bump\":\"minor\""));
    }

    // ========================================================================
    // UpdateSummaryInfo Tests
    // ========================================================================

    #[test]
    fn test_update_summary_info_new() {
        let summary = UpdateSummaryInfo::new(3, 2, true, 1);

        assert_eq!(summary.packages_added, 3);
        assert_eq!(summary.commits_added, 2);
        assert!(summary.bump_updated);
        assert_eq!(summary.environments_added, 1);
        assert!(summary.has_changes());
    }

    #[test]
    fn test_update_summary_info_empty() {
        let summary = UpdateSummaryInfo::empty();

        assert_eq!(summary.packages_added, 0);
        assert_eq!(summary.commits_added, 0);
        assert!(!summary.bump_updated);
        assert_eq!(summary.environments_added, 0);
        assert!(!summary.has_changes());
    }

    // ========================================================================
    // ReleasedVersionEntry Tests
    // ========================================================================

    #[test]
    fn test_released_version_entry_new() {
        let entry = ReleasedVersionEntry::new("@scope/core", "2.0.0");

        assert_eq!(entry.package_name, "@scope/core");
        assert_eq!(entry.version, "2.0.0");
    }

    // ========================================================================
    // ReleaseInfoData Tests
    // ========================================================================

    #[test]
    fn test_release_info_data_new() {
        let versions = vec![
            ReleasedVersionEntry::new("@scope/core", "2.0.0"),
            ReleasedVersionEntry::new("@scope/utils", "1.5.0"),
        ];
        let info = ReleaseInfoData::new(
            "2024-01-15T10:00:00Z",
            "developer@example.com",
            "abc123def456",
            versions,
        );

        assert_eq!(info.released_at, "2024-01-15T10:00:00Z");
        assert_eq!(info.released_by, "developer@example.com");
        assert_eq!(info.release_commit, "abc123def456");
        assert_eq!(info.released_versions.len(), 2);
    }

    // ========================================================================
    // ArchivedChangesetInfo Tests
    // ========================================================================

    #[test]
    fn test_archived_changeset_info_new() {
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "major",
            "2024-01-10T10:00:00Z",
            "2024-01-14T10:00:00Z",
        )
        .with_packages(vec!["@scope/core".to_string()]);

        let release_info = ReleaseInfoData::new(
            "2024-01-15T10:00:00Z",
            "developer@example.com",
            "abc123",
            vec![ReleasedVersionEntry::new("@scope/core", "2.0.0")],
        );

        let archived = ArchivedChangesetInfo::new(changeset, release_info);

        assert_eq!(archived.changeset.id, "feature-api");
        assert_eq!(archived.release_info.released_by, "developer@example.com");
    }

    // ========================================================================
    // ChangesetAddData Tests
    // ========================================================================

    #[test]
    fn test_changeset_add_data_new() {
        let data =
            ChangesetAddData::new("feature-api", "feature/api", "minor", "2024-01-15T10:00:00Z");

        assert_eq!(data.id, "feature-api");
        assert_eq!(data.branch, "feature/api");
        assert_eq!(data.bump, "minor");
        assert!(data.packages.is_empty());
        assert!(data.environments.is_empty());
        assert_eq!(data.created_at, "2024-01-15T10:00:00Z");
    }

    #[test]
    fn test_changeset_add_data_with_packages_and_environments() {
        let data =
            ChangesetAddData::new("feature-api", "feature/api", "major", "2024-01-15T10:00:00Z")
                .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
                .with_environments(vec!["staging".to_string()]);

        assert_eq!(data.packages.len(), 2);
        assert_eq!(data.environments.len(), 1);
    }

    // ========================================================================
    // ChangesetUpdateData Tests
    // ========================================================================

    #[test]
    fn test_changeset_update_data_success() {
        let summary = UpdateSummaryInfo::new(1, 1, false, 0);
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "minor",
            "2024-01-15T10:00:00Z",
            "2024-01-15T12:00:00Z",
        );
        let data = ChangesetUpdateData::success(summary, changeset);

        assert!(data.updated);
        assert_eq!(data.summary.packages_added, 1);
        assert_eq!(data.summary.commits_added, 1);
        assert_eq!(data.changeset.branch, "feature/api");
    }

    #[test]
    fn test_changeset_update_data_no_changes() {
        let summary = UpdateSummaryInfo::empty();
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "minor",
            "2024-01-15T10:00:00Z",
            "2024-01-15T10:00:00Z",
        );
        let data = ChangesetUpdateData::new(false, summary, changeset);

        assert!(!data.updated);
        assert_eq!(data.summary.commits_added, 0);
        assert!(!data.summary.has_changes());
    }

    #[test]
    fn test_changeset_update_data_new() {
        let summary = UpdateSummaryInfo::new(0, 0, true, 0);
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "major",
            "2024-01-15T10:00:00Z",
            "2024-01-15T12:00:00Z",
        );
        let data = ChangesetUpdateData::new(true, summary, changeset);

        assert!(data.updated);
        assert!(data.summary.bump_updated);
    }

    // ========================================================================
    // ChangesetListData Tests
    // ========================================================================

    #[test]
    fn test_changeset_list_data_new() {
        let changesets = vec![
            ChangesetListItemInfo::new(
                "cs-1",
                "branch-1",
                "minor",
                3, // commit_count
                "2024-01-15T10:00:00Z",
                "2024-01-15T10:00:00Z",
            ),
            ChangesetListItemInfo::new(
                "cs-2",
                "branch-2",
                "patch",
                1, // commit_count
                "2024-01-14T10:00:00Z",
                "2024-01-14T10:00:00Z",
            ),
        ];
        let data = ChangesetListData::new(changesets);

        assert_eq!(data.count, 2);
        assert_eq!(data.changesets.len(), 2);
        assert_eq!(data.changesets[0].commit_count, 3);
        assert_eq!(data.changesets[1].commit_count, 1);
    }

    #[test]
    fn test_changeset_list_data_empty() {
        let data = ChangesetListData::empty();

        assert_eq!(data.count, 0);
        assert!(data.changesets.is_empty());
    }

    // ========================================================================
    // ChangesetShowData Tests
    // ========================================================================

    #[test]
    fn test_changeset_show_data_new() {
        let changeset = ChangesetDetailInfo::new(
            "feature-api",
            "feature/api",
            "major",
            "2024-01-15T10:00:00Z",
            "2024-01-15T10:00:00Z",
        );
        let data = ChangesetShowData::new(changeset);

        assert_eq!(data.changeset.id, "feature-api");
    }

    // ========================================================================
    // ChangesetRemoveData Tests
    // ========================================================================

    #[test]
    fn test_changeset_remove_data_new() {
        let data = ChangesetRemoveData::new(true, "feature/old");

        assert!(data.removed);
        assert_eq!(data.branch, "feature/old");
    }

    #[test]
    fn test_changeset_remove_data_success() {
        let data = ChangesetRemoveData::success("feature/old");

        assert!(data.removed);
        assert_eq!(data.branch, "feature/old");
    }

    // ========================================================================
    // ChangesetHistoryData Tests
    // ========================================================================

    #[test]
    fn test_changeset_history_data_new() {
        let changeset = ChangesetDetailInfo::new(
            "feature-old",
            "feature/old",
            "minor",
            "2024-01-10T10:00:00Z",
            "2024-01-10T10:00:00Z",
        );
        let release_info =
            ReleaseInfoData::new("2024-01-15T10:00:00Z", "dev@example.com", "abc123", vec![]);
        let archived = ArchivedChangesetInfo::new(changeset, release_info);
        let data = ChangesetHistoryData::new(vec![archived]);

        assert_eq!(data.count, 1);
        assert_eq!(data.archived.len(), 1);
    }

    #[test]
    fn test_changeset_history_data_empty() {
        let data = ChangesetHistoryData::empty();

        assert_eq!(data.count, 0);
        assert!(data.archived.is_empty());
    }

    // ========================================================================
    // ChangesetCheckData Tests
    // ========================================================================

    #[test]
    fn test_changeset_check_data_exists() {
        let data = ChangesetCheckData::exists("feature/api");

        assert!(data.has_changeset);
        assert_eq!(data.branch, Some("feature/api".to_string()));
    }

    #[test]
    fn test_changeset_check_data_not_found() {
        let data = ChangesetCheckData::not_found();

        assert!(!data.has_changeset);
        assert!(data.branch.is_none());
    }
}

// ============================================================================
// Bump Types Tests (Story 5.1)
// ============================================================================

/// Tests for bump command type definitions.
#[cfg(test)]
mod bump_types_tests {
    use crate::error::ErrorInfo;
    use crate::types::bump::{
        BumpApplyApiResponse, BumpApplyData, BumpApplyParams, BumpPreviewApiResponse,
        BumpPreviewData, BumpPreviewParams, BumpSnapshotApiResponse, BumpSnapshotData,
        BumpSnapshotParams, BumpSummaryInfo, COMMON_PRERELEASE_TAGS, DEFAULT_SNAPSHOT_FORMAT,
        DependencyUpdateInfo, PackageVersionInfo, SnapshotVersionInfo, VALID_DEPENDENCY_TYPES,
    };

    // ========================================================================
    // Constants Tests
    // ========================================================================

    #[test]
    fn test_common_prerelease_tags() {
        assert!(COMMON_PRERELEASE_TAGS.contains(&"alpha"));
        assert!(COMMON_PRERELEASE_TAGS.contains(&"beta"));
        assert!(COMMON_PRERELEASE_TAGS.contains(&"rc"));
        assert_eq!(COMMON_PRERELEASE_TAGS.len(), 3);
    }

    #[test]
    fn test_valid_dependency_types() {
        assert!(VALID_DEPENDENCY_TYPES.contains(&"regular"));
        assert!(VALID_DEPENDENCY_TYPES.contains(&"dev"));
        assert!(VALID_DEPENDENCY_TYPES.contains(&"peer"));
        assert!(VALID_DEPENDENCY_TYPES.contains(&"optional"));
        assert_eq!(VALID_DEPENDENCY_TYPES.len(), 4);
    }

    #[test]
    fn test_default_snapshot_format() {
        assert!(DEFAULT_SNAPSHOT_FORMAT.contains("{version}"));
        assert!(DEFAULT_SNAPSHOT_FORMAT.contains("{short_commit}"));
        assert_eq!(DEFAULT_SNAPSHOT_FORMAT, "{version}-snapshot.{short_commit}");
    }

    // ========================================================================
    // BumpPreviewParams Tests
    // ========================================================================

    #[test]
    fn test_bump_preview_params_new() {
        let params = BumpPreviewParams::new("/workspace");

        assert_eq!(params.root, "/workspace");
        assert!(params.config_path.is_none());
        assert!(params.packages.is_none());
        assert!(params.show_diff.is_none());
    }

    #[test]
    fn test_bump_preview_params_builder_chain() {
        let params = BumpPreviewParams::new("/workspace")
            .with_config_path("/workspace/repo.config.json")
            .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
            .with_show_diff(true);

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.config_path, Some("/workspace/repo.config.json".to_string()));
        assert_eq!(
            params.packages,
            Some(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
        );
        assert_eq!(params.show_diff, Some(true));
    }

    #[test]
    fn test_bump_preview_params_clone() {
        let params = BumpPreviewParams::new("/workspace")
            .with_show_diff(true)
            .with_packages(vec!["@scope/core".to_string()]);
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.show_diff, params.show_diff);
        assert_eq!(cloned.packages, params.packages);
    }

    #[test]
    fn test_bump_preview_params_serialize() {
        let params = BumpPreviewParams::new("/workspace").with_show_diff(true);
        let json = serde_json::to_string(&params).unwrap_or_default();

        assert!(json.contains("\"root\":\"/workspace\""));
        assert!(json.contains("\"show_diff\":true"));
        // Optional fields that are None should not be present
        assert!(!json.contains("\"config_path\""));
        assert!(!json.contains("\"packages\""));
    }

    // ========================================================================
    // BumpApplyParams Tests
    // ========================================================================

    #[test]
    fn test_bump_apply_params_new() {
        let params = BumpApplyParams::new("/workspace");

        assert_eq!(params.root, "/workspace");
        assert!(params.config_path.is_none());
        assert!(params.packages.is_none());
        assert!(params.git_commit.is_none());
        assert!(params.git_tag.is_none());
        assert!(params.git_push.is_none());
        assert!(params.prerelease.is_none());
        assert!(params.no_changelog.is_none());
        assert!(params.no_archive.is_none());
        assert!(params.always_archive.is_none());
        assert!(params.force.is_none());
    }

    #[test]
    fn test_bump_apply_params_builder_chain() {
        let params = BumpApplyParams::new("/workspace")
            .with_config_path("/workspace/repo.config.json")
            .with_packages(vec!["@scope/core".to_string()])
            .with_git_commit(true)
            .with_git_tag(true)
            .with_git_push(false)
            .with_prerelease("beta")
            .with_no_changelog(false)
            .with_no_archive(false)
            .with_always_archive(true)
            .with_force(true);

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.config_path, Some("/workspace/repo.config.json".to_string()));
        assert_eq!(params.packages, Some(vec!["@scope/core".to_string()]));
        assert_eq!(params.git_commit, Some(true));
        assert_eq!(params.git_tag, Some(true));
        assert_eq!(params.git_push, Some(false));
        assert_eq!(params.prerelease, Some("beta".to_string()));
        assert_eq!(params.no_changelog, Some(false));
        assert_eq!(params.no_archive, Some(false));
        assert_eq!(params.always_archive, Some(true));
        assert_eq!(params.force, Some(true));
    }

    #[test]
    fn test_bump_apply_params_with_git_options() {
        let params = BumpApplyParams::new("/workspace").with_git_options(true, true, false);

        assert_eq!(params.git_commit, Some(true));
        assert_eq!(params.git_tag, Some(true));
        assert_eq!(params.git_push, Some(false));
    }

    #[test]
    fn test_bump_apply_params_clone() {
        let params =
            BumpApplyParams::new("/workspace").with_prerelease("alpha").with_git_commit(true);
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.prerelease, params.prerelease);
        assert_eq!(cloned.git_commit, params.git_commit);
    }

    #[test]
    fn test_bump_apply_params_serialize() {
        let params = BumpApplyParams::new("/workspace").with_git_commit(true).with_prerelease("rc");
        let json = serde_json::to_string(&params).unwrap_or_default();

        assert!(json.contains("\"root\":\"/workspace\""));
        assert!(json.contains("\"git_commit\":true"));
        assert!(json.contains("\"prerelease\":\"rc\""));
        // Optional fields that are None should not be present
        assert!(!json.contains("\"git_tag\""));
    }

    // ========================================================================
    // BumpSnapshotParams Tests
    // ========================================================================

    #[test]
    fn test_bump_snapshot_params_new() {
        let params = BumpSnapshotParams::new("/workspace");

        assert_eq!(params.root, "/workspace");
        assert!(params.config_path.is_none());
        assert!(params.packages.is_none());
        assert!(params.format.is_none());
    }

    #[test]
    fn test_bump_snapshot_params_builder_chain() {
        let params = BumpSnapshotParams::new("/workspace")
            .with_config_path("/workspace/repo.config.json")
            .with_packages(vec!["@scope/core".to_string()])
            .with_format("{version}-{branch}.{short_commit}");

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.config_path, Some("/workspace/repo.config.json".to_string()));
        assert_eq!(params.packages, Some(vec!["@scope/core".to_string()]));
        assert_eq!(params.format, Some("{version}-{branch}.{short_commit}".to_string()));
    }

    #[test]
    fn test_bump_snapshot_params_clone() {
        let params =
            BumpSnapshotParams::new("/workspace").with_format("{version}-snapshot.{timestamp}");
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.format, params.format);
    }

    #[test]
    fn test_bump_snapshot_params_serialize() {
        let params =
            BumpSnapshotParams::new("/workspace").with_format("{version}-dev.{short_commit}");
        let json = serde_json::to_string(&params).unwrap_or_default();

        assert!(json.contains("\"root\":\"/workspace\""));
        assert!(json.contains("\"format\":\"{version}-dev.{short_commit}\""));
    }

    // ========================================================================
    // DependencyUpdateInfo Tests
    // ========================================================================

    #[test]
    fn test_dependency_update_info_new() {
        let update = DependencyUpdateInfo::new("@scope/core", "regular", "^1.0.0", "^1.1.0");

        assert_eq!(update.name, "@scope/core");
        assert_eq!(update.dependency_type, "regular");
        assert_eq!(update.old_version, "^1.0.0");
        assert_eq!(update.new_version, "^1.1.0");
    }

    #[test]
    fn test_dependency_update_info_regular() {
        let update = DependencyUpdateInfo::regular("@scope/utils", "^2.0.0", "^2.1.0");

        assert_eq!(update.name, "@scope/utils");
        assert_eq!(update.dependency_type, "regular");
        assert_eq!(update.old_version, "^2.0.0");
        assert_eq!(update.new_version, "^2.1.0");
    }

    #[test]
    fn test_dependency_update_info_dev() {
        let update = DependencyUpdateInfo::dev("typescript", "^4.0.0", "^5.0.0");

        assert_eq!(update.name, "typescript");
        assert_eq!(update.dependency_type, "dev");
    }

    #[test]
    fn test_dependency_update_info_peer() {
        let update = DependencyUpdateInfo::peer("react", "^17.0.0", "^18.0.0");

        assert_eq!(update.name, "react");
        assert_eq!(update.dependency_type, "peer");
    }

    #[test]
    fn test_dependency_update_info_optional() {
        let update = DependencyUpdateInfo::optional("lodash", "^4.0.0", "^4.1.0");

        assert_eq!(update.name, "lodash");
        assert_eq!(update.dependency_type, "optional");
    }

    #[test]
    fn test_dependency_update_info_clone() {
        let update = DependencyUpdateInfo::regular("@scope/core", "^1.0.0", "^1.1.0");
        let cloned = update.clone();

        assert_eq!(cloned.name, update.name);
        assert_eq!(cloned.dependency_type, update.dependency_type);
        assert_eq!(cloned.old_version, update.old_version);
        assert_eq!(cloned.new_version, update.new_version);
    }

    // ========================================================================
    // PackageVersionInfo Tests
    // ========================================================================

    #[test]
    fn test_package_version_info_new() {
        let info =
            PackageVersionInfo::new("@scope/core", "packages/core", "1.0.0", "1.1.0", "minor");

        assert_eq!(info.name, "@scope/core");
        assert_eq!(info.path, "packages/core");
        assert_eq!(info.current_version, "1.0.0");
        assert_eq!(info.next_version, "1.1.0");
        assert_eq!(info.bump, "minor");
        assert!(info.dependency_updates.is_empty());
    }

    #[test]
    fn test_package_version_info_with_dependency_updates() {
        let updates = vec![
            DependencyUpdateInfo::regular("@scope/utils", "^1.0.0", "^1.1.0"),
            DependencyUpdateInfo::dev("typescript", "^4.0.0", "^5.0.0"),
        ];

        let info =
            PackageVersionInfo::new("@scope/core", "packages/core", "1.0.0", "2.0.0", "major")
                .with_dependency_updates(updates);

        assert_eq!(info.dependency_updates.len(), 2);
        assert_eq!(info.dependency_updates[0].name, "@scope/utils");
        assert_eq!(info.dependency_updates[1].name, "typescript");
    }

    #[test]
    fn test_package_version_info_add_dependency_update() {
        let info =
            PackageVersionInfo::new("@scope/core", "packages/core", "1.0.0", "1.1.0", "minor")
                .add_dependency_update(DependencyUpdateInfo::regular("dep1", "^1.0.0", "^1.1.0"))
                .add_dependency_update(DependencyUpdateInfo::dev("dep2", "^2.0.0", "^2.1.0"));

        assert_eq!(info.dependency_updates.len(), 2);
    }

    #[test]
    fn test_package_version_info_bump_type_checks() {
        let major = PackageVersionInfo::new("pkg", "path", "1.0.0", "2.0.0", "major");
        assert!(major.is_major());
        assert!(!major.is_minor());
        assert!(!major.is_patch());
        assert!(!major.is_none());

        let minor = PackageVersionInfo::new("pkg", "path", "1.0.0", "1.1.0", "minor");
        assert!(!minor.is_major());
        assert!(minor.is_minor());
        assert!(!minor.is_patch());
        assert!(!minor.is_none());

        let patch = PackageVersionInfo::new("pkg", "path", "1.0.0", "1.0.1", "patch");
        assert!(!patch.is_major());
        assert!(!patch.is_minor());
        assert!(patch.is_patch());
        assert!(!patch.is_none());

        let none = PackageVersionInfo::new("pkg", "path", "1.0.0", "1.0.0", "none");
        assert!(!none.is_major());
        assert!(!none.is_minor());
        assert!(!none.is_patch());
        assert!(none.is_none());
    }

    #[test]
    fn test_package_version_info_clone() {
        let info =
            PackageVersionInfo::new("@scope/core", "packages/core", "1.0.0", "1.1.0", "minor");
        let cloned = info.clone();

        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.path, info.path);
        assert_eq!(cloned.current_version, info.current_version);
        assert_eq!(cloned.next_version, info.next_version);
        assert_eq!(cloned.bump, info.bump);
    }

    // ========================================================================
    // SnapshotVersionInfo Tests
    // ========================================================================

    #[test]
    fn test_snapshot_version_info_new() {
        let info = SnapshotVersionInfo::new(
            "@scope/core",
            "packages/core",
            "1.0.0",
            "1.0.0-snapshot.abc123f",
        );

        assert_eq!(info.name, "@scope/core");
        assert_eq!(info.path, "packages/core");
        assert_eq!(info.original_version, "1.0.0");
        assert_eq!(info.snapshot_version, "1.0.0-snapshot.abc123f");
    }

    #[test]
    fn test_snapshot_version_info_clone() {
        let info = SnapshotVersionInfo::new(
            "@scope/core",
            "packages/core",
            "1.0.0",
            "1.0.0-feature-x.abc123f",
        );
        let cloned = info.clone();

        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.snapshot_version, info.snapshot_version);
    }

    // ========================================================================
    // BumpSummaryInfo Tests
    // ========================================================================

    #[test]
    fn test_bump_summary_info_new() {
        let summary = BumpSummaryInfo::new(10, 2, 5, 3);

        assert_eq!(summary.total_packages, 10);
        assert_eq!(summary.major_bumps, 2);
        assert_eq!(summary.minor_bumps, 5);
        assert_eq!(summary.patch_bumps, 3);
    }

    #[test]
    fn test_bump_summary_info_empty() {
        let summary = BumpSummaryInfo::empty();

        assert_eq!(summary.total_packages, 0);
        assert_eq!(summary.major_bumps, 0);
        assert_eq!(summary.minor_bumps, 0);
        assert_eq!(summary.patch_bumps, 0);
    }

    #[test]
    fn test_bump_summary_info_from_packages() {
        let packages = vec![
            PackageVersionInfo::new("pkg1", "path1", "1.0.0", "2.0.0", "major"),
            PackageVersionInfo::new("pkg2", "path2", "1.0.0", "1.1.0", "minor"),
            PackageVersionInfo::new("pkg3", "path3", "1.0.0", "1.1.0", "minor"),
            PackageVersionInfo::new("pkg4", "path4", "1.0.0", "1.0.1", "patch"),
        ];

        let summary = BumpSummaryInfo::from_packages(&packages);

        assert_eq!(summary.total_packages, 4);
        assert_eq!(summary.major_bumps, 1);
        assert_eq!(summary.minor_bumps, 2);
        assert_eq!(summary.patch_bumps, 1);
    }

    #[test]
    fn test_bump_summary_info_has_breaking_changes() {
        let with_major = BumpSummaryInfo::new(5, 1, 2, 2);
        assert!(with_major.has_breaking_changes());

        let without_major = BumpSummaryInfo::new(5, 0, 3, 2);
        assert!(!without_major.has_breaking_changes());
    }

    // ========================================================================
    // BumpPreviewData Tests
    // ========================================================================

    #[test]
    fn test_bump_preview_data_new() {
        let packages = vec![PackageVersionInfo::new(
            "@scope/core",
            "packages/core",
            "1.0.0",
            "1.1.0",
            "minor",
        )];
        let changesets = vec!["feature-api".to_string()];

        let data = BumpPreviewData::new("independent", packages, changesets);

        assert_eq!(data.strategy, "independent");
        assert_eq!(data.packages.len(), 1);
        assert_eq!(data.changesets.len(), 1);
        assert_eq!(data.summary.total_packages, 1);
        assert_eq!(data.summary.minor_bumps, 1);
    }

    #[test]
    fn test_bump_preview_data_empty() {
        let data = BumpPreviewData::empty("unified");

        assert_eq!(data.strategy, "unified");
        assert!(data.packages.is_empty());
        assert!(data.changesets.is_empty());
        assert_eq!(data.summary.total_packages, 0);
    }

    #[test]
    fn test_bump_preview_data_has_packages() {
        let empty = BumpPreviewData::empty("independent");
        assert!(!empty.has_packages());

        let with_packages = BumpPreviewData::new(
            "independent",
            vec![PackageVersionInfo::new("pkg", "path", "1.0.0", "1.1.0", "minor")],
            vec![],
        );
        assert!(with_packages.has_packages());
    }

    #[test]
    fn test_bump_preview_data_has_breaking_changes() {
        let with_major = BumpPreviewData::new(
            "independent",
            vec![PackageVersionInfo::new("pkg", "path", "1.0.0", "2.0.0", "major")],
            vec![],
        );
        assert!(with_major.has_breaking_changes());

        let without_major = BumpPreviewData::new(
            "independent",
            vec![PackageVersionInfo::new("pkg", "path", "1.0.0", "1.1.0", "minor")],
            vec![],
        );
        assert!(!without_major.has_breaking_changes());
    }

    // ========================================================================
    // BumpApplyData Tests
    // ========================================================================

    #[test]
    fn test_bump_apply_data_new() {
        let data = BumpApplyData::new("independent", 5, 2);

        assert_eq!(data.strategy, "independent");
        assert_eq!(data.packages_updated, 5);
        assert_eq!(data.changesets_archived, 2);
        assert!(data.files_modified.is_empty());
        assert!(data.tags_created.is_empty());
        assert!(data.commit_sha.is_none());
    }

    #[test]
    fn test_bump_apply_data_builder_chain() {
        let data = BumpApplyData::new("independent", 3, 1)
            .with_files_modified(vec![
                "packages/core/package.json".to_string(),
                "packages/core/CHANGELOG.md".to_string(),
            ])
            .with_tags_created(vec!["@scope/core@1.1.0".to_string()])
            .with_commit_sha("abc123def456789");

        assert_eq!(data.files_modified.len(), 2);
        assert_eq!(data.tags_created.len(), 1);
        assert_eq!(data.commit_sha, Some("abc123def456789".to_string()));
    }

    #[test]
    fn test_bump_apply_data_has_commit() {
        let without_commit = BumpApplyData::new("independent", 1, 1);
        assert!(!without_commit.has_commit());

        let with_commit = BumpApplyData::new("independent", 1, 1).with_commit_sha("abc123");
        assert!(with_commit.has_commit());
    }

    #[test]
    fn test_bump_apply_data_has_tags() {
        let without_tags = BumpApplyData::new("independent", 1, 1);
        assert!(!without_tags.has_tags());

        let with_tags = BumpApplyData::new("independent", 1, 1)
            .with_tags_created(vec!["@scope/core@1.0.0".to_string()]);
        assert!(with_tags.has_tags());
    }

    // ========================================================================
    // BumpSnapshotData Tests
    // ========================================================================

    #[test]
    fn test_bump_snapshot_data_new() {
        let packages = vec![SnapshotVersionInfo::new(
            "@scope/core",
            "packages/core",
            "1.0.0",
            "1.0.0-snapshot.abc123f",
        )];

        let data =
            BumpSnapshotData::new("independent", packages, "{version}-snapshot.{short_commit}");

        assert_eq!(data.strategy, "independent");
        assert_eq!(data.packages.len(), 1);
        assert_eq!(data.format, "{version}-snapshot.{short_commit}");
    }

    #[test]
    fn test_bump_snapshot_data_empty() {
        let data = BumpSnapshotData::empty("unified", "{version}-dev.{timestamp}");

        assert_eq!(data.strategy, "unified");
        assert!(data.packages.is_empty());
        assert_eq!(data.format, "{version}-dev.{timestamp}");
    }

    #[test]
    fn test_bump_snapshot_data_package_count() {
        let empty = BumpSnapshotData::empty("independent", "format");
        assert_eq!(empty.package_count(), 0);

        let with_packages = BumpSnapshotData::new(
            "independent",
            vec![
                SnapshotVersionInfo::new("pkg1", "path1", "1.0.0", "1.0.0-snapshot"),
                SnapshotVersionInfo::new("pkg2", "path2", "2.0.0", "2.0.0-snapshot"),
            ],
            "format",
        );
        assert_eq!(with_packages.package_count(), 2);
    }

    // ========================================================================
    // API Response Tests
    // ========================================================================

    #[test]
    fn test_bump_preview_api_response_success() {
        let data = BumpPreviewData::empty("independent");
        let response = BumpPreviewApiResponse::success(data);

        assert!(response.success);
        assert!(response.is_success());
        assert!(!response.is_failure());
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_bump_preview_api_response_failure() {
        let error = ErrorInfo::validation("Invalid root path", Some("root"));
        let response = BumpPreviewApiResponse::failure(error);

        assert!(!response.success);
        assert!(!response.is_success());
        assert!(response.is_failure());
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, "EVALIDATION");
    }

    #[test]
    fn test_bump_apply_api_response_success() {
        let data = BumpApplyData::new("independent", 3, 1);
        let response = BumpApplyApiResponse::success(data);

        assert!(response.success);
        assert!(response.is_success());
        assert!(response.data.is_some());
        assert_eq!(response.data.as_ref().unwrap().packages_updated, 3);
    }

    #[test]
    fn test_bump_apply_api_response_failure() {
        let error = ErrorInfo::git("Failed to create commit");
        let response = BumpApplyApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.is_failure());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, "EGIT");
    }

    #[test]
    fn test_bump_snapshot_api_response_success() {
        let data = BumpSnapshotData::empty("independent", "format");
        let response = BumpSnapshotApiResponse::success(data);

        assert!(response.success);
        assert!(response.is_success());
        assert!(response.data.is_some());
    }

    #[test]
    fn test_bump_snapshot_api_response_failure() {
        let error = ErrorInfo::validation("Invalid format template", Some("format"));
        let response = BumpSnapshotApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.is_failure());
        assert!(response.error.is_some());
    }

    // ========================================================================
    // Serialization Tests
    // ========================================================================

    #[test]
    fn test_package_version_info_serialize() {
        let info =
            PackageVersionInfo::new("@scope/core", "packages/core", "1.0.0", "1.1.0", "minor");
        let json = serde_json::to_string(&info).unwrap_or_default();

        assert!(json.contains("\"name\":\"@scope/core\""));
        assert!(json.contains("\"path\":\"packages/core\""));
        assert!(json.contains("\"current_version\":\"1.0.0\""));
        assert!(json.contains("\"next_version\":\"1.1.0\""));
        assert!(json.contains("\"bump\":\"minor\""));
    }

    #[test]
    fn test_bump_preview_data_serialize() {
        let packages = vec![PackageVersionInfo::new("pkg", "path", "1.0.0", "1.1.0", "minor")];
        let data = BumpPreviewData::new("independent", packages, vec!["cs1".to_string()]);
        let json = serde_json::to_string(&data).unwrap_or_default();

        assert!(json.contains("\"strategy\":\"independent\""));
        assert!(json.contains("\"packages\""));
        assert!(json.contains("\"summary\""));
        assert!(json.contains("\"changesets\""));
    }

    #[test]
    fn test_bump_apply_data_serialize() {
        let data = BumpApplyData::new("unified", 5, 2).with_commit_sha("abc123");
        let json = serde_json::to_string(&data).unwrap_or_default();

        assert!(json.contains("\"strategy\":\"unified\""));
        assert!(json.contains("\"packages_updated\":5"));
        assert!(json.contains("\"changesets_archived\":2"));
        assert!(json.contains("\"commit_sha\":\"abc123\""));
    }

    #[test]
    fn test_bump_snapshot_data_serialize() {
        let packages = vec![SnapshotVersionInfo::new("pkg", "path", "1.0.0", "1.0.0-snapshot.abc")];
        let data =
            BumpSnapshotData::new("independent", packages, "{version}-snapshot.{short_commit}");
        let json = serde_json::to_string(&data).unwrap_or_default();

        assert!(json.contains("\"strategy\":\"independent\""));
        assert!(json.contains("\"format\":\"{version}-snapshot.{short_commit}\""));
        assert!(json.contains("\"snapshot_version\":\"1.0.0-snapshot.abc\""));
    }
}

// =============================================================================
// Execute Types Tests (Story 6.2)
// =============================================================================

/// Tests for execute command type definitions.
#[cfg(test)]
mod execute_types_tests {
    use crate::error::ErrorInfo;
    use crate::types::execute::{
        ExecuteApiResponse, ExecuteData, ExecuteParams, ExecuteSummary, PackageExecutionResult,
    };

    // ========================================================================
    // ExecuteParams Tests
    // ========================================================================

    #[test]
    fn test_execute_params_new() {
        let params = ExecuteParams::new("/workspace", "npm:test");

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.cmd, "npm:test");
        assert!(params.filter_package.is_none());
        assert!(params.affected.is_none());
        assert!(params.since.is_none());
        assert!(params.until.is_none());
        assert!(params.branch.is_none());
        assert!(params.parallel.is_none());
        assert!(params.args.is_none());
        assert!(params.timeout_secs.is_none());
        assert!(params.per_package_timeout_secs.is_none());
    }

    #[test]
    fn test_execute_params_builder_chain() {
        let params = ExecuteParams::new("/workspace", "npm:test")
            .with_filter_package(vec!["@scope/core".to_string()])
            .with_parallel(true)
            .with_timeout_secs(300)
            .with_per_package_timeout_secs(60)
            .with_args(vec!["--coverage".to_string()]);

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.cmd, "npm:test");
        assert_eq!(params.filter_package, Some(vec!["@scope/core".to_string()]));
        assert_eq!(params.parallel, Some(true));
        assert_eq!(params.timeout_secs, Some(300));
        assert_eq!(params.per_package_timeout_secs, Some(60));
        assert_eq!(params.args, Some(vec!["--coverage".to_string()]));
    }

    #[test]
    fn test_execute_params_affected_options() {
        let params = ExecuteParams::new("/workspace", "npm:test")
            .with_affected(true)
            .with_branch("main")
            .with_since("HEAD~5")
            .with_until("HEAD");

        assert_eq!(params.affected, Some(true));
        assert_eq!(params.branch, Some("main".to_string()));
        assert_eq!(params.since, Some("HEAD~5".to_string()));
        assert_eq!(params.until, Some("HEAD".to_string()));
    }

    #[test]
    fn test_execute_params_has_filter_package() {
        let params_none = ExecuteParams::new(".", "npm:test");
        assert!(!params_none.has_filter_package());

        let params_empty = ExecuteParams::new(".", "npm:test").with_filter_package(vec![]);
        assert!(!params_empty.has_filter_package());

        let params_with_packages =
            ExecuteParams::new(".", "npm:test").with_filter_package(vec!["pkg".to_string()]);
        assert!(params_with_packages.has_filter_package());
    }

    #[test]
    fn test_execute_params_is_affected() {
        let params_none = ExecuteParams::new(".", "npm:test");
        assert!(!params_none.is_affected());

        let params_false = ExecuteParams::new(".", "npm:test").with_affected(false);
        assert!(!params_false.is_affected());

        let params_true = ExecuteParams::new(".", "npm:test").with_affected(true);
        assert!(params_true.is_affected());
    }

    #[test]
    fn test_execute_params_is_parallel() {
        let params_none = ExecuteParams::new(".", "npm:test");
        assert!(!params_none.is_parallel());

        let params_false = ExecuteParams::new(".", "npm:test").with_parallel(false);
        assert!(!params_false.is_parallel());

        let params_true = ExecuteParams::new(".", "npm:test").with_parallel(true);
        assert!(params_true.is_parallel());
    }

    #[test]
    fn test_execute_params_clone() {
        let params =
            ExecuteParams::new("/workspace", "npm:test").with_parallel(true).with_timeout_secs(300);
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.cmd, params.cmd);
        assert_eq!(cloned.parallel, params.parallel);
        assert_eq!(cloned.timeout_secs, params.timeout_secs);
    }

    #[test]
    fn test_execute_params_serialize() {
        let params =
            ExecuteParams::new("/workspace", "npm:test").with_parallel(true).with_timeout_secs(300);
        let json = serde_json::to_string(&params).unwrap_or_default();

        assert!(json.contains("\"root\":\"/workspace\""));
        assert!(json.contains("\"cmd\":\"npm:test\""));
        assert!(json.contains("\"parallel\":true"));
        assert!(json.contains("\"timeout_secs\":300"));
        // Optional fields that are None should not be present
        assert!(!json.contains("\"filter_package\""));
        assert!(!json.contains("\"affected\""));
    }

    // ========================================================================
    // PackageExecutionResult Tests
    // ========================================================================

    #[test]
    fn test_package_execution_result_new() {
        let result = PackageExecutionResult::new("@scope/core", true, 0, 1500.0);

        assert_eq!(result.package, "@scope/core");
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!((result.duration_ms - 1500.0).abs() < f64::EPSILON);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_package_execution_result_success() {
        let result = PackageExecutionResult::success("@scope/core", 2000.0);

        assert_eq!(result.package, "@scope/core");
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!((result.duration_ms - 2000.0).abs() < f64::EPSILON);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_package_execution_result_failure() {
        let result = PackageExecutionResult::failure("@scope/core", 1, 500.0, "Test failed");

        assert_eq!(result.package, "@scope/core");
        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
        assert!((result.duration_ms - 500.0).abs() < f64::EPSILON);
        assert_eq!(result.error, Some("Test failed".to_string()));
    }

    #[test]
    fn test_package_execution_result_with_error() {
        let result = PackageExecutionResult::new("@scope/core", false, 1, 500.0)
            .with_error("Command not found");

        assert!(!result.success);
        assert_eq!(result.error, Some("Command not found".to_string()));
    }

    #[test]
    fn test_package_execution_result_clone() {
        let result = PackageExecutionResult::failure("@scope/core", 1, 500.0, "Error");
        let cloned = result.clone();

        assert_eq!(cloned.package, result.package);
        assert_eq!(cloned.success, result.success);
        assert_eq!(cloned.exit_code, result.exit_code);
        assert!((cloned.duration_ms - result.duration_ms).abs() < f64::EPSILON);
        assert_eq!(cloned.error, result.error);
    }

    #[test]
    fn test_package_execution_result_serialize() {
        let result = PackageExecutionResult::failure("@scope/core", 1, 500.0, "Error");
        let json = serde_json::to_string(&result).unwrap_or_default();

        assert!(json.contains("\"package\":\"@scope/core\""));
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"exit_code\":1"));
        assert!(json.contains("\"duration_ms\":500"));
        assert!(json.contains("\"error\":\"Error\""));
    }

    #[test]
    fn test_package_execution_result_serialize_without_error() {
        let result = PackageExecutionResult::success("@scope/core", 1500.0);
        let json = serde_json::to_string(&result).unwrap_or_default();

        assert!(json.contains("\"package\":\"@scope/core\""));
        assert!(json.contains("\"success\":true"));
        // error field should not be present when None
        assert!(!json.contains("\"error\""));
    }

    // ========================================================================
    // ExecuteSummary Tests
    // ========================================================================

    #[test]
    fn test_execute_summary_new() {
        let summary = ExecuteSummary::new(5, 4, 1, 15000.0);

        assert_eq!(summary.total, 5);
        assert_eq!(summary.succeeded, 4);
        assert_eq!(summary.failed, 1);
        assert!((summary.total_duration_ms - 15000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_execute_summary_empty() {
        let summary = ExecuteSummary::empty();

        assert_eq!(summary.total, 0);
        assert_eq!(summary.succeeded, 0);
        assert_eq!(summary.failed, 0);
        assert!((summary.total_duration_ms - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_execute_summary_from_results() {
        let results = vec![
            PackageExecutionResult::success("pkg1", 1000.0),
            PackageExecutionResult::success("pkg2", 500.0),
            PackageExecutionResult::failure("pkg3", 1, 300.0, "Error"),
        ];
        let summary = ExecuteSummary::from_results(&results);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.succeeded, 2);
        assert_eq!(summary.failed, 1);
        assert!((summary.total_duration_ms - 1800.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_execute_summary_from_results_empty() {
        let results: Vec<PackageExecutionResult> = vec![];
        let summary = ExecuteSummary::from_results(&results);

        assert_eq!(summary.total, 0);
        assert_eq!(summary.succeeded, 0);
        assert_eq!(summary.failed, 0);
        assert!((summary.total_duration_ms - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_execute_summary_all_succeeded() {
        let all_pass = ExecuteSummary::new(3, 3, 0, 1000.0);
        assert!(all_pass.all_succeeded());

        let some_fail = ExecuteSummary::new(3, 2, 1, 1000.0);
        assert!(!some_fail.all_succeeded());

        let empty = ExecuteSummary::empty();
        assert!(!empty.all_succeeded());
    }

    #[test]
    fn test_execute_summary_has_failures() {
        let all_pass = ExecuteSummary::new(3, 3, 0, 1000.0);
        assert!(!all_pass.has_failures());

        let some_fail = ExecuteSummary::new(3, 2, 1, 1000.0);
        assert!(some_fail.has_failures());

        let all_fail = ExecuteSummary::new(3, 0, 3, 1000.0);
        assert!(all_fail.has_failures());
    }

    #[test]
    fn test_execute_summary_clone() {
        let summary = ExecuteSummary::new(5, 4, 1, 15000.0);
        let cloned = summary.clone();

        assert_eq!(cloned.total, summary.total);
        assert_eq!(cloned.succeeded, summary.succeeded);
        assert_eq!(cloned.failed, summary.failed);
        assert!((cloned.total_duration_ms - summary.total_duration_ms).abs() < f64::EPSILON);
    }

    #[test]
    fn test_execute_summary_serialize() {
        let summary = ExecuteSummary::new(5, 4, 1, 15000.0);
        let json = serde_json::to_string(&summary).unwrap_or_default();

        assert!(json.contains("\"total\":5"));
        assert!(json.contains("\"succeeded\":4"));
        assert!(json.contains("\"failed\":1"));
        assert!(json.contains("\"total_duration_ms\":15000"));
    }

    // ========================================================================
    // ExecuteData Tests
    // ========================================================================

    #[test]
    fn test_execute_data_new() {
        let results = vec![PackageExecutionResult::success("pkg1", 1000.0)];
        let summary = ExecuteSummary::new(1, 1, 0, 1000.0);
        let data = ExecuteData::new("npm:test", results, summary);

        assert_eq!(data.command, "npm:test");
        assert_eq!(data.results.len(), 1);
        assert_eq!(data.summary.total, 1);
    }

    #[test]
    fn test_execute_data_from_results() {
        let results = vec![
            PackageExecutionResult::success("pkg1", 1000.0),
            PackageExecutionResult::failure("pkg2", 1, 500.0, "Error"),
        ];
        let data = ExecuteData::from_results("npm:build", results);

        assert_eq!(data.command, "npm:build");
        assert_eq!(data.results.len(), 2);
        assert_eq!(data.summary.total, 2);
        assert_eq!(data.summary.succeeded, 1);
        assert_eq!(data.summary.failed, 1);
        assert!((data.summary.total_duration_ms - 1500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_execute_data_empty() {
        let data = ExecuteData::empty("npm:lint");

        assert_eq!(data.command, "npm:lint");
        assert!(data.results.is_empty());
        assert_eq!(data.summary.total, 0);
    }

    #[test]
    fn test_execute_data_package_count() {
        let data = ExecuteData::from_results(
            "npm:test",
            vec![
                PackageExecutionResult::success("pkg1", 1000.0),
                PackageExecutionResult::success("pkg2", 500.0),
            ],
        );

        assert_eq!(data.package_count(), 2);
    }

    #[test]
    fn test_execute_data_all_succeeded() {
        let all_pass = ExecuteData::from_results(
            "npm:test",
            vec![
                PackageExecutionResult::success("pkg1", 1000.0),
                PackageExecutionResult::success("pkg2", 500.0),
            ],
        );
        assert!(all_pass.all_succeeded());

        let some_fail = ExecuteData::from_results(
            "npm:test",
            vec![
                PackageExecutionResult::success("pkg1", 1000.0),
                PackageExecutionResult::failure("pkg2", 1, 500.0, "Error"),
            ],
        );
        assert!(!some_fail.all_succeeded());
    }

    #[test]
    fn test_execute_data_has_failures() {
        let all_pass = ExecuteData::from_results(
            "npm:test",
            vec![PackageExecutionResult::success("pkg1", 1000.0)],
        );
        assert!(!all_pass.has_failures());

        let some_fail = ExecuteData::from_results(
            "npm:test",
            vec![PackageExecutionResult::failure("pkg1", 1, 500.0, "Error")],
        );
        assert!(some_fail.has_failures());
    }

    #[test]
    fn test_execute_data_clone() {
        let data = ExecuteData::from_results(
            "npm:test",
            vec![PackageExecutionResult::success("pkg1", 1000.0)],
        );
        let cloned = data.clone();

        assert_eq!(cloned.command, data.command);
        assert_eq!(cloned.results.len(), data.results.len());
        assert_eq!(cloned.summary.total, data.summary.total);
    }

    #[test]
    fn test_execute_data_serialize() {
        let data = ExecuteData::from_results(
            "npm:test",
            vec![PackageExecutionResult::success("@scope/core", 1000.0)],
        );
        let json = serde_json::to_string(&data).unwrap_or_default();

        assert!(json.contains("\"command\":\"npm:test\""));
        assert!(json.contains("\"package\":\"@scope/core\""));
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("\"succeeded\":1"));
    }

    // ========================================================================
    // ExecuteApiResponse Tests
    // ========================================================================

    #[test]
    fn test_execute_api_response_success() {
        let data = ExecuteData::empty("npm:test");
        let response = ExecuteApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
        assert!(response.is_success());
        assert!(!response.is_failure());
    }

    #[test]
    fn test_execute_api_response_failure() {
        let error = ErrorInfo::validation("Invalid command", Some("cmd"));
        let response = ExecuteApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert!(!response.is_success());
        assert!(response.is_failure());
    }

    #[test]
    fn test_execute_api_response_failure_with_different_error_codes() {
        // EVALIDATION
        let validation_error = ErrorInfo::validation("Invalid root", Some("root"));
        let validation_response = ExecuteApiResponse::failure(validation_error);
        assert_eq!(
            validation_response.error.as_ref().map(|e| e.code.as_str()),
            Some("EVALIDATION")
        );

        // ENOENT (Entity Not Found - Unix/Node.js standard)
        let not_found_error = ErrorInfo::not_found("Path not found", Some("root"));
        let not_found_response = ExecuteApiResponse::failure(not_found_error);
        assert_eq!(not_found_response.error.as_ref().map(|e| e.code.as_str()), Some("ENOENT"));

        // ETIMEOUT
        let timeout_error = ErrorInfo::timeout("Operation timed out");
        let timeout_response = ExecuteApiResponse::failure(timeout_error);
        assert_eq!(timeout_response.error.as_ref().map(|e| e.code.as_str()), Some("ETIMEOUT"));
    }

    #[test]
    fn test_execute_api_response_clone() {
        let data = ExecuteData::empty("npm:test");
        let response = ExecuteApiResponse::success(data);
        let cloned = response.clone();

        assert_eq!(cloned.success, response.success);
        assert!(cloned.data.is_some());
    }

    #[test]
    fn test_execute_api_response_serialize_success() {
        let data = ExecuteData::empty("npm:test");
        let response = ExecuteApiResponse::success(data);
        let json = serde_json::to_string(&response).unwrap_or_default();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"command\":\"npm:test\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_execute_api_response_serialize_failure() {
        let error = ErrorInfo::validation("Invalid command", Some("cmd"));
        let response = ExecuteApiResponse::failure(error);
        let json = serde_json::to_string(&response).unwrap_or_default();

        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"code\":\"EVALIDATION\""));
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn test_execute_api_response_with_full_data() {
        let results = vec![
            PackageExecutionResult::success("@scope/core", 1500.0),
            PackageExecutionResult::failure("@scope/utils", 1, 800.0, "Test failed"),
        ];
        let data = ExecuteData::from_results("npm:test", results);
        let response = ExecuteApiResponse::success(data);

        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert_eq!(data.command, "npm:test");
        assert_eq!(data.results.len(), 2);
        assert_eq!(data.summary.total, 2);
        assert_eq!(data.summary.succeeded, 1);
        assert_eq!(data.summary.failed, 1);
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_execute_complete_scenario_parallel() {
        // Simulate parallel execution on affected packages
        let params = ExecuteParams::new("/workspace", "npm:test")
            .with_affected(true)
            .with_branch("main")
            .with_parallel(true)
            .with_timeout_secs(300)
            .with_per_package_timeout_secs(60);

        assert!(params.is_affected());
        assert!(params.is_parallel());
        assert!(!params.has_filter_package());

        // Simulate results
        let results = vec![
            PackageExecutionResult::success("@scope/core", 2000.0),
            PackageExecutionResult::success("@scope/utils", 1500.0),
            PackageExecutionResult::success("@scope/cli", 3000.0),
        ];
        let data = ExecuteData::from_results(&params.cmd, results);
        let response = ExecuteApiResponse::success(data);

        assert!(response.is_success());
        let data = response.data.as_ref().unwrap();
        assert!(data.all_succeeded());
        assert!(!data.has_failures());
        assert_eq!(data.summary.total, 3);
        assert_eq!(data.summary.succeeded, 3);
    }

    #[test]
    fn test_execute_complete_scenario_filtered() {
        // Simulate execution on specific packages
        let params = ExecuteParams::new("/workspace", "npm:build")
            .with_filter_package(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
            .with_parallel(false);

        assert!(!params.is_affected());
        assert!(!params.is_parallel());
        assert!(params.has_filter_package());

        // Simulate results with one failure
        let results = vec![
            PackageExecutionResult::success("@scope/core", 5000.0),
            PackageExecutionResult::failure(
                "@scope/utils",
                1,
                2000.0,
                "Build failed: missing dependency",
            ),
        ];
        let data = ExecuteData::from_results(&params.cmd, results);
        let response = ExecuteApiResponse::success(data);

        assert!(response.is_success());
        let data = response.data.as_ref().unwrap();
        assert!(!data.all_succeeded());
        assert!(data.has_failures());
        assert_eq!(data.summary.succeeded, 1);
        assert_eq!(data.summary.failed, 1);
    }

    #[test]
    fn test_execute_system_command() {
        // Test with a system command (not npm script)
        let params =
            ExecuteParams::new("/workspace", "echo hello").with_args(vec!["world".to_string()]);

        assert_eq!(params.cmd, "echo hello");
        assert_eq!(params.args, Some(vec!["world".to_string()]));

        let results = vec![PackageExecutionResult::success("root", 50.0)];
        let data = ExecuteData::from_results(&params.cmd, results);

        assert_eq!(data.command, "echo hello");
        assert!((data.summary.total_duration_ms - 50.0).abs() < f64::EPSILON);
    }
}

/// Tests for config types (Story 7.1).
/// Tests for ConfigShowParams, ConfigValidateParams, and related structures.
#[cfg(test)]
mod config_params_tests {
    use crate::types::config::{ConfigShowParams, ConfigValidateParams};

    #[test]
    fn test_config_show_params_new() {
        let params = ConfigShowParams::new(".".to_string());

        assert_eq!(params.root, ".");
        assert!(params.config_path.is_none());
    }

    #[test]
    fn test_config_show_params_with_config() {
        let params =
            ConfigShowParams::with_config("/workspace".to_string(), "repo.config.json".to_string());

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.config_path, Some("repo.config.json".to_string()));
    }

    #[test]
    fn test_config_show_params_clone() {
        let params = ConfigShowParams::with_config(".".to_string(), "custom.json".to_string());
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
        assert_eq!(cloned.config_path, params.config_path);
    }

    #[test]
    fn test_config_show_params_debug() {
        let params = ConfigShowParams::new(".".to_string());
        let debug_str = format!("{params:?}");

        assert!(debug_str.contains("ConfigShowParams"));
        assert!(debug_str.contains("root"));
    }

    #[test]
    fn test_config_show_params_serialize() {
        let params = ConfigShowParams::new("/path/to/workspace".to_string());
        let json = serde_json::to_string(&params).unwrap();

        assert!(json.contains("root"));
        assert!(json.contains("/path/to/workspace"));
    }

    #[test]
    fn test_config_validate_params_new() {
        let params = ConfigValidateParams::new(".".to_string());

        assert_eq!(params.root, ".");
        assert!(params.config_path.is_none());
    }

    #[test]
    fn test_config_validate_params_with_config() {
        let params = ConfigValidateParams::with_config(
            "/workspace".to_string(),
            "repo.config.toml".to_string(),
        );

        assert_eq!(params.root, "/workspace");
        assert_eq!(params.config_path, Some("repo.config.toml".to_string()));
    }

    #[test]
    fn test_config_validate_params_clone() {
        let params = ConfigValidateParams::new("/project".to_string());
        let cloned = params.clone();

        assert_eq!(cloned.root, params.root);
    }
}

/// Tests for config info structures (Story 7.1).
#[cfg(test)]
mod config_info_tests {
    use crate::types::config::{
        AuditConfigInfo, AuditSectionsConfigInfo, BackupConfigInfo, ChangelogConfigInfo,
        ChangesetConfigInfo, DependencyConfigInfo, ExecuteConfigInfo, GitConfigInfo,
        HealthScoreWeightsInfo, RegistryConfigInfo, ScopedRegistryEntry, UpgradeConfigInfo,
        VersionConfigInfo,
    };

    #[test]
    fn test_changeset_config_info_new() {
        let config = ChangesetConfigInfo::new(
            ".changesets".to_string(),
            ".changesets/history".to_string(),
            vec!["production".to_string(), "staging".to_string()],
            vec!["production".to_string()],
        );

        assert_eq!(config.path, ".changesets");
        assert_eq!(config.history_path, ".changesets/history");
        assert_eq!(config.available_environments.len(), 2);
        assert_eq!(config.default_environments.len(), 1);
    }

    #[test]
    fn test_changeset_config_info_default() {
        let config = ChangesetConfigInfo::default();

        assert_eq!(config.path, ".changesets");
        assert_eq!(config.history_path, ".changesets/history");
        assert!(config.available_environments.is_empty());
        assert!(config.default_environments.is_empty());
    }

    #[test]
    fn test_version_config_info_new() {
        let config = VersionConfigInfo::new(
            "independent".to_string(),
            "minor".to_string(),
            "{version}-snapshot".to_string(),
        );

        assert_eq!(config.strategy, "independent");
        assert_eq!(config.default_bump, "minor");
        assert_eq!(config.snapshot_format, "{version}-snapshot");
    }

    #[test]
    fn test_version_config_info_default() {
        let config = VersionConfigInfo::default();

        assert_eq!(config.strategy, "independent");
        assert_eq!(config.default_bump, "patch");
        assert!(config.snapshot_format.contains("{version}"));
    }

    #[test]
    fn test_dependency_config_info_new() {
        let config = DependencyConfigInfo::new(
            "patch".to_string(),
            true,
            false,
            false,
            10,
            false,
            true,
            true,
            true,
            true,
        );

        assert_eq!(config.propagation_bump, "patch");
        assert!(config.propagate_dependencies);
        assert!(!config.propagate_dev_dependencies);
        assert_eq!(config.max_depth, 10);
        assert!(config.skip_workspace_protocol);
    }

    #[test]
    fn test_dependency_config_info_default() {
        let config = DependencyConfigInfo::default();

        assert_eq!(config.propagation_bump, "patch");
        assert!(config.propagate_dependencies);
        assert!(!config.propagate_dev_dependencies);
        assert!(!config.propagate_peer_dependencies);
        assert_eq!(config.max_depth, 10);
        assert!(!config.fail_on_circular);
    }

    #[test]
    fn test_registry_config_info_new() {
        let scoped = vec![ScopedRegistryEntry::new(
            "@myorg".to_string(),
            "https://npm.myorg.com".to_string(),
        )];
        let config =
            RegistryConfigInfo::new("https://registry.npmjs.org".to_string(), scoped, 30, 3, true);

        assert_eq!(config.default_registry, "https://registry.npmjs.org");
        assert_eq!(config.scoped_registries.len(), 1);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.retry_attempts, 3);
        assert!(config.read_npmrc);
    }

    #[test]
    fn test_registry_config_info_default() {
        let config = RegistryConfigInfo::default();

        assert_eq!(config.default_registry, "https://registry.npmjs.org");
        assert!(config.scoped_registries.is_empty());
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.retry_attempts, 3);
        assert!(config.read_npmrc);
    }

    #[test]
    fn test_scoped_registry_entry_new() {
        let entry = ScopedRegistryEntry::new(
            "@websublime".to_string(),
            "https://npm.websublime.dev".to_string(),
        );

        assert_eq!(entry.scope, "@websublime");
        assert_eq!(entry.registry, "https://npm.websublime.dev");
    }

    #[test]
    fn test_backup_config_info_new() {
        let config = BackupConfigInfo::new(true, ".backups".to_string(), 5);

        assert!(config.enabled);
        assert_eq!(config.path, ".backups");
        assert_eq!(config.keep_count, 5);
    }

    #[test]
    fn test_backup_config_info_default() {
        let config = BackupConfigInfo::default();

        assert!(config.enabled);
        assert_eq!(config.path, ".backups");
        assert_eq!(config.keep_count, 5);
    }

    #[test]
    fn test_upgrade_config_info_new() {
        let registry = RegistryConfigInfo::default();
        let backup = BackupConfigInfo::default();
        let config =
            UpgradeConfigInfo::new(true, "patch".to_string(), registry.clone(), backup.clone());

        assert!(config.auto_changeset);
        assert_eq!(config.changeset_bump, "patch");
        assert_eq!(config.registry.default_registry, registry.default_registry);
    }

    #[test]
    fn test_upgrade_config_info_default() {
        let config = UpgradeConfigInfo::default();

        assert!(config.auto_changeset);
        assert_eq!(config.changeset_bump, "patch");
        assert_eq!(config.registry.timeout_secs, 30);
        assert!(config.backup.enabled);
    }

    #[test]
    fn test_changelog_config_info_new() {
        let config = ChangelogConfigInfo::new(
            true,
            "keep-a-changelog".to_string(),
            true,
            Some("https://github.com/org/repo".to_string()),
            true,
            None,
            vec![],
            "per-package".to_string(),
        );

        assert!(config.enabled);
        assert_eq!(config.format, "keep-a-changelog");
        assert!(config.include_commit_links);
        assert!(config.repository_url.is_some());
        assert!(config.conventional);
        assert!(config.template.is_none());
    }

    #[test]
    fn test_changelog_config_info_default() {
        let config = ChangelogConfigInfo::default();

        assert!(config.enabled);
        assert_eq!(config.format, "keep-a-changelog");
        assert!(config.include_commit_links);
        assert!(config.conventional);
        assert_eq!(config.monorepo_mode, "per-package");
    }

    #[test]
    fn test_audit_sections_config_info_new() {
        let config = AuditSectionsConfigInfo::new(true, true, false, false);

        assert!(config.upgrades);
        assert!(config.dependencies);
        assert!(!config.version_consistency);
        assert!(!config.breaking_changes);
    }

    #[test]
    fn test_audit_sections_config_info_default() {
        let config = AuditSectionsConfigInfo::default();

        assert!(config.upgrades);
        assert!(config.dependencies);
        assert!(config.version_consistency);
        assert!(config.breaking_changes);
    }

    #[test]
    fn test_health_score_weights_info_new() {
        let config = HealthScoreWeightsInfo::new(0.4, 0.3, 0.2, 0.1);

        assert!((config.upgrades_weight - 0.4).abs() < f64::EPSILON);
        assert!((config.dependencies_weight - 0.3).abs() < f64::EPSILON);
        assert!((config.version_consistency_weight - 0.2).abs() < f64::EPSILON);
        assert!((config.breaking_changes_weight - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_health_score_weights_info_default() {
        let config = HealthScoreWeightsInfo::default();

        // All weights should be 0.25 by default
        assert!((config.upgrades_weight - 0.25).abs() < f64::EPSILON);
        assert!((config.dependencies_weight - 0.25).abs() < f64::EPSILON);
        assert!((config.version_consistency_weight - 0.25).abs() < f64::EPSILON);
        assert!((config.breaking_changes_weight - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_audit_config_info_new() {
        let sections = AuditSectionsConfigInfo::default();
        let weights = HealthScoreWeightsInfo::default();
        let config = AuditConfigInfo::new(true, "medium".to_string(), sections, weights);

        assert!(config.enabled);
        assert_eq!(config.min_severity, "medium");
    }

    #[test]
    fn test_audit_config_info_default() {
        let config = AuditConfigInfo::default();

        assert!(config.enabled);
        assert_eq!(config.min_severity, "low");
        assert!(config.sections.upgrades);
    }

    #[test]
    fn test_git_config_info_new() {
        let config = GitConfigInfo::new("develop".to_string(), true);

        assert_eq!(config.branch_base, "develop");
        assert!(config.detect_affected_packages);
    }

    #[test]
    fn test_git_config_info_default() {
        let config = GitConfigInfo::default();

        assert_eq!(config.branch_base, "main");
        assert!(config.detect_affected_packages);
    }

    #[test]
    fn test_execute_config_info_new() {
        let config = ExecuteConfigInfo::new(600, 120, 8);

        assert_eq!(config.timeout_secs, 600);
        assert_eq!(config.per_package_timeout_secs, 120);
        assert_eq!(config.max_parallel, 8);
    }

    #[test]
    fn test_execute_config_info_default() {
        let config = ExecuteConfigInfo::default();

        assert_eq!(config.timeout_secs, 300);
        assert_eq!(config.per_package_timeout_secs, 60);
        assert_eq!(config.max_parallel, 4);
    }
}

/// Tests for ConfigData and ConfigShowData (Story 7.1).
#[cfg(test)]
mod config_data_tests {
    use crate::types::config::{
        AuditConfigInfo, ChangelogConfigInfo, ChangesetConfigInfo, ConfigData, ConfigShowData,
        DependencyConfigInfo, ExecuteConfigInfo, GitConfigInfo, UpgradeConfigInfo,
        VersionConfigInfo,
    };

    #[test]
    fn test_config_data_new() {
        let config = ConfigData::new(
            ChangesetConfigInfo::default(),
            VersionConfigInfo::default(),
            DependencyConfigInfo::default(),
            UpgradeConfigInfo::default(),
            ChangelogConfigInfo::default(),
            AuditConfigInfo::default(),
            GitConfigInfo::default(),
            ExecuteConfigInfo::default(),
        );

        assert_eq!(config.changeset.path, ".changesets");
        assert_eq!(config.version.strategy, "independent");
        assert!(config.dependency.propagate_dependencies);
    }

    #[test]
    fn test_config_data_default() {
        let config = ConfigData::default();

        assert_eq!(config.changeset.path, ".changesets");
        assert_eq!(config.version.strategy, "independent");
        assert_eq!(config.version.default_bump, "patch");
        assert!(config.dependency.propagate_dependencies);
        assert!(config.upgrade.auto_changeset);
        assert!(config.changelog.enabled);
        assert!(config.audit.enabled);
        assert_eq!(config.git.branch_base, "main");
        assert_eq!(config.execute.max_parallel, 4);
    }

    #[test]
    fn test_config_data_clone() {
        let config = ConfigData::default();
        let cloned = config.clone();

        assert_eq!(cloned.changeset.path, config.changeset.path);
        assert_eq!(cloned.version.strategy, config.version.strategy);
    }

    #[test]
    fn test_config_data_serialize() {
        let config = ConfigData::default();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("changeset"));
        assert!(json.contains("version"));
        assert!(json.contains("dependency"));
        assert!(json.contains("upgrade"));
        assert!(json.contains("changelog"));
        assert!(json.contains("audit"));
        assert!(json.contains("git"));
        assert!(json.contains("execute"));
    }

    #[test]
    fn test_config_show_data_new() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.json".to_string(), "json".to_string(), config);

        assert_eq!(show_data.config_path, "repo.config.json");
        assert_eq!(show_data.config_format, "json");
        assert_eq!(show_data.config.version.strategy, "independent");
    }

    #[test]
    fn test_config_show_data_with_toml_format() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.toml".to_string(), "toml".to_string(), config);

        assert_eq!(show_data.config_format, "toml");
    }

    #[test]
    fn test_config_show_data_with_yaml_format() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.yaml".to_string(), "yaml".to_string(), config);

        assert_eq!(show_data.config_format, "yaml");
    }

    #[test]
    fn test_config_show_data_clone() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.json".to_string(), "json".to_string(), config);
        let cloned = show_data.clone();

        assert_eq!(cloned.config_path, show_data.config_path);
        assert_eq!(cloned.config_format, show_data.config_format);
    }

    #[test]
    fn test_config_show_data_serialize() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.json".to_string(), "json".to_string(), config);
        let json = serde_json::to_string(&show_data).unwrap();

        assert!(json.contains("config_path"));
        assert!(json.contains("config_format"));
        assert!(json.contains("repo.config.json"));
    }
}

/// Tests for ConfigValidationIssue and ConfigValidateData (Story 7.1).
#[cfg(test)]
mod config_validation_tests {
    use crate::types::config::{ConfigValidateData, ConfigValidationIssue};

    #[test]
    fn test_config_validation_issue_error() {
        let issue = ConfigValidationIssue::error(
            "version.strategy".to_string(),
            "Invalid strategy value".to_string(),
        );

        assert_eq!(issue.severity, "error");
        assert_eq!(issue.field, "version.strategy");
        assert_eq!(issue.message, "Invalid strategy value");
        assert!(issue.suggestion.is_none());
    }

    #[test]
    fn test_config_validation_issue_error_with_suggestion() {
        let issue = ConfigValidationIssue::error_with_suggestion(
            "version.strategy".to_string(),
            "Invalid strategy value".to_string(),
            "Use 'independent' or 'unified'".to_string(),
        );

        assert_eq!(issue.severity, "error");
        assert!(issue.is_error());
        assert!(!issue.is_warning());
        assert!(issue.suggestion.is_some());
        assert_eq!(issue.suggestion.unwrap(), "Use 'independent' or 'unified'");
    }

    #[test]
    fn test_config_validation_issue_warning() {
        let issue = ConfigValidationIssue::warning(
            "changelog.repositoryUrl".to_string(),
            "Repository URL not set".to_string(),
        );

        assert_eq!(issue.severity, "warning");
        assert!(issue.is_warning());
        assert!(!issue.is_error());
        assert!(!issue.is_info());
    }

    #[test]
    fn test_config_validation_issue_warning_with_suggestion() {
        let issue = ConfigValidationIssue::warning_with_suggestion(
            "changelog.repositoryUrl".to_string(),
            "Repository URL not set".to_string(),
            "Add repository URL for commit links".to_string(),
        );

        assert!(issue.is_warning());
        assert!(issue.suggestion.is_some());
    }

    #[test]
    fn test_config_validation_issue_info() {
        let issue = ConfigValidationIssue::info(
            "execute.maxParallel".to_string(),
            "Consider increasing for faster builds".to_string(),
        );

        assert_eq!(issue.severity, "info");
        assert!(issue.is_info());
        assert!(!issue.is_error());
        assert!(!issue.is_warning());
    }

    #[test]
    fn test_config_validation_issue_new() {
        let issue = ConfigValidationIssue::new(
            "warning".to_string(),
            "test.field".to_string(),
            "Test message".to_string(),
            Some("Fix suggestion".to_string()),
        );

        assert_eq!(issue.severity, "warning");
        assert_eq!(issue.field, "test.field");
        assert_eq!(issue.message, "Test message");
        assert_eq!(issue.suggestion, Some("Fix suggestion".to_string()));
    }

    #[test]
    fn test_config_validation_issue_clone() {
        let issue = ConfigValidationIssue::error("field".to_string(), "message".to_string());
        let cloned = issue.clone();

        assert_eq!(cloned.severity, issue.severity);
        assert_eq!(cloned.field, issue.field);
    }

    #[test]
    fn test_config_validate_data_new() {
        let errors = vec![ConfigValidationIssue::error(
            "version.strategy".to_string(),
            "Invalid".to_string(),
        )];
        let warnings = vec![ConfigValidationIssue::warning(
            "changelog.repositoryUrl".to_string(),
            "Missing".to_string(),
        )];

        let data = ConfigValidateData::new(false, "repo.config.json".to_string(), errors, warnings);

        assert!(!data.valid);
        assert_eq!(data.config_path, "repo.config.json");
        assert_eq!(data.errors.len(), 1);
        assert_eq!(data.warnings.len(), 1);
    }

    #[test]
    fn test_config_validate_data_valid() {
        let data = ConfigValidateData::valid("repo.config.json".to_string());

        assert!(data.valid);
        assert!(data.errors.is_empty());
        assert!(data.warnings.is_empty());
        assert!(!data.has_errors());
        assert!(!data.has_warnings());
        assert_eq!(data.total_issues(), 0);
    }

    #[test]
    fn test_config_validate_data_valid_with_warnings() {
        let warnings =
            vec![ConfigValidationIssue::warning("field".to_string(), "warning".to_string())];
        let data =
            ConfigValidateData::valid_with_warnings("repo.config.json".to_string(), warnings);

        assert!(data.valid);
        assert!(data.errors.is_empty());
        assert!(!data.warnings.is_empty());
        assert!(!data.has_errors());
        assert!(data.has_warnings());
        assert_eq!(data.total_issues(), 1);
    }

    #[test]
    fn test_config_validate_data_invalid() {
        let errors = vec![
            ConfigValidationIssue::error("field1".to_string(), "error1".to_string()),
            ConfigValidationIssue::error("field2".to_string(), "error2".to_string()),
        ];
        let data = ConfigValidateData::invalid("repo.config.json".to_string(), errors);

        assert!(!data.valid);
        assert_eq!(data.errors.len(), 2);
        assert!(data.warnings.is_empty());
        assert!(data.has_errors());
        assert!(!data.has_warnings());
        assert_eq!(data.total_issues(), 2);
    }

    #[test]
    fn test_config_validate_data_invalid_with_warnings() {
        let errors = vec![ConfigValidationIssue::error("field".to_string(), "error".to_string())];
        let warnings =
            vec![ConfigValidationIssue::warning("field".to_string(), "warning".to_string())];
        let data = ConfigValidateData::invalid_with_warnings(
            "repo.config.json".to_string(),
            errors,
            warnings,
        );

        assert!(!data.valid);
        assert!(data.has_errors());
        assert!(data.has_warnings());
        assert_eq!(data.total_issues(), 2);
    }

    #[test]
    fn test_config_validate_data_serialize() {
        let data = ConfigValidateData::valid("repo.config.json".to_string());
        let json = serde_json::to_string(&data).unwrap();

        assert!(json.contains("valid"));
        assert!(json.contains("config_path"));
        assert!(json.contains("errors"));
        assert!(json.contains("warnings"));
    }
}

/// Tests for ConfigShowApiResponse and ConfigValidateApiResponse (Story 7.1).
#[cfg(test)]
mod config_api_response_tests {
    use crate::error::ErrorInfo;
    use crate::types::config::{
        ConfigData, ConfigShowApiResponse, ConfigShowData, ConfigValidateApiResponse,
        ConfigValidateData,
    };

    #[test]
    fn test_config_show_api_response_success() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.json".to_string(), "json".to_string(), config);
        let response = ConfigShowApiResponse::success(show_data);

        assert!(response.success);
        assert!(response.is_success());
        assert!(!response.is_failure());
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_config_show_api_response_failure() {
        let error = ErrorInfo::not_found("Config file not found", Some("repo.config.json"));
        let response = ConfigShowApiResponse::failure(error);

        assert!(!response.success);
        assert!(!response.is_success());
        assert!(response.is_failure());
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, "ENOENT");
    }

    #[test]
    fn test_config_show_api_response_failure_with_different_error_codes() {
        // Test ECONFIG
        let error = ErrorInfo::configuration("Invalid configuration format");
        let response = ConfigShowApiResponse::failure(error);
        assert_eq!(response.error.as_ref().unwrap().code, "ECONFIG");

        // Test EVALIDATION
        let error = ErrorInfo::validation("Invalid root path", Some("root"));
        let response = ConfigShowApiResponse::failure(error);
        assert_eq!(response.error.as_ref().unwrap().code, "EVALIDATION");
    }

    #[test]
    fn test_config_show_api_response_clone() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.json".to_string(), "json".to_string(), config);
        let response = ConfigShowApiResponse::success(show_data);
        let cloned = response.clone();

        assert_eq!(cloned.success, response.success);
    }

    #[test]
    fn test_config_show_api_response_serialize_success() {
        let config = ConfigData::default();
        let show_data =
            ConfigShowData::new("repo.config.json".to_string(), "json".to_string(), config);
        let response = ConfigShowApiResponse::success(show_data);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("data"));
    }

    #[test]
    fn test_config_show_api_response_serialize_failure() {
        let error = ErrorInfo::not_found("Not found", None::<String>);
        let response = ConfigShowApiResponse::failure(error);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":false"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_config_validate_api_response_success() {
        let data = ConfigValidateData::valid("repo.config.json".to_string());
        let response = ConfigValidateApiResponse::success(data);

        assert!(response.success);
        assert!(response.is_success());
        assert!(!response.is_failure());
        assert!(response.data.is_some());
        assert!(response.data.as_ref().unwrap().valid);
    }

    #[test]
    fn test_config_validate_api_response_success_with_validation_errors() {
        // The API response is still "success" because the command executed
        // The validation data shows whether the config is valid
        let data = ConfigValidateData::invalid(
            "repo.config.json".to_string(),
            vec![crate::types::config::ConfigValidationIssue::error(
                "field".to_string(),
                "error".to_string(),
            )],
        );
        let response = ConfigValidateApiResponse::success(data);

        assert!(response.success); // API call succeeded
        assert!(!response.data.as_ref().unwrap().valid); // But config is invalid
    }

    #[test]
    fn test_config_validate_api_response_failure() {
        let error = ErrorInfo::not_found("Config file not found", Some("repo.config.json"));
        let response = ConfigValidateApiResponse::failure(error);

        assert!(!response.success);
        assert!(response.is_failure());
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_config_validate_api_response_clone() {
        let data = ConfigValidateData::valid("repo.config.json".to_string());
        let response = ConfigValidateApiResponse::success(data);
        let cloned = response.clone();

        assert_eq!(cloned.success, response.success);
    }

    #[test]
    fn test_config_validate_api_response_serialize_success() {
        let data = ConfigValidateData::valid("repo.config.json".to_string());
        let response = ConfigValidateApiResponse::success(data);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"valid\":true"));
    }

    #[test]
    fn test_config_validate_api_response_serialize_failure() {
        let error = ErrorInfo::configuration("Parse error");
        let response = ConfigValidateApiResponse::failure(error);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":false"));
        assert!(json.contains("ECONFIG"));
    }
}

/// Tests for config constants (Story 7.1).
#[cfg(test)]
mod config_constants_tests {
    use crate::types::config::{
        VALID_BUMP_TYPES, VALID_CHANGELOG_FORMATS, VALID_MONOREPO_MODES, VALID_SEVERITY_LEVELS,
        VALID_STRATEGIES,
    };

    #[test]
    fn test_valid_strategies() {
        assert_eq!(VALID_STRATEGIES.len(), 2);
        assert!(VALID_STRATEGIES.contains(&"independent"));
        assert!(VALID_STRATEGIES.contains(&"unified"));
    }

    #[test]
    fn test_valid_bump_types() {
        assert_eq!(VALID_BUMP_TYPES.len(), 4);
        assert!(VALID_BUMP_TYPES.contains(&"major"));
        assert!(VALID_BUMP_TYPES.contains(&"minor"));
        assert!(VALID_BUMP_TYPES.contains(&"patch"));
        assert!(VALID_BUMP_TYPES.contains(&"none"));
    }

    #[test]
    fn test_valid_changelog_formats() {
        assert_eq!(VALID_CHANGELOG_FORMATS.len(), 3);
        assert!(VALID_CHANGELOG_FORMATS.contains(&"keep-a-changelog"));
        assert!(VALID_CHANGELOG_FORMATS.contains(&"conventional-commits"));
        assert!(VALID_CHANGELOG_FORMATS.contains(&"custom"));
    }

    #[test]
    fn test_valid_monorepo_modes() {
        assert_eq!(VALID_MONOREPO_MODES.len(), 3);
        assert!(VALID_MONOREPO_MODES.contains(&"per-package"));
        assert!(VALID_MONOREPO_MODES.contains(&"root"));
        assert!(VALID_MONOREPO_MODES.contains(&"both"));
    }

    #[test]
    fn test_valid_severity_levels() {
        assert_eq!(VALID_SEVERITY_LEVELS.len(), 3);
        assert!(VALID_SEVERITY_LEVELS.contains(&"error"));
        assert!(VALID_SEVERITY_LEVELS.contains(&"warning"));
        assert!(VALID_SEVERITY_LEVELS.contains(&"info"));
    }
}

/// Complete scenario tests for config commands (Story 7.1).
#[cfg(test)]
mod config_scenario_tests {
    use crate::error::ErrorInfo;
    use crate::types::config::{
        ConfigData, ConfigShowApiResponse, ConfigShowData, ConfigShowParams,
        ConfigValidateApiResponse, ConfigValidateData, ConfigValidateParams, ConfigValidationIssue,
        VersionConfigInfo,
    };

    #[test]
    fn test_complete_config_show_scenario() {
        // Simulate a complete configShow workflow
        let params = ConfigShowParams::new("/path/to/workspace".to_string());
        assert_eq!(params.root, "/path/to/workspace");

        // Simulate loaded config with custom version strategy
        let config = ConfigData {
            version: VersionConfigInfo::new(
                "unified".to_string(),
                "minor".to_string(),
                "{version}-dev".to_string(),
            ),
            ..Default::default()
        };

        let show_data = ConfigShowData::new(
            "/path/to/workspace/repo.config.json".to_string(),
            "json".to_string(),
            config,
        );

        let response = ConfigShowApiResponse::success(show_data);

        assert!(response.is_success());
        let data = response.data.unwrap();
        assert_eq!(data.config_format, "json");
        assert_eq!(data.config.version.strategy, "unified");
        assert_eq!(data.config.version.default_bump, "minor");
    }

    #[test]
    fn test_complete_config_validate_scenario_valid() {
        // Simulate a complete configValidate workflow for valid config
        let params = ConfigValidateParams::new(".".to_string());
        assert_eq!(params.root, ".");

        let data = ConfigValidateData::valid("repo.config.json".to_string());
        let response = ConfigValidateApiResponse::success(data);

        assert!(response.is_success());
        let data = response.data.unwrap();
        assert!(data.valid);
        assert!(data.errors.is_empty());
    }

    #[test]
    fn test_complete_config_validate_scenario_with_warnings() {
        // Simulate validation with warnings but no errors
        let warnings = vec![
            ConfigValidationIssue::warning(
                "changelog.repositoryUrl".to_string(),
                "Repository URL not set, commit links will not work".to_string(),
            ),
            ConfigValidationIssue::warning_with_suggestion(
                "execute.maxParallel".to_string(),
                "Low parallelism may slow down builds".to_string(),
                "Consider increasing to match CPU cores".to_string(),
            ),
        ];

        let data =
            ConfigValidateData::valid_with_warnings("repo.config.json".to_string(), warnings);

        assert!(data.valid);
        assert!(!data.has_errors());
        assert!(data.has_warnings());
        assert_eq!(data.warnings.len(), 2);
        assert!(data.warnings[1].suggestion.is_some());
    }

    #[test]
    fn test_complete_config_validate_scenario_invalid() {
        // Simulate validation with errors
        let errors = vec![
            ConfigValidationIssue::error(
                "version.strategy".to_string(),
                "Invalid strategy 'wrong'".to_string(),
            ),
            ConfigValidationIssue::error_with_suggestion(
                "changeset.path".to_string(),
                "Path does not exist".to_string(),
                "Create the directory or update the path".to_string(),
            ),
        ];
        let warnings = vec![ConfigValidationIssue::warning(
            "git.branchBase".to_string(),
            "Branch 'master' is deprecated, consider using 'main'".to_string(),
        )];

        let data = ConfigValidateData::invalid_with_warnings(
            "repo.config.json".to_string(),
            errors,
            warnings,
        );

        assert!(!data.valid);
        assert!(data.has_errors());
        assert!(data.has_warnings());
        assert_eq!(data.total_issues(), 3);

        // Verify error details
        assert!(data.errors[0].is_error());
        assert_eq!(data.errors[0].field, "version.strategy");
        assert!(data.errors[1].suggestion.is_some());
    }

    #[test]
    fn test_config_show_error_scenario() {
        // Simulate configShow failing because config file not found
        let params = ConfigShowParams::new("/invalid/path".to_string());
        assert_eq!(params.root, "/invalid/path");

        let error = ErrorInfo::not_found(
            "Configuration file not found in /invalid/path",
            Some("repo.config.json"),
        );
        let response = ConfigShowApiResponse::failure(error);

        assert!(response.is_failure());
        let error = response.error.unwrap();
        assert_eq!(error.code, "ENOENT");
        assert!(error.message.contains("not found"));
    }

    #[test]
    fn test_config_validate_error_scenario() {
        // Simulate configValidate failing because of parse error
        let params =
            ConfigValidateParams::with_config(".".to_string(), "broken.config.json".to_string());
        assert!(params.config_path.is_some());

        let error =
            ErrorInfo::configuration("Failed to parse configuration: unexpected token at line 5");
        let response = ConfigValidateApiResponse::failure(error);

        assert!(response.is_failure());
        let error = response.error.unwrap();
        assert_eq!(error.code, "ECONFIG");
    }
}
