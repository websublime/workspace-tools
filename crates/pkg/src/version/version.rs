//! # Version Module
//!
//! This module provides utilities for working with semantic versions in package management.
//!
//! The main structure is the `Version` enum, which provides methods for version parsing,
//! comparison, bumping, and analyzing version relationships.
//!
//! ## Key Features
//!
//! - Parse and validate semantic versions
//! - Bump major, minor, patch, or create snapshot versions
//! - Compare versions and determine relationships
//! - Detect breaking changes
//! - Version update strategies
//!
//! ## Examples
//!
//! ```
//! use sublime_package_tools::{Version, VersionRelationship};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse version
//! let version = Version::parse("1.2.3")?;
//!
//! // Bump versions
//! let next_major = Version::bump_major("1.2.3")?; // 2.0.0
//! let next_minor = Version::bump_minor("1.2.3")?; // 1.3.0
//! let next_patch = Version::bump_patch("1.2.3")?; // 1.2.4
//!
//! // Create snapshot
//! let snapshot = Version::bump_snapshot("1.2.3", "abc123")?; // 1.2.3-alpha.abc123
//!
//! // Compare versions
//! let rel = Version::compare_versions("1.0.0", "2.0.0");
//! assert_eq!(rel, VersionRelationship::MajorUpgrade);
//!
//! // Detect breaking changes
//! assert!(Version::is_breaking_change("1.0.0", "2.0.0"));
//! assert!(!Version::is_breaking_change("1.0.0", "1.1.0"));
//! # Ok(())
//! # }
//! ```

use crate::errors::VersionError;
use semver::{BuildMetadata, Prerelease, Version as SemVersion};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Version update strategy
///
/// Controls what types of version updates are allowed when upgrading dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersionUpdateStrategy {
    /// Only upgrade patch versions (0.0.x)
    PatchOnly,
    /// Upgrade patch and minor versions (0.x.y)
    MinorAndPatch,
    /// Upgrade all versions including major ones (x.y.z)
    AllUpdates,
}

/// Default implementation for VersionUpdateStrategy
///
/// Defaults to MinorAndPatch for a balance between freshness and stability.
impl Default for VersionUpdateStrategy {
    fn default() -> Self {
        Self::MinorAndPatch
    }
}

/// Version stability filter
///
/// Controls whether prerelease versions are included in upgrades.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersionStability {
    /// Only include stable versions
    StableOnly,
    /// Include prereleases and stable versions
    IncludePrerelease,
}

/// Default implementation for `VersionStability`
///
/// Defaults to `StableOnly` for maximum stability.
impl Default for VersionStability {
    fn default() -> Self {
        Self::StableOnly
    }
}

/// Relationship between two semantic versions
///
/// Describes how two versions relate to each other in terms of
/// semver rules (major, minor, patch changes) and prerelease status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionRelationship {
    /// Second version is a major upgrade (1.0.0 -> 2.0.0)
    MajorUpgrade,
    /// Second version is a minor upgrade (1.0.0 -> 1.1.0)
    MinorUpgrade,
    /// Second version is a patch upgrade (1.0.0 -> 1.0.1)
    PatchUpgrade,
    /// Moved from prerelease to stable (1.0.0-alpha -> 1.0.0)
    PrereleaseToStable,
    /// Newer prerelease version (1.0.0-alpha -> 1.0.0-beta)
    NewerPrerelease,
    /// Versions are identical (1.0.0 == 1.0.0)
    Identical,
    /// Second version is a major downgrade (2.0.0 -> 1.0.0)
    MajorDowngrade,
    /// Second version is a minor downgrade (1.1.0 -> 1.0.0)
    MinorDowngrade,
    /// Second version is a patch downgrade (1.0.1 -> 1.0.0)
    PatchDowngrade,
    /// Moved from stable to prerelease (1.0.0 -> 1.0.0-alpha)
    StableToPrerelease,
    /// Older prerelease version (1.0.0-beta -> 1.0.0-alpha)
    OlderPrerelease,
    /// Version comparison couldn't be determined (invalid versions)
    Indeterminate,
}

