//! Integration tests for registry client using mock HTTP server.
//!
//! **What**: Comprehensive tests for RegistryClient functionality using mockito
//! to simulate NPM registry responses.
//!
//! **How**: Sets up mock HTTP endpoints that return realistic NPM registry responses,
//! then validates client behavior for success cases, error cases, retries, and edge cases.
//!
//! **Why**: To ensure the registry client handles all scenarios correctly without
//! depending on external NPM registry availability or network conditions.

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
#[allow(clippy::panic)]
mod integration_tests {
    use crate::config::RegistryConfig;
    use crate::error::UpgradeError;
    use crate::upgrade::{RegistryClient, UpgradeType};
    use mockito::Server;
    use std::path::PathBuf;

    /// Helper to create a test RegistryConfig with .npmrc reading disabled.
    fn test_config() -> RegistryConfig {
        let mut config = RegistryConfig::default();
        config.read_npmrc = false;
        config
    }

    /// Helper to create a mock registry response for a package.
    fn create_package_response(
        name: &str,
        versions: &[&str],
        latest: &str,
        deprecated: Option<&str>,
    ) -> serde_json::Value {
        let mut versions_map = serde_json::Map::new();

        for version in versions {
            let mut version_obj = serde_json::Map::new();
            if let Some(dep_msg) = deprecated
                && *version == latest
            {
                version_obj.insert("deprecated".to_string(), serde_json::json!(dep_msg));
            }
            versions_map.insert(version.to_string(), serde_json::json!(version_obj));
        }

        let mut dist_tags = serde_json::Map::new();
        dist_tags.insert("latest".to_string(), serde_json::json!(latest));

        let mut time = serde_json::Map::new();
        time.insert("created".to_string(), serde_json::json!("2020-01-01T00:00:00.000Z"));
        time.insert("modified".to_string(), serde_json::json!("2024-01-01T00:00:00.000Z"));
        for version in versions {
            time.insert(version.to_string(), serde_json::json!("2023-01-01T00:00:00.000Z"));
        }

        let mut repo = serde_json::Map::new();
        repo.insert("type".to_string(), serde_json::json!("git"));
        repo.insert("url".to_string(), serde_json::json!("https://github.com/test/repo.git"));

        serde_json::json!({
            "name": name,
            "versions": versions_map,
            "dist-tags": dist_tags,
            "time": time,
            "repository": repo,
        })
    }

    #[tokio::test]
    async fn test_get_package_info_success() {
        let mut server = Server::new_async().await;

        let response = create_package_response(
            "express",
            &["4.17.0", "4.17.1", "4.18.0", "4.18.1"],
            "4.18.1",
            None,
        );

        let mock = server
            .mock("GET", "/express")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let metadata =
            client.get_package_info("express").await.expect("Failed to get package info");

        mock.assert_async().await;

        assert_eq!(metadata.name, "express");
        assert_eq!(metadata.latest, "4.18.1");
        assert_eq!(metadata.versions.len(), 4);
        assert!(metadata.versions.contains(&"4.18.1".to_string()));
        assert!(!metadata.is_deprecated());
        assert!(metadata.repository.is_some());
    }

    #[tokio::test]
    async fn test_get_package_info_deprecated() {
        let mut server = Server::new_async().await;

        let response = create_package_response(
            "left-pad",
            &["1.0.0", "1.1.0"],
            "1.1.0",
            Some("This package is deprecated. Use leftpad instead."),
        );

        let mock = server
            .mock("GET", "/left-pad")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let metadata =
            client.get_package_info("left-pad").await.expect("Failed to get package info");

        mock.assert_async().await;

        assert_eq!(metadata.name, "left-pad");
        assert!(metadata.is_deprecated());
        assert_eq!(
            metadata.deprecation_message(),
            Some("This package is deprecated. Use leftpad instead.")
        );
    }

