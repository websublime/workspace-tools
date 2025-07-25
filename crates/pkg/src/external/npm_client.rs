//! Package registry client implementations
//!
//! This module provides implementations for accessing package registries like npm,
//! retrieving package metadata, and managing version information.

use crate::{
    errors::PackageRegistryError, CacheEntry,
    network::{ResilientClient, ResilientClientConfig, RetryConfig, CircuitBreakerConfig},
    config::NetworkConfig,
};
use async_trait::async_trait;
use flate2::read::GzDecoder;
use serde_json::Value;
use std::{
    any::Any,
    collections::HashMap,
    io,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tar::Archive;
use tokio::fs;

/// Interface for package registry operations
///
/// This trait defines the common operations expected from a package registry,
/// such as retrieving version information and package metadata.
///
/// Implementors should provide efficient caching and appropriate error handling.
/// All implementations must be Send + Sync for thread safety.
#[async_trait]
pub trait PackageRegistry: Send + Sync {
    /// Get the latest version of a package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to query
    ///
    /// # Returns
    ///
    /// `Ok(Some(version))` if the package exists, `Ok(None)` if the package doesn't exist,
    /// or a `PackageRegistryError` if the query fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    async fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError>;

    /// Get all available versions of a package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to query
    ///
    /// # Returns
    ///
    /// A list of version strings, or a `PackageRegistryError` if the query fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    async fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;

    /// Get metadata about a package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to query
    /// * `version` - Specific version to retrieve
    ///
    /// # Returns
    ///
    /// Package metadata as JSON, or a `PackageRegistryError` if the query fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The specified package or version is not found
    /// - The response cannot be parsed as JSON
    async fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Value, PackageRegistryError>;

    /// Get the registry as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get the registry as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Download a package tarball and return the bytes
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to download
    /// * `version` - Version of the package to download
    ///
    /// # Returns
    ///
    /// Package tarball as bytes, or a `PackageRegistryError` if the download fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to download the package fails
    /// - The specified package or version is not found
    /// - The downloaded data cannot be read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{NpmRegistry, PackageRegistry};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), sublime_package_tools::PackageRegistryError> {
    /// let registry = NpmRegistry::default();
    /// let tarball_bytes = registry.download_package("lodash", "4.17.21").await?;
    /// println!("Downloaded {} bytes", tarball_bytes.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn download_package(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, PackageRegistryError>;

    /// Download and extract a package to a destination directory
    ///
    /// This method downloads the package tarball and extracts it to the specified
    /// destination directory. The extracted contents will include the `package/`
    /// directory structure as provided by npm.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to download and extract
    /// * `version` - Version of the package to download and extract
    /// * `destination` - Path where the package should be extracted
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `PackageRegistryError` if any operation fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The destination directory cannot be created
    /// - The package download fails
    /// - The tarball extraction fails
    /// - File system operations fail
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{NpmRegistry, PackageRegistry};
    /// use std::path::Path;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), sublime_package_tools::PackageRegistryError> {
    /// let registry = NpmRegistry::default();
    /// let dest = Path::new("./packages/lodash");
    /// registry.download_and_extract_package("lodash", "4.17.21", dest).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn download_and_extract_package(
        &self,
        package_name: &str,
        version: &str,
        destination: &Path,
    ) -> Result<(), PackageRegistryError>;
}

/// NPM registry client implementation with network resilience
///
/// Provides access to the NPM package registry with built-in caching,
/// retry policies, and circuit breaker protection for robust network operations.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::NpmRegistry;
///
/// // Create with the default npm registry URL
/// let registry = NpmRegistry::default();
///
/// // Or with a custom registry URL
/// let custom_registry = NpmRegistry::new("https://my-custom-registry.example.com");
///
/// // With custom network configuration
/// let config = sublime_package_tools::NetworkConfig::default();
/// let resilient_registry = NpmRegistry::with_network_config(
///     "https://registry.npmjs.org",
///     config
/// );
/// ```
pub struct NpmRegistry {
    base_url: String,
    resilient_client: ResilientClient,
    cache_ttl: Duration,
    versions_cache: Arc<Mutex<HashMap<String, CacheEntry<Vec<String>>>>>,
    latest_version_cache: Arc<Mutex<HashMap<String, CacheEntry<Option<String>>>>>,
    auth_token: Option<String>,
    auth_type: Option<String>,
}

