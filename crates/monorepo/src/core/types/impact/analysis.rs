//! Version impact analysis types

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
