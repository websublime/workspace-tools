//! Version types and operations for semantic versioning.
//!
//! **What**: Provides types and utilities for working with semantic versions (semver),
//! including version parsing, comparison, bumping, and snapshot generation.
//!
//! **How**: Wraps the `semver` crate's `Version` type to provide a domain-specific API
//! with error handling, serialization support, and integration with the package tools
//! error system. Implements version bumping logic for major, minor, and patch releases.
//!
//! **Why**: To provide a type-safe, well-tested foundation for version management across
//! the package tools system, ensuring consistent version handling and preventing common
//! errors like invalid version strings or incorrect version bumps.
//!
//! # Version Format
//!
//! Versions follow the semantic versioning specification (semver 2.0.0):
//! - Format: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]`
//! - Example: `1.2.3`, `2.0.0-beta.1`, `1.0.0+20231201`
//!
//! # Examples
//!
//! ## Parsing versions
//!
//! ```rust
//! use sublime_pkg_tools::types::{Version, VersionBump};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse a valid version
//! let version = Version::parse("1.2.3")?;
//! assert_eq!(version.to_string(), "1.2.3");
//!
//! // Parse with prerelease
//! let prerelease = Version::parse("2.0.0-beta.1")?;
//! assert_eq!(prerelease.to_string(), "2.0.0-beta.1");
//! # Ok(())
//! # }
//! ```
//!
//! ## Bumping versions
//!
//! ```rust
//! use sublime_pkg_tools::types::{Version, VersionBump};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let version = Version::parse("1.2.3")?;
//!
//! // Bump major version
//! let major = version.bump(VersionBump::Major)?;
//! assert_eq!(major.to_string(), "2.0.0");
//!
//! // Bump minor version
//! let minor = version.bump(VersionBump::Minor)?;
//! assert_eq!(minor.to_string(), "1.3.0");
//!
//! // Bump patch version
//! let patch = version.bump(VersionBump::Patch)?;
//! assert_eq!(patch.to_string(), "1.2.4");
//!
//! // No bump
//! let none = version.bump(VersionBump::None)?;
//! assert_eq!(none.to_string(), "1.2.3");
//! # Ok(())
//! # }
//! ```
//!
//! ## Comparing versions
//!
//! ```rust
//! use sublime_pkg_tools::types::Version;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let v1 = Version::parse("1.2.3")?;
//! let v2 = Version::parse("2.0.0")?;
//! let v3 = Version::parse("1.2.3")?;
//!
//! assert!(v1 < v2);
//! assert!(v2 > v1);
//! assert_eq!(v1, v3);
//! # Ok(())
//! # }
//! ```
//!
//! ## Snapshot versions
//!
//! ```rust
//! use sublime_pkg_tools::types::{Version, VersionBump};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let version = Version::parse("1.2.3")?;
//! let snapshot = version.snapshot("abc123def")?;
//!
//! // Snapshot format: MAJOR.MINOR.PATCH-snapshot-TIMESTAMP-HASH
//! assert!(snapshot.to_string().starts_with("1.2.3-snapshot-"));
//! assert!(snapshot.to_string().contains("-abc123def"));
//! # Ok(())
//! # }
//! ```

use crate::error::{VersionError, VersionResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A semantic version number.
///
/// This type wraps `semver::Version` to provide a domain-specific API with
/// proper error handling and integration with the package tools error system.
///
/// # Format
///
/// Follows semantic versioning 2.0.0 specification:
/// - `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
/// - Optional prerelease: `MAJOR.MINOR.PATCH-PRERELEASE` (e.g., `1.0.0-beta.1`)
/// - Optional build metadata: `MAJOR.MINOR.PATCH+BUILD` (e.g., `1.0.0+20231201`)
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::Version;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let version = Version::parse("1.2.3")?;
/// assert_eq!(version.major(), 1);
/// assert_eq!(version.minor(), 2);
/// assert_eq!(version.patch(), 3);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Version {
    inner: semver::Version,
}

impl Version {
    /// Parses a version string into a `Version`.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::ParseError` if the version string is invalid or does not
    /// conform to semantic versioning format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Valid versions
    /// let v1 = Version::parse("1.2.3")?;
    /// let v2 = Version::parse("0.0.1")?;
    /// let v3 = Version::parse("2.0.0-beta.1")?;
    /// let v4 = Version::parse("1.0.0+build.123")?;
    ///
    /// // Invalid versions
    /// assert!(Version::parse("").is_err());
    /// assert!(Version::parse("1.2").is_err());
    /// assert!(Version::parse("v1.2.3").is_err());
    /// assert!(Version::parse("1.2.3.4").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(s: &str) -> VersionResult<Self> {
        if s.is_empty() {
            return Err(VersionError::ParseError {
                version: s.to_string(),
                reason: "version string cannot be empty".to_string(),
            });
        }