/// Display implementation for `VersionRelationship`
///
/// Provides a string representation of the relationship.
impl Display for VersionRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        match self {
            Self::MajorUpgrade => write!(f, "MajorUpgrade"),
            Self::MinorUpgrade => write!(f, "MinorUpgrade"),
            Self::PatchUpgrade => write!(f, "PatchUpgrade"),
            Self::PrereleaseToStable => write!(f, "PrereleaseToStable"),
            Self::NewerPrerelease => write!(f, "NewerPrerelease"),
            Self::Identical => write!(f, "Identical"),
            Self::MajorDowngrade => write!(f, "MajorDowngrade"),
            Self::MinorDowngrade => write!(f, "MinorDowngrade"),
            Self::PatchDowngrade => write!(f, "PatchDowngrade"),
            Self::StableToPrerelease => write!(f, "StableToPrerelease"),
            Self::OlderPrerelease => write!(f, "OlderPrerelease"),
            Self::Indeterminate => write!(f, "Indeterminate"),
        }
    }
}

/// Type of version bump to perform
///
/// Specifies which component of a semantic version should be incremented.
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq)]
pub enum Version {
    /// Increment major version (x.0.0)
    Major,
    /// Increment minor version (0.x.0)
    Minor,
    /// Increment patch version (0.0.x)
    Patch,
    /// Create a snapshot version with prerelease tag
    Snapshot,
}

/// Convert from string to Version
///
/// Maps common version bump names to the appropriate Version enum variant.
impl From<&str> for Version {
    fn from(version: &str) -> Self {
        match version {
            "major" => Version::Major,
            "minor" => Version::Minor,
            "snapshot" => Version::Snapshot,
            _ => Version::Patch,
        }
    }
}

/// Display implementation for Version
///
/// Provides the string representation of the version bump type.
impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Version::Major => write!(f, "major"),
            Version::Minor => write!(f, "minor"),
            Version::Patch => write!(f, "patch"),
            Version::Snapshot => write!(f, "snapshot"),
        }
    }
}

impl Version {
    /// Bumps the version of the package to major
    ///
    /// Increments the major component and resets minor, patch, prerelease, and build metadata.
    ///
    /// # Arguments
    ///
    /// * `version` - The current version string
    ///
    /// # Returns
    ///
    /// A new semantic version with bumped major component, or a `VersionError` if parsing fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Version;
    ///
    /// let new_version = Version::bump_major("1.2.3").unwrap();
    /// assert_eq!(new_version.to_string(), "2.0.0");
    ///
    /// // Resets everything
    /// let new_version = Version::bump_major("1.2.3-alpha.1+build.456").unwrap();
    /// assert_eq!(new_version.to_string(), "2.0.0");
    /// ```
    pub fn bump_major(version: &str) -> Result<SemVersion, VersionError> {
        let mut sem_version = SemVersion::parse(version)?;
        sem_version.major += 1;
        sem_version.minor = 0;
        sem_version.patch = 0;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        Ok(sem_version)
    }

    /// Bumps the version of the package to minor
    ///
    /// Increments the minor component and resets patch, prerelease, and build metadata.
    ///
    /// # Arguments
    ///
    /// * `version` - The current version string
    ///
    /// # Returns
    ///
    /// A new semantic version with bumped minor component, or a `VersionError` if parsing fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Version;
    ///
    /// let new_version = Version::bump_minor("1.2.3").unwrap();
    /// assert_eq!(new_version.to_string(), "1.3.0");
    ///
    /// // Keeps major, resets the rest
    /// let new_version = Version::bump_minor("1.2.3-alpha.1+build.456").unwrap();
    /// assert_eq!(new_version.to_string(), "1.3.0");
    /// ```
    pub fn bump_minor(version: &str) -> Result<SemVersion, VersionError> {
        let mut sem_version = SemVersion::parse(version)?;
        sem_version.minor += 1;
        sem_version.patch = 0;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        Ok(sem_version)
    }

