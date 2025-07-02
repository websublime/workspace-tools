//! Version management types and result structures

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
    /// Incompatible versions between packages
    IncompatibleVersions,
    /// Invalid version format detected
    InvalidVersionFormat,
}
