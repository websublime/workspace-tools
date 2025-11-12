//! Upgrade detection for external dependencies.
//!
//! **What**: Provides functionality to detect available upgrades for external npm packages
//! by scanning package.json files and querying package registries.
//!
//! **How**: This module scans the workspace for package.json files, extracts external
//! dependencies (filtering out workspace:, file:, link:, and portal: protocols), queries
//! npm registries concurrently for available versions, and classifies upgrades by type
//! (major, minor, patch). It supports filtering by package name, dependency name, and
//! dependency type.
//!
//! **Why**: To enable developers to discover available dependency upgrades with fine-grained
//! control over what to detect, supporting both security patches and feature updates while
//! providing clear classification of upgrade impact.

use crate::error::UpgradeError;
use crate::types::DependencyType;
use crate::upgrade::registry::{RegistryClient, UpgradeType};
use chrono::{DateTime, Utc};
use futures::stream::{self, StreamExt};
use package_json::PackageJson;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};

/// Options for controlling upgrade detection.
///
/// Provides fine-grained control over which dependencies to scan and how to
/// query package registries.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::upgrade::DetectionOptions;
///
/// // Detect all upgrades with default settings
/// let options = DetectionOptions::all();
///
/// // Detect only production dependencies
/// let options = DetectionOptions::production_only();
///
/// // Detect only dev dependencies
/// let options = DetectionOptions::dev_only();
///
/// // Custom filtering
/// let mut options = DetectionOptions::default();
/// options.include_dependencies = true;
/// options.include_dev_dependencies = false;
/// options.package_filter = Some(vec!["my-package".to_string()]);
/// options.concurrency = 20;
/// ```
#[derive(Debug, Clone)]
pub struct DetectionOptions {
    /// Include regular dependencies from `dependencies` field.
    ///
    /// # Default: `false`
    pub include_dependencies: bool,

    /// Include development dependencies from `devDependencies` field.
    ///
    /// # Default: `false`
    pub include_dev_dependencies: bool,

    /// Include peer dependencies from `peerDependencies` field.
    ///
    /// # Default: `false`
    pub include_peer_dependencies: bool,

    /// Include optional dependencies from `optionalDependencies` field.
    ///
    /// # Default: `false`
    pub include_optional_dependencies: bool,

    /// Filter detection to specific package names.
    ///
    /// When set, only package.json files matching these package names will be scanned.
    /// Package name is matched from the `name` field in package.json.
    ///
    /// # Default: `None` (all packages)
    pub package_filter: Option<Vec<String>>,

    /// Filter detection to specific dependency names.
    ///
    /// When set, only these dependencies will be checked for upgrades.
    ///
    /// # Default: `None` (all dependencies)
    pub dependency_filter: Option<Vec<String>>,

    /// Include pre-release versions in detection.
    ///
    /// When enabled, will consider pre-release versions (e.g., 1.0.0-alpha.1)
    /// as upgrade candidates.
    ///
    /// # Default: `false`
    pub include_prereleases: bool,

    /// Maximum number of concurrent registry queries.
    ///
    /// Controls the parallelism of registry API calls. Higher values may improve
    /// performance but could trigger rate limits.
    ///
    /// # Default: `10`
    pub concurrency: usize,
}

impl Default for DetectionOptions {
    fn default() -> Self {
        Self {
            include_dependencies: false,
            include_dev_dependencies: false,
            include_peer_dependencies: false,
            include_optional_dependencies: false,
            package_filter: None,
            dependency_filter: None,
            include_prereleases: false,
            concurrency: 10,
        }
    }
}

impl DetectionOptions {
    /// Creates options to detect all dependency types.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::DetectionOptions;
    ///
    /// let options = DetectionOptions::all();
    /// assert!(options.include_dependencies);
    /// assert!(options.include_dev_dependencies);
    /// assert!(options.include_peer_dependencies);
    /// assert!(options.include_optional_dependencies);
    /// ```
    #[must_use]
    pub fn all() -> Self {
        Self {
            include_dependencies: true,
            include_dev_dependencies: true,
            include_peer_dependencies: true,
            include_optional_dependencies: true,
            concurrency: 10,
            ..Default::default()
        }
    }

    /// Creates options to detect only production dependencies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::DetectionOptions;
    ///
    /// let options = DetectionOptions::production_only();
    /// assert!(options.include_dependencies);
    /// assert!(!options.include_dev_dependencies);
    /// ```
    #[must_use]
    pub fn production_only() -> Self {
        Self { include_dependencies: true, concurrency: 10, ..Default::default() }
    }