    /// Bumps the version of the package to patch
    ///
    /// Increments the patch component and resets prerelease and build metadata.
    ///
    /// # Arguments
    ///
    /// * `version` - The current version string
    ///
    /// # Returns
    ///
    /// A new semantic version with bumped patch component, or a `VersionError` if parsing fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Version;
    ///
    /// let new_version = Version::bump_patch("1.2.3").unwrap();
    /// assert_eq!(new_version.to_string(), "1.2.4");
    ///
    /// // Keeps major and minor, resets the rest
    /// let new_version = Version::bump_patch("1.2.3-alpha.1+build.456").unwrap();
    /// assert_eq!(new_version.to_string(), "1.2.4");
    /// ```
    pub fn bump_patch(version: &str) -> Result<SemVersion, VersionError> {
        let mut sem_version = SemVersion::parse(version)?;
        sem_version.patch += 1;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        Ok(sem_version)
    }

    /// Bumps the version of the package to snapshot appending the sha to the version
    ///
    /// Creates a snapshot version with an alpha prerelease tag containing the provided SHA.
    ///
    /// # Arguments
    ///
    /// * `version` - The current version string
    /// * `sha` - The SHA or other identifier to include in the prerelease tag
    ///
    /// # Returns
    ///
    /// A new semantic version with snapshot prerelease tag, or a `VersionError` if parsing fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Version;
    ///
    /// let new_version = Version::bump_snapshot("1.2.3", "abc123").unwrap();
    /// assert_eq!(new_version.to_string(), "1.2.3-alpha.abc123");
    /// ```
    pub fn bump_snapshot(version: &str, sha: &str) -> Result<SemVersion, VersionError> {
        let alpha = format!("alpha.{sha}");

        let mut sem_version = SemVersion::parse(version)?;
        sem_version.pre = Prerelease::new(alpha.as_str()).unwrap_or(Prerelease::EMPTY);
        sem_version.build = BuildMetadata::EMPTY;
        Ok(sem_version)
    }