impl Default for NpmRegistry {
    fn default() -> Self {
        Self::new("https://registry.npmjs.org")
    }
}

#[async_trait]
impl PackageRegistry for NpmRegistry {
    async fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError> {
        // Check cache first
        {
            if let Ok(cache) = self.latest_version_cache.lock() {
                if let Some(cache_entry) = cache.get(package_name) {
                    if cache_entry.is_valid(self.cache_ttl) {
                        return Ok(cache_entry.get());
                    }
                }
            }
        }

        let url = format!("{}/latest", self.package_url(package_name));

        let response = self
            .resilient_client
            .get(&url)
            .await
            .map_err(Self::convert_resilient_error)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let data: Value = response
            .json()
            .await
            .map_err(PackageRegistryError::FetchFailure)?;

        let version = data.get("version").and_then(|v| v.as_str()).map(ToString::to_string);

        // Cache the result
        if let Ok(mut cache) = self.latest_version_cache.lock() {
            cache.insert(package_name.to_string(), CacheEntry::new(version.clone()));
        }

        Ok(version)
    }

    async fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError> {
        // Check cache first
        {
            if let Ok(cache) = self.versions_cache.lock() {
                if let Some(cache_entry) = cache.get(package_name) {
                    if cache_entry.is_valid(self.cache_ttl) {
                        return Ok(cache_entry.get());
                    }
                }
            }
        }

        let url = self.package_url(package_name);

        let response = self
            .resilient_client
            .get(&url)
            .await
            .map_err(Self::convert_resilient_error)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        let data: Value = response
            .json()
            .await
            .map_err(PackageRegistryError::JsonParseFailure)?;

        let versions = data
            .get("versions")
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect::<Vec<String>>())
            .unwrap_or_default();

        // Cache the result
        if let Ok(mut cache) = self.versions_cache.lock() {
            cache.insert(package_name.to_string(), CacheEntry::new(versions.clone()));
        }

        Ok(versions)
    }

    async fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Value, PackageRegistryError> {
        let url = format!("{}/{}", self.package_url(package_name), version);

        let response = self
            .resilient_client
            .get(&url)
            .await
            .map_err(Self::convert_resilient_error)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PackageRegistryError::NotFound {
                package_name: package_name.to_string(),
                version: version.to_string(),
            });
        }

        response
            .json()
            .await
            .map_err(PackageRegistryError::JsonParseFailure)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn download_package(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, PackageRegistryError> {
        let download_url = self.get_download_url(package_name, version);

        let response = self
            .resilient_client
            .get(&download_url)
            .await
            .map_err(|e| match Self::convert_resilient_error(e) {
                PackageRegistryError::FetchFailure(req_err) => PackageRegistryError::DownloadFailure {
                    package_name: package_name.to_string(),
                    version: version.to_string(),
                    source: req_err,
                },
                other => other,
            })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PackageRegistryError::NotFound {
                package_name: package_name.to_string(),
                version: version.to_string(),
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| PackageRegistryError::DownloadFailure {
                package_name: package_name.to_string(),
                version: version.to_string(),
                source: e,
            })?;

        Ok(bytes.to_vec())
    }

    async fn download_and_extract_package(
        &self,
        package_name: &str,
        version: &str,
        destination: &Path,
    ) -> Result<(), PackageRegistryError> {
        // Create destination directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(destination).await {
            return Err(PackageRegistryError::DirectoryCreationFailure {
                path: destination.display().to_string(),
                source: e,
            });
        }

        // Download the package tarball
        let tarball_bytes = self.download_package(package_name, version).await?;

        // Create a cursor from the bytes for reading
        let cursor = io::Cursor::new(tarball_bytes);

        // Create a gzip decoder
        let gz_decoder = GzDecoder::new(cursor);

        // Create a tar archive from the decompressed data
        let mut archive = Archive::new(gz_decoder);

        // Extract the archive to the destination
        archive.unpack(destination).map_err(|e| PackageRegistryError::ExtractionFailure {
            package_name: package_name.to_string(),
            version: version.to_string(),
            destination: destination.display().to_string(),
            source: e,
        })?;

        Ok(())
    }
}

