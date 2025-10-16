//! # Mock NPM Registry Implementation
//!
//! This module provides an in-memory mock NPM registry implementation for testing.
//!
//! ## What
//!
//! `MockRegistry` is an in-memory implementation that simulates an NPM registry
//! for testing package upgrade operations without making real network calls.
//!
//! ## How
//!
//! Package metadata is stored in memory. The mock provides methods to add packages,
//! set versions, and query package information.
//!
//! ## Why
//!
//! Mock registry provides:
//! - Fast test execution without network overhead
//! - Predictable test behavior
//! - Easy setup of package scenarios
//! - Ability to test error conditions

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory mock NPM registry for testing
///
/// This struct maintains an in-memory representation of an NPM registry,
/// allowing tests to run without making real network calls.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::common::mocks::MockRegistry;
///
/// let registry = MockRegistry::new();
/// registry.add_package("react", vec!["18.0.0", "18.1.0", "18.2.0"]);
/// let latest = registry.get_latest_version("react");
/// assert_eq!(latest, Some("18.2.0".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct MockRegistry {
    /// Package metadata storage
    packages: Arc<Mutex<HashMap<String, PackageMetadata>>>,
    /// Registry URL
    registry_url: String,
    /// Whether to simulate network errors
    should_fail: Arc<Mutex<bool>>,
}

/// Metadata for a package in the registry
///
/// This struct contains all the information about a package that would
/// be returned by a real NPM registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// The package name
    pub name: String,
    /// All available versions
    pub versions: HashMap<String, VersionMetadata>,
    /// The latest version tag
    pub latest: String,
    /// Whether the package is deprecated
    pub deprecated: Option<String>,
    /// Time information for each version
    pub time: HashMap<String, DateTime<Utc>>,
    /// Repository information
    pub repository: Option<RepositoryInfo>,
}

/// Metadata for a specific version of a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    /// The version string
    pub version: String,
    /// Version description
    pub description: Option<String>,
    /// Dependencies
    pub dependencies: HashMap<String, String>,
    /// Dev dependencies
    pub dev_dependencies: HashMap<String, String>,
    /// Peer dependencies
    pub peer_dependencies: HashMap<String, String>,
    /// Whether this version is deprecated
    pub deprecated: Option<String>,
    /// Published date
    pub published_at: DateTime<Utc>,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository type (e.g., "git")
    #[serde(rename = "type")]
    pub type_: String,
    /// Repository URL
    pub url: String,
}

impl MockRegistry {
    /// Creates a new mock registry
    ///
    /// # Returns
    ///
    /// A new `MockRegistry` instance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            packages: Arc::new(Mutex::new(HashMap::new())),
            registry_url: "https://registry.npmjs.org".to_string(),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Creates a new mock registry with a custom URL
    ///
    /// # Arguments
    ///
    /// * `url` - The registry URL
    ///
    /// # Returns
    ///
    /// A new `MockRegistry` instance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::with_url("https://npm.example.com");
    /// ```
    #[must_use]
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            packages: Arc::new(Mutex::new(HashMap::new())),
            registry_url: url.into(),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Gets the registry URL
    ///
    /// # Returns
    ///
    /// The registry URL
    #[must_use]
    pub fn url(&self) -> &str {
        &self.registry_url
    }

    /// Adds a package with multiple versions
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `versions` - List of version strings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("lodash", vec!["4.17.0", "4.17.21"]);
    /// ```
    pub fn add_package(&self, name: impl Into<String>, versions: Vec<&str>) {
        let name = name.into();
        let mut version_map = HashMap::new();
        let mut time_map = HashMap::new();
        let mut latest = String::new();
        let mut latest_version = Version::new(0, 0, 0);

        for version_str in versions {
            let version = Version::parse(version_str).unwrap_or_else(|_| Version::new(0, 0, 0));
            if version > latest_version {
                latest_version = version;
                latest = version_str.to_string();
            }

            let now = Utc::now();
            let metadata = VersionMetadata {
                version: version_str.to_string(),
                description: Some(format!("Test package {}", name)),
                dependencies: HashMap::new(),
                dev_dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                deprecated: None,
                published_at: now,
            };

            version_map.insert(version_str.to_string(), metadata);
            time_map.insert(version_str.to_string(), now);
        }

        let metadata = PackageMetadata {
            name: name.clone(),
            versions: version_map,
            latest,
            deprecated: None,
            time: time_map,
            repository: None,
        };

        let mut packages = self.packages.lock().unwrap();
        packages.insert(name, metadata);
    }

    /// Adds a package with detailed metadata
    ///
    /// # Arguments
    ///
    /// * `metadata` - The package metadata
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// let metadata = PackageMetadata { /* ... */ };
    /// registry.add_package_metadata(metadata);
    /// ```
    pub fn add_package_metadata(&self, metadata: PackageMetadata) {
        let mut packages = self.packages.lock().unwrap();
        packages.insert(metadata.name.clone(), metadata);
    }