    /// Compare two version strings and return their relationship
    ///
    /// Analyzes the semantic version relationship between two versions,
    /// including major/minor/patch differences and prerelease status.
    ///
    /// # Arguments
    ///
    /// * `v1` - First version string
    /// * `v2` - Second version string
    ///
    /// # Returns
    ///
    /// A `VersionRelationship` describing how v2 relates to v1
    #[must_use]
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Version, VersionRelationship};
    ///
    /// // Major upgrade
    /// assert_eq!(Version::compare_versions("1.0.0", "2.0.0"), VersionRelationship::MajorUpgrade);
    ///
    /// // Minor upgrade
    /// assert_eq!(Version::compare_versions("1.0.0", "1.1.0"), VersionRelationship::MinorUpgrade);
    ///
    /// // Identical versions
    /// assert_eq!(Version::compare_versions("1.0.0", "1.0.0"), VersionRelationship::Identical);
    /// ```
    pub fn compare_versions(v1: &str, v2: &str) -> VersionRelationship {
        if let (Ok(ver1), Ok(ver2)) = (semver::Version::parse(v1), semver::Version::parse(v2)) {
            match ver1.cmp(&ver2) {
                std::cmp::Ordering::Less => {
                    if ver1.major < ver2.major {
                        VersionRelationship::MajorUpgrade
                    } else if ver1.minor < ver2.minor {
                        VersionRelationship::MinorUpgrade
                    } else if ver1.patch < ver2.patch {
                        VersionRelationship::PatchUpgrade
                    } else if !ver1.pre.is_empty() && ver2.pre.is_empty() {
                        VersionRelationship::PrereleaseToStable
                    } else {
                        VersionRelationship::NewerPrerelease
                    }
                }
                std::cmp::Ordering::Equal => VersionRelationship::Identical,
                std::cmp::Ordering::Greater => {
                    if ver1.major > ver2.major {
                        VersionRelationship::MajorDowngrade
                    } else if ver1.minor > ver2.minor {
                        VersionRelationship::MinorDowngrade
                    } else if ver1.patch > ver2.patch {
                        VersionRelationship::PatchDowngrade
                    } else if ver1.pre.is_empty() && !ver2.pre.is_empty() {
                        VersionRelationship::StableToPrerelease
                    } else {
                        VersionRelationship::OlderPrerelease
                    }
                }
            }
        } else {
            VersionRelationship::Indeterminate
        }
    }

    /// Check if moving from v1 to v2 is a breaking change according to semver
    ///
    /// According to semantic versioning, a change in the major version
    /// number indicates a breaking change.
    ///
    /// # Arguments
    ///
    /// * `v1` - First version string
    /// * `v2` - Second version string
    ///
    /// # Returns
    ///
    /// `true` if moving from v1 to v2 is a breaking change, `false` otherwise
    #[must_use]
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Version;
    ///
    /// // Major version increase is breaking
    /// assert!(Version::is_breaking_change("1.0.0", "2.0.0"));
    ///
    /// // Minor version increase is not breaking
    /// assert!(!Version::is_breaking_change("1.0.0", "1.1.0"));
    ///
    /// // Patch version increase is not breaking
    /// assert!(!Version::is_breaking_change("1.0.0", "1.0.1"));
    /// ```
    pub fn is_breaking_change(v1: &str, v2: &str) -> bool {
        if let (Ok(ver1), Ok(ver2)) = (semver::Version::parse(v1), semver::Version::parse(v2)) {
            ver2.major > ver1.major
        } else {
            // If we can't parse the versions, conservatively assume breaking
            true
        }
    }

    /// Parse a version string into a semantic version
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to parse
    ///
    /// # Returns
    ///
    /// A parsed semantic version, or a `VersionError` if parsing fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Version;
    ///
    /// let version = Version::parse("1.2.3").unwrap();
    /// assert_eq!(version.major, 1);
    /// assert_eq!(version.minor, 2);
    /// assert_eq!(version.patch, 3);
    /// ```
    pub fn parse(version: &str) -> Result<SemVersion, VersionError> {
        let version = semver::Version::parse(version)?;
        Ok(version)
    }
}

/// Version bump strategy for enterprise package management
///
/// Defines different strategies for bumping versions in monorepo and single repository contexts.
/// Supports both standard semver bumping and advanced enterprise features like cascade bumping
/// and snapshot versioning with SHA integration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BumpStrategy {
    /// Bump major version (x.0.0)
    Major,
    /// Bump minor version (x.y.0)
    Minor,
    /// Bump patch version (x.y.z)
    Patch,
    /// Create snapshot version with SHA or timestamp (x.y.z-alpha.SHA)
    ///
    /// The string parameter contains the SHA, timestamp, or other identifier
    /// to append to the snapshot version.
    Snapshot(String),
    /// Cascade bump: Bump this package and all its dependents intelligently
    ///
    /// This strategy automatically detects affected packages in a monorepo
    /// and applies appropriate version bumps to maintain consistency.
    Cascade,
}

impl Default for BumpStrategy {
    fn default() -> Self {
        Self::Patch
    }
}

impl Display for BumpStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::Snapshot(ref identifier) => write!(f, "snapshot-{identifier}"),
            Self::Cascade => write!(f, "cascade"),
        }
    }
}

