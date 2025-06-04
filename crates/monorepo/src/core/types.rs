//! Core types for monorepo project representation

use crate::config::Environment;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sublime_package_tools::PackageInfo;
use sublime_standard_tools::monorepo::WorkspacePackage;

/// Status of a package version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionStatus {
    /// Version is stable and released
    Stable,
    /// Version is a snapshot/development version
    Snapshot { sha: String },
    /// Version is a pre-release
    PreRelease { tag: String },
    /// Version has pending changes
    Dirty,
}

/// Changeset information for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    /// Unique identifier for the changeset
    pub id: String,
    
    /// Package this changeset applies to
    pub package: String,
    
    /// Type of version bump
    pub version_bump: crate::config::VersionBumpType,
    
    /// Description of the changes
    pub description: String,
    
    /// Branch where the changeset was created
    pub branch: String,
    
    /// Development environments where this has been deployed
    pub development_environments: Vec<Environment>,
    
    /// Whether this has been deployed to production
    pub production_deployment: bool,
    
    /// When the changeset was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Author of the changeset
    pub author: String,
    
    /// Status of the changeset
    pub status: ChangesetStatus,
}

/// Status of a changeset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangesetStatus {
    /// Changeset is pending application
    Pending,
    /// Changeset has been partially deployed
    PartiallyDeployed { environments: Vec<Environment> },
    /// Changeset has been fully deployed
    FullyDeployed { deployed_at: chrono::DateTime<chrono::Utc> },
    /// Changeset has been merged
    Merged { 
        merged_at: chrono::DateTime<chrono::Utc>,
        final_version: String,
    },
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
    pub changesets: Vec<Changeset>,
}

impl MonorepoPackageInfo {
    /// Create a new MonorepoPackageInfo
    pub fn new(
        package_info: PackageInfo,
        workspace_package: WorkspacePackage,
        is_internal: bool,
    ) -> Self {
        Self {
            package_info,
            workspace_package,
            is_internal,
            dependents: Vec::new(),
            dependencies_external: Vec::new(),
            version_status: VersionStatus::Stable,
            changesets: Vec::new(),
        }
    }
    
    /// Get the package name
    pub fn name(&self) -> &str {
        &self.workspace_package.name
    }
    
    /// Get the package version
    pub fn version(&self) -> &str {
        &self.workspace_package.version
    }
    
    /// Get the package path
    pub fn path(&self) -> &PathBuf {
        &self.workspace_package.absolute_path
    }
    
    /// Get the relative path from monorepo root
    pub fn relative_path(&self) -> &PathBuf {
        &self.workspace_package.location
    }
    
    /// Check if this package has pending changesets
    pub fn has_pending_changesets(&self) -> bool {
        self.changesets.iter().any(|cs| matches!(cs.status, ChangesetStatus::Pending))
    }
    
    /// Get pending changesets
    pub fn pending_changesets(&self) -> Vec<&Changeset> {
        self.changesets
            .iter()
            .filter(|cs| matches!(cs.status, ChangesetStatus::Pending))
            .collect()
    }
    
    /// Check if package is dirty (has uncommitted changes)
    pub fn is_dirty(&self) -> bool {
        matches!(self.version_status, VersionStatus::Dirty)
    }
}