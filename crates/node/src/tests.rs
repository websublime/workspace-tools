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
}

/// Tests for validation module (validation.rs).
#[cfg(test)]
mod validation_tests {
    use crate::validation::{
        validate_bump_type, validate_message_not_empty, validate_mutual_exclusion,
        validate_optional_timeout, validate_packages_not_empty, validate_root, validate_semver,
        validate_timeout,
    };
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_root_empty() {
        let result = validate_root("");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("root".to_string()));
    }

    #[test]
    fn test_validate_root_not_exists() {
        let result = validate_root("/this/path/does/not/exist/at/all");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "ENOENT");
    }

    #[test]
    fn test_validate_root_valid() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_root(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_root_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let result = validate_root(file_path.to_str().unwrap());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
    }

    #[test]
    fn test_validate_packages_not_empty_valid() {
        let packages = vec!["@scope/pkg1".to_string()];
        let result = validate_packages_not_empty(&packages);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_packages_not_empty_empty() {
        let packages: Vec<String> = vec![];
        let result = validate_packages_not_empty(&packages);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("packages".to_string()));
    }

    #[test]
    fn test_validate_bump_type_valid() {
        assert!(validate_bump_type("major").is_ok());
        assert!(validate_bump_type("minor").is_ok());
        assert!(validate_bump_type("patch").is_ok());
        assert!(validate_bump_type("premajor").is_ok());
        assert!(validate_bump_type("preminor").is_ok());
        assert!(validate_bump_type("prepatch").is_ok());
        assert!(validate_bump_type("prerelease").is_ok());
    }

    #[test]
    fn test_validate_bump_type_invalid() {
        let result = validate_bump_type("invalid");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("bumpType".to_string()));
    }

    #[test]
    fn test_validate_message_not_empty_valid() {
        let result = validate_message_not_empty("Add feature", "message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_message_not_empty_empty() {
        let result = validate_message_not_empty("", "message");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
    }

    #[test]
    fn test_validate_message_not_empty_whitespace() {
        let result = validate_message_not_empty("   ", "message");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_semver_valid() {
        assert!(validate_semver("1.0.0", "version").is_ok());
        assert!(validate_semver("2.3.4", "version").is_ok());
        assert!(validate_semver("10.20.30", "version").is_ok());
        assert!(validate_semver("1.0.0-beta.1", "version").is_ok());
    }

    #[test]
    fn test_validate_semver_invalid() {
        let result = validate_semver("invalid", "version");
        assert!(result.is_err());

        let result = validate_semver("1.0", "version");
        assert!(result.is_err());

        let result = validate_semver("a.b.c", "version");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_mutual_exclusion_none_set() {
        let result = validate_mutual_exclusion(&[("a", false), ("b", false)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_mutual_exclusion_one_set() {
        let result = validate_mutual_exclusion(&[("a", true), ("b", false)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_mutual_exclusion_both_set() {
        let result = validate_mutual_exclusion(&[("filterPackage", true), ("affected", true)]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
    }

    #[test]
    fn test_validate_timeout_valid() {
        assert!(validate_timeout(1, "timeout").is_ok());
        assert!(validate_timeout(30, "timeout").is_ok());
        assert!(validate_timeout(3600, "timeout").is_ok());
    }

    #[test]
    fn test_validate_timeout_zero() {
        let result = validate_timeout(0, "timeoutSecs");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "EVALIDATION");
        assert_eq!(error.context, Some("timeoutSecs".to_string()));
    }

    #[test]
    fn test_validate_optional_timeout_none() {
        let result = validate_optional_timeout(None, "timeout");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_optional_timeout_valid() {
        let result = validate_optional_timeout(Some(30), "timeout");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_optional_timeout_zero() {
        let result = validate_optional_timeout(Some(0), "timeout");
        assert!(result.is_err());
    }
}

/// Tests for response module (response.rs).
#[cfg(test)]
mod response_tests {
    use crate::error::ErrorInfo;
    use crate::response::{ApiResponseExt, JsonResponse, result_to_response};

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