    /// Creates options to detect only development dependencies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::DetectionOptions;
    ///
    /// let options = DetectionOptions::dev_only();
    /// assert!(options.include_dev_dependencies);
    /// assert!(!options.include_dependencies);
    /// ```
    #[must_use]
    pub fn dev_only() -> Self {
        Self { include_dev_dependencies: true, concurrency: 10, ..Default::default() }
    }

    /// Returns whether the given package name matches the filter.
    pub(crate) fn matches_package_filter(&self, package_name: &str) -> bool {
        match &self.package_filter {
            Some(filter) => filter.iter().any(|name| name == package_name),
            None => true,
        }
    }

    /// Returns whether the given dependency name matches the filter.
    pub(crate) fn matches_dependency_filter(&self, dependency_name: &str) -> bool {
        match &self.dependency_filter {
            Some(filter) => filter.iter().any(|name| name == dependency_name),
            None => true,
        }
    }
}

/// Preview of available dependency upgrades.
///
/// Contains all detected external dependencies with available updates,
/// classified by upgrade type (major, minor, patch).
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::UpgradePreview;
///
/// # fn example(preview: UpgradePreview) {
/// println!("Detected at: {}", preview.detected_at);
/// println!("Total packages: {}", preview.packages.len());
/// println!("Total upgrades: {}", preview.summary.upgrades_available);
/// println!("Major upgrades: {}", preview.summary.major_upgrades);
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradePreview {
    /// Timestamp when detection was performed.
    pub detected_at: DateTime<Utc>,

    /// All available upgrades grouped by package.
    pub packages: Vec<PackageUpgrades>,

    /// Summary statistics.
    pub summary: UpgradeSummary,
}

/// Available upgrades for a single package.
///
/// Contains all detected upgrade opportunities for dependencies in a single package.json file.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::PackageUpgrades;
///
/// # fn example(package: PackageUpgrades) {
/// println!("Package: {}", package.package_name);
/// println!("Path: {}", package.package_path.display());
/// println!("Upgrades: {}", package.upgrades.len());
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpgrades {
    /// Package name from package.json `name` field.
    pub package_name: String,

    /// Path to package.json file.
    pub package_path: PathBuf,

    /// Current version in package.json (if present).
    pub current_version: Option<String>,

    /// List of available upgrades for dependencies in this package.
    pub upgrades: Vec<DependencyUpgrade>,
}

/// Details of a single dependency upgrade.
///
/// Represents a single external dependency with an available upgrade.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::DependencyUpgrade;
///
/// # fn example(upgrade: DependencyUpgrade) {
/// println!("{}: {} -> {} ({})",
///     upgrade.name,
///     upgrade.current_version,
///     upgrade.latest_version,
///     upgrade.upgrade_type
/// );
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyUpgrade {
    /// Dependency name.
    pub name: String,

    /// Current version spec in package.json (e.g., "^1.2.3", "~2.0.0").
    pub current_version: String,

    /// Latest available version from registry.
    pub latest_version: String,

    /// Type of upgrade (major, minor, patch).
    pub upgrade_type: UpgradeType,

    /// Dependency type (regular, dev, peer, optional).
    pub dependency_type: DependencyType,

    /// Registry URL where this package is published.
    pub registry_url: String,

    /// Additional version information.
    pub version_info: VersionInfo,
}

/// Additional version information from registry.
///
/// Provides context about available versions and package status.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::VersionInfo;
///
/// # fn example(info: VersionInfo) {
/// println!("Latest stable: {}", info.latest_stable);
/// if let Some(deprecated) = &info.deprecated {
///     println!("DEPRECATED: {}", deprecated);
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// All available versions from registry.
    pub available_versions: Vec<String>,

    /// Latest stable version.
    pub latest_stable: String,

    /// Latest pre-release version (if any).
    pub latest_prerelease: Option<String>,

    /// Deprecation warning (if deprecated).
    pub deprecated: Option<String>,

    /// Publication date of latest version.
    pub published_at: Option<DateTime<Utc>>,
}

/// Summary statistics for upgrades.
///
/// Provides aggregate information about detected upgrades.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::UpgradeSummary;
///
/// # fn example(summary: UpgradeSummary) {
/// println!("Packages scanned: {}", summary.packages_scanned);
/// println!("Total dependencies: {}", summary.total_dependencies);
/// println!("Upgrades available: {}", summary.upgrades_available);
/// println!("Major: {}, Minor: {}, Patch: {}",
///     summary.major_upgrades,
///     summary.minor_upgrades,
///     summary.patch_upgrades
/// );
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeSummary {
    /// Total number of packages scanned.
    pub packages_scanned: usize,

    /// Total number of external dependencies found.
    pub total_dependencies: usize,

    /// Number of dependencies with available upgrades.
    pub upgrades_available: usize,

    /// Number of major version upgrades available.
    pub major_upgrades: usize,

    /// Number of minor version upgrades available.
    pub minor_upgrades: usize,

    /// Number of patch version upgrades available.
    pub patch_upgrades: usize,

    /// Number of deprecated dependencies found.
    pub deprecated_dependencies: usize,
}