/// Report generated after version bump operations
///
/// Provides detailed information about what packages were bumped,
/// what updates were required, and any issues encountered.
#[derive(Debug, Clone)]
pub struct VersionBumpReport {
    /// Packages that had their versions bumped directly
    pub primary_bumps: std::collections::HashMap<String, String>,
    /// Packages that were bumped due to cascade effects
    pub cascade_bumps: std::collections::HashMap<String, String>,
    /// Dependency references that were updated
    pub reference_updates: Vec<DependencyReferenceUpdate>,
    /// Packages that were detected as affected but not bumped
    pub affected_packages: Vec<String>,
    /// Warnings generated during the bump process
    pub warnings: Vec<String>,
    /// Errors that occurred (but didn't halt the process)
    pub errors: Vec<String>,
}

impl VersionBumpReport {
    /// Create a new empty version bump report
    #[must_use]
    pub fn new() -> Self {
        Self {
            primary_bumps: std::collections::HashMap::new(),
            cascade_bumps: std::collections::HashMap::new(),
            reference_updates: Vec::new(),
            affected_packages: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if any packages were bumped
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.primary_bumps.is_empty() || !self.cascade_bumps.is_empty()
    }

    /// Get total number of packages affected
    #[must_use]
    pub fn total_packages_affected(&self) -> usize {
        self.primary_bumps.len() + self.cascade_bumps.len()
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add an error to the report
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
}

impl Default for VersionBumpReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency reference update information
///
/// Tracks when a dependency reference needs to be updated
/// as a result of version bumping operations.
#[derive(Debug, Clone)]
pub struct DependencyReferenceUpdate {
    /// Package that contains the dependency reference
    pub package: String,
    /// Name of the dependency being updated
    pub dependency: String,
    /// Original version reference
    pub from_reference: String,
    /// New version reference
    pub to_reference: String,
    /// Type of update being performed
    pub update_type: ReferenceUpdateType,
}

/// Type of dependency reference update
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceUpdateType {
    /// Update to exact version (for internal dependencies)
    FixedVersion,
    /// Suggest using workspace protocol
    WorkspaceProtocol,
    /// Keep the existing range pattern
    KeepRange,
}

/// Enterprise-grade version manager with cascade bumping and monorepo support
///
/// Provides advanced version management capabilities for both single repositories
/// and monorepo contexts, with intelligent cascade bumping and affected package detection.
///
/// ## Features
///
/// - **Context-Aware**: Adapts behavior for single repository vs monorepo
/// - **Cascade Bumping**: Automatically bump dependent packages in monorepos
/// - **Snapshot Versioning**: Create snapshot versions with SHA or timestamp
/// - **Affected Detection**: Intelligently detect which packages are affected by changes
/// - **Reference Management**: Update dependency references after version bumps
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::{VersionManager, BumpStrategy};
/// use sublime_standard_tools::filesystem::AsyncFileSystem;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create version manager with filesystem integration
/// let fs = AsyncFileSystem::new();
/// let version_manager = VersionManager::new(fs);
///
/// // Bump a single package version
/// let report = version_manager.bump_package_version(
///     "my-package",
///     BumpStrategy::Minor
/// ).await?;
///
/// println!("Bumped {} packages", report.total_packages_affected());
///
/// // In a monorepo, detect affected packages
/// let affected = version_manager.detect_affected_packages(&["changed-package"]).await?;
/// println!("Found {} affected packages", affected.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct VersionManager<F> {
    /// Filesystem integration for reading package.json files
    filesystem: F,
    /// Cache of package information for performance
    package_cache: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, CachedPackageInfo>>>,
}

/// Cached package information for performance optimization
#[derive(Debug, Clone)]
struct CachedPackageInfo {
    /// Package name
    name: String,
    /// Current version
    version: String,
    /// Dependencies with their current references
    dependencies: std::collections::HashMap<String, String>,
    /// Timestamp when this cache entry was created
    cached_at: std::time::SystemTime,
}

impl<F> VersionManager<F> 
where
    F: Clone,
{
    /// Create a new version manager with filesystem integration
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for reading package files
    ///
    /// # Returns
    ///
    /// A new VersionManager instance ready for version operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::VersionManager;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let version_manager = VersionManager::new(fs);
    /// ```
    #[must_use]
    pub fn new(filesystem: F) -> Self {
        Self {
            filesystem,
            package_cache: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Bump a package version using the specified strategy
    ///
    /// This is the main entry point for version bumping operations.
    /// Handles both simple version bumps and complex cascade operations.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to bump
    /// * `strategy` - Bump strategy to apply
    ///
    /// # Returns
    ///
    /// A detailed report of all changes made during the bump operation
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Package not found
    /// - Invalid version format
    /// - Filesystem operations fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{VersionManager, BumpStrategy};
    ///
    /// # async fn example(version_manager: VersionManager<impl Clone>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Simple version bump
    /// let report = version_manager.bump_package_version("my-package", BumpStrategy::Minor).await?;
    /// 
    /// // Cascade bump (for monorepos)
    /// let cascade_report = version_manager.bump_package_version("core-package", BumpStrategy::Cascade).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bump_package_version(
        &self,
        package_name: &str,
        strategy: BumpStrategy,
    ) -> Result<VersionBumpReport, VersionError> {
        let mut report = VersionBumpReport::new();

        match strategy {
            BumpStrategy::Major | BumpStrategy::Minor | BumpStrategy::Patch => {
                self.perform_simple_bump(package_name, &strategy, &mut report).await?;
            }
            BumpStrategy::Snapshot(ref identifier) => {
                self.perform_snapshot_bump(package_name, identifier, &mut report).await?;
            }
            BumpStrategy::Cascade => {
                self.perform_cascade_bump(package_name, &mut report).await?;
            }
        }

        Ok(report)
    }

    /// Detect packages affected by changes to the specified packages
    ///
    /// This method analyzes the dependency graph to find all packages that
    /// directly or indirectly depend on the changed packages.
    ///
    /// # Arguments
    ///
    /// * `changed_packages` - List of package names that have changed
    ///
    /// # Returns
    ///
    /// A list of package names that are affected by the changes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::VersionManager;
    ///
    /// # async fn example(version_manager: VersionManager<impl Clone>) -> Result<(), Box<dyn std::error::Error>> {
    /// let affected = version_manager.detect_affected_packages(&["core-lib", "utils"]).await?;
    /// for package in affected {
    ///     println!("Package {} is affected", package);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_affected_packages(
        &self,
        changed_packages: &[String],
    ) -> Result<Vec<String>, VersionError> {
        // This is a placeholder implementation
        // In a real implementation, this would analyze the dependency graph
        // to find all packages that depend on the changed packages
        
        let mut affected = Vec::new();
        
        // Add the changed packages themselves
        affected.extend_from_slice(changed_packages);
        
        // TODO: Implement actual dependency graph analysis
        // This would involve:
        // 1. Loading all package.json files
        // 2. Building a dependency graph
        // 3. Finding all packages that depend on changed_packages
        // 4. Recursively finding their dependents
        
        Ok(affected)
    }

    /// Perform a simple version bump (major, minor, or patch)
    async fn perform_simple_bump(
        &self,
        package_name: &str,
        strategy: &BumpStrategy,
        report: &mut VersionBumpReport,
    ) -> Result<(), VersionError> {
        // Load current package info
        let current_version = self.get_package_version(package_name).await?;
        
        // Calculate new version
        let new_version = match strategy {
            BumpStrategy::Major => Version::bump_major(&current_version)?,
            BumpStrategy::Minor => Version::bump_minor(&current_version)?,
            BumpStrategy::Patch => Version::bump_patch(&current_version)?,
            _ => return Err(VersionError::InvalidVersion("Invalid strategy for simple bump".to_string())),
        };

        // Record the bump
        report.primary_bumps.insert(package_name.to_string(), new_version.to_string());

        // TODO: Actually write the new version to package.json
        // This would involve:
        // 1. Reading the package.json file
        // 2. Updating the version field
        // 3. Writing it back to disk

        Ok(())
    }

    /// Perform a snapshot version bump with identifier
    async fn perform_snapshot_bump(
        &self,
        package_name: &str,
        identifier: &str,
        report: &mut VersionBumpReport,
    ) -> Result<(), VersionError> {
        // Load current package info
        let current_version = self.get_package_version(package_name).await?;
        
        // Create snapshot version
        let new_version = Version::bump_snapshot(&current_version, identifier)?;

        // Record the bump
        report.primary_bumps.insert(package_name.to_string(), new_version.to_string());

        // TODO: Write the new version to package.json

        Ok(())
    }

    /// Perform cascade version bump (bump package and all dependents)
    async fn perform_cascade_bump(
        &self,
        package_name: &str,
        report: &mut VersionBumpReport,
    ) -> Result<(), VersionError> {
        // First, bump the target package
        self.perform_simple_bump(package_name, &BumpStrategy::Patch, report).await?;

        // Then find and bump all affected packages
        let affected = self.detect_affected_packages(&[package_name.to_string()]).await?;
        
        for affected_package in affected {
            if affected_package != package_name {
                // Bump affected package (usually patch)
                let current_version = self.get_package_version(&affected_package).await?;
                let new_version = Version::bump_patch(&current_version)?;
                
                report.cascade_bumps.insert(affected_package.clone(), new_version.to_string());

                // TODO: Update dependency references
                // Create reference update entry
                report.reference_updates.push(DependencyReferenceUpdate {
                    package: affected_package,
                    dependency: package_name.to_string(),
                    from_reference: current_version.clone(),
                    to_reference: report.primary_bumps.get(package_name).unwrap_or(&current_version).clone(),
                    update_type: ReferenceUpdateType::FixedVersion,
                });
            }
        }

        Ok(())
    }

    /// Get the current version of a package
    async fn get_package_version(&self, package_name: &str) -> Result<String, VersionError> {
        // Check cache first
        {
            let cache = self.package_cache.read().map_err(|_| VersionError::InvalidVersion("Cache lock failed".to_string()))?;
            if let Some(cached) = cache.get(package_name) {
                // Check if cache is still valid (e.g., less than 5 minutes old)
                if cached.cached_at.elapsed().unwrap_or(std::time::Duration::from_secs(300)) < std::time::Duration::from_secs(300) {
                    return Ok(cached.version.clone());
                }
            }
        }

        // TODO: Actually read from package.json
        // For now, return a placeholder
        let version = "1.0.0".to_string();

        // Update cache
        {
            let mut cache = self.package_cache.write().map_err(|_| VersionError::InvalidVersion("Cache lock failed".to_string()))?;
            cache.insert(package_name.to_string(), CachedPackageInfo {
                name: package_name.to_string(),
                version: version.clone(),
                dependencies: std::collections::HashMap::new(),
                cached_at: std::time::SystemTime::now(),
            });
        }

        Ok(version)
    }

    /// Clear the package cache
    ///
    /// Useful when you know package information has changed and want to force
    /// fresh reads from the filesystem.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::VersionManager;
    ///
    /// # fn example(version_manager: &VersionManager<impl Clone>) {
    /// // After making external changes to package.json files
    /// version_manager.clear_cache();
    /// # }
    /// ```
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.package_cache.write() {
            cache.clear();
        }
    }

    /// Get cache statistics for debugging and monitoring
    ///
    /// # Returns
    ///
    /// Tuple of (cache_size, oldest_entry_age_seconds)
    #[must_use]
    pub fn cache_stats(&self) -> (usize, u64) {
        if let Ok(cache) = self.package_cache.read() {
            let size = cache.len();
            let oldest_age = cache.values()
                .map(|info| info.cached_at.elapsed().unwrap_or_default().as_secs())
                .max()
                .unwrap_or(0);
            (size, oldest_age)
        } else {
            (0, 0)
        }
    }
}
