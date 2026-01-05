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

// ============================================================================
// Changeset Add Command Tests (Story 4.2)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod changeset_add_tests {
    use std::io::Write;

    use crate::commands::changeset::{
        CliChangesetInfo, SharedBuffer, convert_params_to_args, convert_to_napi_add_data,
        parse_changeset_add_response, validate_params,
    };
    use crate::types::changeset::ChangesetAddParams;

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
            let written = buffer.write(b"hello").unwrap();
            assert_eq!(written, 5);
            assert_eq!(buffer.take_bytes(), b"hello");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"hello ");
            let _ = buffer.write(b"world");
            assert_eq!(buffer.take_bytes(), b"hello world");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();

            let _ = buffer.write(b"test data");

            // Both should see the same data
            assert_eq!(buffer.take_bytes(), b"test data");
            assert_eq!(buffer_clone.take_bytes(), b"test data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            // Flush should always succeed (no-op for Vec)
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"preserved");

            // take_bytes clones, so data is preserved
            let first_take = buffer.take_bytes();
            let second_take = buffer.take_bytes();
            assert_eq!(first_take, second_take);
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_changeset_add_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changeset": {
                        "branch": "feature/new-api",
                        "bump": "minor",
                        "packages": ["@scope/core", "@scope/utils"],
                        "environments": ["staging", "production"],
                        "commits": ["abc123", "def456"],
                        "created_at": "2025-01-20T10:00:00Z",
                        "updated_at": "2025-01-20T10:00:00Z"
                    },
                    "message": "Add new feature"
                }
            }"#;

            let result = parse_changeset_add_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.id, "feature/new-api");
            assert_eq!(data.branch, "feature/new-api");
            assert_eq!(data.bump, "minor");
            assert_eq!(data.packages, vec!["@scope/core", "@scope/utils"]);
            assert_eq!(data.environments, vec!["staging", "production"]);
            assert_eq!(data.created_at, "2025-01-20T10:00:00Z");
        }

        #[test]
        fn test_parse_changeset_add_response_minimal() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changeset": {
                        "branch": "main",
                        "bump": "patch",
                        "packages": ["my-package"],
                        "environments": [],
                        "commits": [],
                        "created_at": "2025-01-20T10:00:00Z",
                        "updated_at": "2025-01-20T10:00:00Z"
                    }
                }
            }"#;

            let result = parse_changeset_add_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.id, "main");
            assert_eq!(data.packages, vec!["my-package"]);
            assert!(data.environments.is_empty());
        }

        #[test]
        fn test_parse_changeset_add_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Changeset already exists for branch 'feature/test'"
            }"#;

            let result = parse_changeset_add_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("already exists"));
        }

        #[test]
        fn test_parse_changeset_add_response_empty() {
            let result = parse_changeset_add_response(b"");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_changeset_add_response_whitespace_only() {
            let result = parse_changeset_add_response(b"   \n\t  ");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_changeset_add_response_invalid_json() {
            let result = parse_changeset_add_response(b"not valid json");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_changeset_add_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_changeset_add_response(&invalid_utf8);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_changeset_add_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_changeset_add_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("success but no data"));
        }

        #[test]
        fn test_parse_changeset_add_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_changeset_add_response(json.as_bytes());
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
        fn test_convert_to_napi_add_data() {
            let cli_info = CliChangesetInfo {
                branch: "feature/test".to_string(),
                bump: "minor".to_string(),
                packages: vec!["pkg-a".to_string(), "pkg-b".to_string()],
                environments: vec!["production".to_string()],
                commits: vec!["commit1".to_string()],
                created_at: "2025-01-20T10:00:00Z".to_string(),
                updated_at: "2025-01-20T12:00:00Z".to_string(),
            };

            let data = convert_to_napi_add_data(cli_info);

            assert_eq!(data.id, "feature/test");
            assert_eq!(data.branch, "feature/test");
            assert_eq!(data.bump, "minor");
            assert_eq!(data.packages, vec!["pkg-a", "pkg-b"]);
            assert_eq!(data.environments, vec!["production"]);
            assert_eq!(data.created_at, "2025-01-20T10:00:00Z");
        }

        #[test]
        fn test_convert_params_to_args_defaults() {
            let params = ChangesetAddParams::new(".");

            let args = convert_params_to_args(&params);

            assert!(args.bump.is_none());
            assert!(args.env.is_none());
            assert!(args.branch.is_none());
            assert!(args.message.is_none());
            assert!(args.packages.is_none());
            assert!(args.non_interactive); // Always true
            assert!(!args.force);
        }

        #[test]
        fn test_convert_params_to_args_custom() {
            let params = ChangesetAddParams::new(".")
                .with_bump("major")
                .with_environments(vec!["staging".to_string()])
                .with_branch("feature/test")
                .with_message("Test message")
                .with_packages(vec!["my-pkg".to_string()])
                .with_force(true);

            let args = convert_params_to_args(&params);

            assert_eq!(args.bump, Some("major".to_string()));
            assert_eq!(args.env, Some(vec!["staging".to_string()]));
            assert_eq!(args.branch, Some("feature/test".to_string()));
            assert_eq!(args.message, Some("Test message".to_string()));
            assert_eq!(args.packages, Some(vec!["my-pkg".to_string()]));
            assert!(args.non_interactive);
            assert!(args.force);
        }

        #[test]
        fn test_convert_params_to_args_force_false_explicit() {
            let params = ChangesetAddParams::new(".").with_force(false);

            let args = convert_params_to_args(&params);

            assert!(!args.force);
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn test_validate_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetAddParams::new(temp_dir.path().to_str().unwrap());

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_nonexistent_path() {
            let params = ChangesetAddParams::new("/nonexistent/path/that/does/not/exist");

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_params_empty_root() {
            let params = ChangesetAddParams::new("");

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            fs::write(&file_path, "test").unwrap();

            let params = ChangesetAddParams::new(file_path.to_str().unwrap());

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_params_valid_bump_type() {
            let temp_dir = TempDir::new().unwrap();
            let params =
                ChangesetAddParams::new(temp_dir.path().to_str().unwrap()).with_bump("minor");

            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_invalid_bump_type() {
            let temp_dir = TempDir::new().unwrap();
            let params =
                ChangesetAddParams::new(temp_dir.path().to_str().unwrap()).with_bump("invalid");

            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("invalid bump type"));
        }

        #[test]
        fn test_validate_params_all_bump_types() {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path().to_str().unwrap();

            for bump in &["major", "minor", "patch", "none"] {
                let params = ChangesetAddParams::new(root).with_bump(*bump);
                let result = validate_params(&params);
                assert!(result.is_ok(), "Bump type '{bump}' should be valid");
            }
        }

        #[test]
        fn test_validate_params_with_all_options() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetAddParams::new(temp_dir.path().to_str().unwrap())
                .with_bump("minor")
                .with_environments(vec!["staging".to_string()])
                .with_branch("feature/test")
                .with_message("Test message")
                .with_packages(vec!["my-pkg".to_string()])
                .with_force(true);

            let result = validate_params(&params);
            assert!(result.is_ok());
        }
    }
}

// =============================================================================
// Changeset Update Tests (Story 4.3)
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod changeset_update_tests {
    use std::io::Write;

    use crate::commands::changeset::{
        CliUpdateSummary, CliUpdatedChangesetInfo, SharedBuffer, convert_to_napi_changeset_detail,
        convert_to_napi_update_summary, convert_update_params_to_args,
        parse_changeset_update_response, validate_update_params,
    };
    use crate::types::changeset::ChangesetUpdateParams;

    // -------------------------------------------------------------------------
    // SharedBuffer Tests (Reused from changeset_add, but included for completeness)
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
            let written = buffer.write(b"hello").unwrap();
            assert_eq!(written, 5);
            assert_eq!(buffer.take_bytes(), b"hello");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"hello ");
            let _ = buffer.write(b"world");
            assert_eq!(buffer.take_bytes(), b"hello world");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();

            let _ = buffer.write(b"test data");

            // Both should see the same data
            assert_eq!(buffer.take_bytes(), b"test data");
            assert_eq!(buffer_clone.take_bytes(), b"test data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            // Flush should always succeed (no-op for Vec)
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"preserved");

            // take_bytes clones, so data is preserved
            let first_take = buffer.take_bytes();
            let second_take = buffer.take_bytes();
            assert_eq!(first_take, second_take);
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_changeset_update_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "updated": {
                        "packages_added": 2,
                        "commits_added": 1,
                        "bump_updated": true,
                        "environments_added": 1
                    },
                    "changeset": {
                        "branch": "feature/new-api",
                        "bump": "major",
                        "packages": ["@scope/core", "@scope/utils", "@scope/new-pkg"],
                        "environments": ["staging", "production"],
                        "commits": ["abc123", "def456", "ghi789"],
                        "created_at": "2025-01-20T10:00:00Z",
                        "updated_at": "2025-01-20T12:00:00Z"
                    }
                }
            }"#;

            let result = parse_changeset_update_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert!(data.updated);
            assert_eq!(data.summary.packages_added, 2);
            assert_eq!(data.summary.commits_added, 1);
            assert!(data.summary.bump_updated);
            assert_eq!(data.summary.environments_added, 1);
            assert_eq!(data.changeset.branch, "feature/new-api");
            assert_eq!(data.changeset.bump, "major");
            assert_eq!(data.changeset.packages.len(), 3);
        }

        #[test]
        fn test_parse_changeset_update_response_no_changes() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "updated": {
                        "packages_added": 0,
                        "commits_added": 0,
                        "bump_updated": false,
                        "environments_added": 0
                    },
                    "changeset": {
                        "branch": "main",
                        "bump": "patch",
                        "packages": ["my-package"],
                        "environments": [],
                        "commits": [],
                        "created_at": "2025-01-20T10:00:00Z",
                        "updated_at": "2025-01-20T10:00:00Z"
                    }
                }
            }"#;

            let result = parse_changeset_update_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            // When no changes were made, updated should be false
            assert!(!data.updated);
            assert_eq!(data.summary.packages_added, 0);
            assert!(!data.summary.bump_updated);
        }

        #[test]
        fn test_parse_changeset_update_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "No changeset found for branch 'nonexistent'"
            }"#;

            let result = parse_changeset_update_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("No changeset found"));
        }

        #[test]
        fn test_parse_changeset_update_response_empty() {
            let result = parse_changeset_update_response(b"");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_changeset_update_response_whitespace_only() {
            let result = parse_changeset_update_response(b"   \n\t  ");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_changeset_update_response_invalid_json() {
            let result = parse_changeset_update_response(b"not valid json");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_changeset_update_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_changeset_update_response(&invalid_utf8);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_changeset_update_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_changeset_update_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("success but no data"));
        }

        #[test]
        fn test_parse_changeset_update_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_changeset_update_response(json.as_bytes());
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
        fn test_convert_to_napi_update_summary() {
            let cli_summary = CliUpdateSummary {
                packages_added: 3,
                commits_added: 1,
                bump_updated: true,
                environments_added: 2,
            };

            let summary = convert_to_napi_update_summary(&cli_summary);

            assert_eq!(summary.packages_added, 3);
            assert_eq!(summary.commits_added, 1);
            assert!(summary.bump_updated);
            assert_eq!(summary.environments_added, 2);
            assert!(summary.has_changes());
        }

        #[test]
        fn test_convert_to_napi_update_summary_no_changes() {
            let cli_summary = CliUpdateSummary {
                packages_added: 0,
                commits_added: 0,
                bump_updated: false,
                environments_added: 0,
            };

            let summary = convert_to_napi_update_summary(&cli_summary);

            assert_eq!(summary.packages_added, 0);
            assert_eq!(summary.commits_added, 0);
            assert!(!summary.bump_updated);
            assert_eq!(summary.environments_added, 0);
            assert!(!summary.has_changes());
        }

        #[test]
        fn test_convert_to_napi_changeset_detail() {
            let cli_info = CliUpdatedChangesetInfo {
                branch: "feature/test".to_string(),
                bump: "minor".to_string(),
                packages: vec!["pkg-a".to_string(), "pkg-b".to_string()],
                environments: vec!["production".to_string()],
                commits: vec!["commit1".to_string(), "commit2".to_string()],
                created_at: "2025-01-20T10:00:00Z".to_string(),
                updated_at: "2025-01-20T12:00:00Z".to_string(),
            };

            let detail = convert_to_napi_changeset_detail(&cli_info);

            assert_eq!(detail.id, "feature/test");
            assert_eq!(detail.branch, "feature/test");
            assert_eq!(detail.bump, "minor");
            assert_eq!(detail.packages, vec!["pkg-a", "pkg-b"]);
            assert_eq!(detail.environments, vec!["production"]);
            assert_eq!(detail.commits, vec!["commit1", "commit2"]);
            assert!(detail.message.is_none());
            assert_eq!(detail.created_at, "2025-01-20T10:00:00Z");
            assert_eq!(detail.updated_at, "2025-01-20T12:00:00Z");
        }

        #[test]
        fn test_convert_update_params_to_args_defaults() {
            let params = ChangesetUpdateParams::new(".").with_id("feature/test");

            let args = convert_update_params_to_args(&params);

            assert_eq!(args.id, Some("feature/test".to_string()));
            assert!(args.commit.is_none());
            assert!(args.packages.is_none());
            assert!(args.bump.is_none());
            assert!(args.env.is_none());
        }

        #[test]
        fn test_convert_update_params_to_args_full() {
            let params = ChangesetUpdateParams::new(".")
                .with_id("feature/test")
                .with_commit("abc123")
                .with_packages(vec!["pkg-a".to_string(), "pkg-b".to_string()])
                .with_bump("major")
                .with_environments(vec!["staging".to_string()]);

            let args = convert_update_params_to_args(&params);

            assert_eq!(args.id, Some("feature/test".to_string()));
            assert_eq!(args.commit, Some("abc123".to_string()));
            assert_eq!(args.packages, Some(vec!["pkg-a".to_string(), "pkg-b".to_string()]));
            assert_eq!(args.bump, Some("major".to_string()));
            assert_eq!(args.env, Some(vec!["staging".to_string()]));
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn test_validate_update_params_valid() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetUpdateParams::new(temp_dir.path().to_str().unwrap())
                .with_id("feature/test");

            let result = validate_update_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_update_params_missing_id() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetUpdateParams::new(temp_dir.path().to_str().unwrap());

            let result = validate_update_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("id is required"));
        }

        #[test]
        fn test_validate_update_params_nonexistent_path() {
            let params = ChangesetUpdateParams::new("/nonexistent/path/that/does/not/exist")
                .with_id("feature/test");

            let result = validate_update_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_update_params_empty_root() {
            let params = ChangesetUpdateParams::new("").with_id("feature/test");

            let result = validate_update_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_update_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            fs::write(&file_path, "test").unwrap();

            let params =
                ChangesetUpdateParams::new(file_path.to_str().unwrap()).with_id("feature/test");

            let result = validate_update_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_update_params_valid_bump_type() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetUpdateParams::new(temp_dir.path().to_str().unwrap())
                .with_id("feature/test")
                .with_bump("minor");

            let result = validate_update_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_update_params_invalid_bump_type() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetUpdateParams::new(temp_dir.path().to_str().unwrap())
                .with_id("feature/test")
                .with_bump("invalid");

            let result = validate_update_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("invalid bump type"));
        }

        #[test]
        fn test_validate_update_params_all_bump_types() {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path().to_str().unwrap();

            for bump in &["major", "minor", "patch", "none"] {
                let params =
                    ChangesetUpdateParams::new(root).with_id("feature/test").with_bump(*bump);
                let result = validate_update_params(&params);
                assert!(result.is_ok(), "Bump type '{bump}' should be valid");
            }
        }

        #[test]
        fn test_validate_update_params_with_all_options() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetUpdateParams::new(temp_dir.path().to_str().unwrap())
                .with_id("feature/test")
                .with_commit("abc123")
                .with_packages(vec!["pkg-a".to_string()])
                .with_bump("major")
                .with_environments(vec!["staging".to_string()]);

            let result = validate_update_params(&params);
            assert!(result.is_ok());
        }
    }
}

