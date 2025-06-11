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
    Snapshot { 
        /// Git commit SHA for the snapshot
        sha: String 
    },
    /// Version is a pre-release
    PreRelease { 
        /// Pre-release tag (e.g., "alpha", "beta", "rc.1")
        tag: String 
    },
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
    PartiallyDeployed { 
        /// Environments where this has been deployed
        environments: Vec<Environment> 
    },
    /// Changeset has been fully deployed
    FullyDeployed { 
        /// When it was fully deployed
        deployed_at: chrono::DateTime<chrono::Utc> 
    },
    /// Changeset has been merged
    Merged { 
        /// When the changeset was merged
        merged_at: chrono::DateTime<chrono::Utc>,
        /// Final version after merge
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
    /// Create a new `MonorepoPackageInfo`
    #[must_use] pub fn new(
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
    #[must_use] pub fn name(&self) -> &str {
        &self.workspace_package.name
    }
    
    /// Get the package version
    #[must_use] pub fn version(&self) -> &str {
        &self.workspace_package.version
    }
    
    /// Get the package path
    #[must_use] pub fn path(&self) -> &PathBuf {
        &self.workspace_package.absolute_path
    }
    
    /// Get the relative path from monorepo root
    #[must_use] pub fn relative_path(&self) -> &PathBuf {
        &self.workspace_package.location
    }
    
    /// Check if this package has pending changesets
    #[must_use] pub fn has_pending_changesets(&self) -> bool {
        self.changesets.iter().any(|cs| matches!(cs.status, ChangesetStatus::Pending))
    }
    
    /// Get pending changesets
    #[must_use] pub fn pending_changesets(&self) -> Vec<&Changeset> {
        self.changesets
            .iter()
            .filter(|cs| matches!(cs.status, ChangesetStatus::Pending))
            .collect()
    }
    
    /// Check if package is dirty (has uncommitted changes)
    #[must_use] pub fn is_dirty(&self) -> bool {
        matches!(self.version_status, VersionStatus::Dirty)
    }
}

// ===== VERSION MANAGEMENT TYPES =====

/// Result of a versioning operation
#[derive(Debug)]
pub struct VersioningResult {
    /// Primary package updates
    pub primary_updates: Vec<PackageVersionUpdate>,
    /// Updates propagated to dependent packages
    pub propagated_updates: Vec<PackageVersionUpdate>,
    /// Any conflicts detected
    pub conflicts: Vec<VersionConflict>,
    /// Dependency resolution updates
    pub dependency_updates: sublime_package_tools::ResolutionResult,
}

/// Information about a package version update
#[derive(Debug, Clone)]
pub struct PackageVersionUpdate {
    /// Package name
    pub package_name: String,
    /// Previous version
    pub old_version: String,
    /// New version
    pub new_version: String,
    /// Type of version bump applied
    pub bump_type: crate::config::VersionBumpType,
    /// Reason for the update
    pub reason: String,
}

/// Result of version change propagation
#[derive(Debug, Clone, Default)]
pub struct PropagationResult {
    /// List of propagated updates
    pub updates: Vec<PackageVersionUpdate>,
    /// Any conflicts detected during propagation
    pub conflicts: Vec<VersionConflict>,
}

/// A version conflict that needs resolution
#[derive(Debug, Clone)]
pub struct VersionConflict {
    /// Package with the conflict
    pub package_name: String,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Description of the conflict
    pub description: String,
    /// Suggested resolution strategy
    pub resolution_strategy: String,
}

/// Types of version conflicts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// Package has pending changesets
    PendingChangesets,
    /// Working directory is dirty
    DirtyWorkingDirectory,
    /// Potential breaking change in non-major bump
    PotentialBreakingChange,
    /// Dependency version mismatch
    DependencyMismatch,
    /// Circular dependency detected
    CircularDependency,
}

/// Analysis of version impact across the monorepo
#[derive(Debug, Clone)]
pub struct VersionImpactAnalysis {
    /// Impact analysis for each affected package
    pub affected_packages: std::collections::HashMap<String, PackageImpactAnalysis>,
    /// Total number of packages affected
    pub total_packages_affected: usize,
    /// Breaking changes analysis
    pub breaking_changes: Vec<BreakingChangeAnalysis>,
    /// Dependency chain impacts
    pub dependency_chain_impacts: Vec<DependencyChainImpact>,
    /// Maximum depth of propagation
    pub estimated_propagation_depth: usize,
}

/// Impact analysis for a single package
#[derive(Debug, Clone)]
pub struct PackageImpactAnalysis {
    /// Package name
    pub package_name: String,
    /// Number of direct dependents
    pub direct_dependents: usize,
    /// Number of transitive dependents
    pub transitive_dependents: usize,
    /// Suggested version bump
    pub suggested_version_bump: crate::config::VersionBumpType,
    /// Whether this change has breaking potential
    pub breaking_potential: bool,
    /// Risk score for propagation (0.0 to 10.0)
    pub propagation_risk: f32,
}

/// Analysis of breaking changes
#[derive(Debug, Clone)]
pub struct BreakingChangeAnalysis {
    /// Package with breaking change
    pub package_name: String,
    /// Reason for breaking change classification
    pub reason: String,
    /// List of packages affected by this breaking change
    pub affected_dependents: Vec<String>,
}

/// Impact analysis for dependency chains
#[derive(Debug, Clone)]
pub struct DependencyChainImpact {
    /// Root package of the chain
    pub root_package: String,
    /// Length of the dependency chain
    pub chain_length: usize,
    /// All packages in the chain
    pub affected_packages: Vec<String>,
    /// Maximum propagation depth
    pub max_propagation_depth: usize,
}

/// A comprehensive versioning plan
#[derive(Debug, Clone)]
pub struct VersioningPlan {
    /// Steps to execute in order
    pub steps: Vec<VersioningPlanStep>,
    /// Total number of packages to update
    pub total_packages: usize,
    /// Estimated execution duration
    pub estimated_duration: std::time::Duration,
    /// Potential conflicts
    pub conflicts: Vec<VersionConflict>,
    /// Impact analysis
    pub impact_analysis: VersionImpactAnalysis,
}

/// A single step in a versioning plan
#[derive(Debug, Clone)]
pub struct VersioningPlanStep {
    /// Package to update
    pub package_name: String,
    /// Current version
    pub current_version: String,
    /// Planned version bump
    pub planned_version_bump: crate::config::VersionBumpType,
    /// Reason for the bump
    pub reason: String,
    /// Dependencies that will be updated
    pub dependencies_to_update: Vec<String>,
    /// Order of execution (lower numbers execute first)
    pub execution_order: usize,
}

// ===== VERSIONING STRATEGY IMPLEMENTATIONS =====

/// Default versioning strategy implementation
#[derive(Debug, Clone)]
pub struct DefaultVersioningStrategy;

/// Conservative versioning strategy - minimal propagation
#[derive(Debug, Clone)]
pub struct ConservativeVersioningStrategy;

/// Aggressive versioning strategy - propagates all changes
#[derive(Debug, Clone)]
pub struct AggressiveVersioningStrategy;