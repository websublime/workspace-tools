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
//! use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
//!
//! // Create client without registry
//! let client = PackageRegistryClient::new();
//! assert!(!client.has_registry());
//!
//! // With registry configuration
//! let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
//! let versions = client.get_package_versions("react").await.unwrap();
//! ```

use crate::external::npm_client::PackageRegistryClone;
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
/// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
    ///
    /// # async fn example() -> Result<(), PackageRegistryError> {
    /// let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// if let Some(version) = client.get_latest_version("react").await? {
    ///     println!("Latest react version: {}", version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
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
        
        Ok(parsed_versions.last().map(|(_, version)| (*version).clone()))
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
    ///
    /// # async fn example() -> Result<(), PackageRegistryError> {
    /// let client = PackageRegistryClient::with_registry(Box::new(npm_registry));
    /// if let Some(metadata) = client.get_package_metadata("react", "17.0.0").await? {
    ///     println!("React 17.0.0 metadata: {}", metadata);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
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
    /// use sublime_package_tools::external::package_registry_client::PackageRegistryClient;
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