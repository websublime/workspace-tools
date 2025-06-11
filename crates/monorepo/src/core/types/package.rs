//! Package-related types and implementations

use serde::{Deserialize, Serialize};
use sublime_package_tools::PackageInfo;
use sublime_standard_tools::monorepo::WorkspacePackage;

/// Status of a package version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionStatus {
    /// Version is stable and released
    Stable,
    /// Version is a snapshot/development version
    Snapshot {
        /// Git commit SHA for the snapshot
        sha: String,
    },
    /// Version is a pre-release
    PreRelease {
        /// Pre-release tag (e.g., "alpha", "beta", "rc.1")
        tag: String,
    },
    /// Version has pending changes
    Dirty,
}

/// Complete information about a package in the monorepo context
#[derive(Debug, Clone)]
pub struct MonorepoPackageInfo {
    /// Base package information from package-tools
    pub package_info: PackageInfo,

    /// Workspace package information from standard-tools
    pub workspace_package: WorkspacePackage,

    /// Whether this is an internal package (part of the monorepo)
    pub is_internal: bool,

    /// List of packages that depend on this package
    pub dependents: Vec<String>,

    /// External dependencies (not in the monorepo)
    pub dependencies_external: Vec<String>,

    /// Current version status
    pub version_status: VersionStatus,

    /// Changesets associated with this package
    pub changesets: Vec<super::Changeset>,
}

// Implementation moved to ../package.rs for better separation of concerns
