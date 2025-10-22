//! Type definitions for registry operations.
//!
//! **What**: Defines data structures for NPM registry responses, package metadata,
//! and upgrade type classifications.
//!
//! **How**: This module provides serializable types that represent NPM registry API responses,
//! including package metadata, version information, and repository details.
//!
//! **Why**: To provide a type-safe representation of NPM registry data and upgrade classifications
//! for version comparison and upgrade detection.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Package metadata from NPM registry.
///
/// Represents the response from querying a package in the NPM registry.
/// Contains version information, deprecation status, and repository details.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::PackageMetadata;
///
/// # fn example(metadata: PackageMetadata) {
/// println!("Package: {}", metadata.name);
/// println!("Latest version: {}", metadata.latest);
/// println!("Available versions: {}", metadata.versions.len());
///
/// if let Some(deprecated) = metadata.deprecated {
///     println!("DEPRECATED: {}", deprecated);
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageMetadata {
    /// Package name.
    ///
    /// This is the fully qualified name including scope if applicable (e.g., "@scope/package").
    pub name: String,

    /// All available versions for this package.
    ///
    /// List of semver version strings available in the registry.
    pub versions: Vec<String>,

    /// Latest dist-tag version.
    ///
    /// The version associated with the "latest" tag in the registry.
    pub latest: String,

    /// Deprecation notice if the package is deprecated.
    ///
    /// Contains the deprecation message from the package maintainer.
    pub deprecated: Option<String>,

    /// Publication time metadata.
    ///
    /// Maps version strings to their publication timestamps.
    /// Also includes special keys like "created" and "modified".
    pub time: HashMap<String, DateTime<Utc>>,

    /// Repository information if available.
    ///
    /// Contains the source code repository details for the package.
    pub repository: Option<RepositoryInfo>,
}

/// Repository information from package metadata.
///
/// Represents the source code repository details for an NPM package.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::RepositoryInfo;
///
/// let repo = RepositoryInfo {
///     type_: "git".to_string(),
///     url: "https://github.com/user/repo.git".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryInfo {
    /// Repository type (typically "git").
    #[serde(rename = "type")]
    pub type_: String,

    /// Repository URL.
    pub url: String,
}

/// Classification of version upgrade type.
///
/// Determines the semantic versioning magnitude of an upgrade.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::UpgradeType;
///
/// let upgrade = UpgradeType::Minor;
/// assert!(upgrade.is_breaking() == false);
/// assert!(upgrade.is_safe() == true);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UpgradeType {
    /// Major version upgrade (breaking changes).
    ///
    /// Indicates a change in the major version number (X.y.z -> (X+1).0.0).
    /// May contain breaking changes according to semver.
    Major,

    /// Minor version upgrade (new features, backward compatible).
    ///
    /// Indicates a change in the minor version number (x.Y.z -> x.(Y+1).0).
    /// Should be backward compatible according to semver.
    Minor,

    /// Patch version upgrade (bug fixes).
    ///
    /// Indicates a change in the patch version number (x.y.Z -> x.y.(Z+1)).
    /// Should only contain bug fixes according to semver.
    Patch,
}

impl UpgradeType {
    /// Returns whether this upgrade type is considered breaking.
    ///
    /// Only major version upgrades are considered breaking according to semver.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeType;
    ///
    /// assert!(UpgradeType::Major.is_breaking());
    /// assert!(!UpgradeType::Minor.is_breaking());
    /// assert!(!UpgradeType::Patch.is_breaking());
    /// ```
    #[must_use]
    pub const fn is_breaking(self) -> bool {
        matches!(self, Self::Major)
    }

    /// Returns whether this upgrade type is considered safe.
    ///
    /// Patch and minor upgrades are considered safe according to semver.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeType;
    ///
    /// assert!(!UpgradeType::Major.is_safe());
    /// assert!(UpgradeType::Minor.is_safe());
    /// assert!(UpgradeType::Patch.is_safe());
    /// ```
    #[must_use]
    pub const fn is_safe(self) -> bool {
        !self.is_breaking()
    }

    /// Returns the priority of this upgrade type.
    ///
    /// Higher values indicate more significant upgrades.
    /// Major = 3, Minor = 2, Patch = 1.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeType;
    ///
    /// assert!(UpgradeType::Major.priority() > UpgradeType::Minor.priority());
    /// assert!(UpgradeType::Minor.priority() > UpgradeType::Patch.priority());
    /// ```
    #[must_use]
    pub const fn priority(self) -> u8 {
        match self {
            Self::Major => 3,
            Self::Minor => 2,
            Self::Patch => 1,
        }
    }

    /// Returns the string representation of this upgrade type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeType;
    ///
    /// assert_eq!(UpgradeType::Major.as_str(), "major");
    /// assert_eq!(UpgradeType::Minor.as_str(), "minor");
    /// assert_eq!(UpgradeType::Patch.as_str(), "patch");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
        }
    }
}

impl std::fmt::Display for UpgradeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PackageMetadata {
    /// Returns whether this package is deprecated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::PackageMetadata;
    ///
    /// # fn example(metadata: PackageMetadata) {
    /// if metadata.is_deprecated() {
    ///     if let Some(msg) = metadata.deprecation_message() {
    ///         println!("Package is deprecated: {}", msg);
    ///     }
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_deprecated(&self) -> bool {
        self.deprecated.is_some()
    }

    /// Returns the deprecation message if the package is deprecated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::PackageMetadata;
    ///
    /// # fn example(metadata: PackageMetadata) {
    /// if let Some(message) = metadata.deprecation_message() {
    ///     println!("Deprecation notice: {}", message);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn deprecation_message(&self) -> Option<&str> {
        self.deprecated.as_deref()
    }

    /// Returns the creation time of the package if available.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::PackageMetadata;
    ///
    /// # fn example(metadata: PackageMetadata) {
    /// if let Some(created) = metadata.created_at() {
    ///     println!("Package created: {}", created);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.time.get("created").copied()
    }

    /// Returns the last modification time of the package if available.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::PackageMetadata;
    ///
    /// # fn example(metadata: PackageMetadata) {
    /// if let Some(modified) = metadata.modified_at() {
    ///     println!("Last modified: {}", modified);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn modified_at(&self) -> Option<DateTime<Utc>> {
        self.time.get("modified").copied()
    }

    /// Returns the publication time for a specific version if available.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::PackageMetadata;
    ///
    /// # fn example(metadata: PackageMetadata) {
    /// if let Some(published) = metadata.version_published_at("1.2.3") {
    ///     println!("Version 1.2.3 published: {}", published);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn version_published_at(&self, version: &str) -> Option<DateTime<Utc>> {
        self.time.get(version).copied()
    }
}
