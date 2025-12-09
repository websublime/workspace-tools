//! Tests module for command implementations.
//!
//! # What
//!
//! This module contains unit tests for all command implementations in the
//! `commands` module. Tests are organized by command, with each command
//! having its own submodule for clarity and maintainability.
//!
//! # How
//!
//! Tests are organized into submodules:
//! - `status_tests`: Tests for the status command (Story 3.2)
//!
//! Each test submodule is further divided into logical groups:
//! - Component tests (e.g., SharedBuffer, parsing, conversion)
//! - Validation tests
//! - Integration-style tests
//!
//! # Why
//!
//! Centralizing command tests in a separate file:
//! - Keeps implementation files focused on production code
//! - Makes tests easier to find and maintain
//! - Follows the project's convention of grouping tests in dedicated files
//! - Allows for shared test utilities across command tests

// ============================================================================
// Status Command Tests (Story 3.2)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod status_tests {
    use std::io::Write;

    use crate::commands::status::{
        convert_to_napi_status, parse_status_response, validate_params, CliBranchInfo,
        CliChangesetInfo, CliPackageInfo, CliPackageManagerInfo, CliRepositoryInfo, CliStatusData,
        SharedBuffer,
    };
    use crate::types::status::StatusParams;

    // -------------------------------------------------------------------------
    // SharedBuffer Tests
    // -------------------------------------------------------------------------

    mod shared_buffer_tests {
        use super::*;

        #[test]
        fn test_shared_buffer_new() {
            let buffer = SharedBuffer::new();
            assert!(buffer.take_bytes().is_empty());
        }

        #[test]
        fn test_shared_buffer_write() {
            let mut buffer = SharedBuffer::new();
            let bytes_written = buffer.write(b"hello").unwrap();
            assert_eq!(bytes_written, 5);
            assert_eq!(buffer.take_bytes(), b"hello");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"hello ").unwrap();
            buffer.write_all(b"world").unwrap();
            assert_eq!(buffer.take_bytes(), b"hello world");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer1 = SharedBuffer::new();
            let buffer2 = buffer1.clone();

            buffer1.write_all(b"test").unwrap();

            // Both should see the same data
            assert_eq!(buffer1.take_bytes(), b"test");
            assert_eq!(buffer2.take_bytes(), b"test");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"data").unwrap();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"persistent").unwrap();

            // Multiple calls should return the same data
            assert_eq!(buffer.take_bytes(), b"persistent");
            assert_eq!(buffer.take_bytes(), b"persistent");
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_status_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "repository": {
                        "kind": "monorepo",
                        "monorepoType": "pnpm"
                    },
                    "packageManager": {
                        "name": "pnpm",
                        "lockFile": "pnpm-lock.yaml"
                    },
                    "branch": {
                        "name": "main"
                    },
                    "changesets": [
                        { "id": "feature-login" }
                    ],
                    "packages": [
                        {
                            "name": "@org/core",
                            "version": "1.0.0",
                            "path": "packages/core"
                        }
                    ]
                }
            }"#;

            let result = parse_status_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.repository.kind, "monorepo");
            assert_eq!(data.repository.monorepo_type, Some("pnpm".to_string()));
            assert_eq!(data.package_manager.name, "pnpm");
            assert_eq!(data.package_manager.lock_file, "pnpm-lock.yaml");
            assert!(data.branch.is_some());
            assert_eq!(data.branch.unwrap().name, "main");
            assert_eq!(data.changesets.len(), 1);
            assert_eq!(data.changesets[0].id, "feature-login");
            assert_eq!(data.packages.len(), 1);
            assert_eq!(data.packages[0].name, "@org/core");
            assert_eq!(data.packages[0].version, "1.0.0");
            assert_eq!(data.packages[0].path, "packages/core");
        }

        #[test]
        fn test_parse_status_response_simple_repo() {
            let json = r#"{
                "success": true,
                "data": {
                    "repository": {
                        "kind": "simple"
                    },
                    "packageManager": {
                        "name": "npm",
                        "lockFile": "package-lock.json"
                    },
                    "changesets": [],
                    "packages": [
                        {
                            "name": "my-package",
                            "version": "0.1.0",
                            "path": "."
                        }
                    ]
                }
            }"#;

            let result = parse_status_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.repository.kind, "simple");
            assert!(data.repository.monorepo_type.is_none());
            assert!(data.branch.is_none());
            assert!(data.changesets.is_empty());
        }

        #[test]
        fn test_parse_status_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Not a valid Node.js project: package.json not found"
            }"#;

            let result = parse_status_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EEXEC");
            assert!(error.message.contains("package.json not found"));
        }

        #[test]
        fn test_parse_status_response_empty() {
            let result = parse_status_response(b"");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_status_response_whitespace_only() {
            let result = parse_status_response(b"   \n\t  ");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_status_response_invalid_json() {
            let result = parse_status_response(b"not valid json");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_status_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_status_response(&invalid_utf8);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_status_response_success_no_data() {
            let json = r#"{
                "success": true
            }"#;

            let result = parse_status_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("no data"));
        }

        #[test]
        fn test_parse_status_response_cli_error_no_message() {
            let json = r#"{
                "success": false
            }"#;

            let result = parse_status_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Unknown CLI error"));
        }
    }

    // -------------------------------------------------------------------------
    // Conversion Tests
    // -------------------------------------------------------------------------

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_convert_to_napi_status_full() {
            let cli_data = CliStatusData {
                repository: CliRepositoryInfo {
                    kind: "monorepo".to_string(),
                    monorepo_type: Some("yarn".to_string()),
                },
                package_manager: CliPackageManagerInfo {
                    name: "yarn".to_string(),
                    lock_file: "yarn.lock".to_string(),
                },
                branch: Some(CliBranchInfo { name: "develop".to_string() }),
                changesets: vec![
                    CliChangesetInfo { id: "fix-bug".to_string() },
                    CliChangesetInfo { id: "add-feature".to_string() },
                ],
                packages: vec![
                    CliPackageInfo {
                        name: "@org/pkg1".to_string(),
                        version: "1.0.0".to_string(),
                        path: "packages/pkg1".to_string(),
                    },
                    CliPackageInfo {
                        name: "@org/pkg2".to_string(),
                        version: "2.0.0".to_string(),
                        path: "packages/pkg2".to_string(),
                    },
                ],
            };

            let napi_data = convert_to_napi_status(cli_data);

            assert_eq!(napi_data.repository.kind, "monorepo");
            assert_eq!(napi_data.repository.monorepo_type, Some("yarn".to_string()));
            assert_eq!(napi_data.package_manager.name, "yarn");
            assert_eq!(napi_data.package_manager.lock_file, "yarn.lock");
            assert_eq!(napi_data.branch.as_ref().map(|b| &b.name), Some(&"develop".to_string()));
            assert_eq!(napi_data.changesets.len(), 2);
            assert_eq!(napi_data.packages.len(), 2);
        }

        #[test]
        fn test_convert_to_napi_status_minimal() {
            let cli_data = CliStatusData {
                repository: CliRepositoryInfo { kind: "simple".to_string(), monorepo_type: None },
                package_manager: CliPackageManagerInfo {
                    name: "npm".to_string(),
                    lock_file: "package-lock.json".to_string(),
                },
                branch: None,
                changesets: vec![],
                packages: vec![],
            };

            let napi_data = convert_to_napi_status(cli_data);

            assert_eq!(napi_data.repository.kind, "simple");
            assert!(napi_data.repository.monorepo_type.is_none());
            assert!(napi_data.branch.is_none());
            assert!(napi_data.changesets.is_empty());
            assert!(napi_data.packages.is_empty());
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_params_valid_directory() {
            // Use current directory which should always exist
            let params = StatusParams { root: ".".to_string(), config_path: None };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_nonexistent_path() {
            let params = StatusParams {
                root: "/this/path/definitely/does/not/exist/12345".to_string(),
                config_path: None,
            };

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_params_empty_root() {
            let params = StatusParams { root: String::new(), config_path: None };

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_params_file_not_directory() {
            // Create a temporary file
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            let file_path = temp_file.path().to_string_lossy().to_string();

            let params = StatusParams { root: file_path, config_path: None };

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("directory"));
        }

        #[test]
        fn test_validate_params_with_config_path() {
            let params = StatusParams {
                root: ".".to_string(),
                config_path: Some("/some/config/path.json".to_string()),
            };

            // Config path is not validated by validate_params
            // (it's passed to CLI which handles it)
            let result = validate_params(&params);
            assert!(result.is_ok());
        }
    }
}
