//! Configuration for dependency upgrades.

use crate::graph::DependencyFilter;
use crate::types::version::{VersionStability, VersionUpdateStrategy};
use serde::{Deserialize, Serialize};

/// Execution mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Only report potential upgrades without applying them
    DryRun,
    /// Apply upgrades to packages
    Apply,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::DryRun
    }
}

/// Configuration for the dependency upgrader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeConfig {
    /// Which types of dependencies to include
    pub dependency_types: DependencyFilter,
    /// Which types of version updates to include
    pub update_strategy: VersionUpdateStrategy,
    /// Whether to include prerelease versions
    pub version_stability: VersionStability,
    /// Specific packages to upgrade (if empty, upgrade all)
    pub target_packages: Vec<String>,
    /// Specific dependencies to upgrade (if empty, upgrade all)
    pub target_dependencies: Vec<String>,
    /// Additional registries to check for updates
    pub registries: Vec<String>,
    /// Whether to actually apply the upgrades or just report them
    pub execution_mode: ExecutionMode,
}

impl From<&VersionUpdateStrategy> for UpgradeConfig {
    fn from(update_strategy: &VersionUpdateStrategy) -> Self {
        Self {
            dependency_types: DependencyFilter::WithDevelopment,
            update_strategy: update_strategy.clone(),
            version_stability: VersionStability::StableOnly,
            target_packages: Vec::new(),
            target_dependencies: Vec::new(),
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        }
    }
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            dependency_types: DependencyFilter::WithDevelopment,
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            version_stability: VersionStability::StableOnly,
            target_packages: Vec::new(),
            target_dependencies: Vec::new(),
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        }
    }
}

impl UpgradeConfig {
    pub fn with_registries(registries: Vec<String>) -> Self {
        Self {
            dependency_types: DependencyFilter::WithDevelopment,
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            version_stability: VersionStability::StableOnly,
            target_packages: Vec::new(),
            target_dependencies: Vec::new(),
            registries,
            execution_mode: ExecutionMode::DryRun,
        }
    }
}
