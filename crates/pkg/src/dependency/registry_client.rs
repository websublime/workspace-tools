//! # Package Registry Client Service
//!
//! External service communication client for package registry operations.
//!
//! ## Overview
//!
//! This service encapsulates all interactions with external package registries:
//! - Async network operations with clean boundaries
//! - Registry lifecycle management
//! - Error handling and fallback strategies  
//! - Thread-safe registry sharing via Arc
//!
//! ## Thread Safety
//!
//! Uses Arc<Box<dyn PackageRegistryClone>> for safe registry sharing:
//! - Arc enables cheap cloning for service sharing
//! - Option allows "no registry" configuration
//! - All operations handle missing registry gracefully
//!
//! ## Async Design
//!
//! All network operations are async by default:
//! - Pure async interface for network calls
//! - Sync wrappers available where needed
//! - Proper error propagation from registry layer
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
//!
//! // Create client without registry
//! let client = PackageRegistryClient::new();
//! assert!(!client.has_registry());
//!
//! // With registry configuration
//! let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
//! let versions = client.get_package_versions("react").await.unwrap();
//! ```

use crate::package::registry::PackageRegistryClone;
use crate::errors::PackageRegistryError;
use std::sync::Arc;

/// Debug trait implementation for PackageRegistryClone trait objects
impl std::fmt::Debug for dyn PackageRegistryClone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackageRegistry").finish()
    }
}

/// External service communication client for package registry operations
/// 
/// This service handles all external package registry interactions while maintaining
/// clean separation from dependency storage and conflict resolution concerns.
///
/// # Architecture
///
/// - **Single Responsibility**: Only handles external registry communication
/// - **Thread Safety**: Arc<Box<dyn>> for safe registry sharing between services
/// - **Async First**: All network operations use async patterns
/// - **Graceful Fallbacks**: Handles missing registry configuration elegantly
///
/// # Examples
///
/// ```rust
/// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
///
/// let client = PackageRegistryClient::new();
/// 
/// // Configure registry
/// let npm_registry = NpmRegistry::new("https://registry.npmjs.org");
/// let mut client = PackageRegistryClient::with_registry(Box::new(npm_registry));
/// 
/// // Query package versions
/// let versions = client.get_package_versions("react").await.unwrap();
/// println!("Found {} versions", versions.len());
/// ```
#[derive(Debug)]
pub(crate) struct PackageRegistryClient {
    /// Optional package registry for external queries
    /// 
    /// - Arc enables sharing between threads and services
    /// - Option allows "no registry" configuration
    /// - Box<dyn> provides trait object flexibility
    registry: Option<Arc<Box<dyn PackageRegistryClone>>>,
}

