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
//! - `init_tests`: Tests for the init command (Story 3.4)
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
        CliBranchInfo, CliChangesetInfo, CliPackageInfo, CliPackageManagerInfo, CliRepositoryInfo,
        CliStatusData, SharedBuffer, convert_to_napi_status, parse_status_response,
        validate_params,
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

// ============================================================================
// Init Command Tests (Story 3.4)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod init_tests {
    use std::io::Write;

    use crate::commands::init::{
        CliInitData, SharedBuffer, convert_params_to_args, convert_to_napi_init,
        parse_init_response, validate_params,
    };
    use crate::types::init::InitParams;

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
        fn test_parse_init_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "configFile": "repo.config.toml",
                    "configFormat": "toml",
                    "strategy": "independent",
                    "changesetPath": ".changesets",
                    "environments": ["dev", "staging", "production"],
                    "defaultEnvironments": ["production"],
                    "registry": "https://registry.npmjs.org"
                }
            }"#;

            let result = parse_init_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.config_file, "repo.config.toml");
            assert_eq!(data.config_format, "toml");
            assert_eq!(data.strategy, "independent");
            assert_eq!(data.changeset_path, ".changesets");
            assert_eq!(data.environments.len(), 3);
            assert_eq!(data.environments[0], "dev");
            assert_eq!(data.environments[1], "staging");
            assert_eq!(data.environments[2], "production");
            assert_eq!(data.default_environments.len(), 1);
            assert_eq!(data.default_environments[0], "production");
            assert_eq!(data.registry, "https://registry.npmjs.org");
        }

        #[test]
        fn test_parse_init_response_unified_strategy() {
            let json = r#"{
                "success": true,
                "data": {
                    "configFile": "repo.config.json",
                    "configFormat": "json",
                    "strategy": "unified",
                    "changesetPath": ".changes",
                    "environments": ["prod"],
                    "defaultEnvironments": ["prod"],
                    "registry": "https://npm.pkg.github.com"
                }
            }"#;

            let result = parse_init_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.config_file, "repo.config.json");
            assert_eq!(data.config_format, "json");
            assert_eq!(data.strategy, "unified");
            assert_eq!(data.changeset_path, ".changes");
            assert_eq!(data.registry, "https://npm.pkg.github.com");
        }

        #[test]
        fn test_parse_init_response_yaml_format() {
            let json = r#"{
                "success": true,
                "data": {
                    "configFile": "repo.config.yaml",
                    "configFormat": "yaml",
                    "strategy": "independent",
                    "changesetPath": ".changesets",
                    "environments": [],
                    "defaultEnvironments": [],
                    "registry": "https://registry.npmjs.org"
                }
            }"#;

            let result = parse_init_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.config_file, "repo.config.yaml");
            assert_eq!(data.config_format, "yaml");
            assert!(data.environments.is_empty());
            assert!(data.default_environments.is_empty());
        }

        #[test]
        fn test_parse_init_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Configuration file already exists: repo.config.toml. Use --force to overwrite."
            }"#;

            let result = parse_init_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            // Should be ECONFIG for config-related errors
            assert_eq!(error.code, "ECONFIG");
            assert!(error.message.contains("already exists"));
        }

        #[test]
        fn test_parse_init_response_validation_error() {
            let json = r#"{
                "success": false,
                "error": "No package.json found. This does not appear to be a Node.js project."
            }"#;

            let result = parse_init_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EEXEC");
            assert!(error.message.contains("package.json"));
        }

        #[test]
        fn test_parse_init_response_empty() {
            let result = parse_init_response(b"");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_init_response_whitespace_only() {
            let result = parse_init_response(b"   \n\t  ");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_init_response_invalid_json() {
            let result = parse_init_response(b"not valid json");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_init_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_init_response(&invalid_utf8);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_init_response_success_no_data() {
            let json = r#"{
                "success": true
            }"#;

            let result = parse_init_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("no data"));
        }

        #[test]
        fn test_parse_init_response_cli_error_no_message() {
            let json = r#"{
                "success": false
            }"#;

            let result = parse_init_response(json.as_bytes());
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
        fn test_convert_to_napi_init_full() {
            let cli_data = CliInitData {
                config_file: "repo.config.toml".to_string(),
                config_format: "toml".to_string(),
                strategy: "independent".to_string(),
                changeset_path: ".changesets".to_string(),
                environments: vec![
                    "dev".to_string(),
                    "staging".to_string(),
                    "production".to_string(),
                ],
                default_environments: vec!["production".to_string()],
                registry: "https://registry.npmjs.org".to_string(),
            };

            let napi_data = convert_to_napi_init(cli_data);

            assert_eq!(napi_data.config_file, "repo.config.toml");
            assert_eq!(napi_data.config_format, "toml");
            assert_eq!(napi_data.strategy, "independent");
            assert_eq!(napi_data.changeset_path, ".changesets");
            assert_eq!(napi_data.environments.len(), 3);
            assert_eq!(napi_data.default_environments.len(), 1);
            assert_eq!(napi_data.registry, "https://registry.npmjs.org");
        }

        #[test]
        fn test_convert_to_napi_init_minimal() {
            let cli_data = CliInitData {
                config_file: "repo.config.json".to_string(),
                config_format: "json".to_string(),
                strategy: "unified".to_string(),
                changeset_path: ".changes".to_string(),
                environments: vec![],
                default_environments: vec![],
                registry: "https://npm.example.com".to_string(),
            };

            let napi_data = convert_to_napi_init(cli_data);

            assert_eq!(napi_data.config_file, "repo.config.json");
            assert_eq!(napi_data.config_format, "json");
            assert_eq!(napi_data.strategy, "unified");
            assert!(napi_data.environments.is_empty());
            assert!(napi_data.default_environments.is_empty());
        }

        #[test]
        fn test_convert_params_to_args_defaults() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: None,
                force: None,
            };

            let args = convert_params_to_args(&params);

            // Check defaults are applied
            assert_eq!(args.changeset_path.to_string_lossy(), ".changesets");
            assert_eq!(args.registry, "https://registry.npmjs.org");
            assert!(!args.force);
            assert!(args.non_interactive); // Always true for API
            assert!(args.environments.is_none());
            assert!(args.default_env.is_none());
            assert!(args.strategy.is_none());
            assert!(args.config_format.is_none());
        }

        #[test]
        fn test_convert_params_to_args_custom() {
            let params = InitParams {
                root: "/path/to/project".to_string(),
                changeset_path: Some(".my-changesets".to_string()),
                environments: Some(vec!["dev".to_string(), "prod".to_string()]),
                default_env: Some(vec!["prod".to_string()]),
                strategy: Some("independent".to_string()),
                registry: Some("https://npm.pkg.github.com".to_string()),
                config_format: Some("yaml".to_string()),
                force: Some(true),
            };

            let args = convert_params_to_args(&params);

            assert_eq!(args.changeset_path.to_string_lossy(), ".my-changesets");
            assert_eq!(args.registry, "https://npm.pkg.github.com");
            assert!(args.force);
            assert!(args.non_interactive); // Always true for API
            assert_eq!(args.environments, Some(vec!["dev".to_string(), "prod".to_string()]));
            assert_eq!(args.default_env, Some(vec!["prod".to_string()]));
            assert_eq!(args.strategy, Some("independent".to_string()));
            assert_eq!(args.config_format, Some("yaml".to_string()));
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
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: None,
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_nonexistent_path() {
            let params = InitParams {
                root: "/this/path/definitely/does/not/exist/12345".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: None,
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_params_empty_root() {
            let params = InitParams {
                root: String::new(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: None,
                force: None,
            };

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

            let params = InitParams {
                root: file_path,
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: None,
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("directory"));
        }

        #[test]
        fn test_validate_params_valid_strategy_independent() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: Some("independent".to_string()),
                registry: None,
                config_format: None,
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_valid_strategy_unified() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: Some("unified".to_string()),
                registry: None,
                config_format: None,
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_valid_strategy_case_insensitive() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: Some("INDEPENDENT".to_string()),
                registry: None,
                config_format: None,
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_invalid_strategy() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: Some("invalid-strategy".to_string()),
                registry: None,
                config_format: None,
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("Invalid strategy"));
            assert!(error.message.contains("independent"));
            assert!(error.message.contains("unified"));
            assert_eq!(error.context, Some("strategy".to_string()));
        }

        #[test]
        fn test_validate_params_valid_config_format_json() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: Some("json".to_string()),
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_valid_config_format_yaml() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: Some("yaml".to_string()),
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_valid_config_format_toml() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: Some("toml".to_string()),
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_valid_config_format_case_insensitive() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: Some("TOML".to_string()),
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_invalid_config_format() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: None,
                environments: None,
                default_env: None,
                strategy: None,
                registry: None,
                config_format: Some("xml".to_string()),
                force: None,
            };

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("Invalid config format"));
            assert!(error.message.contains("json"));
            assert!(error.message.contains("yaml"));
            assert!(error.message.contains("toml"));
            assert_eq!(error.context, Some("configFormat".to_string()));
        }

        #[test]
        fn test_validate_params_all_optional_fields() {
            let params = InitParams {
                root: ".".to_string(),
                changeset_path: Some(".changesets".to_string()),
                environments: Some(vec!["dev".to_string(), "prod".to_string()]),
                default_env: Some(vec!["prod".to_string()]),
                strategy: Some("independent".to_string()),
                registry: Some("https://registry.npmjs.org".to_string()),
                config_format: Some("toml".to_string()),
                force: Some(true),
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
        }
    }
}
