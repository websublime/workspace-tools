//! Registry client for querying NPM package metadata.
//!
//! **What**: Provides HTTP client functionality for querying NPM registries to fetch
//! package metadata, versions, and deprecation information.
//!
//! **How**: Uses reqwest with retry middleware to communicate with NPM registries,
//! handling authentication, timeouts, and scoped packages. Supports both public
//! NPM registry and private registries with authentication.
//!
//! **Why**: To enable reliable package metadata fetching with proper error handling,
//! retry logic, and support for enterprise private registries.

use crate::config::RegistryConfig;
use crate::error::UpgradeError;
use crate::upgrade::registry::npmrc::NpmrcConfig;
use crate::upgrade::registry::types::{PackageMetadata, RepositoryInfo, UpgradeType};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

/// Client for querying NPM package registries.
///
/// Supports public NPM registry, private registries, and scoped packages.
/// Includes retry logic for transient failures and proper authentication handling.
///
/// # Example
///
/// ```rust,no_run
/// use sublime_pkg_tools::upgrade::RegistryClient;
/// use sublime_pkg_tools::config::RegistryConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let config = RegistryConfig::default();
///
/// let client = RegistryClient::new(&workspace_root, config).await?;
/// let metadata = client.get_package_info("express").await?;
///
/// println!("Package: {}", metadata.name);
/// println!("Latest version: {}", metadata.latest);
/// # Ok(())
/// # }
/// ```
pub struct RegistryClient {
    /// Registry configuration
    config: RegistryConfig,

    /// HTTP client with retry middleware
    http_client: ClientWithMiddleware,

    /// .npmrc configuration loaded from workspace
    npmrc: Option<NpmrcConfig>,
}

/// Internal structure for deserializing registry responses.
///
/// The NPM registry API returns a complex JSON structure. This struct
/// represents the top-level response for package metadata queries.
#[derive(Debug, Deserialize)]
struct RegistryResponse {
    name: String,
    versions: HashMap<String, VersionInfo>,
    #[serde(rename = "dist-tags")]
    dist_tags: HashMap<String, String>,
    time: HashMap<String, String>,
    repository: Option<RepositoryInfo>,
}

/// Version-specific information from registry.
#[derive(Debug, Deserialize)]
struct VersionInfo {
    #[serde(default)]
    deprecated: Option<String>,
}

impl RegistryClient {
    /// Creates a new registry client.
    ///
    /// Initializes the HTTP client with retry logic and authentication.
    /// If `config.read_npmrc` is true, will attempt to read .npmrc configuration
    /// from the workspace (to be implemented in story 9.2).
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace
    /// * `config` - Registry configuration including URLs, timeouts, and authentication
    ///
    /// # Returns
    ///
    /// A configured `RegistryClient` ready to query package metadata.
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - HTTP client construction fails
    /// - .npmrc reading fails (when implemented)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sublime_pkg_tools::upgrade::RegistryClient;
    /// use sublime_pkg_tools::config::RegistryConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = RegistryConfig::default();
    /// let client = RegistryClient::new(&PathBuf::from("."), config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(_workspace_root: &Path, config: RegistryConfig) -> Result<Self, UpgradeError> {
        // Build base reqwest client with timeout
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/vnd.npm.install-v1+json"));

        let reqwest_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .default_headers(headers)
            .build()
            .map_err(|e| UpgradeError::NetworkError {
                reason: format!("Failed to build HTTP client: {}", e),
            })?;

        // Configure retry policy
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(
                Duration::from_millis(config.retry_delay_ms),
                Duration::from_secs(config.timeout_secs / 2),
            )
            .build_with_max_retries(config.retry_attempts as u32);