    /// Gets package metadata
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    ///
    /// # Returns
    ///
    /// The package metadata if it exists
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("react", vec!["18.0.0"]);
    /// let metadata = registry.get_package("react");
    /// assert!(metadata.is_some());
    /// ```
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<PackageMetadata> {
        let packages = self.packages.lock().unwrap();
        packages.get(name).cloned()
    }

    /// Gets the latest version of a package
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    ///
    /// # Returns
    ///
    /// The latest version string if the package exists
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("react", vec!["18.0.0", "18.2.0"]);
    /// let latest = registry.get_latest_version("react");
    /// assert_eq!(latest, Some("18.2.0".to_string()));
    /// ```
    #[must_use]
    pub fn get_latest_version(&self, name: &str) -> Option<String> {
        let packages = self.packages.lock().unwrap();
        packages.get(name).map(|p| p.latest.clone())
    }

    /// Gets all versions of a package
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    ///
    /// # Returns
    ///
    /// A vector of all version strings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("react", vec!["18.0.0", "18.2.0"]);
    /// let versions = registry.get_versions("react");
    /// assert_eq!(versions.len(), 2);
    /// ```
    #[must_use]
    pub fn get_versions(&self, name: &str) -> Vec<String> {
        let packages = self.packages.lock().unwrap();
        packages.get(name).map(|p| p.versions.keys().cloned().collect()).unwrap_or_default()
    }

    /// Marks a package as deprecated
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `message` - The deprecation message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("old-pkg", vec!["1.0.0"]);
    /// registry.deprecate_package("old-pkg", "Use new-pkg instead");
    /// ```
    pub fn deprecate_package(&self, name: &str, message: impl Into<String>) {
        let mut packages = self.packages.lock().unwrap();
        if let Some(pkg) = packages.get_mut(name) {
            pkg.deprecated = Some(message.into());
        }
    }

    /// Marks a specific version as deprecated
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `version` - The version to deprecate
    /// * `message` - The deprecation message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("pkg", vec!["1.0.0", "2.0.0"]);
    /// registry.deprecate_version("pkg", "1.0.0", "Use 2.0.0 instead");
    /// ```
    pub fn deprecate_version(&self, name: &str, version: &str, message: impl Into<String>) {
        let mut packages = self.packages.lock().unwrap();
        if let Some(pkg) = packages.get_mut(name) {
            if let Some(ver) = pkg.versions.get_mut(version) {
                ver.deprecated = Some(message.into());
            }
        }
    }

    /// Sets repository information for a package
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `repo_type` - The repository type (e.g., "git")
    /// * `repo_url` - The repository URL
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("pkg", vec!["1.0.0"]);
    /// registry.set_repository("pkg", "git", "https://github.com/user/pkg");
    /// ```
    pub fn set_repository(&self, name: &str, repo_type: &str, repo_url: &str) {
        let mut packages = self.packages.lock().unwrap();
        if let Some(pkg) = packages.get_mut(name) {
            pkg.repository =
                Some(RepositoryInfo { type_: repo_type.to_string(), url: repo_url.to_string() });
        }
    }

    /// Adds dependencies to a specific version
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `version` - The version string
    /// * `dependencies` - Map of dependency names to version specs
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("pkg", vec!["1.0.0"]);
    /// let mut deps = HashMap::new();
    /// deps.insert("lodash".to_string(), "^4.17.0".to_string());
    /// registry.add_dependencies("pkg", "1.0.0", deps);
    /// ```
    pub fn add_dependencies(
        &self,
        name: &str,
        version: &str,
        dependencies: HashMap<String, String>,
    ) {
        let mut packages = self.packages.lock().unwrap();
        if let Some(pkg) = packages.get_mut(name) {
            if let Some(ver) = pkg.versions.get_mut(version) {
                ver.dependencies.extend(dependencies);
            }
        }
    }

    /// Sets whether the registry should simulate failures
    ///
    /// # Arguments
    ///
    /// * `should_fail` - Whether to fail
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.set_should_fail(true);
    /// // Now all operations will fail
    /// ```
    pub fn set_should_fail(&self, should_fail: bool) {
        let mut fail = self.should_fail.lock().unwrap();
        *fail = should_fail;
    }

    /// Checks if the registry is set to fail
    ///
    /// # Returns
    ///
    /// `true` if the registry should simulate failures
    #[must_use]
    pub fn should_fail(&self) -> bool {
        let fail = self.should_fail.lock().unwrap();
        *fail
    }

    /// Clears all packages from the registry
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("pkg", vec!["1.0.0"]);
    /// registry.clear();
    /// assert_eq!(registry.package_count(), 0);
    /// ```
    pub fn clear(&self) {
        let mut packages = self.packages.lock().unwrap();
        packages.clear();
    }

    /// Gets the number of packages in the registry
    ///
    /// # Returns
    ///
    /// The number of packages
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// assert_eq!(registry.package_count(), 0);
    /// ```
    #[must_use]
    pub fn package_count(&self) -> usize {
        let packages = self.packages.lock().unwrap();
        packages.len()
    }

