//! Package-related types and implementations

use serde::{Deserialize, Serialize};
use sublime_package_tools::PackageInfo;
use sublime_standard_tools::monorepo::WorkspacePackage;
use std::collections::HashMap;

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

    /// All dependencies of this package
    pub dependencies: Vec<PackageDependency>,

    /// External dependencies (not in the monorepo)
    pub dependencies_external: Vec<String>,

    /// Current version status
    pub version_status: VersionStatus,

    /// Changesets associated with this package
    pub changesets: Vec<super::Changeset>,
}

/// Type of package based on the package manager and ecosystem
///
/// Used by the PackageDiscoveryService to categorize packages found
/// in the monorepo for proper handling and parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageType {
    /// JavaScript/Node.js package (package.json)
    JavaScript,
    /// Rust package (Cargo.toml)
    Rust,
    /// Python package (pyproject.toml, setup.py)
    Python,
    /// Java package (pom.xml)
    Java,
    /// Go package (go.mod)
    Go,
    /// .NET package (*.csproj, *.fsproj)
    DotNet,
    /// Other/unknown package type
    Other(String),
}

/// Represents a dependency of a package
///
/// Used by the DependencyAnalysisService to track package dependencies
/// and their version requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageDependency {
    /// Name of the dependency
    pub name: String,
    /// Version requirement (e.g., "^1.0.0", ">=2.1.0")
    pub version_requirement: String,
    /// Type of dependency (runtime, dev, peer, etc.)
    pub dependency_type: DependencyType,
    /// Whether this is an optional dependency
    pub optional: bool,
    /// Additional metadata about the dependency
    pub metadata: HashMap<String, String>,
}

/// Type of dependency relationship
///
/// Categorizes dependencies by their usage and importance for the package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Runtime dependency required for normal operation
    Runtime,
    /// Development dependency only needed during development
    Development,
    /// Peer dependency that should be provided by the consuming project
    Peer,
    /// Optional dependency that provides additional features
    Optional,
    /// Build-time dependency needed for compilation/bundling
    Build,
}

// Implementation moved to ../package.rs for better separation of concerns