    #[tokio::test]
    async fn test_get_package_info_not_found() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/nonexistent-package")
            .with_status(404)
            .with_body("Not found")
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let result = client.get_package_info("nonexistent-package").await;

        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UpgradeError::PackageNotFound { package, .. } => {
                assert_eq!(package, "nonexistent-package");
            }
            e => panic!("Expected PackageNotFound, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_package_info_authentication_failed() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/@private/package")
            .with_status(401)
            .with_body("Unauthorized")
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let result = client.get_package_info("@private/package").await;

        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UpgradeError::AuthenticationFailed { .. } => {}
            e => panic!("Expected AuthenticationFailed, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_package_info_with_authentication() {
        let mut server = Server::new_async().await;

        let response = create_package_response("@myorg/utils", &["1.0.0"], "1.0.0", None);

        let mock = server
            .mock("GET", "/@myorg/utils")
            .match_header("authorization", "Bearer test-token-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();
        config.auth_tokens.insert(server.url(), "test-token-123".to_string());

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let metadata =
            client.get_package_info("@myorg/utils").await.expect("Failed to get package info");

        mock.assert_async().await;

        assert_eq!(metadata.name, "@myorg/utils");
        assert_eq!(metadata.latest, "1.0.0");
    }

    #[tokio::test]
    async fn test_get_package_info_scoped_registry() {
        let mut server = Server::new_async().await;

        let response = create_package_response("@myorg/package", &["2.0.0"], "2.0.0", None);

        let mock = server
            .mock("GET", "/@myorg/package")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let mut config = test_config();
        config.scoped_registries.insert("myorg".to_string(), server.url());

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let metadata =
            client.get_package_info("@myorg/package").await.expect("Failed to get package info");

        mock.assert_async().await;

        assert_eq!(metadata.name, "@myorg/package");
    }

    #[tokio::test]
    async fn test_get_package_info_server_error() {
        let mut server = Server::new_async().await;

        // Test 500 internal server error handling
        // The retry middleware will retry on 500 errors, so we expect multiple requests
        let mock = server
            .mock("GET", "/error-package")
            .with_status(500)
            .with_body("Internal Server Error")
            .expect_at_least(1) // Allow retries
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();
        config.retry_attempts = 2; // Limit retries for faster test

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let result = client.get_package_info("error-package").await;

        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UpgradeError::RegistryError { package, .. } => {
                assert_eq!(package, "error-package");
            }
            e => panic!("Expected RegistryError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_package_info_invalid_json() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/broken-package")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{ invalid json }")
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let result = client.get_package_info("broken-package").await;

        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UpgradeError::InvalidResponse { package, .. } => {
                assert_eq!(package, "broken-package");
            }
            e => panic!("Expected InvalidResponse, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_package_info_missing_latest_tag() {
        let mut server = Server::new_async().await;

        let response = serde_json::json!({
            "name": "no-latest",
            "versions": {
                "1.0.0": {}
            },
            "dist-tags": {},
            "time": {}
        });

        let mock = server
            .mock("GET", "/no-latest")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let result = client.get_package_info("no-latest").await;

        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UpgradeError::InvalidResponse { package, reason } => {
                assert_eq!(package, "no-latest");
                assert!(reason.contains("latest"));
            }
            e => panic!("Expected InvalidResponse, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_latest_version() {
        let mut server = Server::new_async().await;

        let response =
            create_package_response("react", &["17.0.0", "18.0.0", "18.2.0"], "18.2.0", None);

        let mock = server
            .mock("GET", "/react")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let latest =
            client.get_latest_version("react").await.expect("Failed to get latest version");

        mock.assert_async().await;

        assert_eq!(latest, "18.2.0");
    }

    #[tokio::test]
    async fn test_retry_on_transient_failure() {
        let mut server = Server::new_async().await;

        let response = create_package_response("retry-test", &["1.0.0"], "1.0.0", None);

        // First request fails with 500
        let mock1 = server
            .mock("GET", "/retry-test")
            .with_status(500)
            .with_body("Internal Server Error")
            .expect(1)
            .create_async()
            .await;

        // Second request succeeds
        let mock2 = server
            .mock("GET", "/retry-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .expect(1)
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();
        config.retry_attempts = 3;
        config.retry_delay_ms = 100;

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let metadata = client
            .get_package_info("retry-test")
            .await
            .expect("Failed to get package info after retry");

        mock1.assert_async().await;
        mock2.assert_async().await;

        assert_eq!(metadata.name, "retry-test");
        assert_eq!(metadata.latest, "1.0.0");
    }

    #[tokio::test]
    async fn test_compare_versions_all_types() {
        let config = test_config();
        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        // Major upgrade
        let upgrade = client
            .compare_versions("test-package", "1.2.3", "2.0.0")
            .expect("Failed to compare versions");
        assert_eq!(upgrade, UpgradeType::Major);
        assert!(upgrade.is_breaking());
        assert!(!upgrade.is_safe());
        assert_eq!(upgrade.priority(), 3);

        // Minor upgrade
        let upgrade = client
            .compare_versions("test-package", "1.2.3", "1.3.0")
            .expect("Failed to compare versions");
        assert_eq!(upgrade, UpgradeType::Minor);
        assert!(!upgrade.is_breaking());
        assert!(upgrade.is_safe());
        assert_eq!(upgrade.priority(), 2);

        // Patch upgrade
        let upgrade = client
            .compare_versions("test-package", "1.2.3", "1.2.4")
            .expect("Failed to compare versions");
        assert_eq!(upgrade, UpgradeType::Patch);
        assert!(!upgrade.is_breaking());
        assert!(upgrade.is_safe());
        assert_eq!(upgrade.priority(), 1);
    }

    #[tokio::test]
    async fn test_package_metadata_helpers() {
        let mut server = Server::new_async().await;

        let response = create_package_response("test-pkg", &["1.0.0", "2.0.0"], "2.0.0", None);

        let mock = server
            .mock("GET", "/test-pkg")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await;

        let mut config = test_config();
        config.default_registry = server.url();

        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        let metadata =
            client.get_package_info("test-pkg").await.expect("Failed to get package info");

        mock.assert_async().await;

        assert!(metadata.created_at().is_some());
        assert!(metadata.modified_at().is_some());
        assert!(metadata.version_published_at("1.0.0").is_some());
        assert!(metadata.version_published_at("nonexistent").is_none());
    }

    #[test]
    fn test_upgrade_type_display() {
        assert_eq!(UpgradeType::Major.to_string(), "major");
        assert_eq!(UpgradeType::Minor.to_string(), "minor");
        assert_eq!(UpgradeType::Patch.to_string(), "patch");
    }

    #[test]
    fn test_upgrade_type_as_str() {
        assert_eq!(UpgradeType::Major.as_str(), "major");
        assert_eq!(UpgradeType::Minor.as_str(), "minor");
        assert_eq!(UpgradeType::Patch.as_str(), "patch");
    }

    #[test]
    fn test_upgrade_type_priority_ordering() {
        assert!(UpgradeType::Major.priority() > UpgradeType::Minor.priority());
        assert!(UpgradeType::Minor.priority() > UpgradeType::Patch.priority());
    }

    #[tokio::test]
    async fn test_compare_versions_major() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let result = client.compare_versions("test-package", "1.2.3", "2.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UpgradeType::Major);
    }

    #[tokio::test]
    async fn test_compare_versions_minor() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let result = client.compare_versions("test-package", "1.2.3", "1.3.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UpgradeType::Minor);
    }

    #[tokio::test]
    async fn test_compare_versions_patch() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let result = client.compare_versions("test-package", "1.2.3", "1.2.4");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UpgradeType::Patch);
    }

    #[tokio::test]
    async fn test_compare_versions_invalid_current() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let result = client.compare_versions("test-package", "invalid", "1.2.3");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compare_versions_invalid_latest() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let result = client.compare_versions("test-package", "1.2.3", "invalid");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compare_versions_not_upgrade() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let result = client.compare_versions("test-package", "2.0.0", "1.0.0");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_messages_contain_package_name() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        // Test invalid current version includes package name
        let result = client.compare_versions("my-package", "invalid-version", "1.2.3");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("my-package"),
            "Error message should contain package name: {}",
            err_msg
        );

        // Test invalid latest version includes package name
        let result = client.compare_versions("another-package", "1.2.3", "not-a-version");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("another-package"),
            "Error message should contain package name: {}",
            err_msg
        );

