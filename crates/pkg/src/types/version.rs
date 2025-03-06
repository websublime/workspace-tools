//! Version types and utilities.

use semver::{BuildMetadata, Prerelease, Version as SemVersion};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

/// Version update strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionUpdateStrategy {
    /// Only upgrade patch versions (0.0.x)
    PatchOnly,
    /// Upgrade patch and minor versions (0.x.y)
    MinorAndPatch,
    /// Upgrade all versions including major ones (x.y.z)
    AllUpdates,
}

impl Default for VersionUpdateStrategy {
    fn default() -> Self {
        Self::MinorAndPatch
    }
}

/// Version stability filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionStability {
    /// Only include stable versions
    StableOnly,
    /// Include prereleases and stable versions
    IncludePrerelease,
}

impl Default for VersionStability {
    fn default() -> Self {
        Self::StableOnly
    }
}

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

impl std::fmt::Display for VersionRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MajorUpgrade => write!(f, "major upgrade"),
            Self::MinorUpgrade => write!(f, "minor upgrade"),
            Self::PatchUpgrade => write!(f, "patch upgrade"),
            Self::PrereleaseToStable => write!(f, "prerelease to stable"),
            Self::NewerPrerelease => write!(f, "newer prerelease"),
            Self::Identical => write!(f, "identical"),
            Self::MajorDowngrade => write!(f, "major downgrade"),
            Self::MinorDowngrade => write!(f, "minor downgrade"),
            Self::PatchDowngrade => write!(f, "patch downgrade"),
            Self::StableToPrerelease => write!(f, "stable to prerelease"),
            Self::OlderPrerelease => write!(f, "older prerelease"),
            Self::Indeterminate => write!(f, "indeterminate"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
/// Enum representing the type of version bump to be performed.
pub enum Version {
    Major,
    Minor,
    Patch,
    Snapshot,
}

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

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Version::Major => write!(f, "major"),
            Version::Minor => write!(f, "minor"),
            Version::Patch => write!(f, "patch"),
            Version::Snapshot => write!(f, "snapshot"),
        }
    }
}

impl Version {
    /// Bumps the version of the package to major.
    pub fn bump_major(version: &str) -> SemVersion {
        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.major += 1;
        sem_version.minor = 0;
        sem_version.patch = 0;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }

    /// Bumps the version of the package to minor.
    pub fn bump_minor(version: &str) -> SemVersion {
        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.minor += 1;
        sem_version.patch = 0;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }

    /// Bumps the version of the package to patch.
    pub fn bump_patch(version: &str) -> SemVersion {
        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.patch += 1;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }

    /// Bumps the version of the package to snapshot appending the sha to the version.
    pub fn bump_snapshot(version: &str, sha: &str) -> SemVersion {
        let alpha = format!("alpha.{sha}");

        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.pre = Prerelease::new(alpha.as_str()).unwrap_or(Prerelease::EMPTY);
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }

    /// Compare two version strings and return their relationship
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
    pub fn is_breaking_change(v1: &str, v2: &str) -> bool {
        if let (Ok(ver1), Ok(ver2)) = (semver::Version::parse(v1), semver::Version::parse(v2)) {
            ver2.major > ver1.major
        } else {
            // If we can't parse the versions, conservatively assume breaking
            true
        }
    }
}