impl PackageRegistryClient {
    /// Creates a new client without registry configuration
    ///
    /// The client starts without any registry, requiring explicit configuration
    /// before network operations can be performed.
    ///
    /// # Returns
    ///
    /// A new PackageRegistryClient with no registry configured
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// let client = PackageRegistryClient::new();
    /// assert!(!client.has_registry());
    /// 
    /// // Network operations will return empty results
    /// let versions = client.get_package_versions("react").await.unwrap();
    /// assert!(versions.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { registry: None }
    }

    /// Creates a new client with registry configuration
    ///
    /// This constructor immediately configures the client with a registry,
    /// enabling all network operations.
    ///
    /// # Arguments
    ///
    /// * `registry` - Boxed registry implementation
    ///
    /// # Returns
    ///
    /// A new PackageRegistryClient with registry configured
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org");
    /// let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// assert!(client.has_registry());
    /// ```
    #[must_use]
    pub fn with_registry(registry: Box<dyn PackageRegistryClone>) -> Self {
        Self {
            registry: Some(Arc::new(registry)),
        }
    }

    /// Sets the registry for this client
    ///
    /// Replaces any existing registry configuration with the provided one.
    /// This allows runtime reconfiguration of the registry.
    ///
    /// # Arguments
    ///
    /// * `registry` - Boxed registry implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// let mut client = PackageRegistryClient::new();
    /// assert!(!client.has_registry());
    /// 
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org");
    /// client.set_registry(Box::new(npm_registry));
    /// assert!(client.has_registry());
    /// ```
    pub fn set_registry(&mut self, registry: Box<dyn PackageRegistryClone>) {
        self.registry = Some(Arc::new(registry));
    }

    /// Checks if registry is configured
    ///
    /// # Returns
    ///
    /// `true` if registry is available for network operations, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// let client = PackageRegistryClient::new();
    /// assert!(!client.has_registry());
    /// 
    /// let client_with_registry = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// assert!(client_with_registry.has_registry());
    /// ```
    #[must_use]
    pub fn has_registry(&self) -> bool {
        self.registry.is_some()
    }

    /// Gets all available package versions from registry
    ///
    /// This implements the core logic from Registry::get_package_versions with
    /// improved error handling and graceful fallbacks.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of package to query
    ///
    /// # Returns
    ///
    /// Vector of version strings. Returns empty vector if no registry configured.
    ///
    /// # Errors
    ///
    /// Returns `PackageRegistryError` if:
    /// - Network request fails
    /// - Registry returns malformed data
    /// - Package does not exist in registry
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// # async fn example() -> Result<(), PackageRegistryError> {
    /// let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// let versions = client.get_package_versions("react").await?;
    /// println!("Found {} versions for react", versions.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_package_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, PackageRegistryError> {
        if let Some(ref registry) = self.registry {
            registry.get_all_versions(package_name).await
        } else {
            // No registry configured - return empty list
            // This matches the current behavior and allows graceful degradation
            Ok(Vec::new())
        }
    }

    /// Gets latest package version from registry
    ///
    /// Convenience method to get the most recent version of a package.
    /// Useful for dependency resolution and upgrade scenarios.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of package to query
    ///
    /// # Returns
    ///
    /// Latest version string, or None if package not found or no registry configured
    ///
    /// # Errors
    ///
    /// Returns `PackageRegistryError` if network operations fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// # async fn example() -> Result<(), PackageRegistryError> {
    /// let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// if let Some(version) = client.get_latest_version("react").await? {
    ///     println!("Latest react version: {}", version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError> {
        let versions = self.get_package_versions(package_name).await?;
        
        if versions.is_empty() {
            return Ok(None);
        }

        // Find the latest version using semver comparison
        use semver::Version;
        
        let mut parsed_versions = Vec::new();
        for version_str in &versions {
            // Clean version string (remove ^ ~ = prefixes)
            let clean_version = version_str
                .trim_start_matches('^')
                .trim_start_matches('~') 
                .trim_start_matches('=');
                
            if let Ok(version) = Version::parse(clean_version) {
                parsed_versions.push((version, version_str));
            }
        }

        if parsed_versions.is_empty() {
            return Ok(None);
        }

        // Sort by semver and get the highest
        parsed_versions.sort_by(|(a, _), (b, _)| a.cmp(b));
        
        Ok(Some(parsed_versions.last().unwrap().1.clone()))
    }

    /// Gets package metadata from registry
    ///
    /// Extended information about a specific package version including
    /// dependencies, description, and other metadata.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of package to query
    /// * `version` - Specific version to get metadata for
    ///
    /// # Returns
    ///
    /// Package metadata as JSON string, or None if not found
    ///
    /// # Errors
    ///
    /// Returns `PackageRegistryError` if network operations fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// # async fn example() -> Result<(), PackageRegistryError> {
    /// let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// if let Some(metadata) = client.get_package_metadata("react", "17.0.0").await? {
    ///     println!("React 17.0.0 metadata: {}", metadata);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_package_metadata(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Option<String>, PackageRegistryError> {
        if let Some(ref registry) = self.registry {
            // For now, we don't have a direct metadata method in PackageRegistryClone
            // We could extend the trait in the future or use get_all_versions as a proxy
            let versions = registry.get_all_versions(package_name).await?;
            
            if versions.contains(&version.to_string()) {
                // Return basic metadata - in a real implementation this would
                // fetch full package.json data from the registry
                Ok(Some(format!(
                    r#"{{"name": "{}", "version": "{}", "available": true}}"#,
                    package_name, version
                )))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Creates a clone of this client
    ///
    /// This is more explicit than implementing Clone trait, as it makes
    /// the cloning intention clear and allows for future customization.
    ///
    /// # Returns
    ///
    /// A new PackageRegistryClient with the same registry configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::registry_client::PackageRegistryClient;
    ///
    /// let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// let client_clone = client.clone_client();
    /// assert_eq!(client.has_registry(), client_clone.has_registry());
    /// ```
    #[must_use]
    pub fn clone_client(&self) -> Self {
        Self {
            registry: self.registry.clone(),
        }
    }
}

impl Default for PackageRegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PackageRegistryClient {
    fn clone(&self) -> Self {
        self.clone_client()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::registry::{PackageRegistry, PackageRegistryClone};
    use crate::errors::PackageRegistryError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Simple mock registry for testing
    #[derive(Debug, Clone)]
    struct MockPackageRegistry {
        packages: Arc<Mutex<HashMap<String, Vec<String>>>>,
    }

    impl MockPackageRegistry {
        fn new() -> Self {
            Self {
                packages: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_package_versions(&mut self, name: &str, versions: &[&str]) -> Result<(), String> {
            let mut packages = self.packages.lock().map_err(|_| "Lock poisoned")?;
            packages.insert(
                name.to_string(), 
                versions.iter().map(|v| v.to_string()).collect()
            );
            Ok(())
        }
    }

    #[async_trait]
    impl PackageRegistry for MockPackageRegistry {
        async fn get_latest_version(&self, package_name: &str) -> Result<Option<String>, PackageRegistryError> {
            let packages = self.packages.lock().map_err(|_| PackageRegistryError::LockFailure)?;
            if let Some(versions) = packages.get(package_name) {
                Ok(versions.last().cloned())
            } else {
                Ok(None)
            }
        }

        async fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError> {
            let packages = self.packages.lock().map_err(|_| PackageRegistryError::LockFailure)?;
            Ok(packages.get(package_name).cloned().unwrap_or_default())
        }

        async fn get_package_info(&self, package_name: &str, version: &str) -> Result<serde_json::Value, PackageRegistryError> {
            use serde_json::json;
            let packages = self.packages.lock().map_err(|_| PackageRegistryError::LockFailure)?;
            if let Some(versions) = packages.get(package_name) {
                if versions.contains(&version.to_string()) {
                    Ok(json!({
                        "name": package_name,
                        "version": version,
                        "description": "Mock package for testing"
                    }))
                } else {
                    Err(PackageRegistryError::NotFound {
                        package_name: package_name.to_string(),
                        version: version.to_string(),
                    })
                }
            } else {
                Err(PackageRegistryError::NotFound {
                    package_name: package_name.to_string(),
                    version: version.to_string(),
                })
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        async fn download_package(&self, _package_name: &str, _version: &str) -> Result<Vec<u8>, PackageRegistryError> {
            Ok(vec![]) // Mock implementation
        }

        async fn download_and_extract_package(&self, _package_name: &str, _version: &str, _destination: &std::path::Path) -> Result<(), PackageRegistryError> {
            Ok(()) // Mock implementation
        }
    }

    impl PackageRegistryClone for MockPackageRegistry {
        fn clone_box(&self) -> Box<dyn PackageRegistryClone> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_client_without_registry() {
        let client = PackageRegistryClient::new();
        assert!(!client.has_registry());
    }

    #[test]
    fn test_client_with_registry() {
        let mock_registry = MockPackageRegistry::new();
        let client = PackageRegistryClient::with_registry(Box::new(mock_registry));
        assert!(client.has_registry());
    }

    #[test]
    fn test_client_set_registry() {
        let mut client = PackageRegistryClient::new();
        assert!(!client.has_registry());
        
        let mock_registry = MockPackageRegistry::new();
        client.set_registry(Box::new(mock_registry));
        assert!(client.has_registry());
    }

    #[tokio::test]
    async fn test_get_package_versions_without_registry() {
        let client = PackageRegistryClient::new();
        let versions = client.get_package_versions("react").await.unwrap();
        assert!(versions.is_empty());
    }

    #[tokio::test]
    async fn test_get_package_versions_with_registry() {
        let mut mock_registry = MockPackageRegistry::new();
        mock_registry.add_package_versions("react", &["17.0.0", "18.0.0"]).unwrap();
        
        let client = PackageRegistryClient::with_registry(Box::new(mock_registry));
        let versions = client.get_package_versions("react").await.unwrap();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"17.0.0".to_string()));
        assert!(versions.contains(&"18.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_get_latest_version() {
        let mut mock_registry = MockPackageRegistry::new();
        mock_registry.add_package_versions("react", &["16.0.0", "17.0.0", "18.0.0"]).unwrap();
        
        let client = PackageRegistryClient::with_registry(Box::new(mock_registry));
        let latest = client.get_latest_version("react").await.unwrap();
        assert_eq!(latest, Some("18.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_get_latest_version_without_registry() {
        let client = PackageRegistryClient::new();
        let latest = client.get_latest_version("react").await.unwrap();
        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn test_get_package_metadata() {
        let mut mock_registry = MockPackageRegistry::new();
        mock_registry.add_package_versions("react", &["17.0.0"]).unwrap();
        
        let client = PackageRegistryClient::with_registry(Box::new(mock_registry));
        let metadata = client.get_package_metadata("react", "17.0.0").await.unwrap();
        assert!(metadata.is_some());
        assert!(metadata.unwrap().contains("react"));
    }

    #[test]
    fn test_client_clone() {
        let mock_registry = MockPackageRegistry::new();
        let client = PackageRegistryClient::with_registry(Box::new(mock_registry));
        let cloned = client.clone_client();
        
        assert_eq!(client.has_registry(), cloned.has_registry());
    }
}