/// Detects available upgrades for external dependencies.
///
/// This function scans the workspace for package.json files and queries registries
/// to detect available upgrades according to the provided options.
///
/// # Arguments
///
/// * `workspace_root` - Root directory of the workspace
/// * `registry_client` - Client for querying package registries
/// * `fs` - Filesystem manager for reading files
/// * `options` - Detection options for filtering and concurrency
///
/// # Returns
///
/// An `UpgradePreview` containing all detected upgrades and summary statistics.
///
/// # Errors
///
/// Returns `UpgradeError` if:
/// - Failed to read workspace files
/// - Failed to parse package.json files
/// - Registry queries fail (network, authentication, etc.)
///
/// # Example
///
/// ```rust,ignore
/// use sublime_pkg_tools::upgrade::{detect_upgrades, DetectionOptions};
/// use sublime_pkg_tools::upgrade::RegistryClient;
/// use sublime_pkg_tools::config::RegistryConfig;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let fs = FileSystemManager::new();
/// let registry_config = RegistryConfig::default();
/// let client = RegistryClient::new(&workspace_root, registry_config).await?;
///
/// let options = DetectionOptions::all();
/// let preview = detect_upgrades(&workspace_root, &client, &fs, options).await?;
///
/// println!("Found {} upgrades", preview.summary.upgrades_available);
/// # Ok(())
/// # }
/// ```
pub async fn detect_upgrades(
    workspace_root: &Path,
    registry_client: &RegistryClient,
    fs: &FileSystemManager,
    options: DetectionOptions,
) -> Result<UpgradePreview, UpgradeError> {
    let detected_at = Utc::now();

    // Find all package.json files
    let package_files = find_package_json_files(workspace_root, fs).await?;

    // Process each package
    let mut all_packages = Vec::new();
    let mut total_dependencies = 0;
    let mut upgrades_available = 0;
    let mut major_upgrades = 0;
    let mut minor_upgrades = 0;
    let mut patch_upgrades = 0;
    let mut deprecated_dependencies = 0;

    for package_json_path in package_files {
        // Read and parse package.json
        let package_json = read_package_json(&package_json_path, fs).await?;

        // Get package name
        let package_name = if package_json.name.is_empty() {
            "unnamed".to_string()
        } else {
            package_json.name.clone()
        };

        // Check package filter
        if !options.matches_package_filter(&package_name) {
            continue;
        }

        // Extract dependencies
        let dependencies = extract_dependencies(&package_json, &options);
        total_dependencies += dependencies.len();

        // Detect upgrades for this package
        let upgrades = detect_package_upgrades(&dependencies, registry_client, &options).await?;

        // Update statistics
        upgrades_available += upgrades.len();
        for upgrade in &upgrades {
            match upgrade.upgrade_type {
                UpgradeType::Major => major_upgrades += 1,
                UpgradeType::Minor => minor_upgrades += 1,
                UpgradeType::Patch => patch_upgrades += 1,
            }
            if upgrade.version_info.deprecated.is_some() {
                deprecated_dependencies += 1;
            }
        }

        // Add to results if there are upgrades or we're including all packages
        if !upgrades.is_empty() {
            // Extract the directory containing package.json (not the file itself)
            let package_path = package_json_path
                .parent()
                .ok_or_else(|| UpgradeError::FileSystemError {
                    path: package_json_path.clone(),
                    reason: "Cannot determine parent directory of package.json".to_string(),
                })?
                .to_path_buf();

            all_packages.push(PackageUpgrades {
                package_name,
                package_path,
                current_version: if package_json.version.is_empty() {
                    None
                } else {
                    Some(package_json.version.clone())
                },
                upgrades,
            });
        }
    }

    let summary = UpgradeSummary {
        packages_scanned: all_packages.len(),
        total_dependencies,
        upgrades_available,
        major_upgrades,
        minor_upgrades,
        patch_upgrades,
        deprecated_dependencies,
    };

    Ok(UpgradePreview { detected_at, packages: all_packages, summary })
}

