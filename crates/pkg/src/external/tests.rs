//! Comprehensive tests for the external module
//!
//! This module contains all tests for external service functionality including
//! package registry clients, npm clients, and external API interactions.

#![allow(clippy::unwrap_used)] // Tests may use unwrap for test failures per CLAUDE.md rules
#![allow(clippy::expect_used)] // Tests may use expect for test failures per CLAUDE.md rules
#![allow(clippy::panic)] // Tests may use panic for test failures per CLAUDE.md rules

mod package_registry_client_tests {
    use crate::external::package_registry_client::PackageRegistryClient;
    use crate::external::npm_client::{PackageRegistry, PackageRegistryClone};
    use crate::errors::PackageRegistryError;
    use async_trait::async_trait;

    /// Mock registry for testing that provides predictable responses
    #[derive(Debug, Clone)]
    struct MockRegistry {
        should_fail: bool,
        versions: Vec<String>,
    }

    impl MockRegistry {
        fn new(versions: Vec<String>) -> Self {
            Self {
                should_fail: false,
                versions,
            }
        }

        fn new_failing() -> Self {
            Self {
                should_fail: true,
                versions: vec![],
            }
        }
    }

    #[async_trait]
    impl PackageRegistry for MockRegistry {
        async fn get_all_versions(&self, _package_name: &str) -> Result<Vec<String>, PackageRegistryError> {
            if self.should_fail {
                Err(PackageRegistryError::NotFound { 
                    package_name: _package_name.to_string(), 
                    version: "mock-failure".to_string() 
                })
            } else {
                Ok(self.versions.clone())
            }
        }

        async fn get_latest_version(&self, _package_name: &str) -> Result<Option<String>, PackageRegistryError> {
            if self.should_fail {
                Err(PackageRegistryError::NotFound { 
                    package_name: _package_name.to_string(), 
                    version: "mock-failure".to_string() 
                })
            } else {
                Ok(self.versions.last().cloned())
            }
        }

        async fn get_package_info(&self, _package_name: &str, _version: &str) -> Result<serde_json::Value, PackageRegistryError> {
            if self.should_fail {
                Err(PackageRegistryError::NotFound { 
                    package_name: _package_name.to_string(), 
                    version: "mock-failure".to_string() 
                })
            } else {
                use serde_json::json;
                Ok(json!({
                    "name": _package_name,
                    "version": _version,
                    "description": "Mock package for testing"
                }))
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        async fn download_package(&self, _package_name: &str, _version: &str) -> Result<Vec<u8>, PackageRegistryError> {
            if self.should_fail {
                Err(PackageRegistryError::NotFound { 
                    package_name: _package_name.to_string(), 
                    version: "mock-failure".to_string() 
                })
            } else {
                Ok(b"mock package content".to_vec())
            }
        }

        async fn download_and_extract_package(
            &self,
            _package_name: &str,
            _version: &str,
            _destination: &std::path::Path,
        ) -> Result<(), PackageRegistryError> {
            if self.should_fail {
                Err(PackageRegistryError::NotFound { 
                    package_name: _package_name.to_string(), 
                    version: "mock-failure".to_string() 
                })
            } else {
                Ok(())
            }
        }
    }

    impl PackageRegistryClone for MockRegistry {
        fn clone_box(&self) -> Box<dyn PackageRegistryClone> {
            Box::new(self.clone())
        }
    }

    #[tokio::test]
    async fn test_get_package_versions_without_registry() {
        let client = PackageRegistryClient::new();
        let versions = client.get_package_versions("react").await.unwrap();
        assert!(versions.is_empty());
    }

    #[tokio::test]
    async fn test_get_package_versions_with_mock_registry() {
        let mut client = PackageRegistryClient::new();
        let mock_registry = MockRegistry::new(vec![
            "16.0.0".to_string(),
            "17.0.0".to_string(),
            "18.0.0".to_string(),
        ]);
        
        client.set_registry(Box::new(mock_registry));
        
        let versions = client.get_package_versions("react").await.unwrap();
        assert_eq!(versions.len(), 3);
        assert!(versions.contains(&"16.0.0".to_string()));
        assert!(versions.contains(&"17.0.0".to_string()));
        assert!(versions.contains(&"18.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_get_package_versions_registry_failure() {
        let mut client = PackageRegistryClient::new();
        let failing_registry = MockRegistry::new_failing();
        
        client.set_registry(Box::new(failing_registry));
        
        let result = client.get_package_versions("react").await;
        assert!(result.is_err());
        
        if let Err(PackageRegistryError::NotFound { package_name, version }) = result {
            assert_eq!(package_name, "react");
            assert_eq!(version, "mock-failure");
        } else {
            panic!("Expected NotFound error, got: {:?}", result);
        }
    }

    // Note: PackageRegistryClient doesn't expose download methods directly
    // Those are part of the PackageRegistry trait, not the client interface

    #[test]
    fn test_has_registry() {
        let client = PackageRegistryClient::new();
        assert!(!client.has_registry());
        
        let mut client_with_registry = PackageRegistryClient::new();
        let mock_registry = MockRegistry::new(vec![]);
        client_with_registry.set_registry(Box::new(mock_registry));
        assert!(client_with_registry.has_registry());
    }

    #[test]
    fn test_clone_client() {
        let mut client = PackageRegistryClient::new();
        let mock_registry = MockRegistry::new(vec!["1.0.0".to_string()]);
        client.set_registry(Box::new(mock_registry));
        
        let cloned = client.clone_client();
        assert_eq!(client.has_registry(), cloned.has_registry());
    }

    #[test]
    fn test_default() {
        let client = PackageRegistryClient::default();
        assert!(!client.has_registry());
    }

    #[test]
    fn test_new() {
        let client = PackageRegistryClient::new();
        assert!(!client.has_registry());
    }
}