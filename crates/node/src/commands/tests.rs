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
                        "createdAt": "2025-01-20T10:00:00Z",
                        "updatedAt": "2025-01-20T10:00:00Z"
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
                        "createdAt": "2025-01-20T10:00:00Z",
                        "updatedAt": "2025-01-20T10:00:00Z"
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
                        "createdAt": "2025-01-20T10:00:00Z",
                        "updatedAt": "2025-01-20T12:00:00Z"
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
                        "createdAt": "2025-01-20T10:00:00Z",
                        "updatedAt": "2025-01-20T10:00:00Z"
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