// =============================================================================
// Changeset List Tests (Story 4.4)
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod changeset_list_tests {
    use std::io::Write;

    use crate::commands::changeset::{
        CliChangesetListItem, CliChangesetListResponseData, SharedBuffer,
        convert_list_item_to_napi, convert_list_params_to_args, convert_to_napi_list_data,
        parse_changeset_list_response, validate_list_params,
    };
    use crate::types::changeset::{ChangesetListParams, VALID_SORT_OPTIONS};

    // -------------------------------------------------------------------------
    // SharedBuffer Tests (Reused from changeset_add, but included for completeness)
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
            let written = buffer.write(b"hello").unwrap();
            assert_eq!(written, 5);
            assert_eq!(buffer.take_bytes(), b"hello");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"hello ");
            let _ = buffer.write(b"world");
            assert_eq!(buffer.take_bytes(), b"hello world");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();

            let _ = buffer.write(b"test data");

            // Both should see the same data
            assert_eq!(buffer.take_bytes(), b"test data");
            assert_eq!(buffer_clone.take_bytes(), b"test data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            // Flush should always succeed (no-op for Vec)
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"preserved");

            // take_bytes clones, so data is preserved
            let first_take = buffer.take_bytes();
            let second_take = buffer.take_bytes();
            assert_eq!(first_take, second_take);
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_changeset_list_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changesets": [
                        {
                            "branch": "feature/new-api",
                            "bump": "minor",
                            "packages": ["@scope/core", "@scope/utils"],
                            "environments": ["staging"],
                            "commit_count": 3,
                            "created_at": "2024-01-15T10:30:00Z",
                            "updated_at": "2024-01-15T14:45:00Z"
                        },
                        {
                            "branch": "feature/breaking-change",
                            "bump": "major",
                            "packages": ["@scope/api"],
                            "environments": ["production"],
                            "commit_count": 1,
                            "created_at": "2024-01-14T09:00:00Z",
                            "updated_at": "2024-01-14T09:00:00Z"
                        }
                    ],
                    "total": 2
                }
            }"#;

            let result = parse_changeset_list_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.count, 2);
            assert_eq!(data.changesets.len(), 2);

            // Check first changeset
            let first = &data.changesets[0];
            assert_eq!(first.branch, "feature/new-api");
            assert_eq!(first.bump, "minor");
            assert_eq!(first.packages, vec!["@scope/core", "@scope/utils"]);
            assert_eq!(first.environments, vec!["staging"]);
            assert_eq!(first.commit_count, 3);
            assert_eq!(first.created_at, "2024-01-15T10:30:00Z");
            assert_eq!(first.updated_at, "2024-01-15T14:45:00Z");

            // Check second changeset
            let second = &data.changesets[1];
            assert_eq!(second.branch, "feature/breaking-change");
            assert_eq!(second.bump, "major");
        }

        #[test]
        fn test_parse_changeset_list_response_empty_list() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changesets": [],
                    "total": 0
                }
            }"#;

            let result = parse_changeset_list_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.count, 0);
            assert!(data.changesets.is_empty());
        }

        #[test]
        fn test_parse_changeset_list_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Workspace not initialized"
            }"#;

            let result = parse_changeset_list_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EEXEC");
            assert!(error.message.contains("Workspace not initialized"));
        }

        #[test]
        fn test_parse_changeset_list_response_empty() {
            let result = parse_changeset_list_response(b"");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_changeset_list_response_whitespace_only() {
            let result = parse_changeset_list_response(b"   \n\t  ");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_changeset_list_response_invalid_json() {
            let result = parse_changeset_list_response(b"not valid json");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse CLI JSON response"));
        }

        #[test]
        fn test_parse_changeset_list_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_changeset_list_response(&invalid_utf8);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_changeset_list_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_changeset_list_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("success but no data"));
        }

        #[test]
        fn test_parse_changeset_list_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_changeset_list_response(json.as_bytes());
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
        fn test_convert_list_item_to_napi() {
            let cli_item = CliChangesetListItem {
                branch: "feature/test".to_string(),
                bump: "minor".to_string(),
                packages: vec!["@scope/pkg1".to_string(), "@scope/pkg2".to_string()],
                environments: vec!["staging".to_string()],
                commit_count: 5,
                created_at: "2024-01-15T10:00:00Z".to_string(),
                updated_at: "2024-01-15T12:00:00Z".to_string(),
            };

            let result = convert_list_item_to_napi(cli_item);

            assert_eq!(result.id, "feature/test");
            assert_eq!(result.branch, "feature/test");
            assert_eq!(result.bump, "minor");
            assert_eq!(result.packages, vec!["@scope/pkg1", "@scope/pkg2"]);
            assert_eq!(result.environments, vec!["staging"]);
            assert_eq!(result.commit_count, 5);
            assert_eq!(result.created_at, "2024-01-15T10:00:00Z");
            assert_eq!(result.updated_at, "2024-01-15T12:00:00Z");
        }

        #[test]
        fn test_convert_to_napi_list_data() {
            let cli_data = CliChangesetListResponseData {
                success: true,
                changesets: vec![
                    CliChangesetListItem {
                        branch: "feature/a".to_string(),
                        bump: "patch".to_string(),
                        packages: vec!["pkg-a".to_string()],
                        environments: vec![],
                        commit_count: 1,
                        created_at: "2024-01-15T10:00:00Z".to_string(),
                        updated_at: "2024-01-15T10:00:00Z".to_string(),
                    },
                    CliChangesetListItem {
                        branch: "feature/b".to_string(),
                        bump: "minor".to_string(),
                        packages: vec!["pkg-b".to_string()],
                        environments: vec!["prod".to_string()],
                        commit_count: 2,
                        created_at: "2024-01-14T09:00:00Z".to_string(),
                        updated_at: "2024-01-14T09:00:00Z".to_string(),
                    },
                ],
                total: 2,
            };

            let result = convert_to_napi_list_data(cli_data);

            assert_eq!(result.count, 2);
            assert_eq!(result.changesets.len(), 2);
            assert_eq!(result.changesets[0].branch, "feature/a");
            assert_eq!(result.changesets[1].branch, "feature/b");
        }

        #[test]
        fn test_convert_to_napi_list_data_empty() {
            let cli_data =
                CliChangesetListResponseData { success: true, changesets: vec![], total: 0 };

            let result = convert_to_napi_list_data(cli_data);

            assert_eq!(result.count, 0);
            assert!(result.changesets.is_empty());
        }

        #[test]
        fn test_convert_list_params_to_args_defaults() {
            let params = ChangesetListParams::new(".");

            let args = convert_list_params_to_args(&params);

            assert!(args.filter_package.is_none());
            assert!(args.filter_bump.is_none());
            assert!(args.filter_env.is_none());
            assert_eq!(args.sort, "date"); // Default value
        }

        #[test]
        fn test_convert_list_params_to_args_with_filters() {
            let params = ChangesetListParams::new(".")
                .with_filter_package("@scope/core")
                .with_filter_bump("major")
                .with_filter_env("production")
                .with_sort("branch");

            let args = convert_list_params_to_args(&params);

            assert_eq!(args.filter_package, Some("@scope/core".to_string()));
            assert_eq!(args.filter_bump, Some("major".to_string()));
            assert_eq!(args.filter_env, Some("production".to_string()));
            assert_eq!(args.sort, "branch");
        }

        #[test]
        fn test_convert_list_params_to_args_custom_sort() {
            let params = ChangesetListParams::new(".").with_sort("bump");

            let args = convert_list_params_to_args(&params);

            assert_eq!(args.sort, "bump");
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;
        use tempfile::TempDir;

        #[test]
        fn test_validate_list_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetListParams::new(temp_dir.path().to_str().unwrap());

            let result = validate_list_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_list_params_nonexistent_path() {
            let params = ChangesetListParams::new("/nonexistent/path/that/does/not/exist");

            let result = validate_list_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_list_params_empty_root() {
            let params = ChangesetListParams::new("");

            let result = validate_list_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("empty"));
        }

        #[test]
        fn test_validate_list_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            std::fs::write(&file_path, "test").unwrap();

            let params = ChangesetListParams::new(file_path.to_str().unwrap());

            let result = validate_list_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("directory"));
        }

        #[test]
        fn test_validate_list_params_valid_bump_filter() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetListParams::new(temp_dir.path().to_str().unwrap())
                .with_filter_bump("minor");

            let result = validate_list_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_list_params_invalid_bump_filter() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetListParams::new(temp_dir.path().to_str().unwrap())
                .with_filter_bump("invalid");

            let result = validate_list_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("invalid bump type"));
        }

        #[test]
        fn test_validate_list_params_all_bump_types() {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path().to_str().unwrap();

            for bump in &["major", "minor", "patch", "none"] {
                let params = ChangesetListParams::new(root).with_filter_bump(*bump);
                let result = validate_list_params(&params);
                assert!(result.is_ok(), "Bump type '{bump}' should be valid");
            }
        }

        #[test]
        fn test_validate_list_params_valid_sort_options() {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path().to_str().unwrap();

            for sort in VALID_SORT_OPTIONS {
                let params = ChangesetListParams::new(root).with_sort(*sort);
                let result = validate_list_params(&params);
                assert!(result.is_ok(), "Sort option '{sort}' should be valid");
            }
        }

        #[test]
        fn test_validate_list_params_invalid_sort_option() {
            let temp_dir = TempDir::new().unwrap();
            let params =
                ChangesetListParams::new(temp_dir.path().to_str().unwrap()).with_sort("invalid");

            let result = validate_list_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("invalid sort option"));
            assert!(error.message.contains("date"));
            assert!(error.message.contains("branch"));
            assert!(error.message.contains("bump"));
        }

        #[test]
        fn test_validate_list_params_with_all_filters() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetListParams::new(temp_dir.path().to_str().unwrap())
                .with_filter_package("@scope/core")
                .with_filter_bump("minor")
                .with_filter_env("staging")
                .with_sort("date");

            let result = validate_list_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_list_params_no_sort_uses_default() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetListParams::new(temp_dir.path().to_str().unwrap());

            let result = validate_list_params(&params);
            assert!(result.is_ok());

            // Verify args conversion uses default
            let args = convert_list_params_to_args(&params);
            assert_eq!(args.sort, "date");
        }
    }
}

// ============================================================================
// Changeset Show Tests (Story 4.5)
// ============================================================================

/// Tests for the `changeset_show` command implementation.
///
/// This module contains tests for:
/// - SharedBuffer functionality (output capture mechanism)
/// - JSON response parsing from CLI output
/// - Parameter conversion from NAPI to CLI types
/// - Parameter validation
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod changeset_show_tests {
    use crate::commands::changeset::{
        CliChangesetShowItem, CliChangesetShowResponseData, SharedBuffer,
        convert_show_item_to_napi, convert_show_params_to_args, convert_to_napi_show_data,
        parse_changeset_show_response, validate_show_params,
    };
    use crate::types::changeset::ChangesetShowParams;
    use std::io::Write;
    use tempfile::TempDir;

    // ------------------------------------------------------------------------
    // SharedBuffer Tests (reused pattern from other command tests)
    // ------------------------------------------------------------------------

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
            let _ = buffer.write(b"test data");
            assert_eq!(buffer.take_bytes(), b"test data");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"first ");
            let _ = buffer.write(b"second");
            assert_eq!(buffer.take_bytes(), b"first second");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();
            let _ = buffer.write(b"shared data");

            // Both should see the same data
            assert_eq!(buffer.take_bytes(), b"shared data");
            assert_eq!(buffer_clone.take_bytes(), b"shared data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"preserved");

            // take_bytes should clone, not drain
            let first = buffer.take_bytes();
            let second = buffer.take_bytes();
            assert_eq!(first, second);
        }
    }

    // ------------------------------------------------------------------------
    // Parse Response Tests
    // ------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_changeset_show_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changeset": {
                        "branch": "feature/new-api",
                        "bump": "minor",
                        "packages": ["@scope/core", "@scope/utils"],
                        "environments": ["staging", "production"],
                        "commits": ["abc123", "def456"],
                        "created_at": "2024-01-15T10:30:00Z",
                        "updated_at": "2024-01-15T14:45:00Z"
                    }
                }
            }"#;

            let result = parse_changeset_show_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.changeset.branch, "feature/new-api");
            assert_eq!(data.changeset.bump, "minor");
            assert_eq!(data.changeset.packages.len(), 2);
            assert_eq!(data.changeset.packages[0], "@scope/core");
            assert_eq!(data.changeset.packages[1], "@scope/utils");
            assert_eq!(data.changeset.environments.len(), 2);
            assert_eq!(data.changeset.environments[0], "staging");
            assert_eq!(data.changeset.environments[1], "production");
            assert_eq!(data.changeset.commits.len(), 2);
            assert_eq!(data.changeset.commits[0], "abc123");
            assert_eq!(data.changeset.created_at, "2024-01-15T10:30:00Z");
            assert_eq!(data.changeset.updated_at, "2024-01-15T14:45:00Z");
        }

        #[test]
        fn test_parse_changeset_show_response_minimal() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changeset": {
                        "branch": "main",
                        "bump": "patch",
                        "packages": [],
                        "environments": [],
                        "commits": [],
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z"
                    }
                }
            }"#;

            let result = parse_changeset_show_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.changeset.branch, "main");
            assert_eq!(data.changeset.bump, "patch");
            assert!(data.changeset.packages.is_empty());
            assert!(data.changeset.environments.is_empty());
            assert!(data.changeset.commits.is_empty());
        }

        #[test]
        fn test_parse_changeset_show_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Changeset 'nonexistent' not found"
            }"#;

            let result = parse_changeset_show_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("not found"));
        }

        #[test]
        fn test_parse_changeset_show_response_empty() {
            let result = parse_changeset_show_response(b"");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Empty response"));
        }

        #[test]
        fn test_parse_changeset_show_response_whitespace_only() {
            let result = parse_changeset_show_response(b"   \n\t  ");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Empty response"));
        }

        #[test]
        fn test_parse_changeset_show_response_invalid_json() {
            let result = parse_changeset_show_response(b"not valid json");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_changeset_show_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_changeset_show_response(&invalid_utf8);
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_changeset_show_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_changeset_show_response(json.as_bytes());
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("no data"));
        }

        #[test]
        fn test_parse_changeset_show_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_changeset_show_response(json.as_bytes());
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Unknown CLI error"));
        }
    }

    // ------------------------------------------------------------------------
    // Conversion Tests
    // ------------------------------------------------------------------------

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_convert_show_item_to_napi() {
            let item = CliChangesetShowItem {
                branch: "feature/test".to_string(),
                bump: "minor".to_string(),
                packages: vec!["@scope/pkg1".to_string()],
                environments: vec!["staging".to_string()],
                commits: vec!["abc123".to_string()],
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T14:45:00Z".to_string(),
            };

            let result = convert_show_item_to_napi(item);

            assert_eq!(result.id, "feature/test");
            assert_eq!(result.branch, "feature/test");
            assert_eq!(result.bump, "minor");
            assert_eq!(result.packages, vec!["@scope/pkg1"]);
            assert_eq!(result.environments, vec!["staging"]);
            assert_eq!(result.commits, vec!["abc123"]);
            assert_eq!(result.created_at, "2024-01-15T10:30:00Z");
            assert_eq!(result.updated_at, "2024-01-15T14:45:00Z");
        }

        #[test]
        fn test_convert_to_napi_show_data() {
            let cli_data = CliChangesetShowResponseData {
                success: true,
                changeset: CliChangesetShowItem {
                    branch: "feature/api".to_string(),
                    bump: "major".to_string(),
                    packages: vec!["@scope/api".to_string(), "@scope/client".to_string()],
                    environments: vec!["production".to_string()],
                    commits: vec!["commit1".to_string(), "commit2".to_string()],
                    created_at: "2024-02-01T08:00:00Z".to_string(),
                    updated_at: "2024-02-01T12:00:00Z".to_string(),
                },
            };

            let result = convert_to_napi_show_data(cli_data);

            assert_eq!(result.changeset.branch, "feature/api");
            assert_eq!(result.changeset.bump, "major");
            assert_eq!(result.changeset.packages.len(), 2);
            assert_eq!(result.changeset.environments.len(), 1);
            assert_eq!(result.changeset.commits.len(), 2);
        }

        #[test]
        fn test_convert_show_params_to_args() {
            let params = ChangesetShowParams::new(".", "feature/new-api");
            let args = convert_show_params_to_args(&params);

            assert_eq!(args.branch, "feature/new-api");
        }

        #[test]
        fn test_convert_show_params_to_args_complex_branch() {
            let params = ChangesetShowParams::new(".", "feature/auth/oauth-integration");
            let args = convert_show_params_to_args(&params);

            assert_eq!(args.branch, "feature/auth/oauth-integration");
        }
    }

    // ------------------------------------------------------------------------
    // Validation Tests
    // ------------------------------------------------------------------------

    mod validation_tests {
        use super::*;
        use std::fs::File;

        #[test]
        fn test_validate_show_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let params =
                ChangesetShowParams::new(temp_dir.path().to_str().unwrap(), "feature/test");

            let result = validate_show_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_show_params_nonexistent_path() {
            let params = ChangesetShowParams::new("/nonexistent/path/12345", "feature/test");

            let result = validate_show_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_show_params_empty_root() {
            let params = ChangesetShowParams::new("", "feature/test");

            let result = validate_show_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("root"));
        }

        #[test]
        fn test_validate_show_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test_file.txt");
            let _ = File::create(&file_path).unwrap();

            let params = ChangesetShowParams::new(file_path.to_str().unwrap(), "feature/test");

            let result = validate_show_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("directory"));
        }

        #[test]
        fn test_validate_show_params_empty_branch() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetShowParams::new(temp_dir.path().to_str().unwrap(), "");

            let result = validate_show_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("branch"));
        }

        #[test]
        fn test_validate_show_params_whitespace_branch() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetShowParams::new(temp_dir.path().to_str().unwrap(), "   ");

            let result = validate_show_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("branch"));
        }

        #[test]
        fn test_validate_show_params_with_config_path() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetShowParams::new(temp_dir.path().to_str().unwrap(), "feature/api")
                .with_config_path("/path/to/config.json");

            let result = validate_show_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_show_params_various_branch_names() {
            let temp_dir = TempDir::new().unwrap();

            // Feature branch
            let params =
                ChangesetShowParams::new(temp_dir.path().to_str().unwrap(), "feature/new-api");
            assert!(validate_show_params(&params).is_ok());

            // Hotfix branch
            let params = ChangesetShowParams::new(
                temp_dir.path().to_str().unwrap(),
                "hotfix/security-patch",
            );
            assert!(validate_show_params(&params).is_ok());

            // Simple branch
            let params = ChangesetShowParams::new(temp_dir.path().to_str().unwrap(), "main");
            assert!(validate_show_params(&params).is_ok());

            // Branch with multiple slashes
            let params = ChangesetShowParams::new(
                temp_dir.path().to_str().unwrap(),
                "feature/auth/oauth-integration",
            );
            assert!(validate_show_params(&params).is_ok());

            // Branch with numbers
            let params =
                ChangesetShowParams::new(temp_dir.path().to_str().unwrap(), "release/v2.0.0");
            assert!(validate_show_params(&params).is_ok());
        }
    }
}

