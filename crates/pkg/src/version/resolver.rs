use std::fmt;

use serde::{Deserialize, Serialize};

use crate::version::{
    snapshot::SnapshotVersion,
    versioning::{Version, VersionComparison},
};

/// Resolved version union type.
///
/// Represents either a release version (from package.json) or
/// a snapshot version (calculated for development branches).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolvedVersion {
    /// Release version from package.json
    Release(Version),
    /// Snapshot version for development
    Snapshot(SnapshotVersion),
}

impl fmt::Display for ResolvedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Release(version) => write!(f, "{}", version),
            Self::Snapshot(snapshot) => write!(f, "{}", snapshot),
        }
    }
}

impl From<Version> for ResolvedVersion {
    fn from(version: Version) -> Self {
        Self::Release(version)
    }
}

impl From<SnapshotVersion> for ResolvedVersion {
    fn from(snapshot: SnapshotVersion) -> Self {
        Self::Snapshot(snapshot)
    }
}

impl ResolvedVersion {
    /// Checks if this is a release version.
    #[must_use]
    pub fn is_release(&self) -> bool {
        matches!(self, Self::Release(_))
    }

    /// Checks if this is a snapshot version.
    #[must_use]
    pub fn is_snapshot(&self) -> bool {
        matches!(self, Self::Snapshot(_))
    }

    /// Gets the base version (release version or snapshot base).
    #[must_use]
    pub fn base_version(&self) -> &Version {
        match self {
            Self::Release(version) => version,
            Self::Snapshot(snapshot) => &snapshot.base_version,
        }
    }

    /// Converts to release version if possible.
    #[must_use]
    pub fn as_release(&self) -> Option<&Version> {
        match self {
            Self::Release(version) => Some(version),
            Self::Snapshot(_) => None,
        }
    }

    /// Converts to snapshot version if possible.
    #[must_use]
    pub fn as_snapshot(&self) -> Option<&SnapshotVersion> {
        match self {
            Self::Release(_) => None,
            Self::Snapshot(snapshot) => Some(snapshot),
        }
    }

    /// Compares this resolved version with another.
    ///
    /// # Arguments
    ///
    /// * `other` - The resolved version to compare against
    #[must_use]
    pub fn compare(&self, other: &Self) -> VersionComparison {
        match (self, other) {
            (Self::Release(a), Self::Release(b)) => a.compare(b),
            (Self::Snapshot(a), Self::Snapshot(b)) => a.compare(b),
            (Self::Release(release), Self::Snapshot(snapshot))
            | (Self::Snapshot(snapshot), Self::Release(release)) => {
                match release.compare(&snapshot.base_version) {
                    VersionComparison::Equal => VersionComparison::Incomparable,
                    other => other,
                }
            }
        }
    }
}
