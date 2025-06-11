//! Versioning plan types

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
    pub conflicts: Vec<super::VersionConflict>,
    /// Impact analysis
    pub impact_analysis: super::VersionImpactAnalysis,
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