impl NpmRegistry {
    /// Create a new npm registry client with the given base URL
    ///
    /// Uses default network resilience configuration with caching, retry,
    /// and circuit breaker protection.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the npm registry
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::NpmRegistry;
    ///
    /// // Connect to the standard npm registry
    /// let registry = NpmRegistry::new("https://registry.npmjs.org");
    ///
    /// // Connect to a private registry
    /// let private_registry = NpmRegistry::new("https://npm.my-company.com");
    /// ```
    pub fn new(base_url: &str) -> Self {
        Self::with_network_config(base_url, NetworkConfig::default())
    }

    /// Create a new npm registry client with custom network configuration
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the npm registry
    /// * `network_config` - Network resilience configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{NpmRegistry, NetworkConfig};
    /// use std::time::Duration;
    ///
    /// let config = NetworkConfig {
    ///     enable_resilience: true,
    ///     request_timeout: Duration::from_secs(60),
    ///     ..Default::default()
    /// };
    /// let registry = NpmRegistry::with_network_config("https://registry.npmjs.org", config);
    /// ```
    pub fn with_network_config(base_url: &str, network_config: NetworkConfig) -> Self {
        let resilient_config = ResilientClientConfig {
            cache_size: 1000,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            retry_config: RetryConfig {
                max_retries: network_config.retry_config.max_retries,
                initial_delay: network_config.retry_config.initial_delay,
                max_delay: network_config.retry_config.max_delay,
                exponential_base: network_config.retry_config.exponential_base,
                jitter: network_config.retry_config.enable_jitter,
            },
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: network_config.circuit_breaker_config.failure_threshold,
                success_threshold: network_config.circuit_breaker_config.success_threshold,
                timeout: network_config.circuit_breaker_config.timeout,
                half_open_max_calls: network_config.circuit_breaker_config.half_open_max_calls,
            },
            timeout: network_config.request_timeout,
            user_agent: "sublime-package-tools/0.1.0".to_string(),
            enable_cache: network_config.enable_caching && network_config.enable_resilience,
            enable_retry: network_config.enable_resilience,
            enable_circuit_breaker: network_config.enable_resilience,
        };

        let resilient_client = ResilientClient::new(
            format!("npm-registry-{}", base_url.replace("https://", "").replace("http://", "")),
            resilient_config
        );