// ============================================================================
// Changeset Remove Command Tests (Story 4.6)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod changeset_remove_tests {
    use std::io::Write;

    use tempfile::TempDir;

    use crate::commands::changeset::{
        CliChangesetRemoveResponseData, CliRemovedChangesetInfo, SharedBuffer,
        convert_remove_params_to_args, convert_to_napi_remove_data,
        parse_changeset_remove_response, validate_remove_params,
    };
    use crate::types::changeset::ChangesetRemoveParams;

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
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();
            buffer.write_all(b"test data").unwrap();

            // Both should see the same data
            assert_eq!(buffer.take_bytes(), b"test data");
            assert_eq!(buffer_clone.take_bytes(), b"test data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"preserved").unwrap();

            let first = buffer.take_bytes();
            let second = buffer.take_bytes();

            assert_eq!(first, second);
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_changeset_remove_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "branch": "feature/test",
                    "archived": true,
                    "changeset": {
                        "branch": "feature/test",
                        "bump": "minor",
                        "packages": ["@scope/pkg1", "@scope/pkg2"],
                        "environments": ["development", "production"],
                        "commit_count": 5
                    }
                }
            }"#;

            let result = parse_changeset_remove_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert!(data.removed);
            assert_eq!(data.branch, "feature/test");
        }

        #[test]
        fn test_parse_changeset_remove_response_minimal() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "branch": "fix/bug",
                    "archived": false,
                    "changeset": {
                        "branch": "fix/bug",
                        "bump": "patch",
                        "packages": [],
                        "environments": [],
                        "commit_count": 0
                    }
                }
            }"#;

            let result = parse_changeset_remove_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert!(data.removed);
            assert_eq!(data.branch, "fix/bug");
        }

        #[test]
        fn test_parse_changeset_remove_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Changeset not found: nonexistent"
            }"#;

            let result = parse_changeset_remove_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Changeset not found"));
        }

        #[test]
        fn test_parse_changeset_remove_response_empty() {
            let result = parse_changeset_remove_response(b"");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("empty"));
        }

        #[test]
        fn test_parse_changeset_remove_response_whitespace_only() {
            let result = parse_changeset_remove_response(b"   \n\t  ");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("empty"));
        }

        #[test]
        fn test_parse_changeset_remove_response_invalid_json() {
            let result = parse_changeset_remove_response(b"not valid json");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("parse"));
        }

        #[test]
        fn test_parse_changeset_remove_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_changeset_remove_response(&invalid_utf8);
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("UTF-8"));
        }

        #[test]
        fn test_parse_changeset_remove_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_changeset_remove_response(json.as_bytes());
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("no data"));
        }

        #[test]
        fn test_parse_changeset_remove_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_changeset_remove_response(json.as_bytes());
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Unknown CLI error"));
        }
    }

    // -------------------------------------------------------------------------
    // Conversion Tests
    // -------------------------------------------------------------------------

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_convert_to_napi_remove_data() {
            let cli_data = CliChangesetRemoveResponseData {
                success: true,
                branch: "feature/test".to_string(),
                archived: true,
                changeset: CliRemovedChangesetInfo {
                    branch: "feature/test".to_string(),
                    bump: "minor".to_string(),
                    packages: vec!["@scope/pkg1".to_string()],
                    environments: vec!["production".to_string()],
                    commit_count: 3,
                },
            };

            let result = convert_to_napi_remove_data(&cli_data);

            assert!(result.removed);
            assert_eq!(result.branch, "feature/test");
        }

        #[test]
        fn test_convert_remove_params_to_args_basic() {
            let params = ChangesetRemoveParams::new(".", "feature/test");

            let args = convert_remove_params_to_args(&params);

            assert_eq!(args.branch, "feature/test");
            // Force is always true in API mode
            assert!(args.force);
        }

        #[test]
        fn test_convert_remove_params_to_args_force_ignored() {
            // Even if force is explicitly set to false, it should be true in API mode
            let params = ChangesetRemoveParams {
                root: ".".to_string(),
                config_path: None,
                branch: "feature/test".to_string(),
                force: Some(false),
            };

            let args = convert_remove_params_to_args(&params);

            // Force is always true in API mode - no interactive prompts
            assert!(args.force);
        }

        #[test]
        fn test_convert_remove_params_to_args_various_branches() {
            // Simple branch
            let params = ChangesetRemoveParams::new(".", "main");
            assert_eq!(convert_remove_params_to_args(&params).branch, "main");

            // Feature branch
            let params = ChangesetRemoveParams::new(".", "feature/new-api");
            assert_eq!(convert_remove_params_to_args(&params).branch, "feature/new-api");

            // Hotfix branch
            let params = ChangesetRemoveParams::new(".", "hotfix/critical-fix");
            assert_eq!(convert_remove_params_to_args(&params).branch, "hotfix/critical-fix");

            // Branch with multiple slashes
            let params = ChangesetRemoveParams::new(".", "feature/auth/oauth");
            assert_eq!(convert_remove_params_to_args(&params).branch, "feature/auth/oauth");
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;
        use std::fs::File;

        #[test]
        fn test_validate_remove_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let params =
                ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "feature/test");

            let result = validate_remove_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_remove_params_nonexistent_path() {
            let params = ChangesetRemoveParams::new("/nonexistent/path/12345", "feature/test");

            let result = validate_remove_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_remove_params_empty_root() {
            let params = ChangesetRemoveParams::new("", "feature/test");

            let result = validate_remove_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("root"));
        }

        #[test]
        fn test_validate_remove_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test_file.txt");
            let _ = File::create(&file_path).unwrap();

            let params = ChangesetRemoveParams::new(file_path.to_str().unwrap(), "feature/test");

            let result = validate_remove_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("directory"));
        }

        #[test]
        fn test_validate_remove_params_empty_branch() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "");

            let result = validate_remove_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("branch"));
        }

        #[test]
        fn test_validate_remove_params_whitespace_branch() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "   ");

            let result = validate_remove_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("branch"));
        }

        #[test]
        fn test_validate_remove_params_with_config_path() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetRemoveParams {
                root: temp_dir.path().to_str().unwrap().to_string(),
                config_path: Some("/path/to/config.json".to_string()),
                branch: "feature/test".to_string(),
                force: None,
            };

            let result = validate_remove_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_remove_params_various_branch_names() {
            let temp_dir = TempDir::new().unwrap();

            // Feature branch
            let params =
                ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "feature/new-api");
            assert!(validate_remove_params(&params).is_ok());

            // Hotfix branch
            let params = ChangesetRemoveParams::new(
                temp_dir.path().to_str().unwrap(),
                "hotfix/security-patch",
            );
            assert!(validate_remove_params(&params).is_ok());

            // Simple branch
            let params = ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "main");
            assert!(validate_remove_params(&params).is_ok());

            // Branch with multiple slashes
            let params = ChangesetRemoveParams::new(
                temp_dir.path().to_str().unwrap(),
                "feature/auth/oauth-integration",
            );
            assert!(validate_remove_params(&params).is_ok());

            // Branch with numbers
            let params =
                ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "release/v2.0.0");
            assert!(validate_remove_params(&params).is_ok());

            // Branch with underscores and dashes
            let params = ChangesetRemoveParams::new(
                temp_dir.path().to_str().unwrap(),
                "feature/my_feature-branch",
            );
            assert!(validate_remove_params(&params).is_ok());
        }

        #[test]
        fn test_validate_remove_params_with_force_flag() {
            let temp_dir = TempDir::new().unwrap();

            // With force = true
            let params =
                ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "feature/test")
                    .with_force(true);
            assert!(validate_remove_params(&params).is_ok());

            // With force = false (should still validate, force is only used at execution)
            let params =
                ChangesetRemoveParams::new(temp_dir.path().to_str().unwrap(), "feature/test")
                    .with_force(false);
            assert!(validate_remove_params(&params).is_ok());
        }
    }
}

// ============================================================================
// Changeset History Tests
// ============================================================================

/// Tests for the changeset history command implementation.
///
/// This module covers:
/// - SharedBuffer functionality for output capture
/// - Response parsing from CLI JSON output
/// - Conversion of CLI types to NAPI types
/// - Parameter validation
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod changeset_history_tests {
    use std::collections::HashMap;

    use tempfile::TempDir;

    use crate::commands::changeset::{
        CliArchivedChangesetInfo, CliChangesetHistoryResponseData, SharedBuffer,
        convert_archived_changeset_to_napi, convert_history_params_to_args,
        convert_to_napi_history_data, parse_changeset_history_response, validate_history_params,
    };
    use crate::types::changeset::ChangesetHistoryParams;

    // ========================================================================
    // SharedBuffer Tests
    // ========================================================================

    mod shared_buffer_tests {
        use super::*;
        use std::io::Write;

        #[test]
        fn test_shared_buffer_new() {
            let buffer = SharedBuffer::new();
            assert!(buffer.take_bytes().is_empty());
        }

        #[test]
        fn test_shared_buffer_write() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"test data");
            assert_eq!(buffer.take_bytes(), b"test data");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"first ");
            let _ = buffer.write(b"second");
            assert_eq!(buffer.take_bytes(), b"first second");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let clone = buffer.clone();
            let _ = buffer.write(b"shared data");
            // Clone should see the same data
            assert_eq!(clone.take_bytes(), b"shared data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"preserved");
            let first = buffer.take_bytes();
            let second = buffer.take_bytes();
            assert_eq!(first, second);
        }
    }

    // ========================================================================
    // Parse Response Tests
    // ========================================================================

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_changeset_history_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changesets": [
                        {
                            "branch": "feature/add-api",
                            "bump": "minor",
                            "packages": ["@scope/core"],
                            "environments": ["production"],
                            "commits": ["abc123", "def456"],
                            "created_at": "2024-01-15T10:30:00Z",
                            "updated_at": "2024-01-15T14:45:00Z",
                            "versions": {"@scope/core": "2.0.0"},
                            "git_commit": "release123",
                            "applied_at": "2024-01-16T10:00:00Z",
                            "applied_by": "CI"
                        }
                    ],
                    "total": 1
                }
            }"#;

            let result = parse_changeset_history_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.count, 1);
            assert_eq!(data.archived.len(), 1);

            let archived = &data.archived[0];
            assert_eq!(archived.changeset.branch, "feature/add-api");
            assert_eq!(archived.changeset.bump, "minor");
            assert_eq!(archived.changeset.packages, vec!["@scope/core"]);
            assert_eq!(archived.changeset.environments, vec!["production"]);
            assert_eq!(archived.changeset.commits, vec!["abc123", "def456"]);
            assert_eq!(archived.release_info.released_by, "CI");
            assert_eq!(archived.release_info.release_commit, "release123");
            assert_eq!(archived.release_info.released_versions.len(), 1);
            assert_eq!(archived.release_info.released_versions[0].package_name, "@scope/core");
            assert_eq!(archived.release_info.released_versions[0].version, "2.0.0");
        }

        #[test]
        fn test_parse_changeset_history_response_empty_list() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changesets": [],
                    "total": 0
                }
            }"#;

            let result = parse_changeset_history_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.count, 0);
            assert!(data.archived.is_empty());
        }

        #[test]
        fn test_parse_changeset_history_response_multiple_items() {
            let json = r#"{
                "success": true,
                "data": {
                    "success": true,
                    "changesets": [
                        {
                            "branch": "feature/first",
                            "bump": "major",
                            "packages": ["@scope/pkg1"],
                            "environments": [],
                            "commits": ["abc"],
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z",
                            "versions": {"@scope/pkg1": "1.0.0"},
                            "git_commit": "commit1",
                            "applied_at": "2024-01-02T00:00:00Z",
                            "applied_by": "user1"
                        },
                        {
                            "branch": "feature/second",
                            "bump": "patch",
                            "packages": ["@scope/pkg2"],
                            "environments": ["staging"],
                            "commits": ["def"],
                            "created_at": "2024-02-01T00:00:00Z",
                            "updated_at": "2024-02-01T00:00:00Z",
                            "versions": {"@scope/pkg2": "1.0.1"},
                            "git_commit": "commit2",
                            "applied_at": "2024-02-02T00:00:00Z",
                            "applied_by": "user2"
                        }
                    ],
                    "total": 2
                }
            }"#;

            let result = parse_changeset_history_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.count, 2);
            assert_eq!(data.archived.len(), 2);

            assert_eq!(data.archived[0].changeset.branch, "feature/first");
            assert_eq!(data.archived[1].changeset.branch, "feature/second");
        }

        #[test]
        fn test_parse_changeset_history_response_cli_error() {
            let json = r#"{"success": false, "error": "Workspace not initialized"}"#;
            let result = parse_changeset_history_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Workspace not initialized"));
        }

        #[test]
        fn test_parse_changeset_history_response_empty() {
            let result = parse_changeset_history_response(b"");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("empty output"));
        }

        #[test]
        fn test_parse_changeset_history_response_whitespace_only() {
            let result = parse_changeset_history_response(b"   \n\t  ");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("empty output"));
        }

        #[test]
        fn test_parse_changeset_history_response_invalid_json() {
            let result = parse_changeset_history_response(b"not json");
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_changeset_history_response_invalid_utf8() {
            let invalid_utf8 = vec![0xFF, 0xFE, 0x00, 0x01];
            let result = parse_changeset_history_response(&invalid_utf8);
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_changeset_history_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_changeset_history_response(json.as_bytes());
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("success but no data"));
        }

        #[test]
        fn test_parse_changeset_history_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_changeset_history_response(json.as_bytes());
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Unknown CLI error"));
        }
    }

    // ========================================================================
    // Conversion Tests
    // ========================================================================

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_convert_archived_changeset_to_napi() {
            let mut versions = HashMap::new();
            versions.insert("@scope/core".to_string(), "2.0.0".to_string());
            versions.insert("@scope/utils".to_string(), "1.5.0".to_string());

            let cli = CliArchivedChangesetInfo {
                branch: "feature/add-api".to_string(),
                bump: "minor".to_string(),
                packages: vec!["@scope/core".to_string(), "@scope/utils".to_string()],
                environments: vec!["production".to_string()],
                commits: vec!["abc123".to_string()],
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T14:45:00Z".to_string(),
                versions,
                git_commit: "release123".to_string(),
                applied_at: "2024-01-16T10:00:00Z".to_string(),
                applied_by: "CI".to_string(),
            };

            let result = convert_archived_changeset_to_napi(cli);

            // Verify changeset details
            assert_eq!(result.changeset.id, "feature/add-api");
            assert_eq!(result.changeset.branch, "feature/add-api");
            assert_eq!(result.changeset.bump, "minor");
            assert_eq!(result.changeset.packages, vec!["@scope/core", "@scope/utils"]);
            assert_eq!(result.changeset.environments, vec!["production"]);
            assert_eq!(result.changeset.commits, vec!["abc123"]);
            assert_eq!(result.changeset.created_at, "2024-01-15T10:30:00Z");
            assert_eq!(result.changeset.updated_at, "2024-01-15T14:45:00Z");

            // Verify release info
            assert_eq!(result.release_info.released_at, "2024-01-16T10:00:00Z");
            assert_eq!(result.release_info.released_by, "CI");
            assert_eq!(result.release_info.release_commit, "release123");
            assert_eq!(result.release_info.released_versions.len(), 2);
        }

        #[test]
        fn test_convert_archived_changeset_empty_fields() {
            let cli = CliArchivedChangesetInfo {
                branch: "empty-branch".to_string(),
                bump: "patch".to_string(),
                packages: vec![],
                environments: vec![],
                commits: vec![],
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                versions: HashMap::new(),
                git_commit: "commit".to_string(),
                applied_at: "2024-01-02T00:00:00Z".to_string(),
                applied_by: "system".to_string(),
            };

            let result = convert_archived_changeset_to_napi(cli);

            assert!(result.changeset.packages.is_empty());
            assert!(result.changeset.environments.is_empty());
            assert!(result.changeset.commits.is_empty());
            assert!(result.release_info.released_versions.is_empty());
        }

        #[test]
        fn test_convert_to_napi_history_data() {
            let cli_data = CliChangesetHistoryResponseData {
                success: true,
                changesets: vec![CliArchivedChangesetInfo {
                    branch: "test-branch".to_string(),
                    bump: "major".to_string(),
                    packages: vec!["pkg1".to_string()],
                    environments: vec![],
                    commits: vec![],
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    updated_at: "2024-01-01T00:00:00Z".to_string(),
                    versions: HashMap::new(),
                    git_commit: "abc".to_string(),
                    applied_at: "2024-01-02T00:00:00Z".to_string(),
                    applied_by: "user".to_string(),
                }],
                total: 1,
            };

            let result = convert_to_napi_history_data(cli_data);

            assert_eq!(result.count, 1);
            assert_eq!(result.archived.len(), 1);
            assert_eq!(result.archived[0].changeset.branch, "test-branch");
        }

        #[test]
        fn test_convert_to_napi_history_data_empty() {
            let cli_data =
                CliChangesetHistoryResponseData { success: true, changesets: vec![], total: 0 };

            let result = convert_to_napi_history_data(cli_data);

            assert_eq!(result.count, 0);
            assert!(result.archived.is_empty());
        }

        #[test]
        fn test_convert_history_params_to_args_defaults() {
            let params = ChangesetHistoryParams::new(".");
            let args = convert_history_params_to_args(&params);

            assert!(args.filter_package.is_none());
            assert!(args.filter_env.is_none());
            assert!(args.filter_bump.is_none());
            assert!(args.since.is_none());
            assert!(args.until.is_none());
            assert!(args.limit.is_none());
        }

        #[test]
        fn test_convert_history_params_to_args_all_filters() {
            let params = ChangesetHistoryParams::new(".")
                .with_filter_package("@scope/core")
                .with_filter_env("production")
                .with_filter_bump("major")
                .with_since("2024-01-01")
                .with_until("2024-12-31")
                .with_limit(10);
            let args = convert_history_params_to_args(&params);

            assert_eq!(args.filter_package, Some("@scope/core".to_string()));
            assert_eq!(args.filter_env, Some("production".to_string()));
            assert_eq!(args.filter_bump, Some("major".to_string()));
            assert_eq!(args.since, Some("2024-01-01".to_string()));
            assert_eq!(args.until, Some("2024-12-31".to_string()));
            assert_eq!(args.limit, Some(10));
        }

        #[test]
        fn test_convert_history_params_limit_conversion() {
            // Test u32 to usize conversion
            let params = ChangesetHistoryParams::new(".").with_limit(100);
            let args = convert_history_params_to_args(&params);
            assert_eq!(args.limit, Some(100));
        }
    }

    // ========================================================================
    // Validation Tests
    // ========================================================================

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_history_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap());
            let result = validate_history_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_history_params_nonexistent_path() {
            let params = ChangesetHistoryParams::new("/nonexistent/path/xyz");
            let result = validate_history_params(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_history_params_empty_root() {
            let params = ChangesetHistoryParams::new("");
            let result = validate_history_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty"));
        }

        #[test]
        fn test_validate_history_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            std::fs::write(&file_path, "test").unwrap();

            let params = ChangesetHistoryParams::new(file_path.to_str().unwrap());
            let result = validate_history_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("directory"));
        }

        #[test]
        fn test_validate_history_params_valid_bump_types() {
            let temp_dir = TempDir::new().unwrap();

            for bump_type in ["major", "minor", "patch", "none"] {
                let params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap())
                    .with_filter_bump(bump_type);
                assert!(
                    validate_history_params(&params).is_ok(),
                    "Expected {bump_type} to be valid"
                );
            }
        }

        #[test]
        fn test_validate_history_params_invalid_bump_type() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap())
                .with_filter_bump("invalid");
            let result = validate_history_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("invalid"));
        }

        #[test]
        fn test_validate_history_params_with_all_filters() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap())
                .with_filter_package("@scope/core")
                .with_filter_env("production")
                .with_filter_bump("major")
                .with_since("2024-01-01")
                .with_until("2024-12-31")
                .with_limit(50);
            assert!(validate_history_params(&params).is_ok());
        }

        #[test]
        fn test_validate_history_params_with_config_path() {
            let temp_dir = TempDir::new().unwrap();
            let mut params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap());
            params.config_path = Some("/path/to/config.json".to_string());
            // Config path validation happens at execution time, not validation time
            assert!(validate_history_params(&params).is_ok());
        }

        #[test]
        fn test_validate_history_params_date_filters() {
            let temp_dir = TempDir::new().unwrap();

            // Only since date
            let params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap())
                .with_since("2024-01-01");
            assert!(validate_history_params(&params).is_ok());

            // Only until date
            let params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap())
                .with_until("2024-12-31");
            assert!(validate_history_params(&params).is_ok());

            // Both dates
            let params = ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap())
                .with_since("2024-01-01")
                .with_until("2024-12-31");
            assert!(validate_history_params(&params).is_ok());
        }

        #[test]
        fn test_validate_history_params_limit_values() {
            let temp_dir = TempDir::new().unwrap();

            // Zero limit is valid (means no results, which is valid)
            let params =
                ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap()).with_limit(0);
            assert!(validate_history_params(&params).is_ok());

            // Large limit is valid
            let params =
                ChangesetHistoryParams::new(temp_dir.path().to_str().unwrap()).with_limit(1000);
            assert!(validate_history_params(&params).is_ok());
        }
    }
}