        // Test version comparison failed includes package name
        let result = client.compare_versions("test-pkg", "2.0.0", "1.0.0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("test-pkg"),
            "Error message should contain package name: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_resolve_registry_url_default() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config.clone())
            .await
            .expect("Failed to create client");

        let url = client.resolve_registry_url("lodash");
        assert_eq!(url, config.default_registry);
    }

    #[tokio::test]
    async fn test_resolve_registry_url_scoped() {
        let mut config = test_config();
        config.scoped_registries.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let url = client.resolve_registry_url("@myorg/package");
        assert_eq!(url, "https://npm.myorg.com");
    }

    #[tokio::test]
    async fn test_resolve_registry_url_scoped_fallback() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config.clone())
            .await
            .expect("Failed to create client");

        // Scoped package but no scoped registry configured
        let url = client.resolve_registry_url("@unknown/package");
        assert_eq!(url, config.default_registry);
    }

    #[tokio::test]
    async fn test_resolve_auth_token_exact_match() {
        let mut config = test_config();
        config.auth_tokens.insert("https://npm.myorg.com".to_string(), "test-token".to_string());

        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let token = client.resolve_auth_token("https://npm.myorg.com");
        assert_eq!(token, Some("test-token".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_auth_token_trailing_slash() {
        let mut config = test_config();
        config.auth_tokens.insert("https://npm.myorg.com".to_string(), "test-token".to_string());

        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let token = client.resolve_auth_token("https://npm.myorg.com/");
        assert_eq!(token, Some("test-token".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_auth_token_no_match() {
        let config = test_config();
        let client = RegistryClient::new(std::path::Path::new("."), config)
            .await
            .expect("Failed to create client");

        let token = client.resolve_auth_token("https://unknown.com");
        assert_eq!(token, None);
    }
}

// ============================================================================
// Npmrc Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::field_reassign_with_default)]
mod npmrc_tests {
    use crate::upgrade::registry::npmrc::NpmrcConfig;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use sublime_standard_tools::filesystem::AsyncFileSystem;

    /// Mock filesystem for testing .npmrc parsing without actual file I/O.
    struct MockFileSystem {
        files: HashMap<PathBuf, String>,
    }

    impl MockFileSystem {
        fn new() -> Self {
            Self { files: HashMap::new() }
        }

        fn add_file(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
            self.files.insert(path.into(), content.into());
        }
    }

    #[async_trait::async_trait]
    impl AsyncFileSystem for MockFileSystem {
        async fn read_file(
            &self,
            path: &Path,
        ) -> Result<Vec<u8>, sublime_standard_tools::error::Error> {
            self.files.get(path).map(|s| s.as_bytes().to_vec()).ok_or_else(|| {
                sublime_standard_tools::error::Error::FileSystem(
                    sublime_standard_tools::error::FileSystemError::NotFound {
                        path: path.to_path_buf(),
                    },
                )
            })
        }

        async fn write_file(
            &self,
            _path: &Path,
            _data: &[u8],
        ) -> Result<(), sublime_standard_tools::error::Error> {
            Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: "Write not needed for tests".to_string(),
                },
            ))
        }

        async fn read_file_string(
            &self,
            path: &Path,
        ) -> Result<String, sublime_standard_tools::error::Error> {
            self.files.get(path).cloned().ok_or_else(|| {
                sublime_standard_tools::error::Error::FileSystem(
                    sublime_standard_tools::error::FileSystemError::NotFound {
                        path: path.to_path_buf(),
                    },
                )
            })
        }

        async fn write_file_string(
            &self,
            _path: &Path,
            _content: &str,
        ) -> Result<(), sublime_standard_tools::error::Error> {
            Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: "Write not needed for tests".to_string(),
                },
            ))
        }

        async fn create_dir_all(
            &self,
            _path: &Path,
        ) -> Result<(), sublime_standard_tools::error::Error> {
            Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: "Create dir not needed for tests".to_string(),
                },
            ))
        }

        async fn remove(&self, _path: &Path) -> Result<(), sublime_standard_tools::error::Error> {
            Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: "Remove not needed for tests".to_string(),
                },
            ))
        }

        async fn exists(&self, path: &Path) -> bool {
            self.files.contains_key(path)
        }

        async fn read_dir(
            &self,
            _path: &Path,
        ) -> Result<Vec<PathBuf>, sublime_standard_tools::error::Error> {
            Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: "Read dir not needed for tests".to_string(),
                },
            ))
        }

        async fn walk_dir(
            &self,
            _path: &Path,
        ) -> Result<Vec<PathBuf>, sublime_standard_tools::error::Error> {
            Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: "Walk dir not needed for tests".to_string(),
                },
            ))
        }

        async fn metadata(
            &self,
            _path: &Path,
        ) -> Result<std::fs::Metadata, sublime_standard_tools::error::Error> {
            Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: "Metadata not needed for tests".to_string(),
                },
            ))
        }
    }

    #[tokio::test]
    async fn test_parse_empty_npmrc() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/workspace/.npmrc", "");

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse empty file");

        assert!(config.registry.is_none());
        assert!(config.scoped_registries.is_empty());
        assert!(config.auth_tokens.is_empty());
        assert!(config.other.is_empty());
    }

    #[tokio::test]
    async fn test_parse_default_registry() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/workspace/.npmrc", "registry=https://registry.npmjs.org\n");

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse registry");

        assert_eq!(config.registry, Some("https://registry.npmjs.org".to_string()));
    }

    #[tokio::test]
    async fn test_parse_scoped_registry() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/workspace/.npmrc", "@myorg:registry=https://npm.myorg.com\n");

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse scoped registry");

        assert_eq!(
            config.scoped_registries.get("myorg"),
            Some(&"https://npm.myorg.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_multiple_scoped_registries() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            r#"
@myorg:registry=https://npm.myorg.com
@internal:registry=https://registry.internal.corp
@external:registry=https://npm.external.com
"#,
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse multiple scoped registries");

        assert_eq!(config.scoped_registries.len(), 3);
        assert_eq!(
            config.scoped_registries.get("myorg"),
            Some(&"https://npm.myorg.com".to_string())
        );
        assert_eq!(
            config.scoped_registries.get("internal"),
            Some(&"https://registry.internal.corp".to_string())
        );
        assert_eq!(
            config.scoped_registries.get("external"),
            Some(&"https://npm.external.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_auth_token() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/workspace/.npmrc", "//npm.myorg.com/:_authToken=npm_AbCdEf123456\n");

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse auth token");

        assert_eq!(config.auth_tokens.get("npm.myorg.com"), Some(&"npm_AbCdEf123456".to_string()));
    }

    #[tokio::test]
    async fn test_parse_auth_token_various_formats() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            r#"
//npm.myorg.com/:_authToken=token1
registry.npmjs.org:_authToken=token2
npm.internal.com/:_authToken=token3
"#,
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse various auth token formats");

        assert_eq!(config.auth_tokens.get("npm.myorg.com"), Some(&"token1".to_string()));
        assert_eq!(config.auth_tokens.get("registry.npmjs.org"), Some(&"token2".to_string()));
        assert_eq!(config.auth_tokens.get("npm.internal.com"), Some(&"token3".to_string()));
    }

    #[tokio::test]
    async fn test_parse_auth_base64() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            "//npm.myorg.com/:_auth=TUlHUkFNTzpjbVZtZEd0dU9qQXhPakF3TURBd01EQXdNREE2U1ZOVVpIcHFNR1V3UkRCbk5uWlJUMDR4YUV4aVZIVlpWbkl3\n",
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse _auth token");

        assert_eq!(
            config.auth_tokens.get("npm.myorg.com"),
            Some(&"TUlHUkFNTzpjbVZtZEd0dU9qQXhPakF3TURBd01EQXdNREE2U1ZOVVpIcHFNR1V3UkRCbk5uWlJUMDR4YUV4aVZIVlpWbkl3".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_auth_and_authtoken_mixed() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            r#"
//npm.myorg.com/:_auth=base64token
//registry.npmjs.org/:_authToken=bearer_token
//internal.corp.com/:_auth=another_base64
"#,
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse mixed _auth and _authToken formats");

        assert_eq!(config.auth_tokens.get("npm.myorg.com"), Some(&"base64token".to_string()));
        assert_eq!(config.auth_tokens.get("registry.npmjs.org"), Some(&"bearer_token".to_string()));
        assert_eq!(
            config.auth_tokens.get("internal.corp.com"),
            Some(&"another_base64".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_comments() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            r#"
# This is a comment
registry=https://registry.npmjs.org

// Another comment style
@myorg:registry=https://npm.myorg.com
"#,
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse with comments");

        assert_eq!(config.registry, Some("https://registry.npmjs.org".to_string()));
        assert_eq!(
            config.scoped_registries.get("myorg"),
            Some(&"https://npm.myorg.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_inline_comments() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            "registry=https://registry.npmjs.org # Default registry\n",
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse with inline comments");

        assert_eq!(config.registry, Some("https://registry.npmjs.org".to_string()));
    }

    #[tokio::test]
    async fn test_parse_with_whitespace() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            r#"
  registry  =  https://registry.npmjs.org

  @myorg:registry  =  https://npm.myorg.com
"#,
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse with whitespace");

        assert_eq!(config.registry, Some("https://registry.npmjs.org".to_string()));
        assert_eq!(
            config.scoped_registries.get("myorg"),
            Some(&"https://npm.myorg.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_other_properties() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            r#"
save-exact=true
legacy-peer-deps=true
package-lock=false
"#,
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse other properties");

        assert_eq!(config.other.get("save-exact"), Some(&"true".to_string()));
        assert_eq!(config.other.get("legacy-peer-deps"), Some(&"true".to_string()));
        assert_eq!(config.other.get("package-lock"), Some(&"false".to_string()));
    }

    #[tokio::test]
    async fn test_environment_variable_substitution() {
        unsafe {
            std::env::set_var("TEST_NPM_TOKEN", "secret_token_123");
        }

        let mut fs = MockFileSystem::new();
        fs.add_file("/workspace/.npmrc", "//npm.myorg.com/:_authToken=${TEST_NPM_TOKEN}\n");

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse with env var substitution");

        assert_eq!(config.auth_tokens.get("npm.myorg.com"), Some(&"secret_token_123".to_string()));

        unsafe {
            std::env::remove_var("TEST_NPM_TOKEN");
        }
    }

    #[tokio::test]
    async fn test_environment_variable_not_set() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/workspace/.npmrc", "//npm.myorg.com/:_authToken=${NONEXISTENT_VAR}\n");

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse even if env var not set");

        // Should keep the placeholder if env var not set
        assert_eq!(
            config.auth_tokens.get("npm.myorg.com"),
            Some(&"${NONEXISTENT_VAR}".to_string())
        );
    }

    #[tokio::test]
    async fn test_resolve_registry_scoped_package() {
        let mut config = NpmrcConfig::default();
        config.registry = Some("https://registry.npmjs.org".to_string());
        config.scoped_registries.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        assert_eq!(config.resolve_registry("@myorg/package"), Some("https://npm.myorg.com"));
    }

    #[tokio::test]
    async fn test_resolve_registry_unscoped_package() {
        let mut config = NpmrcConfig::default();
        config.registry = Some("https://registry.npmjs.org".to_string());
        config.scoped_registries.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        assert_eq!(config.resolve_registry("lodash"), Some("https://registry.npmjs.org"));
    }

    #[tokio::test]
    async fn test_resolve_registry_unknown_scope() {
        let mut config = NpmrcConfig::default();
        config.registry = Some("https://registry.npmjs.org".to_string());
        config.scoped_registries.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        // Unknown scope falls back to default registry
        assert_eq!(config.resolve_registry("@unknown/package"), Some("https://registry.npmjs.org"));
    }

    #[tokio::test]
    async fn test_resolve_registry_no_default() {
        let mut config = NpmrcConfig::default();
        config.scoped_registries.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        assert_eq!(config.resolve_registry("lodash"), None);
    }

    #[tokio::test]
    async fn test_get_auth_token_exact_match() {
        let mut config = NpmrcConfig::default();
        config.auth_tokens.insert("npm.myorg.com".to_string(), "token123".to_string());

        assert_eq!(config.get_auth_token("npm.myorg.com"), Some("token123"));
    }

    #[tokio::test]
    async fn test_get_auth_token_with_protocol() {
        let mut config = NpmrcConfig::default();
        config.auth_tokens.insert("npm.myorg.com".to_string(), "token123".to_string());

        assert_eq!(config.get_auth_token("https://npm.myorg.com"), Some("token123"));
    }

    #[tokio::test]
    async fn test_get_auth_token_with_trailing_slash() {
        let mut config = NpmrcConfig::default();
        config.auth_tokens.insert("npm.myorg.com".to_string(), "token123".to_string());

        assert_eq!(config.get_auth_token("https://npm.myorg.com/"), Some("token123"));
    }

    #[tokio::test]
    async fn test_get_auth_token_stored_with_protocol() {
        let mut config = NpmrcConfig::default();
        config.auth_tokens.insert("//npm.myorg.com".to_string(), "token123".to_string());

        assert_eq!(config.get_auth_token("https://npm.myorg.com"), Some("token123"));
    }

    #[tokio::test]
    async fn test_get_auth_token_not_found() {
        let mut config = NpmrcConfig::default();
        config.auth_tokens.insert("npm.myorg.com".to_string(), "token123".to_string());

        assert_eq!(config.get_auth_token("npm.other.com"), None);
    }

    #[tokio::test]
    async fn test_merge_configs() {
        let mut base = NpmrcConfig::default();
        base.registry = Some("https://registry.npmjs.org".to_string());
        base.scoped_registries.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        let mut override_config = NpmrcConfig::default();
        override_config.registry = Some("https://custom.registry.com".to_string());
        override_config
            .scoped_registries
            .insert("internal".to_string(), "https://npm.internal.com".to_string());
        override_config.auth_tokens.insert("npm.internal.com".to_string(), "token123".to_string());

        base.merge_with(override_config);

        // Registry should be overridden
        assert_eq!(base.registry, Some("https://custom.registry.com".to_string()));

        // Scoped registries should be merged
        assert_eq!(base.scoped_registries.len(), 2);
        assert_eq!(base.scoped_registries.get("myorg"), Some(&"https://npm.myorg.com".to_string()));
        assert_eq!(
            base.scoped_registries.get("internal"),
            Some(&"https://npm.internal.com".to_string())
        );

        // Auth tokens should be added
        assert_eq!(base.auth_tokens.get("npm.internal.com"), Some(&"token123".to_string()));
    }

    #[tokio::test]
    async fn test_complete_npmrc_example() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/workspace/.npmrc",
            r#"
# Default npm registry
registry=https://registry.npmjs.org

# Scoped registries
@myorg:registry=https://npm.myorg.com
@internal:registry=https://registry.internal.corp

# Authentication tokens
//npm.myorg.com/:_authToken=npm_secret_token_123
//registry.internal.corp/:_authToken=internal_token_456

# Other settings
save-exact=true
legacy-peer-deps=false
"#,
        );

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should parse complete example");

        // Verify default registry
        assert_eq!(config.registry, Some("https://registry.npmjs.org".to_string()));

        // Verify scoped registries
        assert_eq!(config.scoped_registries.len(), 2);
        assert_eq!(
            config.scoped_registries.get("myorg"),
            Some(&"https://npm.myorg.com".to_string())
        );
        assert_eq!(
            config.scoped_registries.get("internal"),
            Some(&"https://registry.internal.corp".to_string())
        );

        // Verify auth tokens
        assert_eq!(config.auth_tokens.len(), 2);
        assert_eq!(
            config.auth_tokens.get("npm.myorg.com"),
            Some(&"npm_secret_token_123".to_string())
        );
        assert_eq!(
            config.auth_tokens.get("registry.internal.corp"),
            Some(&"internal_token_456".to_string())
        );

        // Verify other properties
        assert_eq!(config.other.get("save-exact"), Some(&"true".to_string()));
        assert_eq!(config.other.get("legacy-peer-deps"), Some(&"false".to_string()));

        // Test resolution
        assert_eq!(config.resolve_registry("@myorg/package"), Some("https://npm.myorg.com"));
        assert_eq!(config.resolve_registry("lodash"), Some("https://registry.npmjs.org"));
        assert_eq!(config.get_auth_token("https://npm.myorg.com"), Some("npm_secret_token_123"));
    }

    #[tokio::test]
    async fn test_no_npmrc_file() {
        let fs = MockFileSystem::new();

        let config = NpmrcConfig::from_workspace(Path::new("/workspace"), &fs)
            .await
            .expect("Should succeed with no .npmrc file");

        assert!(config.registry.is_none());
        assert!(config.scoped_registries.is_empty());
        assert!(config.auth_tokens.is_empty());
    }
}