        // Build client with retry middleware
        let http_client = ClientBuilder::new(reqwest_client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Load .npmrc if configured
        let npmrc = if config.read_npmrc {
            match NpmrcConfig::from_workspace(
                _workspace_root,
                &sublime_standard_tools::filesystem::FileSystemManager::new(),
            )
            .await
            {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    // Log but don't fail on .npmrc errors
                    eprintln!("Warning: Failed to load .npmrc: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self { config, http_client, npmrc })
    }

    /// Queries package metadata from the registry.
    ///
    /// Fetches complete package information including all versions, deprecation
    /// status, and repository information.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package (e.g., "express" or "@scope/package")
    ///
    /// # Returns
    ///
    /// Complete package metadata including all available versions.
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - Package not found in registry (404)
    /// - Network error occurs
    /// - Request times out
    /// - Registry returns invalid response
    /// - Authentication fails (for private packages)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sublime_pkg_tools::upgrade::RegistryClient;
    /// use sublime_pkg_tools::config::RegistryConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RegistryClient::new(&PathBuf::from("."), RegistryConfig::default()).await?;
    /// let metadata = client.get_package_info("lodash").await?;
    ///
    /// println!("Found {} versions", metadata.versions.len());
    /// if metadata.is_deprecated() {
    ///     println!("Warning: Package is deprecated!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_package_info(
        &self,
        package_name: &str,
    ) -> Result<PackageMetadata, UpgradeError> {
        let registry_url = self.resolve_registry_url(package_name);
        let package_url = format!("{}/{}", registry_url.trim_end_matches('/'), package_name);

        // Build request with authentication if available
        let mut request = self.http_client.get(&package_url);

        if let Some(token) = self.resolve_auth_token(&registry_url) {
            let auth_header = format!("Bearer {}", token);
            request = request.header(AUTHORIZATION, auth_header);
        }

        // Execute request
        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                UpgradeError::RegistryTimeout {
                    package: package_name.to_string(),
                    timeout_secs: self.config.timeout_secs,
                }
            } else {
                UpgradeError::NetworkError {
                    reason: format!("Failed to query registry for '{}': {}", package_name, e),
                }
            }
        })?;

        // Handle HTTP errors
        let status = response.status();
        if !status.is_success() {
            if status.as_u16() == 404 {
                return Err(UpgradeError::PackageNotFound {
                    package: package_name.to_string(),
                    registry: registry_url,
                });
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(UpgradeError::AuthenticationFailed {
                    registry: registry_url,
                    reason: format!("HTTP {}: Authentication required", status.as_u16()),
                });
            } else {
                return Err(UpgradeError::RegistryError {
                    package: package_name.to_string(),
                    reason: format!(
                        "HTTP {}: {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("Unknown error")
                    ),
                });
            }
        }

        // Parse response
        let registry_response: RegistryResponse =
            response.json().await.map_err(|e| UpgradeError::InvalidResponse {
                package: package_name.to_string(),
                reason: format!("Failed to parse JSON response: {}", e),
            })?;