// ============================================================================
// Changeset Check Command Tests (Story 4.8)
// ============================================================================

/// Tests for the changeset check command implementation.
///
/// This module covers:
/// - SharedBuffer functionality for output capture
/// - Response parsing from CLI JSON output
/// - Conversion of CLI types to NAPI types
/// - Parameter validation
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod changeset_check_tests {
    use std::io::Write;

    use tempfile::TempDir;

    use crate::commands::changeset::{
        CliChangesetCheckResponseData, SharedBuffer, convert_check_params_to_args,
        convert_to_napi_check_data, parse_changeset_check_response, validate_check_params,
    };
    use crate::types::changeset::ChangesetCheckParams;

    // ========================================================================
    // SharedBuffer Tests
    // ========================================================================

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
            let _ = buffer.write(b"test data");
            assert_eq!(buffer.take_bytes(), b"test data");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"first ");
            let _ = buffer.write(b"second");
            assert_eq!(buffer.take_bytes(), b"first second");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();
            let _ = buffer.write(b"shared data");
            assert_eq!(buffer_clone.take_bytes(), b"shared data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            let _ = buffer.write(b"preserved");
            let first = buffer.take_bytes();
            let second = buffer.take_bytes();
            assert_eq!(first, second);
        }
    }

    // ========================================================================
    // Parse Response Tests
    // ========================================================================

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_changeset_check_response_exists() {
            let json = r#"{
                "success": true,
                "data": {
                    "exists": true,
                    "branch": "feature/new-api",
                    "message": "Changeset exists for branch 'feature/new-api'"
                }
            }"#;

            let result = parse_changeset_check_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert!(data.has_changeset);
            assert_eq!(data.branch, Some("feature/new-api".to_string()));
        }

        #[test]
        fn test_parse_changeset_check_response_not_exists() {
            let json = r#"{
                "success": true,
                "data": {
                    "exists": false,
                    "branch": "feature/no-changeset",
                    "message": "No changeset found for branch 'feature/no-changeset'"
                }
            }"#;

            let result = parse_changeset_check_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert!(!data.has_changeset);
            assert!(data.branch.is_none());
        }

        #[test]
        fn test_parse_changeset_check_response_minimal() {
            let json = r#"{
                "success": true,
                "data": {
                    "exists": true,
                    "branch": "main"
                }
            }"#;

            let result = parse_changeset_check_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert!(data.has_changeset);
            assert_eq!(data.branch, Some("main".to_string()));
        }

        #[test]
        fn test_parse_changeset_check_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Workspace not initialized"
            }"#;

            let result = parse_changeset_check_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Workspace not initialized"));
        }

        #[test]
        fn test_parse_changeset_check_response_empty() {
            let result = parse_changeset_check_response(b"");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Empty"));
        }

        #[test]
        fn test_parse_changeset_check_response_whitespace_only() {
            let result = parse_changeset_check_response(b"   \n\t  ");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Empty"));
        }

        #[test]
        fn test_parse_changeset_check_response_invalid_json() {
            let result = parse_changeset_check_response(b"not valid json");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_changeset_check_response_invalid_utf8() {
            let invalid_utf8 = vec![0xFF, 0xFE, 0x00, 0x01];
            let result = parse_changeset_check_response(&invalid_utf8);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_changeset_check_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_changeset_check_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("no data"));
        }

        #[test]
        fn test_parse_changeset_check_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_changeset_check_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Unknown CLI error"));
        }
    }

    // ========================================================================
    // Conversion Tests
    // ========================================================================

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_convert_to_napi_check_data_exists() {
            let cli_data = CliChangesetCheckResponseData {
                exists: true,
                branch: "feature/api".to_string(),
                message: Some("Changeset exists".to_string()),
            };

            let napi_data = convert_to_napi_check_data(cli_data);
            assert!(napi_data.has_changeset);
            assert_eq!(napi_data.branch, Some("feature/api".to_string()));
        }

        #[test]
        fn test_convert_to_napi_check_data_not_exists() {
            let cli_data = CliChangesetCheckResponseData {
                exists: false,
                branch: "feature/no-changeset".to_string(),
                message: Some("No changeset found".to_string()),
            };

            let napi_data = convert_to_napi_check_data(cli_data);
            assert!(!napi_data.has_changeset);
            assert!(napi_data.branch.is_none());
        }

        #[test]
        fn test_convert_to_napi_check_data_without_message() {
            let cli_data = CliChangesetCheckResponseData {
                exists: true,
                branch: "main".to_string(),
                message: None,
            };

            let napi_data = convert_to_napi_check_data(cli_data);
            assert!(napi_data.has_changeset);
            assert_eq!(napi_data.branch, Some("main".to_string()));
        }

        #[test]
        fn test_convert_check_params_to_args_no_branch() {
            let params = ChangesetCheckParams::new("/path/to/workspace");
            let args = convert_check_params_to_args(&params);
            assert!(args.branch.is_none());
        }

        #[test]
        fn test_convert_check_params_to_args_with_branch() {
            let params =
                ChangesetCheckParams::new("/path/to/workspace").with_branch("feature/new-api");
            let args = convert_check_params_to_args(&params);
            assert_eq!(args.branch, Some("feature/new-api".to_string()));
        }

        #[test]
        fn test_convert_check_params_to_args_various_branches() {
            let test_cases = [
                "main",
                "develop",
                "feature/auth",
                "release/v1.0.0",
                "hotfix/critical-bug",
                "user/john/experiment",
            ];

            for branch in test_cases {
                let params = ChangesetCheckParams::new("/root").with_branch(branch);
                let args = convert_check_params_to_args(&params);
                assert_eq!(args.branch, Some(branch.to_string()));
            }
        }
    }

    // ========================================================================
    // Validation Tests
    // ========================================================================

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_check_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetCheckParams::new(temp_dir.path().to_str().unwrap());
            let result = validate_check_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_check_params_nonexistent_path() {
            let params = ChangesetCheckParams::new("/nonexistent/path/xyz");
            let result = validate_check_params(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_check_params_empty_root() {
            let params = ChangesetCheckParams::new("");
            let result = validate_check_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty"));
        }

        #[test]
        fn test_validate_check_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            std::fs::write(&file_path, "test").unwrap();

            let params = ChangesetCheckParams::new(file_path.to_str().unwrap());
            let result = validate_check_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("directory"));
        }

        #[test]
        fn test_validate_check_params_with_branch() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetCheckParams::new(temp_dir.path().to_str().unwrap())
                .with_branch("feature/new-api");
            let result = validate_check_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_check_params_with_config_path() {
            let temp_dir = TempDir::new().unwrap();
            let mut params = ChangesetCheckParams::new(temp_dir.path().to_str().unwrap());
            params.config_path = Some("/path/to/config.json".to_string());
            // Config path validation happens at execution time, not validation time
            let result = validate_check_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_check_params_various_branch_names() {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path().to_str().unwrap();

            let valid_branches = [
                "main",
                "develop",
                "feature/new-api",
                "release/v1.0.0",
                "hotfix/critical-fix",
                "user/john/experiment",
                "UPPERCASE",
                "with-dashes",
                "with_underscores",
                "with.dots",
            ];

            for branch in valid_branches {
                let params = ChangesetCheckParams::new(root).with_branch(branch);
                assert!(
                    validate_check_params(&params).is_ok(),
                    "Expected branch '{branch}' to be valid"
                );
            }
        }

        #[test]
        fn test_validate_check_params_no_branch_uses_current() {
            let temp_dir = TempDir::new().unwrap();
            let params = ChangesetCheckParams::new(temp_dir.path().to_str().unwrap());
            // Branch is optional - when not provided, CLI will use current Git branch
            let result = validate_check_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_check_params_returns_correct_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ChangesetCheckParams::new(path_str);
            let result = validate_check_params(&params);
            assert!(result.is_ok());
            let path = result.unwrap();
            assert_eq!(path.to_str().unwrap(), path_str);
        }
    }
}

