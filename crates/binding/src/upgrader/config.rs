//! JavaScript bindings for upgrade configuration.

use crate::graph::DependencyFilter;
use crate::types::version::{VersionStability, VersionUpdateStrategy};
use napi_derive::napi;
use ws_pkg::upgrader::{ExecutionMode as WsExecutionMode, UpgradeConfig as WsUpgradeConfig};

/// JavaScript binding for ws_pkg::upgrader::config::ExecutionMode
#[napi]
#[derive(Clone)]
pub enum ExecutionMode {
    /// Only report potential upgrades without applying them
    DryRun,
    /// Apply upgrades to packages
    Apply,
}

impl From<ExecutionMode> for WsExecutionMode {
    fn from(mode: ExecutionMode) -> Self {
        match mode {
            ExecutionMode::DryRun => Self::DryRun,
            ExecutionMode::Apply => Self::Apply,
        }
    }
}

impl From<WsExecutionMode> for ExecutionMode {
    fn from(mode: WsExecutionMode) -> Self {
        match mode {
            WsExecutionMode::DryRun => Self::DryRun,
            WsExecutionMode::Apply => Self::Apply,
        }
    }
}

/// JavaScript binding for ws_pkg::upgrader::config::UpgradeConfig
#[napi(object)]
#[derive(Clone)]
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

impl From<UpgradeConfig> for WsUpgradeConfig {
    fn from(config: UpgradeConfig) -> Self {
        Self {
            dependency_types: config.dependency_types.into(),
            update_strategy: config.update_strategy.into(),
            version_stability: config.version_stability.into(),
            target_packages: config.target_packages,
            target_dependencies: config.target_dependencies,
            registries: config.registries,
            execution_mode: config.execution_mode.into(),
        }
    }
}

impl From<WsUpgradeConfig> for UpgradeConfig {
    fn from(config: WsUpgradeConfig) -> Self {
        Self {
            dependency_types: config.dependency_types.into(),
            update_strategy: config.update_strategy.into(),
            version_stability: config.version_stability.into(),
            target_packages: config.target_packages,
            target_dependencies: config.target_dependencies,
            registries: config.registries,
            execution_mode: config.execution_mode.into(),
        }
    }
}

#[napi]
pub fn create_default_upgrade_config() -> UpgradeConfig {
    UpgradeConfig::from(WsUpgradeConfig::default())
}

#[napi]
pub fn create_upgrade_config_from_strategy(strategy: VersionUpdateStrategy) -> UpgradeConfig {
    UpgradeConfig::from(WsUpgradeConfig::from(&strategy.into()))
}

#[napi]
pub fn create_upgrade_config_with_registries(registries: Vec<String>) -> UpgradeConfig {
    UpgradeConfig::from(WsUpgradeConfig::with_registries(registries))
}

#[cfg(test)]
mod config_binding_tests {
    use super::*;

    #[test]
    fn test_execution_mode_conversion() {
        // Test conversion from ExecutionMode to WsExecutionMode
        let mode = ExecutionMode::Apply;
        let ws_mode = WsExecutionMode::from(mode);
        assert!(matches!(ws_mode, WsExecutionMode::Apply));

        // Test conversion from WsExecutionMode to ExecutionMode
        let mode_back = ExecutionMode::from(ws_mode);
        assert!(matches!(mode_back, ExecutionMode::Apply));
    }

    #[test]
    fn test_upgrade_config_conversion() {
        // Create an UpgradeConfig
        let config = UpgradeConfig {
            dependency_types: DependencyFilter::WithDevelopment,
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            version_stability: VersionStability::StableOnly,
            target_packages: vec!["pkg1".to_string(), "pkg2".to_string()],
            target_dependencies: vec!["dep1".to_string()],
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        };

        // Convert to WsUpgradeConfig
        let ws_config = WsUpgradeConfig::from(config.clone());

        // Convert back to UpgradeConfig
        let config_back = UpgradeConfig::from(ws_config);

        // Verify fields
        assert!(matches!(config_back.dependency_types, DependencyFilter::WithDevelopment));
        assert!(matches!(config_back.update_strategy, VersionUpdateStrategy::MinorAndPatch));
        assert!(matches!(config_back.version_stability, VersionStability::StableOnly));
        assert_eq!(config_back.target_packages, vec!["pkg1".to_string(), "pkg2".to_string()]);
        assert_eq!(config_back.target_dependencies, vec!["dep1".to_string()]);
        assert_eq!(config_back.registries, vec!["https://registry.npmjs.org".to_string()]);
        assert!(matches!(config_back.execution_mode, ExecutionMode::DryRun));
    }

    #[test]
    fn test_create_default_upgrade_config() {
        let config = create_default_upgrade_config();
        assert!(matches!(config.dependency_types, DependencyFilter::WithDevelopment));
        assert!(matches!(config.update_strategy, VersionUpdateStrategy::MinorAndPatch));
        assert!(matches!(config.version_stability, VersionStability::StableOnly));
        assert!(config.target_packages.is_empty());
        assert!(config.target_dependencies.is_empty());
        assert_eq!(config.registries, vec!["https://registry.npmjs.org".to_string()]);
        assert!(matches!(config.execution_mode, ExecutionMode::DryRun));
    }

    #[test]
    fn test_create_upgrade_config_from_strategy() {
        let config = create_upgrade_config_from_strategy(VersionUpdateStrategy::AllUpdates);
        assert!(matches!(config.update_strategy, VersionUpdateStrategy::AllUpdates));
    }

    #[test]
    fn test_create_upgrade_config_with_registries() {
        let registries = vec!["https://custom-registry.com".to_string()];
        let config = create_upgrade_config_with_registries(registries.clone());
        assert_eq!(config.registries, registries);
    }
}
