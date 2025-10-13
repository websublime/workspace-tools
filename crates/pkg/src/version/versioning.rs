use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{error::VersionError, version::bump::VersionBump, PackageResult};

/// Standard semantic version representation.
///
/// Wraps the `semver::Version` type with additional functionality
/// for package management operations.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::version::Version;
/// use std::str::FromStr;
///
/// let version = Version::from_str("1.2.3")?;
/// assert_eq!(version.major(), 1);
/// assert_eq!(version.minor(), 2);
/// assert_eq!(version.patch(), 3);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    /// The underlying semantic version
    pub(crate) inner: semver::Version,
}

/// Version comparison result.
///
/// Represents the relationship between two versions for
/// dependency resolution and conflict detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// First version is less than second
    Less,
    /// Versions are equal
    Equal,
    /// First version is greater than second
    Greater,
    /// Versions are incomparable (e.g., snapshot vs release)
    Incomparable,
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = semver::Version::from_str(s)
            .map_err(|e| VersionError::ParseFailed { version: s.to_string(), source: e })?;
        Ok(Self { inner })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<semver::Version> for Version {
    fn from(version: semver::Version) -> Self {
        Self { inner: version }
    }
}

impl From<Version> for semver::Version {
    fn from(version: Version) -> Self {
        version.inner
    }
}

impl Version {
    /// Creates a new version from major, minor, and patch components.
    ///
    /// # Arguments
    ///
    /// * `major` - Major version number
    /// * `minor` - Minor version number
    /// * `patch` - Patch version number
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    /// assert_eq!(version.to_string(), "1.2.3");
    /// ```
    #[must_use]
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { inner: semver::Version::new(major, minor, patch) }
    }

    /// Gets the major version number.
    #[must_use]
    pub fn major(&self) -> u64 {
        self.inner.major
    }

    /// Gets the minor version number.
    #[must_use]
    pub fn minor(&self) -> u64 {
        self.inner.minor
    }

    /// Gets the patch version number.
    #[must_use]
    pub fn patch(&self) -> u64 {
        self.inner.patch
    }

    /// Gets the pre-release version.
    #[must_use]
    pub fn pre_release(&self) -> &semver::Prerelease {
        &self.inner.pre
    }

    /// Gets the build metadata.
    #[must_use]
    pub fn build_metadata(&self) -> &semver::BuildMetadata {
        &self.inner.build
    }

    /// Applies a version bump to create a new version.
    ///
    /// # Arguments
    ///
    /// * `bump` - The type of version bump to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{Version, VersionBump};
    /// use std::str::FromStr;
    ///
    /// let version = Version::from_str("1.2.3")?;
    /// let bumped = version.bump(VersionBump::Minor);
    /// assert_eq!(bumped.to_string(), "1.3.0");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn bump(&self, bump: VersionBump) -> Self {
        match bump {
            VersionBump::Major => Self::new(self.major() + 1, 0, 0),
            VersionBump::Minor => Self::new(self.major(), self.minor() + 1, 0),
            VersionBump::Patch => Self::new(self.major(), self.minor(), self.patch() + 1),
            VersionBump::None => self.clone(),
        }
    }

    /// Checks if this version is a pre-release.
    #[must_use]
    pub fn is_prerelease(&self) -> bool {
        !self.inner.pre.is_empty()
    }

    /// Checks if this version has build metadata.
    #[must_use]
    pub fn has_build_metadata(&self) -> bool {
        !self.inner.build.is_empty()
    }

    /// Compares this version with another version.
    ///
    /// # Arguments
    ///
    /// * `other` - The version to compare against
    ///
    /// # Returns
    ///
    /// Version comparison result
    #[must_use]
    pub fn compare(&self, other: &Self) -> VersionComparison {
        match self.inner.cmp(&other.inner) {
            std::cmp::Ordering::Less => VersionComparison::Less,
            std::cmp::Ordering::Equal => VersionComparison::Equal,
            std::cmp::Ordering::Greater => VersionComparison::Greater,
        }
    }

    /// Creates a version with pre-release identifier.
    ///
    /// # Arguments
    ///
    /// * `prerelease` - Pre-release identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::Version;
    ///
    /// let version = Version::new(1, 2, 3).with_prerelease("alpha.1")?;
    /// assert_eq!(version.to_string(), "1.2.3-alpha.1");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_prerelease(&self, prerelease: &str) -> PackageResult<Self> {
        let mut new_version = self.inner.clone();
        new_version.pre = semver::Prerelease::from_str(prerelease).map_err(|e| {
            VersionError::PreReleaseError { version: self.to_string(), reason: e.to_string() }
        })?;
        Ok(Self { inner: new_version })
    }

    /// Creates a version with build metadata.
    ///
    /// # Arguments
    ///
    /// * `build` - Build metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::Version;
    ///
    /// let version = Version::new(1, 2, 3).with_build_metadata("20240115.abc123")?;
    /// assert_eq!(version.to_string(), "1.2.3+20240115.abc123");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_build_metadata(&self, build: &str) -> PackageResult<Self> {
        let mut new_version = self.inner.clone();
        new_version.build = semver::BuildMetadata::from_str(build).map_err(|e| {
            VersionError::PreReleaseError { version: self.to_string(), reason: e.to_string() }
        })?;
        Ok(Self { inner: new_version })
    }
}