// ============================================================================
// Bump Preview Command Tests (Story 5.2)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod bump_preview_tests {
    use std::io::Write;

    use tempfile::TempDir;

    use crate::commands::bump::{
        CliBumpSnapshot, CliBumpSummary, CliChangesetInfo, CliPackageBumpInfo, SharedBuffer,
        convert_params_to_args, convert_to_napi_preview, parse_preview_response,
        validate_preview_params,
    };
    use crate::types::bump::BumpPreviewParams;

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
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();
            buffer.write_all(b"shared data").unwrap();

            // Both buffers should see the same data
            assert_eq!(buffer.take_bytes(), b"shared data");
            assert_eq!(buffer_clone.take_bytes(), b"shared data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"test data").unwrap();

            // Multiple takes should return same data
            let first_take = buffer.take_bytes();
            let second_take = buffer.take_bytes();
            assert_eq!(first_take, second_take);
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_preview_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Independent",
                    "packages": [
                        {
                            "name": "@scope/core",
                            "path": "packages/core",
                            "currentVersion": "1.0.0",
                            "nextVersion": "1.1.0",
                            "bumpType": "Minor",
                            "willBump": true,
                            "reason": "direct change from changeset"
                        }
                    ],
                    "changesets": [
                        {
                            "id": "feature-new-api",
                            "branch": "feature/new-api",
                            "bumpType": "Minor",
                            "packages": ["@scope/core"],
                            "commitCount": 5
                        }
                    ],
                    "summary": {
                        "totalPackages": 1,
                        "packagesToBump": 1,
                        "packagesUnchanged": 0,
                        "totalChangesets": 1,
                        "hasCircularDependencies": false
                    }
                }
            }"#;

            let result = parse_preview_response(json.as_bytes());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.strategy, "independent");
            assert_eq!(data.packages.len(), 1);
            assert_eq!(data.packages[0].name, "@scope/core");
            assert_eq!(data.packages[0].current_version, "1.0.0");
            assert_eq!(data.packages[0].next_version, "1.1.0");
            assert_eq!(data.packages[0].bump, "minor");
            assert_eq!(data.changesets.len(), 1);
            assert_eq!(data.changesets[0], "feature-new-api");
            assert_eq!(data.summary.total_packages, 1);
            assert_eq!(data.summary.minor_bumps, 1);
        }

        #[test]
        fn test_parse_preview_response_no_packages_to_bump() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Unified",
                    "packages": [
                        {
                            "name": "@scope/utils",
                            "path": "packages/utils",
                            "currentVersion": "2.0.0",
                            "nextVersion": "2.0.0",
                            "bumpType": "None",
                            "willBump": false,
                            "reason": "not in any changeset"
                        }
                    ],
                    "changesets": [],
                    "summary": {
                        "totalPackages": 1,
                        "packagesToBump": 0,
                        "packagesUnchanged": 1,
                        "totalChangesets": 0,
                        "hasCircularDependencies": false
                    }
                }
            }"#;

            let result = parse_preview_response(json.as_bytes());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.strategy, "unified");
            // Packages with willBump=false are filtered out
            assert_eq!(data.packages.len(), 0);
            assert_eq!(data.changesets.len(), 0);
            assert_eq!(data.summary.total_packages, 0);
        }

        #[test]
        fn test_parse_preview_response_multiple_bump_types() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Independent",
                    "packages": [
                        {
                            "name": "@scope/core",
                            "path": "packages/core",
                            "currentVersion": "1.0.0",
                            "nextVersion": "2.0.0",
                            "bumpType": "Major",
                            "willBump": true,
                            "reason": "breaking change"
                        },
                        {
                            "name": "@scope/utils",
                            "path": "packages/utils",
                            "currentVersion": "1.0.0",
                            "nextVersion": "1.1.0",
                            "bumpType": "Minor",
                            "willBump": true,
                            "reason": "new feature"
                        },
                        {
                            "name": "@scope/config",
                            "path": "packages/config",
                            "currentVersion": "1.0.0",
                            "nextVersion": "1.0.1",
                            "bumpType": "Patch",
                            "willBump": true,
                            "reason": "bug fix"
                        }
                    ],
                    "changesets": [],
                    "summary": {
                        "totalPackages": 3,
                        "packagesToBump": 3,
                        "packagesUnchanged": 0,
                        "totalChangesets": 1,
                        "hasCircularDependencies": false
                    }
                }
            }"#;

            let result = parse_preview_response(json.as_bytes());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.packages.len(), 3);
            assert_eq!(data.summary.total_packages, 3);
            assert_eq!(data.summary.major_bumps, 1);
            assert_eq!(data.summary.minor_bumps, 1);
            assert_eq!(data.summary.patch_bumps, 1);
        }

        #[test]
        fn test_parse_preview_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "No changesets found"
            }"#;

            let result = parse_preview_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EEXEC");
            assert!(error.message.contains("No changesets found"));
        }

        #[test]
        fn test_parse_preview_response_empty() {
            let result = parse_preview_response(b"");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_preview_response_whitespace_only() {
            let result = parse_preview_response(b"   \n  \t  ");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_preview_response_invalid_json() {
            let result = parse_preview_response(b"not valid json");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_preview_response_invalid_utf8() {
            let invalid_utf8 = vec![0xFF, 0xFE, 0x00, 0x01];
            let result = parse_preview_response(&invalid_utf8);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_preview_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_preview_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("no data"));
        }

        #[test]
        fn test_parse_preview_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_preview_response(json.as_bytes());
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
        fn test_convert_to_napi_preview_full() {
            let cli_data = CliBumpSnapshot {
                strategy: "Independent".to_string(),
                packages: vec![
                    CliPackageBumpInfo {
                        name: "@scope/core".to_string(),
                        path: "packages/core".to_string(),
                        current_version: "1.0.0".to_string(),
                        next_version: "2.0.0".to_string(),
                        bump_type: "Major".to_string(),
                        will_bump: true,
                        reason: "breaking change".to_string(),
                    },
                    CliPackageBumpInfo {
                        name: "@scope/utils".to_string(),
                        path: "packages/utils".to_string(),
                        current_version: "1.0.0".to_string(),
                        next_version: "1.0.0".to_string(),
                        bump_type: "None".to_string(),
                        will_bump: false,
                        reason: "not in changeset".to_string(),
                    },
                ],
                changesets: vec![CliChangesetInfo {
                    id: "feature-breaking".to_string(),
                    branch: "feature/breaking".to_string(),
                    bump_type: "Major".to_string(),
                    packages: vec!["@scope/core".to_string()],
                    commit_count: 3,
                }],
                summary: CliBumpSummary {
                    total_packages: 2,
                    packages_to_bump: 1,
                    packages_unchanged: 1,
                    total_changesets: 1,
                    has_circular_dependencies: false,
                },
            };

            let result = convert_to_napi_preview(cli_data);

            // Strategy should be lowercase
            assert_eq!(result.strategy, "independent");

            // Only packages with will_bump=true should be included
            assert_eq!(result.packages.len(), 1);
            assert_eq!(result.packages[0].name, "@scope/core");
            assert_eq!(result.packages[0].bump, "major");

            // Changesets should only have IDs
            assert_eq!(result.changesets.len(), 1);
            assert_eq!(result.changesets[0], "feature-breaking");

            // Summary should be calculated from filtered packages
            assert_eq!(result.summary.total_packages, 1);
            assert_eq!(result.summary.major_bumps, 1);
            assert_eq!(result.summary.minor_bumps, 0);
            assert_eq!(result.summary.patch_bumps, 0);
        }

        #[test]
        fn test_convert_to_napi_preview_empty() {
            let cli_data = CliBumpSnapshot {
                strategy: "Unified".to_string(),
                packages: vec![],
                changesets: vec![],
                summary: CliBumpSummary {
                    total_packages: 0,
                    packages_to_bump: 0,
                    packages_unchanged: 0,
                    total_changesets: 0,
                    has_circular_dependencies: false,
                },
            };

            let result = convert_to_napi_preview(cli_data);
            assert_eq!(result.strategy, "unified");
            assert!(result.packages.is_empty());
            assert!(result.changesets.is_empty());
            assert_eq!(result.summary.total_packages, 0);
        }

        #[test]
        fn test_convert_params_to_args_defaults() {
            let params = BumpPreviewParams::new("/path/to/project");
            let args = convert_params_to_args(&params);

            // Preview mode settings
            assert!(args.dry_run);
            assert!(!args.execute);
            assert!(!args.snapshot);

            // Default values
            assert!(args.packages.is_none());
            assert!(!args.show_diff);
            assert!(args.force);

            // No git operations
            assert!(!args.git_commit);
            assert!(!args.git_tag);
            assert!(!args.git_push);

            // No changelog/archive in preview
            assert!(args.no_changelog);
            assert!(args.no_archive);
        }

        #[test]
        fn test_convert_params_to_args_with_options() {
            let params = BumpPreviewParams::new("/path/to/project")
                .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
                .with_show_diff(true);
            let args = convert_params_to_args(&params);

            assert!(args.dry_run);
            assert_eq!(
                args.packages,
                Some(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
            );
            assert!(args.show_diff);
        }

        #[test]
        fn test_convert_params_to_args_show_diff_false() {
            let params = BumpPreviewParams::new("/path/to/project").with_show_diff(false);
            let args = convert_params_to_args(&params);
            assert!(!args.show_diff);
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_preview_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpPreviewParams::new(path_str);
            let result = validate_preview_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_preview_params_nonexistent_path() {
            let params = BumpPreviewParams::new("/nonexistent/path/to/project");
            let result = validate_preview_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_preview_params_empty_root() {
            let params = BumpPreviewParams::new("");
            let result = validate_preview_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_preview_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            std::fs::write(&file_path, "test content").unwrap();

            let params = BumpPreviewParams::new(file_path.to_str().unwrap());
            let result = validate_preview_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_preview_params_with_packages() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params =
                BumpPreviewParams::new(path_str).with_packages(vec!["@scope/core".to_string()]);
            let result = validate_preview_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_preview_params_with_config_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpPreviewParams::new(path_str).with_config_path("/path/to/config.json");
            let result = validate_preview_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_preview_params_returns_correct_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpPreviewParams::new(path_str);
            let result = validate_preview_params(&params);
            assert!(result.is_ok());
            let path = result.unwrap();
            assert_eq!(path.to_str().unwrap(), path_str);
        }
    }
}

// ============================================================================
// Validator Tests for Bump Commands (Story 5.2-5.4)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod bump_validator_tests {
    use crate::validation::validators;

    // -------------------------------------------------------------------------
    // Prerelease Tag Validator Tests
    // -------------------------------------------------------------------------

    /// Tests for prerelease tag validation.
    ///
    /// The prerelease tag is a simple string (e.g., "alpha", "beta", "rc").
    /// The mode (create, increment, promote) is automatically inferred based
    /// on each package's current version state.
    mod prerelease_tag_tests {
        use super::*;

        #[test]
        fn test_prerelease_tag_valid_alpha() {
            let result = validators::prerelease_tag("alpha");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_valid_beta() {
            let result = validators::prerelease_tag("beta");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_valid_rc() {
            let result = validators::prerelease_tag("rc");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_valid_canary() {
            let result = validators::prerelease_tag("canary");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_valid_with_hyphen() {
            let result = validators::prerelease_tag("beta-1");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_valid_with_numbers() {
            let result = validators::prerelease_tag("beta1");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_valid_uppercase() {
            let result = validators::prerelease_tag("RC1");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_valid_mixed_case() {
            let result = validators::prerelease_tag("Alpha-2");
            assert!(result.is_ok());
        }

        #[test]
        fn test_prerelease_tag_empty() {
            let result = validators::prerelease_tag("");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("cannot be empty"));
        }

        #[test]
        fn test_prerelease_tag_invalid_underscore() {
            let result = validators::prerelease_tag("beta_1");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("invalid characters"));
        }

        #[test]
        fn test_prerelease_tag_invalid_period() {
            // Period is not allowed in simple tag format
            let result = validators::prerelease_tag("beta.1");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("invalid characters"));
        }

        #[test]
        fn test_prerelease_tag_invalid_space() {
            let result = validators::prerelease_tag("beta 1");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("invalid characters"));
        }

        #[test]
        fn test_prerelease_tag_invalid_special_chars() {
            let result = validators::prerelease_tag("alpha@1");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("invalid characters"));
        }
    }

    // -------------------------------------------------------------------------
    // Snapshot Format Validator Tests
    // -------------------------------------------------------------------------

    mod snapshot_format_tests {
        use super::*;

        #[test]
        fn test_snapshot_format_valid_default() {
            let result = validators::snapshot_format("{version}-snapshot.{short_commit}");
            assert!(result.is_ok());
        }

        #[test]
        fn test_snapshot_format_valid_with_branch() {
            let result = validators::snapshot_format("{version}-{branch}.{short_commit}");
            assert!(result.is_ok());
        }

        #[test]
        fn test_snapshot_format_valid_with_timestamp() {
            let result = validators::snapshot_format("{version}-dev.{timestamp}");
            assert!(result.is_ok());
        }

        #[test]
        fn test_snapshot_format_valid_with_commit() {
            let result = validators::snapshot_format("{version}-{commit}");
            assert!(result.is_ok());
        }

        #[test]
        fn test_snapshot_format_valid_version_only() {
            let result = validators::snapshot_format("{version}");
            assert!(result.is_ok());
        }

        #[test]
        fn test_snapshot_format_valid_all_variables() {
            let result = validators::snapshot_format(
                "{version}-{branch}-{short_commit}-{commit}-{timestamp}",
            );
            assert!(result.is_ok());
        }

        #[test]
        fn test_snapshot_format_empty() {
            let result = validators::snapshot_format("");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("cannot be empty"));
        }

        #[test]
        fn test_snapshot_format_no_variables() {
            let result = validators::snapshot_format("no-variables-here");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("must contain at least one"));
        }

        #[test]
        fn test_snapshot_format_invalid_variable() {
            let result = validators::snapshot_format("{invalid}");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("must contain at least one"));
        }

        #[test]
        fn test_snapshot_format_partial_variable_name() {
            let result = validators::snapshot_format("{ver}-{bran}");
            assert!(result.is_err());
        }
    }
}

// ============================================================================
// Bump Apply Tests (Story 5.3)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod bump_apply_tests {
    use std::io::Write;

    use tempfile::TempDir;

    use crate::commands::bump::{
        CliBumpSnapshot, CliBumpSummary, CliExecuteResult, SharedBuffer,
        convert_apply_params_to_args, convert_to_napi_apply, parse_apply_response,
        validate_apply_params,
    };
    use crate::types::bump::BumpApplyParams;

    // -------------------------------------------------------------------------
    // SharedBuffer Tests (reused pattern, but validated for apply context)
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
            let bytes_written = buffer.write(b"apply result").unwrap();
            assert_eq!(bytes_written, 12);
            assert_eq!(buffer.take_bytes(), b"apply result");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"packages ").unwrap();
            buffer.write_all(b"updated").unwrap();
            assert_eq!(buffer.take_bytes(), b"packages updated");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();
            buffer.write_all(b"shared apply data").unwrap();

            // Both buffers should see the same data
            assert_eq!(buffer.take_bytes(), b"shared apply data");
            assert_eq!(buffer_clone.take_bytes(), b"shared apply data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"apply test data").unwrap();

            // Multiple takes should return same data
            let first_take = buffer.take_bytes();
            let second_take = buffer.take_bytes();
            assert_eq!(first_take, second_take);
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_apply_response_success() {
            // Note: CLI uses snake_case (no rename_all attribute)
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Independent",
                    "packages_updated": 2,
                    "changesets_archived": 1,
                    "files_modified": [
                        "packages/core/package.json",
                        "packages/core/CHANGELOG.md",
                        "packages/utils/package.json"
                    ],
                    "tags_created": ["@scope/core@1.1.0", "@scope/utils@2.0.0"],
                    "commit_sha": "abc123def456789",
                    "snapshot": {
                        "strategy": "Independent",
                        "packages": [],
                        "changesets": [],
                        "summary": {
                            "totalPackages": 0,
                            "packagesToBump": 0,
                            "packagesUnchanged": 0,
                            "totalChangesets": 0,
                            "hasCircularDependencies": false
                        }
                    }
                }
            }"#;

            let result = parse_apply_response(json.as_bytes());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.strategy, "independent");
            assert_eq!(data.packages_updated, 2);
            assert_eq!(data.changesets_archived, 1);
            assert_eq!(data.files_modified.len(), 3);
            assert!(data.files_modified.contains(&"packages/core/package.json".to_string()));
            assert!(data.files_modified.contains(&"packages/core/CHANGELOG.md".to_string()));
            assert_eq!(data.tags_created.len(), 2);
            assert!(data.tags_created.contains(&"@scope/core@1.1.0".to_string()));
            assert_eq!(data.commit_sha, Some("abc123def456789".to_string()));
        }

        #[test]
        fn test_parse_apply_response_no_git_operations() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Unified",
                    "packages_updated": 3,
                    "changesets_archived": 2,
                    "files_modified": ["packages/core/package.json"],
                    "tags_created": [],
                    "commit_sha": null,
                    "snapshot": {
                        "strategy": "Unified",
                        "packages": [],
                        "changesets": [],
                        "summary": {
                            "totalPackages": 0,
                            "packagesToBump": 0,
                            "packagesUnchanged": 0,
                            "totalChangesets": 0,
                            "hasCircularDependencies": false
                        }
                    }
                }
            }"#;

            let result = parse_apply_response(json.as_bytes());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.strategy, "unified");
            assert_eq!(data.packages_updated, 3);
            assert_eq!(data.changesets_archived, 2);
            assert!(data.tags_created.is_empty());
            assert!(data.commit_sha.is_none());
        }

        #[test]
        fn test_parse_apply_response_nothing_to_bump() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Independent",
                    "packages_updated": 0,
                    "changesets_archived": 0,
                    "files_modified": [],
                    "tags_created": [],
                    "commit_sha": null,
                    "snapshot": {
                        "strategy": "Independent",
                        "packages": [],
                        "changesets": [],
                        "summary": {
                            "totalPackages": 0,
                            "packagesToBump": 0,
                            "packagesUnchanged": 0,
                            "totalChangesets": 0,
                            "hasCircularDependencies": false
                        }
                    }
                }
            }"#;

            let result = parse_apply_response(json.as_bytes());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.packages_updated, 0);
            assert_eq!(data.changesets_archived, 0);
            assert!(data.files_modified.is_empty());
        }

        #[test]
        fn test_parse_apply_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "Git repository has uncommitted changes"
            }"#;

            let result = parse_apply_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EEXEC");
            assert!(error.message.contains("uncommitted changes"));
        }

        #[test]
        fn test_parse_apply_response_empty() {
            let result = parse_apply_response(b"");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_apply_response_whitespace_only() {
            let result = parse_apply_response(b"   \n  \t  ");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_apply_response_invalid_json() {
            let result = parse_apply_response(b"not valid json");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_apply_response_invalid_utf8() {
            let invalid_utf8 = vec![0xFF, 0xFE, 0x00, 0x01];
            let result = parse_apply_response(&invalid_utf8);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_apply_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_apply_response(json.as_bytes());
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("no data"));
        }

        #[test]
        fn test_parse_apply_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_apply_response(json.as_bytes());
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
        fn test_convert_to_napi_apply_full() {
            let cli_data = CliExecuteResult {
                strategy: "Independent".to_string(),
                packages_updated: 3,
                changesets_archived: 2,
                files_modified: vec![
                    "packages/core/package.json".to_string(),
                    "packages/core/CHANGELOG.md".to_string(),
                    "packages/utils/package.json".to_string(),
                ],
                tags_created: vec![
                    "@scope/core@1.1.0".to_string(),
                    "@scope/utils@2.0.0".to_string(),
                ],
                commit_sha: Some("abc123def456789".to_string()),
                snapshot: CliBumpSnapshot {
                    strategy: "Independent".to_string(),
                    packages: vec![],
                    changesets: vec![],
                    summary: CliBumpSummary {
                        total_packages: 0,
                        packages_to_bump: 0,
                        packages_unchanged: 0,
                        total_changesets: 0,
                        has_circular_dependencies: false,
                    },
                },
            };

            let result = convert_to_napi_apply(cli_data);

            assert_eq!(result.strategy, "independent");
            assert_eq!(result.packages_updated, 3);
            assert_eq!(result.changesets_archived, 2);
            assert_eq!(result.files_modified.len(), 3);
            assert_eq!(result.tags_created.len(), 2);
            assert_eq!(result.commit_sha, Some("abc123def456789".to_string()));
        }

        #[test]
        fn test_convert_to_napi_apply_no_git() {
            let cli_data = CliExecuteResult {
                strategy: "Unified".to_string(),
                packages_updated: 1,
                changesets_archived: 1,
                files_modified: vec!["packages/pkg/package.json".to_string()],
                tags_created: vec![],
                commit_sha: None,
                snapshot: CliBumpSnapshot {
                    strategy: "Unified".to_string(),
                    packages: vec![],
                    changesets: vec![],
                    summary: CliBumpSummary {
                        total_packages: 0,
                        packages_to_bump: 0,
                        packages_unchanged: 0,
                        total_changesets: 0,
                        has_circular_dependencies: false,
                    },
                },
            };

            let result = convert_to_napi_apply(cli_data);

            assert_eq!(result.strategy, "unified");
            assert_eq!(result.packages_updated, 1);
            assert!(result.tags_created.is_empty());
            assert!(result.commit_sha.is_none());
        }

        #[test]
        fn test_convert_to_napi_apply_empty() {
            let cli_data = CliExecuteResult {
                strategy: "Independent".to_string(),
                packages_updated: 0,
                changesets_archived: 0,
                files_modified: vec![],
                tags_created: vec![],
                commit_sha: None,
                snapshot: CliBumpSnapshot {
                    strategy: "Independent".to_string(),
                    packages: vec![],
                    changesets: vec![],
                    summary: CliBumpSummary {
                        total_packages: 0,
                        packages_to_bump: 0,
                        packages_unchanged: 0,
                        total_changesets: 0,
                        has_circular_dependencies: false,
                    },
                },
            };

            let result = convert_to_napi_apply(cli_data);

            assert_eq!(result.packages_updated, 0);
            assert_eq!(result.changesets_archived, 0);
            assert!(result.files_modified.is_empty());
            assert!(result.tags_created.is_empty());
        }

        #[test]
        fn test_convert_apply_params_to_args_defaults() {
            let params = BumpApplyParams::new("/path/to/project");
            let args = convert_apply_params_to_args(&params);

            // Execute mode settings
            assert!(!args.dry_run);
            assert!(args.execute);
            assert!(!args.snapshot);

            // Default git operations (all false)
            assert!(!args.git_commit);
            assert!(!args.git_tag);
            assert!(!args.git_push);

            // No prerelease by default
            assert!(args.prerelease.is_none());

            // Default changelog/archive settings
            assert!(!args.no_changelog);
            assert!(!args.no_archive);
            assert!(!args.always_archive);

            // Force is true for API (non-interactive)
            assert!(args.force);

            // No diff in execute mode
            assert!(!args.show_diff);

            // No package filter by default
            assert!(args.packages.is_none());
        }

        #[test]
        fn test_convert_apply_params_to_args_with_git_operations() {
            let params = BumpApplyParams::new("/path/to/project")
                .with_git_commit(true)
                .with_git_tag(true)
                .with_git_push(true);
            let args = convert_apply_params_to_args(&params);

            assert!(args.git_commit);
            assert!(args.git_tag);
            assert!(args.git_push);
        }

        #[test]
        fn test_convert_apply_params_to_args_with_prerelease() {
            let params = BumpApplyParams::new("/path/to/project").with_prerelease("beta");
            let args = convert_apply_params_to_args(&params);

            assert_eq!(args.prerelease, Some("beta".to_string()));
        }

        #[test]
        fn test_convert_apply_params_to_args_with_changelog_control() {
            let params = BumpApplyParams::new("/path/to/project").with_no_changelog(true);
            let args = convert_apply_params_to_args(&params);

            assert!(args.no_changelog);
        }

        #[test]
        fn test_convert_apply_params_to_args_with_archive_control() {
            let params = BumpApplyParams::new("/path/to/project")
                .with_no_archive(true)
                .with_always_archive(false);
            let args = convert_apply_params_to_args(&params);

            assert!(args.no_archive);
            assert!(!args.always_archive);
        }

        #[test]
        fn test_convert_apply_params_to_args_always_archive_for_prerelease() {
            let params = BumpApplyParams::new("/path/to/project")
                .with_prerelease("rc")
                .with_always_archive(true);
            let args = convert_apply_params_to_args(&params);

            assert_eq!(args.prerelease, Some("rc".to_string()));
            assert!(args.always_archive);
        }

        #[test]
        fn test_convert_apply_params_to_args_with_packages() {
            let params = BumpApplyParams::new("/path/to/project")
                .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()]);
            let args = convert_apply_params_to_args(&params);

            assert_eq!(
                args.packages,
                Some(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
            );
        }

        #[test]
        fn test_convert_apply_params_to_args_force_false() {
            let params = BumpApplyParams::new("/path/to/project").with_force(false);
            let args = convert_apply_params_to_args(&params);

            assert!(!args.force);
        }

        #[test]
        fn test_convert_apply_params_to_args_full_release() {
            // Simulates a full release with git operations and archiving
            let params = BumpApplyParams::new("/path/to/project")
                .with_git_commit(true)
                .with_git_tag(true)
                .with_git_push(true)
                .with_force(true);
            let args = convert_apply_params_to_args(&params);

            assert!(!args.dry_run);
            assert!(args.execute);
            assert!(args.git_commit);
            assert!(args.git_tag);
            assert!(args.git_push);
            assert!(!args.no_changelog);
            assert!(!args.no_archive);
            assert!(args.force);
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_apply_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str);
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_nonexistent_path() {
            let params = BumpApplyParams::new("/nonexistent/path/to/project");
            let result = validate_apply_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_apply_params_empty_root() {
            let params = BumpApplyParams::new("");
            let result = validate_apply_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_apply_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            std::fs::write(&file_path, "test content").unwrap();

            let params = BumpApplyParams::new(file_path.to_str().unwrap());
            let result = validate_apply_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_apply_params_valid_prerelease_alpha() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_prerelease("alpha");
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_valid_prerelease_beta() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_prerelease("beta");
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_valid_prerelease_rc() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_prerelease("rc");
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_valid_prerelease_custom() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_prerelease("next-1");
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_invalid_prerelease_period() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_prerelease("alpha.1");
            let result = validate_apply_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("invalid characters"));
        }

        #[test]
        fn test_validate_apply_params_invalid_prerelease_empty() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_prerelease("");
            let result = validate_apply_params(&params);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("cannot be empty"));
        }

        #[test]
        fn test_validate_apply_params_invalid_prerelease_underscore() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_prerelease("beta_1");
            let result = validate_apply_params(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_apply_params_with_git_options() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str)
                .with_git_commit(true)
                .with_git_tag(true)
                .with_git_push(true);
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_with_packages() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params =
                BumpApplyParams::new(path_str).with_packages(vec!["@scope/core".to_string()]);
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_with_config_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str).with_config_path("/path/to/config.json");
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_apply_params_returns_correct_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str);
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
            let path = result.unwrap();
            assert_eq!(path.to_str().unwrap(), path_str);
        }

        #[test]
        fn test_validate_apply_params_full_prerelease_workflow() {
            // Beta prerelease with git and archiving
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpApplyParams::new(path_str)
                .with_prerelease("beta")
                .with_git_commit(true)
                .with_git_tag(true)
                .with_always_archive(true);
            let result = validate_apply_params(&params);
            assert!(result.is_ok());
        }
    }
}