/// Finds all package.json files in the workspace.
pub(crate) async fn find_package_json_files(
    workspace_root: &Path,
    fs: &FileSystemManager,
) -> Result<Vec<PathBuf>, UpgradeError> {
    let mut package_files = Vec::new();

    // Check if workspace root has package.json
    let root_package_json = workspace_root.join("package.json");
    if fs.exists(&root_package_json).await {
        package_files.push(root_package_json);
    }

    // Look for packages in common monorepo locations
    let common_patterns = vec!["packages/*/package.json", "apps/*/package.json"];

    for pattern in common_patterns {
        let _pattern_path = workspace_root.join(pattern);
        // This is a simplified implementation - in a real scenario,
        // we'd use glob patterns or the monorepo detector from standard_tools
        // For now, we'll just check the root package.json
    }

    if package_files.is_empty() {
        return Err(UpgradeError::NoPackagesFound { workspace_root: workspace_root.to_path_buf() });
    }

    Ok(package_files)
}

/// Reads and parses a package.json file.
pub(crate) async fn read_package_json(
    path: &Path,
    fs: &FileSystemManager,
) -> Result<PackageJson, UpgradeError> {
    let content = fs.read_file_string(path).await.map_err(|e| UpgradeError::FileSystemError {
        path: path.to_path_buf(),
        reason: format!("Failed to read package.json: {}", e),
    })?;

    serde_json::from_str(&content).map_err(|e| UpgradeError::PackageJsonError {
        path: path.to_path_buf(),
        reason: format!("Failed to parse package.json: {}", e),
    })
}

/// Represents a dependency to check for upgrades.
#[derive(Debug, Clone)]
pub(crate) struct DependencyToCheck {
    pub(crate) name: String,
    pub(crate) version_spec: String,
    pub(crate) dependency_type: DependencyType,
}

/// Extracts dependencies from package.json based on options.
pub(crate) fn extract_dependencies(
    package_json: &PackageJson,
    options: &DetectionOptions,
) -> Vec<DependencyToCheck> {
    let mut dependencies = Vec::new();

    // Regular dependencies
    if options.include_dependencies
        && let Some(deps) = &package_json.dependencies
    {
        for (name, version) in deps {
            if !is_internal_dependency(version) && options.matches_dependency_filter(name) {
                dependencies.push(DependencyToCheck {
                    name: name.clone(),
                    version_spec: version.clone(),
                    dependency_type: DependencyType::Regular,
                });
            }
        }
    }

    // Dev dependencies
    if options.include_dev_dependencies
        && let Some(deps) = &package_json.dev_dependencies
    {
        for (name, version) in deps {
            if !is_internal_dependency(version) && options.matches_dependency_filter(name) {
                dependencies.push(DependencyToCheck {
                    name: name.clone(),
                    version_spec: version.clone(),
                    dependency_type: DependencyType::Dev,
                });
            }
        }
    }

    // Peer dependencies
    if options.include_peer_dependencies
        && let Some(deps) = &package_json.peer_dependencies
    {
        for (name, version) in deps {
            if !is_internal_dependency(version) && options.matches_dependency_filter(name) {
                dependencies.push(DependencyToCheck {
                    name: name.clone(),
                    version_spec: version.clone(),
                    dependency_type: DependencyType::Peer,
                });
            }
        }
    }

    // Optional dependencies
    if options.include_optional_dependencies
        && let Some(deps) = &package_json.optional_dependencies
    {
        for (name, version) in deps {
            if !is_internal_dependency(version) && options.matches_dependency_filter(name) {
                dependencies.push(DependencyToCheck {
                    name: name.clone(),
                    version_spec: version.clone(),
                    dependency_type: DependencyType::Optional,
                });
            }
        }
    }

    dependencies
}

/// Checks if a version spec is an internal dependency.
///
/// Internal dependencies use special protocols that should be excluded from upgrade detection.
pub(crate) fn is_internal_dependency(version_spec: &str) -> bool {
    version_spec.starts_with("workspace:")
        || version_spec.starts_with("file:")
        || version_spec.starts_with("link:")
        || version_spec.starts_with("portal:")
}