        // Convert to PackageMetadata
        self.convert_to_metadata(registry_response, package_name)
    }

    /// Gets the latest version for a package.
    ///
    /// Queries the registry and returns the version associated with the "latest" dist-tag.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    ///
    /// # Returns
    ///
    /// The latest version string.
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - Package metadata cannot be fetched
    /// - No "latest" tag is found
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sublime_pkg_tools::upgrade::RegistryClient;
    /// use sublime_pkg_tools::config::RegistryConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RegistryClient::new(&PathBuf::from("."), RegistryConfig::default()).await?;
    /// let latest = client.get_latest_version("react").await?;
    /// println!("Latest version: {}", latest);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_latest_version(&self, package_name: &str) -> Result<String, UpgradeError> {
        let metadata = self.get_package_info(package_name).await?;
        Ok(metadata.latest)
    }

    /// Compares two versions and determines the upgrade type.
    ///
    /// Uses semantic versioning to classify the upgrade as major, minor, or patch.
    ///
    /// # Arguments
    ///
    /// * `current` - Current version string (e.g., "1.2.3")
    /// * `latest` - Latest version string (e.g., "2.0.0")
    ///
    /// # Returns
    ///
    /// The type of upgrade (Major, Minor, or Patch).
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - Either version string is not valid semver
    /// - Latest version is not greater than current version
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{RegistryClient, UpgradeType};
    /// use sublime_pkg_tools::config::RegistryConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RegistryClient::new(&PathBuf::from("."), RegistryConfig::default()).await?;
    ///
    /// let upgrade = client.compare_versions("1.2.3", "2.0.0")?;
    /// assert_eq!(upgrade, UpgradeType::Major);
    ///
    /// let upgrade = client.compare_versions("1.2.3", "1.3.0")?;
    /// assert_eq!(upgrade, UpgradeType::Minor);
    ///
    /// let upgrade = client.compare_versions("1.2.3", "1.2.4")?;
    /// assert_eq!(upgrade, UpgradeType::Patch);
    /// # Ok(())
    /// # }
    /// ```
    pub fn compare_versions(
        &self,
        current: &str,
        latest: &str,
    ) -> Result<UpgradeType, UpgradeError> {
        let current_version =
            Version::parse(current).map_err(|e| UpgradeError::InvalidVersionSpec {
                package: "unknown".to_string(),
                spec: current.to_string(),
                reason: format!("Invalid semver: {}", e),
            })?;

        let latest_version =
            Version::parse(latest).map_err(|e| UpgradeError::InvalidVersionSpec {
                package: "unknown".to_string(),
                spec: latest.to_string(),
                reason: format!("Invalid semver: {}", e),
            })?;

        if latest_version <= current_version {
            return Err(UpgradeError::VersionComparisonFailed {
                package: "unknown".to_string(),
                reason: format!(
                    "Latest version '{}' is not greater than current version '{}'",
                    latest, current
                ),
            });
        }

        // Determine upgrade type based on semantic versioning
        if latest_version.major > current_version.major {
            Ok(UpgradeType::Major)
        } else if latest_version.minor > current_version.minor {
            Ok(UpgradeType::Minor)
        } else {
            Ok(UpgradeType::Patch)
        }
    }

    /// Resolves the registry URL for a package.
    ///
    /// Handles scoped packages by checking the scoped_registries configuration.
    /// If .npmrc is loaded, it takes precedence over config settings.
    /// Falls back to the default registry for unscoped packages.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package (may include scope)
    ///
    /// # Returns
    ///
    /// The registry URL to use for this package.
    ///
    /// # Example
    ///
    /// For a package "@myorg/utils" with scoped_registries containing
    /// "myorg" -> "https://npm.myorg.com", returns "https://npm.myorg.com".
    ///
    /// For unscoped package "lodash", returns the default registry.
    pub(crate) fn resolve_registry_url(&self, package_name: &str) -> String {
        // Try .npmrc first if available
        if let Some(npmrc) = &self.npmrc {
            if let Some(registry) = npmrc.resolve_registry(package_name) {
                return registry.to_string();
            }
        }

        // Check if package is scoped (starts with @)
        if let Some(scope) = package_name.strip_prefix('@') {
            // Extract scope name (everything before the first '/')
            if let Some(scope_end) = scope.find('/') {
                let scope_name = &scope[..scope_end];

                // Check if we have a registry configured for this scope
                if let Some(registry) = self.config.scoped_registries.get(scope_name) {
                    return registry.clone();
                }
            }
        }

        // Fall back to default registry
        self.config.default_registry.clone()
    }

    /// Resolves the authentication token for a registry URL.
    ///
    /// Checks the auth_tokens configuration for a matching token.
    /// If .npmrc is loaded, it takes precedence over config settings.
    ///
    /// # Arguments
    ///
    /// * `registry_url` - The registry URL to check
    ///
    /// # Returns
    ///
    /// The authentication token if one is configured, None otherwise.
    pub(crate) fn resolve_auth_token(&self, registry_url: &str) -> Option<String> {
        // Try .npmrc first if available
        if let Some(npmrc) = &self.npmrc {
            if let Some(token) = npmrc.get_auth_token(registry_url) {
                return Some(token.to_string());
            }
        }

        // Try exact match first
        if let Some(token) = self.config.auth_tokens.get(registry_url) {
            return Some(token.clone());
        }

        // Try without trailing slash
        let url_without_slash = registry_url.trim_end_matches('/');
        if let Some(token) = self.config.auth_tokens.get(url_without_slash) {
            return Some(token.clone());
        }

        None
    }

    /// Converts registry response to PackageMetadata.
    ///
    /// Parses the complex NPM registry response into our simpler metadata structure.
    fn convert_to_metadata(
        &self,
        response: RegistryResponse,
        package_name: &str,
    ) -> Result<PackageMetadata, UpgradeError> {
        // Extract latest version from dist-tags
        let latest = response
            .dist_tags
            .get("latest")
            .ok_or_else(|| UpgradeError::InvalidResponse {
                package: package_name.to_string(),
                reason: "No 'latest' dist-tag found".to_string(),
            })?
            .clone();

        // Extract all version strings
        let mut versions: Vec<String> = response.versions.keys().cloned().collect();
        versions.sort_by(|a, b| {
            // Try to parse as semver for proper sorting
            match (Version::parse(a), Version::parse(b)) {
                (Ok(va), Ok(vb)) => va.cmp(&vb),
                _ => a.cmp(b), // Fallback to string comparison
            }
        });

        // Check if any version is deprecated
        let deprecated = response.versions.get(&latest).and_then(|v| v.deprecated.clone());

        // Parse time metadata
        let mut time = HashMap::new();
        for (key, value) in response.time {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&value) {
                time.insert(key, dt.with_timezone(&chrono::Utc));
            }
        }

        Ok(PackageMetadata {
            name: response.name,
            versions,
            latest,
            deprecated,
            time,
            repository: response.repository,
        })
    }
}