// =============================================================================
// Bump Snapshot Tests (Story 5.4)
// =============================================================================

/// Tests for the `bumpSnapshot` command.
///
/// These tests verify:
/// - `SharedBuffer` functionality for capturing CLI output
/// - JSON response parsing from CLI to NAPI types
/// - Type conversion from CLI structures to NAPI structures
/// - Parameter validation for snapshot format and root path
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod bump_snapshot_tests {
    use std::io::Write;

    use tempfile::TempDir;

    use crate::commands::bump::{
        CliBumpSnapshot, CliBumpSummary, CliChangesetInfo, CliPackageBumpInfo, SharedBuffer,
        convert_snapshot_params_to_args, convert_to_napi_snapshot, parse_snapshot_response,
        validate_snapshot_params,
    };
    use crate::types::bump::BumpSnapshotParams;

    // -------------------------------------------------------------------------
    // SharedBuffer Tests (reused pattern, validated for snapshot context)
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
            let bytes_written = buffer.write(b"snapshot result").unwrap();
            assert_eq!(bytes_written, 15);
            assert_eq!(buffer.take_bytes(), b"snapshot result");
        }

        #[test]
        fn test_shared_buffer_multiple_writes() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"snapshot ").unwrap();
            buffer.write_all(b"versions").unwrap();
            assert_eq!(buffer.take_bytes(), b"snapshot versions");
        }

        #[test]
        fn test_shared_buffer_clone_shares_data() {
            let mut buffer = SharedBuffer::new();
            let buffer_clone = buffer.clone();
            buffer.write_all(b"shared snapshot data").unwrap();

            // Both buffers should see the same data
            assert_eq!(buffer.take_bytes(), b"shared snapshot data");
            assert_eq!(buffer_clone.take_bytes(), b"shared snapshot data");
        }

        #[test]
        fn test_shared_buffer_flush() {
            let mut buffer = SharedBuffer::new();
            assert!(buffer.flush().is_ok());
        }

        #[test]
        fn test_shared_buffer_take_bytes_preserves_data() {
            let mut buffer = SharedBuffer::new();
            buffer.write_all(b"snapshot test data").unwrap();

            // Multiple takes should return same data
            let first_take = buffer.take_bytes();
            let second_take = buffer.take_bytes();
            assert_eq!(first_take, second_take);
        }
    }

    // -------------------------------------------------------------------------
    // Parse Response Tests
    // -------------------------------------------------------------------------

    mod parse_response_tests {
        use super::*;

        #[test]
        fn test_parse_snapshot_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Independent",
                    "packages": [
                        {
                            "name": "@scope/core",
                            "path": "packages/core",
                            "currentVersion": "1.0.0",
                            "nextVersion": "1.0.0-snapshot.abc123f",
                            "bumpType": "Minor",
                            "willBump": true,
                            "reason": "Has pending changesets"
                        },
                        {
                            "name": "@scope/utils",
                            "path": "packages/utils",
                            "currentVersion": "2.0.0",
                            "nextVersion": "2.0.0-snapshot.abc123f",
                            "bumpType": "Patch",
                            "willBump": true,
                            "reason": "Has pending changesets"
                        }
                    ],
                    "changesets": [
                        {
                            "id": "changeset-1",
                            "branch": "feature/test",
                            "bumpType": "Minor",
                            "packages": ["@scope/core"],
                            "commitCount": 3
                        }
                    ],
                    "summary": {
                        "totalPackages": 2,
                        "packagesToBump": 2,
                        "packagesUnchanged": 0,
                        "totalChangesets": 1,
                        "hasCircularDependencies": false
                    }
                }
            }"#;

            let format = "{version}-snapshot.{short_commit}".to_string();
            let result = parse_snapshot_response(json.as_bytes(), format.clone());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.strategy, "independent");
            assert_eq!(data.format, format);
            assert_eq!(data.packages.len(), 2);

            // Verify first package
            let pkg1 = &data.packages[0];
            assert_eq!(pkg1.name, "@scope/core");
            assert_eq!(pkg1.path, "packages/core");
            assert_eq!(pkg1.original_version, "1.0.0");
            assert_eq!(pkg1.snapshot_version, "1.0.0-snapshot.abc123f");

            // Verify second package
            let pkg2 = &data.packages[1];
            assert_eq!(pkg2.name, "@scope/utils");
            assert_eq!(pkg2.original_version, "2.0.0");
            assert_eq!(pkg2.snapshot_version, "2.0.0-snapshot.abc123f");
        }

        #[test]
        fn test_parse_snapshot_response_filters_non_bumping_packages() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Independent",
                    "packages": [
                        {
                            "name": "@scope/core",
                            "path": "packages/core",
                            "currentVersion": "1.0.0",
                            "nextVersion": "1.0.0-snapshot.abc123f",
                            "bumpType": "Minor",
                            "willBump": true,
                            "reason": "Has pending changesets"
                        },
                        {
                            "name": "@scope/unchanged",
                            "path": "packages/unchanged",
                            "currentVersion": "1.0.0",
                            "nextVersion": "1.0.0",
                            "bumpType": "None",
                            "willBump": false,
                            "reason": "No pending changesets"
                        }
                    ],
                    "changesets": [],
                    "summary": {
                        "totalPackages": 2,
                        "packagesToBump": 1,
                        "packagesUnchanged": 1,
                        "totalChangesets": 0,
                        "hasCircularDependencies": false
                    }
                }
            }"#;

            let format = "{version}-snapshot.{short_commit}".to_string();
            let result = parse_snapshot_response(json.as_bytes(), format);
            assert!(result.is_ok());
            let data = result.unwrap();
            // Only the package with willBump = true should be included
            assert_eq!(data.packages.len(), 1);
            assert_eq!(data.packages[0].name, "@scope/core");
        }

        #[test]
        fn test_parse_snapshot_response_empty_packages() {
            let json = r#"{
                "success": true,
                "data": {
                    "strategy": "Unified",
                    "packages": [],
                    "changesets": [],
                    "summary": {
                        "totalPackages": 0,
                        "packagesToBump": 0,
                        "packagesUnchanged": 0,
                        "totalChangesets": 0,
                        "hasCircularDependencies": false
                    }
                }
            }"#;

            let format = "{version}-dev.{timestamp}".to_string();
            let result = parse_snapshot_response(json.as_bytes(), format.clone());
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.strategy, "unified");
            assert_eq!(data.format, format);
            assert!(data.packages.is_empty());
        }

        #[test]
        fn test_parse_snapshot_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "No Git repository found"
            }"#;

            let result = parse_snapshot_response(json.as_bytes(), "{version}-snapshot".to_string());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.message.contains("No Git repository found"));
        }

        #[test]
        fn test_parse_snapshot_response_empty() {
            let result = parse_snapshot_response(b"", "{version}".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_snapshot_response_whitespace_only() {
            let result = parse_snapshot_response(b"   \n\t  ", "{version}".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_snapshot_response_invalid_json() {
            let result = parse_snapshot_response(b"not valid json", "{version}".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_snapshot_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_snapshot_response(&invalid_utf8, "{version}".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_snapshot_response_success_no_data() {
            let json = r#"{"success": true}"#;
            let result = parse_snapshot_response(json.as_bytes(), "{version}".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_snapshot_response_cli_error_no_message() {
            let json = r#"{"success": false}"#;
            let result = parse_snapshot_response(json.as_bytes(), "{version}".to_string());
            assert!(result.is_err());
        }
    }

    // -------------------------------------------------------------------------
    // Conversion Tests
    // -------------------------------------------------------------------------

    mod conversion_tests {
        use super::*;

        fn create_test_cli_snapshot() -> CliBumpSnapshot {
            CliBumpSnapshot {
                strategy: "Independent".to_string(),
                packages: vec![
                    CliPackageBumpInfo {
                        name: "@scope/core".to_string(),
                        path: "packages/core".to_string(),
                        current_version: "1.0.0".to_string(),
                        next_version: "1.0.0-snapshot.abc123f".to_string(),
                        bump_type: "Minor".to_string(),
                        will_bump: true,
                        reason: "Has pending changesets".to_string(),
                    },
                    CliPackageBumpInfo {
                        name: "@scope/utils".to_string(),
                        path: "packages/utils".to_string(),
                        current_version: "2.0.0".to_string(),
                        next_version: "2.0.0-snapshot.abc123f".to_string(),
                        bump_type: "Patch".to_string(),
                        will_bump: true,
                        reason: "Has pending changesets".to_string(),
                    },
                ],
                changesets: vec![CliChangesetInfo {
                    id: "changeset-1".to_string(),
                    branch: "feature/test".to_string(),
                    bump_type: "Minor".to_string(),
                    packages: vec!["@scope/core".to_string()],
                    commit_count: 3,
                }],
                summary: CliBumpSummary {
                    total_packages: 2,
                    packages_to_bump: 2,
                    packages_unchanged: 0,
                    total_changesets: 1,
                    has_circular_dependencies: false,
                },
            }
        }

        #[test]
        fn test_convert_to_napi_snapshot_full() {
            let cli_data = create_test_cli_snapshot();
            let format = "{version}-snapshot.{short_commit}".to_string();
            let napi_data = convert_to_napi_snapshot(cli_data, format.clone());

            assert_eq!(napi_data.strategy, "independent");
            assert_eq!(napi_data.format, format);
            assert_eq!(napi_data.packages.len(), 2);

            // Verify conversion mapping
            let pkg1 = &napi_data.packages[0];
            assert_eq!(pkg1.name, "@scope/core");
            assert_eq!(pkg1.path, "packages/core");
            assert_eq!(pkg1.original_version, "1.0.0");
            assert_eq!(pkg1.snapshot_version, "1.0.0-snapshot.abc123f");

            let pkg2 = &napi_data.packages[1];
            assert_eq!(pkg2.name, "@scope/utils");
            assert_eq!(pkg2.path, "packages/utils");
            assert_eq!(pkg2.original_version, "2.0.0");
            assert_eq!(pkg2.snapshot_version, "2.0.0-snapshot.abc123f");
        }

        #[test]
        fn test_convert_to_napi_snapshot_filters_non_bumping() {
            let cli_data = CliBumpSnapshot {
                strategy: "Unified".to_string(),
                packages: vec![
                    CliPackageBumpInfo {
                        name: "@scope/bumped".to_string(),
                        path: "packages/bumped".to_string(),
                        current_version: "1.0.0".to_string(),
                        next_version: "1.0.0-dev.123".to_string(),
                        bump_type: "Minor".to_string(),
                        will_bump: true,
                        reason: "Has changesets".to_string(),
                    },
                    CliPackageBumpInfo {
                        name: "@scope/unchanged".to_string(),
                        path: "packages/unchanged".to_string(),
                        current_version: "1.0.0".to_string(),
                        next_version: "1.0.0".to_string(),
                        bump_type: "None".to_string(),
                        will_bump: false,
                        reason: "No changesets".to_string(),
                    },
                ],
                changesets: vec![],
                summary: CliBumpSummary {
                    total_packages: 2,
                    packages_to_bump: 1,
                    packages_unchanged: 1,
                    total_changesets: 0,
                    has_circular_dependencies: false,
                },
            };

            let format = "{version}-dev.{timestamp}".to_string();
            let napi_data = convert_to_napi_snapshot(cli_data, format);

            assert_eq!(napi_data.packages.len(), 1);
            assert_eq!(napi_data.packages[0].name, "@scope/bumped");
        }

        #[test]
        fn test_convert_to_napi_snapshot_empty() {
            let cli_data = CliBumpSnapshot {
                strategy: "Independent".to_string(),
                packages: vec![],
                changesets: vec![],
                summary: CliBumpSummary {
                    total_packages: 0,
                    packages_to_bump: 0,
                    packages_unchanged: 0,
                    total_changesets: 0,
                    has_circular_dependencies: false,
                },
            };

            let format = "{version}-snapshot".to_string();
            let napi_data = convert_to_napi_snapshot(cli_data, format.clone());

            assert_eq!(napi_data.strategy, "independent");
            assert_eq!(napi_data.format, format);
            assert!(napi_data.packages.is_empty());
        }

        #[test]
        fn test_convert_snapshot_params_to_args_defaults() {
            let params = BumpSnapshotParams::new(".");
            let args = convert_snapshot_params_to_args(&params);

            // Snapshot mode flags
            assert!(!args.dry_run);
            assert!(!args.execute);
            assert!(args.snapshot);
            assert!(args.snapshot_format.is_none());

            // No prerelease in snapshot mode
            assert!(args.prerelease.is_none());

            // No packages filter by default
            assert!(args.packages.is_none());

            // No git operations
            assert!(!args.git_tag);
            assert!(!args.git_push);
            assert!(!args.git_commit);

            // No changelog or archive
            assert!(args.no_changelog);
            assert!(args.no_archive);
            assert!(!args.always_archive);

            // Non-interactive
            assert!(args.force);
            assert!(!args.show_diff);
        }

        #[test]
        fn test_convert_snapshot_params_to_args_with_format() {
            let params =
                BumpSnapshotParams::new(".").with_format("{version}-{branch}.{short_commit}");
            let args = convert_snapshot_params_to_args(&params);

            assert!(args.snapshot);
            assert_eq!(args.snapshot_format, Some("{version}-{branch}.{short_commit}".to_string()));
        }

        #[test]
        fn test_convert_snapshot_params_to_args_with_packages() {
            let params = BumpSnapshotParams::new(".")
                .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()]);
            let args = convert_snapshot_params_to_args(&params);

            assert!(args.snapshot);
            let packages = args.packages.unwrap();
            assert_eq!(packages.len(), 2);
            assert!(packages.contains(&"@scope/core".to_string()));
            assert!(packages.contains(&"@scope/utils".to_string()));
        }

        #[test]
        fn test_convert_snapshot_params_to_args_full() {
            let params = BumpSnapshotParams::new("/path/to/project")
                .with_config_path("/path/to/config.json")
                .with_packages(vec!["pkg-a".to_string()])
                .with_format("{version}-dev.{timestamp}");
            let args = convert_snapshot_params_to_args(&params);

            assert!(args.snapshot);
            assert_eq!(args.snapshot_format, Some("{version}-dev.{timestamp}".to_string()));
            assert_eq!(args.packages, Some(vec!["pkg-a".to_string()]));

            // Verify snapshot-specific settings
            assert!(!args.dry_run);
            assert!(!args.execute);
            assert!(args.no_changelog);
            assert!(args.no_archive);
        }
    }

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_snapshot_params_valid_directory() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str);
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_nonexistent_path() {
            let params = BumpSnapshotParams::new("/nonexistent/path/12345");
            let result = validate_snapshot_params(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_snapshot_params_empty_root() {
            let params = BumpSnapshotParams::new("");
            let result = validate_snapshot_params(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_snapshot_params_file_not_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            std::fs::write(&file_path, "test").unwrap();
            let params = BumpSnapshotParams::new(file_path.to_str().unwrap());
            let result = validate_snapshot_params(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_snapshot_params_valid_format_default() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params =
                BumpSnapshotParams::new(path_str).with_format("{version}-snapshot.{short_commit}");
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_valid_format_branch() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params =
                BumpSnapshotParams::new(path_str).with_format("{version}-{branch}.{short_commit}");
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_valid_format_timestamp() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str).with_format("{version}-dev.{timestamp}");
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_valid_format_commit() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str).with_format("{version}-{commit}");
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_valid_format_version_only() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str).with_format("{version}-snapshot");
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_valid_format_all_variables() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str)
                .with_format("{version}-{branch}-{short_commit}-{commit}-{timestamp}");
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_invalid_format_empty() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str).with_format("");
            let result = validate_snapshot_params(&params);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.message.contains("empty"));
        }

        #[test]
        fn test_validate_snapshot_params_invalid_format_no_variables() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str).with_format("no-variables-here");
            let result = validate_snapshot_params(&params);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.message.contains("must contain at least one valid variable"));
        }

        #[test]
        fn test_validate_snapshot_params_invalid_format_invalid_variable() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str).with_format("{invalid}");
            let result = validate_snapshot_params(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_snapshot_params_with_config_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str).with_config_path("/path/to/config.json");
            let result = validate_snapshot_params(&params);
            // Config path is not validated for existence
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_with_packages() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str)
                .with_packages(vec!["@scope/core".to_string(), "@scope/utils".to_string()]);
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_snapshot_params_returns_correct_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str);
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
            let path = result.unwrap();
            assert_eq!(path.to_str().unwrap(), path_str);
        }

        #[test]
        fn test_validate_snapshot_params_full_workflow() {
            // Complete snapshot params with all options
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = BumpSnapshotParams::new(path_str)
                .with_config_path("repo.config.json")
                .with_packages(vec!["@scope/core".to_string()])
                .with_format("{version}-{branch}.{short_commit}");
            let result = validate_snapshot_params(&params);
            assert!(result.is_ok());
        }
    }
}

