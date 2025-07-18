//! Configuration for dependency upgrades
//!
//! Defines configuration options that control how dependency upgrades are
//! discovered and applied.

use crate::{Filter, VersionStability, VersionUpdateStrategy};
use serde::{Deserialize, Serialize};

/// Execution mode
///
/// Determines whether upgrades should be applied or only reported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Only report potential upgrades without applying them
    DryRun,
    /// Apply upgrades to packages
    Apply,
}

/// Default implementation for ExecutionMode
///
/// Defaults to DryRun mode for safety.
impl Default for ExecutionMode {
    fn default() -> Self {
        Self::DryRun
    }
}

/// Configuration for the dependency upgrader
///
/// Controls which dependencies to upgrade, what types of version updates to allow,
/// and various other behaviors of the upgrader.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{
///     UpgradeConfig, Filter, VersionUpdateStrategy,
///     VersionStability, ExecutionMode
/// };
///
/// let config = UpgradeConfig {
///     dependency_types: Filter::WithDevelopment,
///     update_strategy: VersionUpdateStrategy::MinorAndPatch,
///     version_stability: VersionStability::StableOnly,
///     target_packages: vec!["specific-package".to_string()],
///     target_dependencies: vec!["react".to_string(), "lodash".to_string()],
///     registries: vec!["https://registry.npmjs.org".to_string()],
///     execution_mode: ExecutionMode::DryRun,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeConfig {
    /// Which types of dependencies to include
    pub dependency_types: Filter,
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

/// Convert a VersionUpdateStrategy directly to a configuration
///
/// Creates a default configuration with the specified update strategy.
impl From<&VersionUpdateStrategy> for UpgradeConfig {
    fn from(update_strategy: &VersionUpdateStrategy) -> Self {
        Self {
            dependency_types: Filter::WithDevelopment,
            update_strategy: update_strategy.clone(),
            version_stability: VersionStability::StableOnly,
            target_packages: Vec::new(),
            target_dependencies: Vec::new(),
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        }
    }
}

/// Default implementation for UpgradeConfig
///
/// Creates a configuration with sensible defaults:
/// - Include development dependencies
/// - Allow minor and patch updates (not major)
/// - Only include stable versions
/// - No specific package or dependency targeting
/// - Use the default npm registry
/// - Run in dry-run mode
impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            dependency_types: Filter::WithDevelopment,
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
    /// Create a configuration with custom registries
    ///
    /// # Arguments
    ///
    /// * `registries` - List of registry URLs to use
    ///
    /// # Returns
    ///
    /// A new configuration with the specified registries and default values for other fields
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::UpgradeConfig;
    ///
    /// let config = UpgradeConfig::with_registries(vec![
    ///     "https://registry.npmjs.org".to_string(),
    ///     "https://npm.pkg.github.com".to_string(),
    /// ]);
    /// ```
    #[must_use]
    pub fn with_registries(registries: Vec<String>) -> Self {
        Self {
            dependency_types: Filter::WithDevelopment,
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            version_stability: VersionStability::StableOnly,
            target_packages: Vec::new(),
            target_dependencies: Vec::new(),
            registries,
            execution_mode: ExecutionMode::DryRun,
        }
    }
}
