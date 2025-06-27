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