    /// Lists all package names in the registry
    ///
    /// # Returns
    ///
    /// A vector of all package names
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = MockRegistry::new();
    /// registry.add_package("react", vec!["18.0.0"]);
    /// registry.add_package("vue", vec!["3.0.0"]);
    /// let names = registry.list_packages();
    /// assert_eq!(names.len(), 2);
    /// ```
    #[must_use]
    pub fn list_packages(&self) -> Vec<String> {
        let packages = self.packages.lock().unwrap();
        packages.keys().cloned().collect()
    }
}

impl Default for MockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = MockRegistry::new();
        assert_eq!(registry.package_count(), 0);
    }

    #[test]
    fn test_add_package() {
        let registry = MockRegistry::new();
        registry.add_package("react", vec!["18.0.0", "18.1.0", "18.2.0"]);

        assert_eq!(registry.package_count(), 1);
        let versions = registry.get_versions("react");
        assert_eq!(versions.len(), 3);
    }

    #[test]
    fn test_get_latest_version() {
        let registry = MockRegistry::new();
        registry.add_package("react", vec!["18.0.0", "18.2.0", "18.1.0"]);

        let latest = registry.get_latest_version("react");
        assert_eq!(latest, Some("18.2.0".to_string()));
    }

    #[test]
    fn test_get_package() {
        let registry = MockRegistry::new();
        registry.add_package("vue", vec!["3.0.0", "3.1.0"]);

        let metadata = registry.get_package("vue");
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "vue");
        assert_eq!(metadata.versions.len(), 2);
    }

    #[test]
    fn test_deprecate_package() {
        let registry = MockRegistry::new();
        registry.add_package("old-pkg", vec!["1.0.0"]);
        registry.deprecate_package("old-pkg", "Use new-pkg instead");

        let metadata = registry.get_package("old-pkg").unwrap();
        assert!(metadata.deprecated.is_some());
        assert_eq!(metadata.deprecated.unwrap(), "Use new-pkg instead");
    }

    #[test]
    fn test_deprecate_version() {
        let registry = MockRegistry::new();
        registry.add_package("pkg", vec!["1.0.0", "2.0.0"]);
        registry.deprecate_version("pkg", "1.0.0", "Use 2.0.0");

        let metadata = registry.get_package("pkg").unwrap();
        let v1 = metadata.versions.get("1.0.0").unwrap();
        let v2 = metadata.versions.get("2.0.0").unwrap();

        assert!(v1.deprecated.is_some());
        assert!(v2.deprecated.is_none());
    }

    #[test]
    fn test_set_repository() {
        let registry = MockRegistry::new();
        registry.add_package("pkg", vec!["1.0.0"]);
        registry.set_repository("pkg", "git", "https://github.com/user/pkg");

        let metadata = registry.get_package("pkg").unwrap();
        assert!(metadata.repository.is_some());

        let repo = metadata.repository.unwrap();
        assert_eq!(repo.type_, "git");
        assert_eq!(repo.url, "https://github.com/user/pkg");
    }

    #[test]
    fn test_add_dependencies() {
        let registry = MockRegistry::new();
        registry.add_package("pkg", vec!["1.0.0"]);

        let mut deps = HashMap::new();
        deps.insert("lodash".to_string(), "^4.17.0".to_string());
        deps.insert("react".to_string(), "^18.0.0".to_string());

        registry.add_dependencies("pkg", "1.0.0", deps);

        let metadata = registry.get_package("pkg").unwrap();
        let version = metadata.versions.get("1.0.0").unwrap();
        assert_eq!(version.dependencies.len(), 2);
    }

    #[test]
    fn test_should_fail() {
        let registry = MockRegistry::new();
        assert!(!registry.should_fail());

        registry.set_should_fail(true);
        assert!(registry.should_fail());

        registry.set_should_fail(false);
        assert!(!registry.should_fail());
    }

    #[test]
    fn test_clear() {
        let registry = MockRegistry::new();
        registry.add_package("pkg1", vec!["1.0.0"]);
        registry.add_package("pkg2", vec!["1.0.0"]);

        assert_eq!(registry.package_count(), 2);

        registry.clear();
        assert_eq!(registry.package_count(), 0);
    }

    #[test]
    fn test_list_packages() {
        let registry = MockRegistry::new();
        registry.add_package("react", vec!["18.0.0"]);
        registry.add_package("vue", vec!["3.0.0"]);
        registry.add_package("angular", vec!["15.0.0"]);

        let packages = registry.list_packages();
        assert_eq!(packages.len(), 3);
        assert!(packages.contains(&"react".to_string()));
        assert!(packages.contains(&"vue".to_string()));
        assert!(packages.contains(&"angular".to_string()));
    }

    #[test]
    fn test_custom_url() {
        let registry = MockRegistry::with_url("https://npm.example.com");
        assert_eq!(registry.url(), "https://npm.example.com");
    }
}
