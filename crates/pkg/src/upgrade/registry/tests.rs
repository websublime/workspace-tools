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
            if let Some(dep_msg) = deprecated {
                if *version == latest {
                    version_obj.insert("deprecated".to_string(), serde_json::json!(dep_msg));
                }
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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

        let mut config = RegistryConfig::default();
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
        let config = RegistryConfig::default();
        let client = RegistryClient::new(&PathBuf::from("."), config)
            .await
            .expect("Failed to create client");

        // Major upgrade
        let upgrade =
            client.compare_versions("1.2.3", "2.0.0").expect("Failed to compare versions");
        assert_eq!(upgrade, UpgradeType::Major);
        assert!(upgrade.is_breaking());
        assert!(!upgrade.is_safe());
        assert_eq!(upgrade.priority(), 3);

        // Minor upgrade
        let upgrade =
            client.compare_versions("1.2.3", "1.3.0").expect("Failed to compare versions");
        assert_eq!(upgrade, UpgradeType::Minor);
        assert!(!upgrade.is_breaking());
        assert!(upgrade.is_safe());
        assert_eq!(upgrade.priority(), 2);

        // Patch upgrade
        let upgrade =
            client.compare_versions("1.2.3", "1.2.4").expect("Failed to compare versions");
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

        let mut config = RegistryConfig::default();
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

    #[test]
    fn test_compare_versions_major() {
        let config = RegistryConfig::default();
        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let result = client.compare_versions("1.2.3", "2.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UpgradeType::Major);
    }

    #[test]
    fn test_compare_versions_minor() {
        let config = RegistryConfig::default();
        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let result = client.compare_versions("1.2.3", "1.3.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UpgradeType::Minor);
    }

    #[test]
    fn test_compare_versions_patch() {
        let config = RegistryConfig::default();
        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let result = client.compare_versions("1.2.3", "1.2.4");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UpgradeType::Patch);
    }

    #[test]
    fn test_compare_versions_invalid_current() {
        let config = RegistryConfig::default();
        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let result = client.compare_versions("invalid", "1.2.3");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_versions_invalid_latest() {
        let config = RegistryConfig::default();
        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let result = client.compare_versions("1.2.3", "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_versions_not_upgrade() {
        let config = RegistryConfig::default();
        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let result = client.compare_versions("2.0.0", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_registry_url_default() {
        let config = RegistryConfig::default();
        let client = futures::executor::block_on(RegistryClient::new(
            std::path::Path::new("."),
            config.clone(),
        ))
        .expect("Failed to create client");

        let url = client.resolve_registry_url("lodash");
        assert_eq!(url, config.default_registry);
    }

    #[test]
    fn test_resolve_registry_url_scoped() {
        let mut config = RegistryConfig::default();
        config.scoped_registries.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let url = client.resolve_registry_url("@myorg/utils");
        assert_eq!(url, "https://npm.myorg.com");
    }

    #[test]
    fn test_resolve_registry_url_scoped_fallback() {
        let config = RegistryConfig::default();
        let client = futures::executor::block_on(RegistryClient::new(
            std::path::Path::new("."),
            config.clone(),
        ))
        .expect("Failed to create client");

        // Scoped package but no scoped registry configured
        let url = client.resolve_registry_url("@unknown/package");
        assert_eq!(url, config.default_registry);
    }

    #[test]
    fn test_resolve_auth_token_exact_match() {
        let mut config = RegistryConfig::default();
        config.auth_tokens.insert("https://npm.myorg.com".to_string(), "test-token".to_string());

        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let token = client.resolve_auth_token("https://npm.myorg.com");
        assert_eq!(token, Some("test-token".to_string()));
    }

    #[test]
    fn test_resolve_auth_token_trailing_slash() {
        let mut config = RegistryConfig::default();
        config.auth_tokens.insert("https://npm.myorg.com".to_string(), "test-token".to_string());

        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let token = client.resolve_auth_token("https://npm.myorg.com/");
        assert_eq!(token, Some("test-token".to_string()));
    }

    #[test]
    fn test_resolve_auth_token_no_match() {
        let config = RegistryConfig::default();
        let client =
            futures::executor::block_on(RegistryClient::new(std::path::Path::new("."), config))
                .expect("Failed to create client");

        let token = client.resolve_auth_token("https://unknown.com");
        assert_eq!(token, None);
    }
}