        let inner = semver::Version::from_str(s).map_err(|e| VersionError::ParseError {
            version: s.to_string(),
            reason: e.to_string(),
        })?;

        Ok(Self { inner })
    }

    /// Creates a new version from components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    /// assert_eq!(version.to_string(), "1.2.3");
    /// ```
    #[must_use]
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { inner: semver::Version::new(major, minor, patch) }
    }

    /// Returns the major version number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::parse("1.2.3")?;
    /// assert_eq!(version.major(), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn major(&self) -> u64 {
        self.inner.major
    }

    /// Returns the minor version number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::parse("1.2.3")?;
    /// assert_eq!(version.minor(), 2);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn minor(&self) -> u64 {
        self.inner.minor
    }

    /// Returns the patch version number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::parse("1.2.3")?;
    /// assert_eq!(version.patch(), 3);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn patch(&self) -> u64 {
        self.inner.patch
    }

    /// Returns the prerelease version string if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stable = Version::parse("1.2.3")?;
    /// assert!(stable.prerelease().is_empty());
    ///
    /// let beta = Version::parse("1.0.0-beta.1")?;
    /// assert_eq!(beta.prerelease(), "beta.1");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn prerelease(&self) -> &str {
        self.inner.pre.as_str()
    }

    /// Returns the build metadata string if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let no_build = Version::parse("1.2.3")?;
    /// assert!(no_build.build().is_empty());
    ///
    /// let with_build = Version::parse("1.0.0+build.123")?;
    /// assert_eq!(with_build.build(), "build.123");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn build(&self) -> &str {
        self.inner.build.as_str()
    }

    /// Bumps the version according to the specified bump type.
    ///
    /// When bumping major or minor versions, lower-priority components are reset to zero.
    /// Prerelease and build metadata are removed after bumping.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidVersion` if the bump would result in an invalid version
    /// (e.g., integer overflow).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Version, VersionBump};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::parse("1.2.3")?;
    ///
    /// // Bump major: 1.2.3 -> 2.0.0
    /// let major = version.bump(VersionBump::Major)?;
    /// assert_eq!(major.to_string(), "2.0.0");
    ///
    /// // Bump minor: 1.2.3 -> 1.3.0
    /// let minor = version.bump(VersionBump::Minor)?;
    /// assert_eq!(minor.to_string(), "1.3.0");
    ///
    /// // Bump patch: 1.2.3 -> 1.2.4
    /// let patch = version.bump(VersionBump::Patch)?;
    /// assert_eq!(patch.to_string(), "1.2.4");
    ///
    /// // No bump: 1.2.3 -> 1.2.3
    /// let none = version.bump(VersionBump::None)?;
    /// assert_eq!(none.to_string(), "1.2.3");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With prerelease versions
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Version, VersionBump};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let prerelease = Version::parse("1.0.0-beta.1")?;
    ///
    /// // Bumping removes prerelease
    /// let bumped = prerelease.bump(VersionBump::Patch)?;
    /// assert_eq!(bumped.to_string(), "1.0.1");
    /// # Ok(())
    /// # }
    /// ```
    pub fn bump(&self, bump_type: VersionBump) -> VersionResult<Self> {
        let bumped = match bump_type {
            VersionBump::Major => {
                let new_major = self.inner.major.checked_add(1).ok_or_else(|| {
                    VersionError::InvalidVersion {
                        version: self.to_string(),
                        reason: "major version overflow".to_string(),
                    }
                })?;
                semver::Version::new(new_major, 0, 0)
            }
            VersionBump::Minor => {
                let new_minor = self.inner.minor.checked_add(1).ok_or_else(|| {
                    VersionError::InvalidVersion {
                        version: self.to_string(),
                        reason: "minor version overflow".to_string(),
                    }
                })?;
                semver::Version::new(self.inner.major, new_minor, 0)
            }
            VersionBump::Patch => {
                let new_patch = self.inner.patch.checked_add(1).ok_or_else(|| {
                    VersionError::InvalidVersion {
                        version: self.to_string(),
                        reason: "patch version overflow".to_string(),
                    }
                })?;
                semver::Version::new(self.inner.major, self.inner.minor, new_patch)
            }
            VersionBump::None => self.inner.clone(),
        };

        Ok(Self { inner: bumped })
    }

    /// Generates a snapshot version for testing or pre-release builds.
    ///
    /// Snapshot versions follow the format: `MAJOR.MINOR.PATCH-snapshot-TIMESTAMP-HASH`
    /// where:
    /// - `TIMESTAMP` is UTC Unix timestamp
    /// - `HASH` is a short commit hash or identifier (first 7 characters)
    ///
    /// # Errors
    ///
    /// Returns `VersionError::SnapshotFailed` if snapshot generation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::parse("1.2.3")?;
    /// let snapshot = version.snapshot("abc123def456")?;
    ///
    /// // Format: 1.2.3-snapshot-TIMESTAMP-abc123d
    /// let snapshot_str = snapshot.to_string();
    /// assert!(snapshot_str.starts_with("1.2.3-snapshot-"));
    /// assert!(snapshot_str.contains("-abc123d"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn snapshot(&self, hash: &str) -> VersionResult<Self> {
        if hash.is_empty() {
            return Err(VersionError::SnapshotFailed {
                package: "unknown".to_string(),
                reason: "hash cannot be empty".to_string(),
            });
        }

        let timestamp = Utc::now().timestamp();
        let short_hash = if hash.len() > 7 { &hash[..7] } else { hash };

        let prerelease = format!("snapshot-{}-{}", timestamp, short_hash);

        let mut snapshot_version =
            semver::Version::new(self.inner.major, self.inner.minor, self.inner.patch);
        snapshot_version.pre =
            semver::Prerelease::new(&prerelease).map_err(|e| VersionError::SnapshotFailed {
                package: "unknown".to_string(),
                reason: format!("invalid prerelease format: {}", e),
            })?;

        Ok(Self { inner: snapshot_version })
    }

    /// Returns whether this is a prerelease version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stable = Version::parse("1.2.3")?;
    /// assert!(!stable.is_prerelease());
    ///
    /// let beta = Version::parse("1.0.0-beta.1")?;
    /// assert!(beta.is_prerelease());
    ///
    /// let snapshot = stable.snapshot("abc123")?;
    /// assert!(snapshot.is_prerelease());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_prerelease(&self) -> bool {
        !self.inner.pre.is_empty()
    }

    /// Returns the inner `semver::Version` reference.
    ///
    /// This is useful for interoperability with other libraries that use `semver::Version`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::parse("1.2.3")?;
    /// let semver_version = version.as_semver();
    /// assert_eq!(semver_version.major, 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn as_semver(&self) -> &semver::Version {
        &self.inner
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Type of version bump to apply.
///
/// Defines the semantic versioning bump types according to semver 2.0.0 specification:
/// - **Major**: Incompatible API changes (X.0.0)
/// - **Minor**: Backwards-compatible functionality additions (0.X.0)
/// - **Patch**: Backwards-compatible bug fixes (0.0.X)
/// - **None**: No version change
///
/// # Bump Rules
///
/// When bumping versions:
/// - **Major bump**: Increments major version, resets minor and patch to 0
/// - **Minor bump**: Increments minor version, resets patch to 0
/// - **Patch bump**: Increments patch version only
/// - **None**: No change to version
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::{Version, VersionBump};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let version = Version::parse("1.2.3")?;
///
/// // Major: 1.2.3 -> 2.0.0
/// assert_eq!(
///     version.bump(VersionBump::Major)?.to_string(),
///     "2.0.0"
/// );
///
/// // Minor: 1.2.3 -> 1.3.0
/// assert_eq!(
///     version.bump(VersionBump::Minor)?.to_string(),
///     "1.3.0"
/// );
///
/// // Patch: 1.2.3 -> 1.2.4
/// assert_eq!(
///     version.bump(VersionBump::Patch)?.to_string(),
///     "1.2.4"
/// );
///
/// // None: 1.2.3 -> 1.2.3
/// assert_eq!(
///     version.bump(VersionBump::None)?.to_string(),
///     "1.2.3"
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionBump {
    /// Major version bump (X.0.0) - breaking changes.
    Major,
    /// Minor version bump (0.X.0) - new features, backwards compatible.
    Minor,
    /// Patch version bump (0.0.X) - bug fixes, backwards compatible.
    Patch,
    /// No version bump - version stays the same.
    None,
}

