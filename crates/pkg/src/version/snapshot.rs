use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::version::versioning::{Version, VersionComparison};

/// Snapshot version for development branches.
///
/// Snapshot versions provide unique identifiers for each commit on
/// development branches, enabling continuous deployment without
/// version conflicts.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::version::{Version, SnapshotVersion};
/// use std::str::FromStr;
///
/// let base = Version::from_str("1.2.3")?;
/// let snapshot = SnapshotVersion::new(base, "abc123d".to_string());
///
/// assert_eq!(snapshot.to_string(), "1.2.3-abc123d.snapshot");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotVersion {
    /// Base version from package.json
    pub base_version: Version,
    /// Git commit identifier
    pub commit_id: String,
    /// Timestamp when snapshot was created
    pub created_at: DateTime<Utc>,
}

impl fmt::Display for SnapshotVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}.snapshot", self.base_version, self.commit_id)
    }
}

impl SnapshotVersion {
    /// Creates a new snapshot version.
    ///
    /// # Arguments
    ///
    /// * `base_version` - Base version from package.json
    /// * `commit_id` - Git commit identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{Version, SnapshotVersion};
    /// use std::str::FromStr;
    ///
    /// let base = Version::from_str("1.2.3")?;
    /// let snapshot = SnapshotVersion::new(base, "abc123d".to_string());
    ///
    /// assert_eq!(snapshot.base_version().to_string(), "1.2.3");
    /// assert_eq!(snapshot.commit_id(), "abc123d");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn new(base_version: Version, commit_id: String) -> Self {
        Self { base_version, commit_id, created_at: Utc::now() }
    }

    /// Creates a snapshot version with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `base_version` - Base version from package.json
    /// * `commit_id` - Git commit identifier
    /// * `created_at` - Timestamp when snapshot was created
    #[must_use]
    pub fn new_with_timestamp(
        base_version: Version,
        commit_id: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self { base_version, commit_id, created_at }
    }

    /// Gets the base version.
    #[must_use]
    pub fn base_version(&self) -> &Version {
        &self.base_version
    }

    /// Gets the commit identifier.
    #[must_use]
    pub fn commit_id(&self) -> &str {
        &self.commit_id
    }

    /// Gets the creation timestamp.
    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Checks if this snapshot is based on a specific version.
    ///
    /// # Arguments
    ///
    /// * `version` - Version to check against
    #[must_use]
    pub fn is_based_on(&self, version: &Version) -> bool {
        &self.base_version == version
    }

    /// Compares this snapshot version with another.
    ///
    /// Snapshots are compared first by base version, then by creation timestamp.
    ///
    /// # Arguments
    ///
    /// * `other` - The snapshot version to compare against
    #[must_use]
    pub fn compare(&self, other: &Self) -> VersionComparison {
        match self.base_version.compare(&other.base_version) {
            VersionComparison::Equal => match self.created_at.cmp(&other.created_at) {
                std::cmp::Ordering::Less => VersionComparison::Less,
                std::cmp::Ordering::Equal => VersionComparison::Equal,
                std::cmp::Ordering::Greater => VersionComparison::Greater,
            },
            other => other,
        }
    }
}