// ============================================================================
// Execute Command Tests (Story 6.3)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod execute_tests {
    use std::io::Write;

    use crate::commands::execute::{
        CliExecuteData, CliExecuteSummary, CliPackageExecutionResult, SharedBuffer,
        convert_params_to_args, convert_to_napi_execute, parse_execute_response, resolve_timeouts,
        validate_params,
    };
    use crate::types::execute::ExecuteParams;
    use tempfile::TempDir;

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
    // Validation Tests
    // -------------------------------------------------------------------------

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_params_valid() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test");
            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_empty_root() {
            let params = ExecuteParams::new("", "npm:test");
            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("root"));
        }

        #[test]
        fn test_validate_params_nonexistent_root() {
            let params = ExecuteParams::new("/nonexistent/path/12345", "npm:test");
            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_params_empty_cmd() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "");
            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("empty"));
        }

        #[test]
        fn test_validate_params_whitespace_cmd() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "   ");
            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_params_mutual_exclusion() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test")
                .with_filter_package(vec!["@scope/pkg".to_string()])
                .with_affected(true);
            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("mutually exclusive"));
        }

        #[test]
        fn test_validate_params_filter_package_only() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test")
                .with_filter_package(vec!["@scope/pkg".to_string()]);
            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_affected_only() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test").with_affected(true);
            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_timeout_valid() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test")
                .with_timeout_secs(300)
                .with_per_package_timeout_secs(60);
            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_timeout_exceeds_max() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test").with_timeout_secs(100_000);
            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
            assert!(error.message.contains("exceed"));
        }

        #[test]
        fn test_validate_params_per_package_timeout_exceeds_max() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params =
                ExecuteParams::new(path_str, "npm:test").with_per_package_timeout_secs(10000);
            let result = validate_params(&params);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_params_with_all_options() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:build")
                .with_parallel(true)
                .with_args(vec!["--verbose".to_string()])
                .with_timeout_secs(600);
            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_with_affected_and_branch() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test")
                .with_affected(true)
                .with_branch("main".to_string());
            let result = validate_params(&params);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_params_returns_correct_path() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test");
            let result = validate_params(&params);
            assert!(result.is_ok());
            let path = result.unwrap();
            assert_eq!(path.to_str().unwrap(), path_str);
        }
    }

    // -------------------------------------------------------------------------
    // Timeout Resolution Tests
    // -------------------------------------------------------------------------

    mod timeout_resolution_tests {
        use super::*;

        #[test]
        fn test_resolve_timeouts_defaults() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test");
            let (global, per_pkg) = resolve_timeouts(&params);

            // Should use ExecuteConfig defaults: 300 and 60
            assert_eq!(global, 300);
            assert_eq!(per_pkg, 60);
        }

        #[test]
        fn test_resolve_timeouts_with_overrides() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test")
                .with_timeout_secs(600)
                .with_per_package_timeout_secs(120);
            let (global, per_pkg) = resolve_timeouts(&params);

            assert_eq!(global, 600);
            assert_eq!(per_pkg, 120);
        }

        #[test]
        fn test_resolve_timeouts_zero_means_no_timeout() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let mut params = ExecuteParams::new(path_str, "npm:test");
            params.timeout_secs = Some(0);
            params.per_package_timeout_secs = Some(0);
            let (global, per_pkg) = resolve_timeouts(&params);

            assert_eq!(global, 0);
            assert_eq!(per_pkg, 0);
        }

        #[test]
        fn test_resolve_timeouts_partial_override() {
            let temp_dir = TempDir::new().unwrap();
            let path_str = temp_dir.path().to_str().unwrap();
            let params = ExecuteParams::new(path_str, "npm:test").with_timeout_secs(900);
            let (global, per_pkg) = resolve_timeouts(&params);

            assert_eq!(global, 900);
            assert_eq!(per_pkg, 60); // Default
        }
    }

    // -------------------------------------------------------------------------
    // Args Conversion Tests
    // -------------------------------------------------------------------------

    mod args_conversion_tests {
        use super::*;

        #[test]
        fn test_convert_params_to_args_basic() {
            let params = ExecuteParams::new(".", "npm:test");
            let args = convert_params_to_args(&params);

            assert_eq!(args.cmd, "npm:test");
            assert!(args.filter_package.is_none());
            assert!(!args.affected);
            assert!(!args.parallel);
            assert!(args.args.is_empty());
        }

        #[test]
        fn test_convert_params_to_args_with_filter() {
            let params = ExecuteParams::new(".", "npm:build")
                .with_filter_package(vec!["@scope/core".to_string(), "@scope/utils".to_string()]);
            let args = convert_params_to_args(&params);

            assert_eq!(args.cmd, "npm:build");
            assert_eq!(
                args.filter_package,
                Some(vec!["@scope/core".to_string(), "@scope/utils".to_string()])
            );
            assert!(!args.affected);
        }

        #[test]
        fn test_convert_params_to_args_with_affected() {
            let params = ExecuteParams::new(".", "npm:lint")
                .with_affected(true)
                .with_branch("main".to_string());
            let args = convert_params_to_args(&params);

            assert_eq!(args.cmd, "npm:lint");
            assert!(args.affected);
            assert_eq!(args.branch, Some("main".to_string()));
        }

        #[test]
        fn test_convert_params_to_args_with_parallel() {
            let params = ExecuteParams::new(".", "npm:test").with_parallel(true);
            let args = convert_params_to_args(&params);

            assert!(args.parallel);
        }

        #[test]
        fn test_convert_params_to_args_with_extra_args() {
            let params = ExecuteParams::new(".", "npm:test")
                .with_args(vec!["--coverage".to_string(), "--verbose".to_string()]);
            let args = convert_params_to_args(&params);

            assert_eq!(args.args, vec!["--coverage".to_string(), "--verbose".to_string()]);
        }

        #[test]
        fn test_convert_params_to_args_with_since_until() {
            let params = ExecuteParams::new(".", "npm:test")
                .with_affected(true)
                .with_since("v1.0.0".to_string())
                .with_until("HEAD".to_string());
            let args = convert_params_to_args(&params);

            assert!(args.affected);
            assert_eq!(args.since, Some("v1.0.0".to_string()));
            assert_eq!(args.until, Some("HEAD".to_string()));
        }
    }

    // -------------------------------------------------------------------------
    // Response Parsing Tests
    // -------------------------------------------------------------------------

    mod parsing_tests {
        use super::*;

        #[test]
        fn test_parse_execute_response_success() {
            let json = r#"{
                "success": true,
                "data": {
                    "command": "npm run test",
                    "results": [
                        {
                            "package": "@scope/core",
                            "success": true,
                            "exitCode": 0,
                            "durationMs": 1500
                        },
                        {
                            "package": "@scope/utils",
                            "success": true,
                            "exitCode": 0,
                            "durationMs": 1200
                        }
                    ],
                    "summary": {
                        "total": 2,
                        "succeeded": 2,
                        "failed": 0,
                        "totalDurationMs": 2700
                    }
                }
            }"#;

            let result = parse_execute_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.command, "npm run test");
            assert_eq!(data.results.len(), 2);
            assert_eq!(data.summary.total, 2);
            assert_eq!(data.summary.succeeded, 2);
            assert_eq!(data.summary.failed, 0);
        }

        #[test]
        fn test_parse_execute_response_with_failures() {
            let json = r#"{
                "success": true,
                "data": {
                    "command": "npm run build",
                    "results": [
                        {
                            "package": "@scope/core",
                            "success": true,
                            "exitCode": 0,
                            "durationMs": 3000
                        },
                        {
                            "package": "@scope/broken",
                            "success": false,
                            "exitCode": 1,
                            "durationMs": 500,
                            "error": "Build failed: compilation error"
                        }
                    ],
                    "summary": {
                        "total": 2,
                        "succeeded": 1,
                        "failed": 1,
                        "totalDurationMs": 3500
                    }
                }
            }"#;

            let result = parse_execute_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert_eq!(data.summary.failed, 1);
            assert_eq!(data.results[1].error, Some("Build failed: compilation error".to_string()));
        }

        #[test]
        fn test_parse_execute_response_cli_error() {
            let json = r#"{
                "success": false,
                "error": "No packages found in workspace"
            }"#;

            let result = parse_execute_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.code, "EEXEC");
            assert!(error.message.contains("No packages found"));
        }

        #[test]
        fn test_parse_execute_response_empty() {
            let result = parse_execute_response(b"");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_execute_response_whitespace_only() {
            let result = parse_execute_response(b"   \n\t  ");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("empty response"));
        }

        #[test]
        fn test_parse_execute_response_invalid_json() {
            let result = parse_execute_response(b"not valid json");
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Failed to parse"));
        }

        #[test]
        fn test_parse_execute_response_invalid_utf8() {
            let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
            let result = parse_execute_response(&invalid_utf8);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid UTF-8"));
        }

        #[test]
        fn test_parse_execute_response_success_no_data() {
            let json = r#"{
                "success": true
            }"#;

            let result = parse_execute_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("no data"));
        }

        #[test]
        fn test_parse_execute_response_cli_error_no_message() {
            let json = r#"{
                "success": false
            }"#;

            let result = parse_execute_response(json.as_bytes());
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.message.contains("Unknown CLI error"));
        }

        #[test]
        fn test_parse_execute_response_empty_results() {
            let json = r#"{
                "success": true,
                "data": {
                    "command": "npm run test",
                    "results": [],
                    "summary": {
                        "total": 0,
                        "succeeded": 0,
                        "failed": 0,
                        "totalDurationMs": 0
                    }
                }
            }"#;

            let result = parse_execute_response(json.as_bytes());
            assert!(result.is_ok());

            let data = result.unwrap();
            assert!(data.results.is_empty());
            assert_eq!(data.summary.total, 0);
        }
    }

    // -------------------------------------------------------------------------
    // Conversion Tests
    // -------------------------------------------------------------------------

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_convert_to_napi_execute_full() {
            let cli_data = CliExecuteData {
                command: "npm run build".to_string(),
                results: vec![
                    CliPackageExecutionResult {
                        package: "@org/core".to_string(),
                        success: true,
                        exit_code: 0,
                        duration_ms: 2500,
                        error: None,
                    },
                    CliPackageExecutionResult {
                        package: "@org/utils".to_string(),
                        success: false,
                        exit_code: 1,
                        duration_ms: 500,
                        error: Some("Build failed".to_string()),
                    },
                ],
                summary: CliExecuteSummary {
                    total: 2,
                    succeeded: 1,
                    failed: 1,
                    total_duration_ms: 3000,
                },
            };

            let napi_data = convert_to_napi_execute(cli_data);

            assert_eq!(napi_data.command, "npm run build");
            assert_eq!(napi_data.results.len(), 2);
            assert_eq!(napi_data.results[0].package, "@org/core");
            assert!(napi_data.results[0].success);
            assert_eq!(napi_data.results[0].exit_code, 0);
            assert!((napi_data.results[0].duration_ms - 2500.0).abs() < f64::EPSILON);
            assert!(napi_data.results[0].error.is_none());

            assert_eq!(napi_data.results[1].package, "@org/utils");
            assert!(!napi_data.results[1].success);
            assert_eq!(napi_data.results[1].exit_code, 1);
            assert_eq!(napi_data.results[1].error, Some("Build failed".to_string()));

            assert_eq!(napi_data.summary.total, 2);
            assert_eq!(napi_data.summary.succeeded, 1);
            assert_eq!(napi_data.summary.failed, 1);
            assert!((napi_data.summary.total_duration_ms - 3000.0).abs() < f64::EPSILON);
        }

        #[test]
        fn test_convert_to_napi_execute_empty() {
            let cli_data = CliExecuteData {
                command: "ls -la".to_string(),
                results: vec![],
                summary: CliExecuteSummary {
                    total: 0,
                    succeeded: 0,
                    failed: 0,
                    total_duration_ms: 0,
                },
            };

            let napi_data = convert_to_napi_execute(cli_data);

            assert_eq!(napi_data.command, "ls -la");
            assert!(napi_data.results.is_empty());
            assert_eq!(napi_data.summary.total, 0);
        }

        #[test]
        fn test_convert_to_napi_execute_preserves_error_messages() {
            let cli_data = CliExecuteData {
                command: "npm run test".to_string(),
                results: vec![CliPackageExecutionResult {
                    package: "@org/failing".to_string(),
                    success: false,
                    exit_code: 127,
                    duration_ms: 100,
                    error: Some("Command not found: jest".to_string()),
                }],
                summary: CliExecuteSummary {
                    total: 1,
                    succeeded: 0,
                    failed: 1,
                    total_duration_ms: 100,
                },
            };

            let napi_data = convert_to_napi_execute(cli_data);

            assert_eq!(napi_data.results[0].exit_code, 127);
            assert_eq!(napi_data.results[0].error, Some("Command not found: jest".to_string()));
        }
    }

    // -------------------------------------------------------------------------
    // ExecuteParams Builder Tests
    // -------------------------------------------------------------------------

    mod params_builder_tests {
        use super::*;

        #[test]
        fn test_execute_params_new() {
            let params = ExecuteParams::new("/path/to/project", "npm:test");
            assert_eq!(params.root, "/path/to/project");
            assert_eq!(params.cmd, "npm:test");
            assert!(params.filter_package.is_none());
            assert!(params.affected.is_none());
            assert!(params.parallel.is_none());
        }

        #[test]
        fn test_execute_params_builder_chain() {
            let params = ExecuteParams::new(".", "npm:build")
                .with_filter_package(vec!["@scope/core".to_string()])
                .with_parallel(true)
                .with_timeout_secs(600)
                .with_per_package_timeout_secs(120)
                .with_args(vec!["--verbose".to_string()]);

            assert_eq!(params.filter_package, Some(vec!["@scope/core".to_string()]));
            assert_eq!(params.parallel, Some(true));
            assert_eq!(params.timeout_secs, Some(600));
            assert_eq!(params.per_package_timeout_secs, Some(120));
            assert_eq!(params.args, Some(vec!["--verbose".to_string()]));
        }

        #[test]
        fn test_execute_params_affected_chain() {
            let params = ExecuteParams::new(".", "npm:test")
                .with_affected(true)
                .with_branch("main".to_string())
                .with_since("v1.0.0".to_string())
                .with_until("HEAD".to_string());

            assert_eq!(params.affected, Some(true));
            assert_eq!(params.branch, Some("main".to_string()));
            assert_eq!(params.since, Some("v1.0.0".to_string()));
            assert_eq!(params.until, Some("HEAD".to_string()));
        }

        #[test]
        fn test_execute_params_helper_methods() {
            let params = ExecuteParams::new(".", "npm:test")
                .with_filter_package(vec!["@scope/core".to_string()])
                .with_affected(false)
                .with_parallel(true);

            assert!(params.has_filter_package());
            assert!(!params.is_affected());
            assert!(params.is_parallel());
        }

        #[test]
        fn test_execute_params_helper_methods_defaults() {
            let params = ExecuteParams::new(".", "npm:test");

            assert!(!params.has_filter_package());
            assert!(!params.is_affected());
            assert!(!params.is_parallel());
        }
    }
}

