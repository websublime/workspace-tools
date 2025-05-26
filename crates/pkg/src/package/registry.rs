//! Package registry client implementations
//!
//! This module provides implementations for accessing package registries like npm,
//! retrieving package metadata, and managing version information.

use crate::{CacheEntry, PackageRegistryError};
use flate2::read::GzDecoder;
use reqwest::blocking::{Client, RequestBuilder};
use serde_json::Value;
use std::{
    any::Any,
    collections::HashMap,
    fs,
    io,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tar::Archive;

/// Interface for package registry operations
///
/// This trait defines the common operations expected from a package registry,
/// such as retrieving version information and package metadata.
///
/// Implementors should provide efficient caching and appropriate error handling.
pub trait PackageRegistry {
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
    fn get_latest_version(
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
    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;

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
    fn get_package_info(
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
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{NpmRegistry, PackageRegistry};
    ///
    /// let registry = NpmRegistry::default();
    /// let tarball_bytes = registry.download_package("lodash", "4.17.21")?;
    /// println!("Downloaded {} bytes", tarball_bytes.len());
    /// # Ok::<(), sublime_package_tools::PackageRegistryError>(())
    /// ```
    fn download_package(
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
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{NpmRegistry, PackageRegistry};
    /// use std::path::Path;
    ///
    /// let registry = NpmRegistry::default();
    /// let dest = Path::new("./packages/lodash");
    /// registry.download_and_extract_package("lodash", "4.17.21", dest)?;
    /// # Ok::<(), sublime_package_tools::PackageRegistryError>(())
    /// ```
    fn download_and_extract_package(
        &self,
        package_name: &str,
        version: &str,
        destination: &Path,
    ) -> Result<(), PackageRegistryError>;
}

/// NPM registry client implementation
///
/// Provides access to the NPM package registry and implements caching
/// for efficient queries.
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
/// ```
pub struct NpmRegistry {
    base_url: String,
    client: Client,
    user_agent: String,
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

impl PackageRegistry for NpmRegistry {
    fn get_latest_version(
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

        let response =
            self.build_request(&url).send().map_err(PackageRegistryError::FetchFailure)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let data: Value = response.json().map_err(PackageRegistryError::FetchFailure)?;

        let version = data.get("version").and_then(|v| v.as_str()).map(ToString::to_string);

        // Cache the result
        if let Ok(mut cache) = self.latest_version_cache.lock() {
            cache.insert(package_name.to_string(), CacheEntry::new(version.clone()));
        }

        Ok(version)
    }

    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError> {
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

        let response =
            self.build_request(&url).send().map_err(PackageRegistryError::FetchFailure)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        let data: Value = response.json().map_err(PackageRegistryError::JsonParseFailure)?;

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

    fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Value, PackageRegistryError> {
        let url = format!("{}/{}", self.package_url(package_name), version);

        let response =
            self.build_request(&url).send().map_err(PackageRegistryError::FetchFailure)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PackageRegistryError::NotFound {
                package_name: package_name.to_string(),
                version: version.to_string(),
            });
        }

        response.json().map_err(PackageRegistryError::JsonParseFailure)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn download_package(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, PackageRegistryError> {
        let download_url = self.get_download_url(package_name, version);

        let response = self
            .build_request(&download_url)
            .send()
            .map_err(|e| PackageRegistryError::DownloadFailure {
                package_name: package_name.to_string(),
                version: version.to_string(),
                source: e,
            })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PackageRegistryError::NotFound {
                package_name: package_name.to_string(),
                version: version.to_string(),
            });
        }

        let bytes = response.bytes().map_err(|e| PackageRegistryError::DownloadFailure {
            package_name: package_name.to_string(),
            version: version.to_string(),
            source: e,
        })?;

        Ok(bytes.to_vec())
    }

    fn download_and_extract_package(
        &self,
        package_name: &str,
        version: &str,
        destination: &Path,
    ) -> Result<(), PackageRegistryError> {
        // Create destination directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(destination) {
            return Err(PackageRegistryError::DirectoryCreationFailure {
                path: destination.display().to_string(),
                source: e,
            });
        }

        // Download the package tarball
        let tarball_bytes = self.download_package(package_name, version)?;

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
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
            user_agent: "ws-pkg/0.1.0".to_string(),
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
    /// # Returns
    ///
    /// Mutable reference to self for method chaining
    pub fn set_user_agent(&mut self, user_agent: &str) -> &mut Self {
        self.user_agent = user_agent.to_string();
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
    pub fn set_cache_ttl(&mut self, ttl: Duration) -> &mut Self {
        self.cache_ttl = ttl;
        self
    }

    /// Clear all caches
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

    /// Build a request with appropriate headers
    fn build_request(&self, url: &str) -> RequestBuilder {
        let mut builder = self.client.get(url).header("User-Agent", &self.user_agent);

        // Add auth if configured
        if let (Some(token), Some(auth_type)) = (&self.auth_token, &self.auth_type) {
            let auth_header = if auth_type.eq_ignore_ascii_case("bearer") {
                format!("Bearer {token}")
            } else if auth_type.eq_ignore_ascii_case("basic") {
                format!("Basic {token}")
            } else {
                token.clone()
            };

            builder = builder.header("Authorization", auth_header);
        }

        builder
    }
}

impl Clone for NpmRegistry {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            client: Client::new(),
            user_agent: self.user_agent.clone(),
            cache_ttl: self.cache_ttl,
            versions_cache: Arc::new(Mutex::new(HashMap::new())),
            latest_version_cache: Arc::new(Mutex::new(HashMap::new())),
            auth_token: self.auth_token.clone(),
            auth_type: self.auth_type.clone(),
        }
    }
}

/// Trait for package registries that can be cloned
pub trait PackageRegistryClone: PackageRegistry {
    /// Clone the package registry implementation
    fn clone_box(&self) -> Box<dyn PackageRegistryClone>;
}

impl PackageRegistryClone for NpmRegistry {
    fn clone_box(&self) -> Box<dyn PackageRegistryClone> {
        Box::new(self.clone())
    }
}