impl VersionBump {
    /// Parses a version bump type from a string.
    ///
    /// # Valid values
    ///
    /// - `"major"` or `"Major"` -> `VersionBump::Major`
    /// - `"minor"` or `"Minor"` -> `VersionBump::Minor`
    /// - `"patch"` or `"Patch"` -> `VersionBump::Patch`
    /// - `"none"` or `"None"` -> `VersionBump::None`
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidBump` if the string is not a valid bump type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::VersionBump;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// assert_eq!(VersionBump::parse("major")?, VersionBump::Major);
    /// assert_eq!(VersionBump::parse("Minor")?, VersionBump::Minor);
    /// assert_eq!(VersionBump::parse("PATCH")?, VersionBump::Patch);
    /// assert_eq!(VersionBump::parse("none")?, VersionBump::None);
    ///
    /// assert!(VersionBump::parse("invalid").is_err());
    /// assert!(VersionBump::parse("").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(s: &str) -> VersionResult<Self> {
        match s.to_lowercase().as_str() {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            "none" => Ok(Self::None),
            _ => Err(VersionError::InvalidBump {
                bump: s.to_string(),
                reason: "expected 'major', 'minor', 'patch', or 'none'".to_string(),
            }),
        }
    }

    /// Returns the string representation of the bump type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::VersionBump;
    ///
    /// assert_eq!(VersionBump::Major.as_str(), "major");
    /// assert_eq!(VersionBump::Minor.as_str(), "minor");
    /// assert_eq!(VersionBump::Patch.as_str(), "patch");
    /// assert_eq!(VersionBump::None.as_str(), "none");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::None => "none",
        }
    }
}

impl fmt::Display for VersionBump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for VersionBump {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Versioning strategy for monorepo package management.
///
/// Defines how versions are coordinated across packages in a monorepo:
///
/// - **Independent**: Each package maintains its own version, bumped independently
/// - **Unified**: All packages share the same version, bumped together
///
/// # Independent Strategy
///
/// Each package has its own version that can be bumped independently. This is useful
/// when packages have different release cycles or when you want fine-grained control
/// over versions.
///
/// Example:
/// - `@myorg/core`: 1.2.3
/// - `@myorg/utils`: 2.1.0
/// - `@myorg/cli`: 0.5.2
///
/// # Unified Strategy
///
/// All packages share the same version and are bumped together. This simplifies
/// version management and makes it clear which packages were released together.
///
/// Example (all packages at same version):
/// - `@myorg/core`: 1.2.3
/// - `@myorg/utils`: 1.2.3
/// - `@myorg/cli`: 1.2.3
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::VersioningStrategy;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Parse from string
/// let independent = VersioningStrategy::parse("independent")?;
/// assert_eq!(independent, VersioningStrategy::Independent);
///
/// let unified = VersioningStrategy::parse("unified")?;
/// assert_eq!(unified, VersioningStrategy::Unified);
///
/// // Check strategy type
/// if independent.is_independent() {
///     println!("Using independent versioning");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersioningStrategy {
    /// Each package maintains its own independent version.
    Independent,
    /// All packages share the same version.
    Unified,
}

impl VersioningStrategy {
    /// Parses a versioning strategy from a string.
    ///
    /// # Valid values
    ///
    /// - `"independent"` or `"Independent"` -> `VersioningStrategy::Independent`
    /// - `"unified"` or `"Unified"` -> `VersioningStrategy::Unified`
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidStrategy` if the string is not a valid strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::VersioningStrategy;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// assert_eq!(
    ///     VersioningStrategy::parse("independent")?,
    ///     VersioningStrategy::Independent
    /// );
    /// assert_eq!(
    ///     VersioningStrategy::parse("Unified")?,
    ///     VersioningStrategy::Unified
    /// );
    ///
    /// assert!(VersioningStrategy::parse("invalid").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(s: &str) -> VersionResult<Self> {
        match s.to_lowercase().as_str() {
            "independent" => Ok(Self::Independent),
            "unified" => Ok(Self::Unified),
            _ => Err(VersionError::InvalidStrategy { strategy: s.to_string() }),
        }
    }

    /// Returns the string representation of the strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::VersioningStrategy;
    ///
    /// assert_eq!(VersioningStrategy::Independent.as_str(), "independent");
    /// assert_eq!(VersioningStrategy::Unified.as_str(), "unified");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::Unified => "unified",
        }
    }

    /// Returns `true` if the strategy is `Independent`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::VersioningStrategy;
    ///
    /// assert!(VersioningStrategy::Independent.is_independent());
    /// assert!(!VersioningStrategy::Unified.is_independent());
    /// ```
    #[must_use]
    pub fn is_independent(&self) -> bool {
        matches!(self, Self::Independent)
    }

    /// Returns `true` if the strategy is `Unified`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::VersioningStrategy;
    ///
    /// assert!(VersioningStrategy::Unified.is_unified());
    /// assert!(!VersioningStrategy::Independent.is_unified());
    /// ```
    #[must_use]
    pub fn is_unified(&self) -> bool {
        matches!(self, Self::Unified)
    }
}

impl fmt::Display for VersioningStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for VersioningStrategy {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Default for VersioningStrategy {
    /// Returns the default versioning strategy: `Independent`.
    fn default() -> Self {
        Self::Independent
    }
}