        Self {
            base_url: base_url.to_string(),
            resilient_client,
            cache_ttl: Duration::from_secs(300), // 5 minutes default
            versions_cache: Arc::new(Mutex::new(HashMap::new())),
            latest_version_cache: Arc::new(Mutex::new(HashMap::new())),
            auth_token: None,
            auth_type: None,
        }
    }

    /// Set the user agent string
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The User-Agent header value to use for requests
    ///
    /// Note: This method is deprecated as user agent is now configured
    /// at creation time through ResilientClientConfig.
    #[deprecated(note = "User agent is now configured at creation time")]
    #[allow(dead_code)]
    pub fn set_user_agent(&mut self, _user_agent: &str) -> &mut Self {
        // No-op: user agent is now handled by ResilientClient
        self
    }

    /// Set authentication
    ///
    /// # Arguments
    ///
    /// * `token` - Authentication token
    /// * `auth_type` - Type of authentication (e.g., "bearer", "basic", etc.)
    ///
    /// # Returns
    ///
    /// Mutable reference to self for method chaining
    #[allow(dead_code)]
    pub fn set_auth(&mut self, token: &str, auth_type: &str) -> &mut Self {
        self.auth_token = Some(token.to_string());
        self.auth_type = Some(auth_type.to_string());
        self
    }

    /// Set cache TTL
    ///
    /// # Arguments
    ///
    /// * `ttl` - Time-to-live duration for cached items
    ///
    /// # Returns
    ///
    /// Mutable reference to self for method chaining
    #[allow(dead_code)]
    pub fn set_cache_ttl(&mut self, ttl: Duration) -> &mut Self {
        self.cache_ttl = ttl;
        self
    }

    /// Clear all caches
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.versions_cache = Arc::new(Mutex::new(HashMap::new()));
        self.latest_version_cache = Arc::new(Mutex::new(HashMap::new()));
    }

    /// Build a URL for the package
    fn package_url(&self, package_name: &str) -> String {
        // Handle scoped packages correctly
        let encoded_name = if package_name.starts_with('@') {
            // URL encode the @ character and the / character
            package_name.replace('@', "%40").replace('/', "%2F")
        } else {
            package_name.to_string()
        };

        format!("{}/{}", self.base_url, encoded_name)
    }

    /// Build a download URL for the package tarball
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    /// * `version` - Version of the package
    ///
    /// # Returns
    ///
    /// Complete URL to download the package tarball
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::NpmRegistry;
    ///
    /// let registry = NpmRegistry::default();
    /// let url = registry.get_download_url("lodash", "4.17.21");
    /// assert!(url.contains("lodash"));
    /// assert!(url.contains("4.17.21"));
    /// ```
    #[must_use]
    pub fn get_download_url(&self, package_name: &str, version: &str) -> String {
        if package_name.starts_with('@') {
            // Handle scoped packages: @scope/package -> @scope/package/-/package-version.tgz
            let parts: Vec<&str> = package_name.splitn(2, '/').collect();
            if parts.len() == 2 {
                let _scope = parts[0]; // @scope
                let name = parts[1]; // package
                format!("{}/{}/-/{}-{}.tgz", self.base_url, package_name, name, version)
            } else {
                // Fallback for malformed scoped package names
                format!("{}/{}/-/{}-{}.tgz", self.base_url, package_name, package_name, version)
            }
        } else {
            // Regular packages: package -> package/-/package-version.tgz
            format!("{}/{}/-/{}-{}.tgz", self.base_url, package_name, package_name, version)
        }
    }

    /// Get resilience statistics
    ///
    /// Returns cache and circuit breaker statistics for monitoring
    /// network performance and resilience.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Cache stats: (hits, misses, evictions, expirations)
    /// - Circuit breaker stats: (total_calls, successes, failures, rejections)
    pub async fn resilience_stats(&self) -> ((u64, u64, u64, u64), (u64, u64, u64, u64)) {
        let cache_stats = self.resilient_client.cache_stats().await;
        let circuit_stats = self.resilient_client.circuit_breaker_stats().await;
        (cache_stats, circuit_stats)
    }

    /// Clear the network cache
    ///
    /// Clears both the internal cache (HashMap-based) and the resilient client cache
    pub async fn clear_all_caches(&self) {
        // Clear internal caches
        if let Ok(mut cache) = self.versions_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.latest_version_cache.lock() {
            cache.clear();
        }

        // Clear resilient client cache
        self.resilient_client.clear_cache().await;
    }

    /// Reset the circuit breaker
    ///
    /// Manually reset the circuit breaker to closed state
    pub async fn reset_circuit_breaker(&self) {
        self.resilient_client.reset_circuit_breaker().await;
    }

    /// Convert ResilientClientError to PackageRegistryError
    fn convert_resilient_error(error: crate::network::ResilientClientError) -> PackageRegistryError {
        use crate::network::ResilientClientError;
        
        match error {
            ResilientClientError::RequestError(req_err) => {
                PackageRegistryError::FetchFailure(req_err)
            }
            ResilientClientError::HttpError { status: _, url: _ } => {
                // For HTTP errors, we'll return a generic fetch failure
                // The status will be checked separately by the calling code
                PackageRegistryError::LockFailure // Use this as a temporary placeholder
            }
            ResilientClientError::JsonError(json_err) => {
                PackageRegistryError::JsonParseFailure(json_err)
            }
            ResilientClientError::CircuitBreakerOpen { service: _ } => {
                PackageRegistryError::LockFailure // Use this as a temporary placeholder
            }
            ResilientClientError::RequestCloneError => {
                PackageRegistryError::LockFailure // Use this as a temporary placeholder
            }
        }
    }
}

impl Clone for NpmRegistry {
    fn clone(&self) -> Self {
        // Create a new registry with the same configuration
        // Note: This creates a new resilient client and fresh caches
        let mut cloned = Self::new(&self.base_url);
        cloned.auth_token = self.auth_token.clone();
        cloned.auth_type = self.auth_type.clone();
        cloned.cache_ttl = self.cache_ttl;
        cloned
    }
}

/// Trait for package registries that can be cloned
/// 
/// This trait extends PackageRegistry with the ability to clone the registry.
/// All implementations must be Send + Sync for thread safety.
pub trait PackageRegistryClone: PackageRegistry + Send + Sync {
    /// Clone the package registry implementation
    fn clone_box(&self) -> Box<dyn PackageRegistryClone>;
}

impl PackageRegistryClone for NpmRegistry {
    fn clone_box(&self) -> Box<dyn PackageRegistryClone> {
        Box::new(self.clone())
    }
}