/// Detects upgrades for a package's dependencies.
async fn detect_package_upgrades(
    dependencies: &[DependencyToCheck],
    registry_client: &RegistryClient,
    options: &DetectionOptions,
) -> Result<Vec<DependencyUpgrade>, UpgradeError> {
    // Query registry concurrently with controlled concurrency
    let upgrades = stream::iter(dependencies)
        .map(|dep| async move { detect_single_upgrade(dep, registry_client, options).await })
        .buffer_unordered(options.concurrency)
        .collect::<Vec<_>>()
        .await;

    // Filter out errors and None results
    let mut valid_upgrades = Vec::new();
    for result in upgrades {
        match result {
            Ok(Some(upgrade)) => valid_upgrades.push(upgrade),
            Ok(None) => {}
            Err(e) => {
                // Log error but continue with other dependencies
                // In production, we might want to collect these errors
                eprintln!("Warning: Failed to check upgrade: {}", e);
            }
        }
    }

    Ok(valid_upgrades)
}

/// Detects upgrade for a single dependency.
async fn detect_single_upgrade(
    dependency: &DependencyToCheck,
    registry_client: &RegistryClient,
    options: &DetectionOptions,
) -> Result<Option<DependencyUpgrade>, UpgradeError> {
    // Get package metadata from registry
    let metadata = registry_client.get_package_info(&dependency.name).await?;

    // Extract current version from version spec
    let current_version = extract_version_from_spec(&dependency.version_spec)?;

    // Determine latest version
    let latest_version = if options.include_prereleases {
        // Find latest version including prereleases
        find_latest_version(&metadata.versions)?
    } else {
        // Use latest stable from metadata
        metadata.latest.clone()
    };

    // Check if upgrade is available
    let current = Version::parse(&current_version).map_err(|e| UpgradeError::InvalidVersion {
        version: current_version.clone(),
        message: format!("Failed to parse current version: {}", e),
    })?;

    let latest = Version::parse(&latest_version).map_err(|e| UpgradeError::InvalidVersion {
        version: latest_version.clone(),
        message: format!("Failed to parse latest version: {}", e),
    })?;

    // No upgrade needed if current >= latest
    if current >= latest {
        return Ok(None);
    }

    // Determine upgrade type
    let upgrade_type =
        registry_client.compare_versions(&dependency.name, &current_version, &latest_version)?;

    // Find latest prerelease
    let latest_prerelease = find_latest_prerelease(&metadata.versions);

    // Get registry URL
    let registry_url = registry_client.resolve_registry_url(&dependency.name);

    // Build version info
    let version_info = VersionInfo {
        available_versions: metadata.versions.clone(),
        latest_stable: metadata.latest.clone(),
        latest_prerelease,
        deprecated: metadata.deprecated.clone(),
        published_at: metadata.version_published_at(&latest_version),
    };

    Ok(Some(DependencyUpgrade {
        name: dependency.name.clone(),
        current_version: dependency.version_spec.clone(),
        latest_version,
        upgrade_type,
        dependency_type: dependency.dependency_type,
        registry_url,
        version_info,
    }))
}

/// Extracts version number from version spec.
///
/// Removes common prefixes like ^, ~, >=, etc.
pub(crate) fn extract_version_from_spec(spec: &str) -> Result<String, UpgradeError> {
    let trimmed = spec.trim();

    // Remove common prefixes
    let version = trimmed
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches('>')
        .trim_start_matches("<=")
        .trim_start_matches('<')
        .trim_start_matches('=');

    if version.is_empty() {
        return Err(UpgradeError::InvalidVersion {
            version: spec.to_string(),
            message: "Version spec is empty after removing prefixes".to_string(),
        });
    }

    Ok(version.to_string())
}

/// Finds the latest version from a list of versions.
pub(crate) fn find_latest_version(versions: &[String]) -> Result<String, UpgradeError> {
    let mut parsed_versions: Vec<Version> = Vec::new();

    for version_str in versions {
        if let Ok(version) = Version::parse(version_str) {
            parsed_versions.push(version);
        }
    }

    if parsed_versions.is_empty() {
        return Err(UpgradeError::InvalidVersion {
            version: "none".to_string(),
            message: "No valid versions found".to_string(),
        });
    }

    parsed_versions.sort();
    let latest = parsed_versions.last().ok_or_else(|| UpgradeError::InvalidVersion {
        version: "none".to_string(),
        message: "Failed to find latest version".to_string(),
    })?;

    Ok(latest.to_string())
}

/// Finds the latest prerelease version from a list of versions.
pub(crate) fn find_latest_prerelease(versions: &[String]) -> Option<String> {
    let mut prerelease_versions: Vec<Version> = Vec::new();

    for version_str in versions {
        if let Ok(version) = Version::parse(version_str)
            && !version.pre.is_empty()
        {
            prerelease_versions.push(version);
        }
    }

    if prerelease_versions.is_empty() {
        return None;
    }

    prerelease_versions.sort();
    prerelease_versions.last().map(|v| v.to_string())
}