// ============================================================================
// Config Show Tests (Story 7.2)
// ============================================================================

/// Tests for the `config_show` command implementation.
///
/// These tests verify:
/// - Parameter validation (root path validation)
/// - Configuration file discovery
/// - Configuration parsing and conversion
/// - Type conversions from pkg crate to NAPI types
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod config_show_tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    use crate::commands::config::{
        convert_backup_config, convert_changelog_config, convert_changeset_config,
        convert_dependency_config, convert_execute_config, convert_git_config,
        convert_health_score_weights, convert_registry_config, convert_upgrade_config,
        convert_version_config, format_to_string, validate_params,
    };
    use crate::types::config::ConfigShowParams;
    use sublime_pkg_tools::config::{
        AuditConfig, AuditSectionsConfig, BackupConfig, ChangelogConfig, ChangelogFormat,
        ChangesetConfig, ConfigFormat, DependencyConfig, ExecuteConfig, GitConfig,
        HealthScoreWeightsConfig, MonorepoMode, RegistryConfig, UpgradeConfig, VersionConfig,
    };
    use sublime_pkg_tools::types::VersioningStrategy;

    // ========================================================================
    // Validation Tests
    // ========================================================================

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_params_valid_directory() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let params = ConfigShowParams {
                root: temp_dir.path().to_string_lossy().to_string(),
                config_path: None,
            };

            let result = validate_params(&params);
            assert!(result.is_ok());
            assert_eq!(result.ok(), Some(PathBuf::from(temp_dir.path())));
        }

        #[test]
        fn test_validate_params_nonexistent_path() {
            let params = ConfigShowParams {
                root: "/nonexistent/path/that/does/not/exist".to_string(),
                config_path: None,
            };

            let result = validate_params(&params);
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, "ENOENT");
        }

        #[test]
        fn test_validate_params_empty_root() {
            let params = ConfigShowParams { root: String::new(), config_path: None };

            let result = validate_params(&params);
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_params_file_not_directory() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test_file.txt");
            fs::write(&file_path, "test content").expect("Failed to write file");

            let params = ConfigShowParams {
                root: file_path.to_string_lossy().to_string(),
                config_path: None,
            };

            let result = validate_params(&params);
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, "EVALIDATION");
        }

        #[test]
        fn test_validate_params_with_config_path() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let params = ConfigShowParams {
                root: temp_dir.path().to_string_lossy().to_string(),
                config_path: Some("custom/repo.config.json".to_string()),
            };

            // This should pass validation (config path existence is checked later)
            let result = validate_params(&params);
            assert!(result.is_ok());
        }
    }

    // ========================================================================
    // Format Conversion Tests
    // ========================================================================

    mod format_tests {
        use super::*;

        #[test]
        fn test_format_to_string_json() {
            assert_eq!(format_to_string(ConfigFormat::Json), "json");
        }

        #[test]
        fn test_format_to_string_toml() {
            assert_eq!(format_to_string(ConfigFormat::Toml), "toml");
        }

        #[test]
        fn test_format_to_string_yaml() {
            assert_eq!(format_to_string(ConfigFormat::Yaml), "yaml");
        }
    }

    // ========================================================================
    // Type Conversion Tests
    // ========================================================================

    mod conversion_tests {
        use super::*;
        use std::collections::HashMap;

        #[test]
        fn test_convert_changeset_config() {
            let config = ChangesetConfig {
                path: ".custom-changesets".to_string(),
                history_path: ".custom-changesets/history".to_string(),
                available_environments: vec!["prod".to_string(), "staging".to_string()],
                default_environments: vec!["prod".to_string()],
            };

            let result = convert_changeset_config(&config);

            assert_eq!(result.path, ".custom-changesets");
            assert_eq!(result.history_path, ".custom-changesets/history");
            assert_eq!(result.available_environments, vec!["prod", "staging"]);
            assert_eq!(result.default_environments, vec!["prod"]);
        }

        #[test]
        fn test_convert_version_config_independent() {
            let config = VersionConfig {
                strategy: VersioningStrategy::Independent,
                default_bump: "minor".to_string(),
                snapshot_format: "{version}-{branch}.{commit}".to_string(),
            };

            let result = convert_version_config(&config);

            assert_eq!(result.strategy, "independent");
            assert_eq!(result.default_bump, "minor");
            assert_eq!(result.snapshot_format, "{version}-{branch}.{commit}");
        }

        #[test]
        fn test_convert_version_config_unified() {
            let config = VersionConfig {
                strategy: VersioningStrategy::Unified,
                default_bump: "patch".to_string(),
                snapshot_format: "{version}-snapshot".to_string(),
            };

            let result = convert_version_config(&config);

            assert_eq!(result.strategy, "unified");
            assert_eq!(result.default_bump, "patch");
        }

        #[test]
        fn test_convert_dependency_config() {
            let config = DependencyConfig {
                propagation_bump: "minor".to_string(),
                propagate_dependencies: true,
                propagate_dev_dependencies: false,
                propagate_peer_dependencies: true,
                max_depth: 15,
                fail_on_circular: true,
                skip_workspace_protocol: false,
                skip_file_protocol: true,
                skip_link_protocol: true,
                skip_portal_protocol: false,
            };

            let result = convert_dependency_config(&config);

            assert_eq!(result.propagation_bump, "minor");
            assert!(result.propagate_dependencies);
            assert!(!result.propagate_dev_dependencies);
            assert!(result.propagate_peer_dependencies);
            assert_eq!(result.max_depth, 15);
            assert!(result.fail_on_circular);
            assert!(!result.skip_workspace_protocol);
            assert!(result.skip_file_protocol);
        }

        #[test]
        fn test_convert_registry_config() {
            let mut scoped_registries = HashMap::new();
            scoped_registries.insert("@myorg".to_string(), "https://npm.myorg.com".to_string());

            let config = RegistryConfig {
                default_registry: "https://registry.npmjs.org".to_string(),
                scoped_registries,
                timeout_secs: 60,
                retry_attempts: 5,
                read_npmrc: false,
                ..Default::default()
            };

            let result = convert_registry_config(&config);

            assert_eq!(result.default_registry, "https://registry.npmjs.org");
            assert_eq!(result.scoped_registries.len(), 1);
            assert_eq!(result.scoped_registries[0].scope, "@myorg");
            assert_eq!(result.scoped_registries[0].registry, "https://npm.myorg.com");
            assert_eq!(result.timeout_secs, 60);
            assert_eq!(result.retry_attempts, 5);
            assert!(!result.read_npmrc);
        }

        #[test]
        fn test_convert_backup_config() {
            let config = BackupConfig {
                enabled: true,
                backup_dir: ".backups".to_string(),
                keep_after_success: true,
                max_backups: 10,
            };

            let result = convert_backup_config(&config);

            assert!(result.enabled);
            assert_eq!(result.path, ".backups");
            assert_eq!(result.keep_count, 10);
        }

        #[test]
        fn test_convert_upgrade_config() {
            let config = UpgradeConfig::default();

            let result = convert_upgrade_config(&config);

            assert!(result.auto_changeset);
            assert_eq!(result.changeset_bump, "patch");
        }

        #[test]
        fn test_convert_changelog_config_keep_a_changelog() {
            let config = ChangelogConfig {
                enabled: true,
                format: ChangelogFormat::KeepAChangelog,
                include_commit_links: true,
                repository_url: Some("https://github.com/org/repo".to_string()),
                monorepo_mode: MonorepoMode::PerPackage,
                ..Default::default()
            };

            let result = convert_changelog_config(&config);

            assert!(result.enabled);
            assert_eq!(result.format, "keep-a-changelog");
            assert!(result.include_commit_links);
            assert_eq!(result.repository_url, Some("https://github.com/org/repo".to_string()));
            assert_eq!(result.monorepo_mode, "per-package");
        }

        #[test]
        fn test_convert_changelog_config_conventional() {
            let config = ChangelogConfig {
                format: ChangelogFormat::Conventional,
                monorepo_mode: MonorepoMode::Root,
                ..Default::default()
            };

            let result = convert_changelog_config(&config);

            assert_eq!(result.format, "conventional-commits");
            assert_eq!(result.monorepo_mode, "root");
        }

        #[test]
        fn test_convert_changelog_config_custom() {
            let config = ChangelogConfig {
                format: ChangelogFormat::Custom,
                monorepo_mode: MonorepoMode::Both,
                ..Default::default()
            };

            let result = convert_changelog_config(&config);

            assert_eq!(result.format, "custom");
            assert_eq!(result.monorepo_mode, "both");
        }

        #[test]
        fn test_convert_health_score_weights() {
            let config = HealthScoreWeightsConfig {
                critical_weight: 15.0,
                warning_weight: 5.0,
                info_weight: 1.0,
                security_multiplier: 1.5,
                breaking_changes_multiplier: 1.3,
                dependencies_multiplier: 1.2,
                version_consistency_multiplier: 1.0,
                upgrades_multiplier: 0.8,
                other_multiplier: 1.0,
            };

            let result = convert_health_score_weights(&config);

            // Check that the weights are normalized
            let total = result.upgrades_weight
                + result.dependencies_weight
                + result.version_consistency_weight
                + result.breaking_changes_weight;
            // Should be approximately 1.0 (allow for floating point)
            assert!((total - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_convert_git_config() {
            let config = GitConfig::default();

            let result = convert_git_config(&config);

            // GitConfig from pkg crate has different fields (commit templates)
            // We provide defaults for the NAPI expected fields
            assert_eq!(result.branch_base, "main");
            assert!(result.detect_affected_packages);
        }

        #[test]
        fn test_convert_execute_config() {
            let config = ExecuteConfig {
                timeout_secs: 600,
                per_package_timeout_secs: 120,
                max_parallel: 16,
            };

            let result = convert_execute_config(&config);

            assert_eq!(result.timeout_secs, 600);
            assert_eq!(result.per_package_timeout_secs, 120);
            assert_eq!(result.max_parallel, 16);
        }

        #[test]
        fn test_convert_execute_config_defaults() {
            let config = ExecuteConfig::default();

            let result = convert_execute_config(&config);

            assert_eq!(result.timeout_secs, 300);
            assert_eq!(result.per_package_timeout_secs, 60);
            assert_eq!(result.max_parallel, 8);
        }
    }

    // ========================================================================
    // ConfigShowParams Builder Tests
    // ========================================================================

    mod params_builder_tests {
        use super::*;

        #[test]
        fn test_config_show_params_new() {
            let params = ConfigShowParams::new(".".to_string());

            assert_eq!(params.root, ".");
            assert!(params.config_path.is_none());
        }

        #[test]
        fn test_config_show_params_with_config() {
            let params = ConfigShowParams::with_config(
                "/path/to/workspace".to_string(),
                "custom/repo.config.json".to_string(),
            );

            assert_eq!(params.root, "/path/to/workspace");
            assert_eq!(params.config_path, Some("custom/repo.config.json".to_string()));
        }
    }

    // ========================================================================
    // Integration-like Tests (Config File Discovery)
    // ========================================================================

    mod discovery_tests {
        use super::*;

        #[test]
        fn test_params_with_various_roots() {
            // Test relative path
            let params_relative = ConfigShowParams::new(".".to_string());
            assert_eq!(params_relative.root, ".");

            // Test absolute path format
            let params_absolute = ConfigShowParams::new("/abs/path".to_string());
            assert_eq!(params_absolute.root, "/abs/path");

            // Test with trailing slash
            let params_trailing = ConfigShowParams::new("./my/path/".to_string());
            assert_eq!(params_trailing.root, "./my/path/");
        }

        #[test]
        fn test_params_preserves_config_path() {
            let params = ConfigShowParams {
                root: ".".to_string(),
                config_path: Some("custom.toml".to_string()),
            };

            assert_eq!(params.config_path, Some("custom.toml".to_string()));
        }

        #[test]
        fn test_params_none_config_path() {
            let params = ConfigShowParams { root: ".".to_string(), config_path: None };

            assert!(params.config_path.is_none());
        }
    }

    // ========================================================================
    // Default Values Tests
    // ========================================================================

    mod default_values_tests {
        use super::*;

        #[test]
        fn test_default_changeset_config() {
            let config = ChangesetConfig::default();
            let result = convert_changeset_config(&config);

            assert_eq!(result.path, ".changesets");
            assert_eq!(result.history_path, ".changesets/history");
        }

        #[test]
        fn test_default_version_config() {
            let config = VersionConfig::default();
            let result = convert_version_config(&config);

            assert_eq!(result.strategy, "independent");
            assert_eq!(result.default_bump, "patch");
        }

        #[test]
        fn test_default_dependency_config() {
            let config = DependencyConfig::default();
            let result = convert_dependency_config(&config);

            // Note: Actual pkg crate defaults may differ from original spec
            // Just verify the conversion works correctly
            assert!(result.propagate_dependencies);
            // The actual defaults are checked, not the expected values from docs
            assert_eq!(result.max_depth, u32::try_from(config.max_depth).unwrap_or(u32::MAX));
        }

        #[test]
        fn test_default_audit_config() {
            let config = AuditConfig::default();

            assert!(config.enabled);
            // Note: Actual default from pkg crate is "warning", not "low"
            assert_eq!(config.min_severity, "warning");
        }

        #[test]
        fn test_default_audit_sections_config() {
            let config = AuditSectionsConfig::default();

            assert!(config.upgrades);
            assert!(config.dependencies);
            assert!(config.version_consistency);
            assert!(config.breaking_changes);
        }

        #[test]
        fn test_default_execute_config() {
            let config = ExecuteConfig::default();
            let result = convert_execute_config(&config);

            assert_eq!(result.timeout_secs, 300);
            assert_eq!(result.per_package_timeout_secs, 60);
            assert_eq!(result.max_parallel, 8);
        }

        #[test]
        fn test_default_changelog_config() {
            let config = ChangelogConfig::default();
            let result = convert_changelog_config(&config);

            assert!(result.enabled);
            assert_eq!(result.format, "keep-a-changelog");
            assert!(result.include_commit_links);
            assert!(result.conventional); // conventional.enabled is true by default
            assert_eq!(result.monorepo_mode, "per-package");
        }
    }
}
