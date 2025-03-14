//! JavaScript bindings for dependency upgrader.

use crate::errors::handle_pkg_result;
use crate::types::package::Package;
use crate::upgrader::config::UpgradeConfig;
use crate::upgrader::status::UpgradeStatus;
use napi::Result as NapiResult;
use napi_derive::napi;
use std::cell::RefCell;
use std::rc::Rc;
use ws_pkg::upgrader::{
    AvailableUpgrade as WsAvailableUpgrade, DependencyUpgrader as WsDependencyUpgrader,
};

/// JavaScript binding for ws_pkg::upgrader::AvailableUpgrade
#[napi(object)]
pub struct AvailableUpgrade {
    /// Package name containing the dependency
    pub package_name: String,
    /// Dependency name
    pub dependency_name: String,
    /// Current version of the dependency
    pub current_version: String,
    /// Latest available version that's compatible with requirements
    pub compatible_version: Option<String>,
    /// Latest overall version (may not be compatible with current requirements)
    pub latest_version: Option<String>,
    /// Status of this dependency's upgradability
    pub status: UpgradeStatus,
}

impl From<WsAvailableUpgrade> for AvailableUpgrade {
    fn from(upgrade: WsAvailableUpgrade) -> Self {
        Self {
            package_name: upgrade.package_name,
            dependency_name: upgrade.dependency_name,
            current_version: upgrade.current_version,
            compatible_version: upgrade.compatible_version,
            latest_version: upgrade.latest_version,
            status: upgrade.status.into(),
        }
    }
}

/// JavaScript binding for ws_pkg::upgrader::DependencyUpgrader
#[napi]
pub struct DependencyUpgrader {
    inner: WsDependencyUpgrader,
}

#[napi]
#[allow(clippy::new_without_default)]
impl DependencyUpgrader {
    /// Create a new dependency upgrader with default configuration
    ///
    /// @returns {DependencyUpgrader} A new dependency upgrader
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { inner: WsDependencyUpgrader::new() }
    }

    /// Create a new dependency upgrader with the given configuration
    ///
    /// @param {UpgradeConfig} config - The upgrade configuration
    /// @returns {DependencyUpgrader} A new dependency upgrader
    #[napi(factory)]
    pub fn with_config(config: UpgradeConfig) -> Self {
        Self { inner: WsDependencyUpgrader::with_config(config.into()) }
    }

    /// Set the configuration for the upgrader
    ///
    /// @param {UpgradeConfig} config - The upgrade configuration
    #[napi]
    pub fn set_config(&mut self, config: UpgradeConfig) {
        self.inner.set_config(config.into());
    }

    /// Check for upgrades for a single dependency
    ///
    /// @param {string} packageName - The name of the package containing the dependency
    /// @param {Dependency} dependency - The dependency to check for upgrades
    /// @returns {AvailableUpgrade} Information about available upgrades
    #[napi(ts_return_type = "AvailableUpgrade")]
    pub fn check_dependency_upgrade(
        &mut self,
        package_name: String,
        dependency: &crate::types::dependency::Dependency,
    ) -> NapiResult<AvailableUpgrade> {
        let dep = dependency.inner.borrow();
        let result = handle_pkg_result(self.inner.check_dependency_upgrade(&package_name, &dep))?;
        Ok(AvailableUpgrade::from(result))
    }

    /// Check all dependencies in a package for available upgrades
    ///
    /// @param {Package} package - The package to check for upgrades
    /// @returns {AvailableUpgrade[]} Array of available upgrades
    #[napi(ts_return_type = "AvailableUpgrade[]")]
    pub fn check_package_upgrades(
        &mut self,
        package: &Package,
    ) -> NapiResult<Vec<AvailableUpgrade>> {
        let result = handle_pkg_result(self.inner.check_package_upgrades(&package.inner))?;
        Ok(result.into_iter().map(AvailableUpgrade::from).collect())
    }

    /// Check all packages in a collection for available upgrades
    ///
    /// @param {Package[]} packages - The packages to check for upgrades
    /// @returns {AvailableUpgrade[]} Array of available upgrades
    #[napi(ts_return_type = "AvailableUpgrade[]")]
    pub fn check_all_upgrades(
        &mut self,
        packages: Vec<&Package>,
    ) -> NapiResult<Vec<AvailableUpgrade>> {
        // Create a vector of the inner packages
        let pkg_refs: Vec<ws_pkg::Package> =
            packages.into_iter().map(|p| p.inner.clone()).collect();

        // Now we can pass a slice reference to the vector
        let result = handle_pkg_result(self.inner.check_all_upgrades(&pkg_refs))?;

        Ok(result.into_iter().map(AvailableUpgrade::from).collect())
    }

    /// Apply upgrades to packages based on what was found
    ///
    /// @param {Package[]} packages - The packages to apply upgrades to
    /// @param {AvailableUpgrade[]} upgrades - The upgrades to apply
    /// @returns {AvailableUpgrade[]} The upgrades that were applied
    #[napi(ts_return_type = "AvailableUpgrade[]")]
    pub fn apply_upgrades(
        &self,
        packages: Vec<&Package>,
        upgrades: Vec<AvailableUpgrade>,
    ) -> NapiResult<Vec<AvailableUpgrade>> {
        // Convert JS packages to Rust Rc<RefCell<Package>> format
        let pkg_rcs: Vec<Rc<RefCell<ws_pkg::Package>>> =
            packages.into_iter().map(|p| Rc::new(RefCell::new(p.inner.clone()))).collect();

        // Convert JS upgrades to Rust AvailableUpgrade format
        let ws_upgrades: Vec<WsAvailableUpgrade> = upgrades
            .into_iter()
            .map(|u| WsAvailableUpgrade {
                package_name: u.package_name,
                dependency_name: u.dependency_name,
                current_version: u.current_version,
                compatible_version: u.compatible_version,
                latest_version: u.latest_version,
                status: match u.status {
                    UpgradeStatus::UpToDate => ws_pkg::upgrader::UpgradeStatus::UpToDate,
                    UpgradeStatus::PatchAvailable => {
                        ws_pkg::upgrader::UpgradeStatus::PatchAvailable("".to_string())
                    }
                    UpgradeStatus::MinorAvailable => {
                        ws_pkg::upgrader::UpgradeStatus::MinorAvailable("".to_string())
                    }
                    UpgradeStatus::MajorAvailable => {
                        ws_pkg::upgrader::UpgradeStatus::MajorAvailable("".to_string())
                    }
                    UpgradeStatus::Constrained => {
                        ws_pkg::upgrader::UpgradeStatus::Constrained("".to_string())
                    }
                    UpgradeStatus::CheckFailed => {
                        ws_pkg::upgrader::UpgradeStatus::CheckFailed("".to_string())
                    }
                },
            })
            .collect();

        // Apply upgrades
        let applied = handle_pkg_result(self.inner.apply_upgrades(&pkg_rcs, &ws_upgrades))?;

        // Convert back to JS format
        Ok(applied.into_iter().map(AvailableUpgrade::from).collect())
    }

    /// Generate a report of upgrades in a human-readable format
    ///
    /// @param {AvailableUpgrade[]} upgrades - The upgrades to report
    /// @returns {string} A human-readable report
    #[napi]
    pub fn generate_upgrade_report(&self, upgrades: Vec<AvailableUpgrade>) -> String {
        // Convert upgrades to Rust format
        let ws_upgrades: Vec<WsAvailableUpgrade> = upgrades
            .into_iter()
            .map(|u| WsAvailableUpgrade {
                package_name: u.package_name,
                dependency_name: u.dependency_name,
                current_version: u.current_version,
                compatible_version: u.compatible_version,
                latest_version: u.latest_version,
                status: match u.status {
                    UpgradeStatus::UpToDate => ws_pkg::upgrader::UpgradeStatus::UpToDate,
                    UpgradeStatus::PatchAvailable => {
                        ws_pkg::upgrader::UpgradeStatus::PatchAvailable("".to_string())
                    }
                    UpgradeStatus::MinorAvailable => {
                        ws_pkg::upgrader::UpgradeStatus::MinorAvailable("".to_string())
                    }
                    UpgradeStatus::MajorAvailable => {
                        ws_pkg::upgrader::UpgradeStatus::MajorAvailable("".to_string())
                    }
                    UpgradeStatus::Constrained => {
                        ws_pkg::upgrader::UpgradeStatus::Constrained("".to_string())
                    }
                    UpgradeStatus::CheckFailed => {
                        ws_pkg::upgrader::UpgradeStatus::CheckFailed("".to_string())
                    }
                },
            })
            .collect();

        self.inner.generate_upgrade_report(&ws_upgrades)
    }
}

#[cfg(test)]
mod upgrader_binding_tests {
    use super::*;
    use crate::upgrader::config::{ExecutionMode, UpgradeConfig};

    #[test]
    fn test_dependency_upgrader_creation() {
        // Test default creation
        DependencyUpgrader::new();

        // Test with config
        let config = UpgradeConfig {
            dependency_types: crate::graph::DependencyFilter::WithDevelopment,
            update_strategy: crate::types::version::VersionUpdateStrategy::MinorAndPatch,
            version_stability: crate::types::version::VersionStability::StableOnly,
            target_packages: vec![],
            target_dependencies: vec![],
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        };

        // Pass the config directly, not as a reference
        DependencyUpgrader::with_config(config.clone());

        // Set config
        let mut upgrader = DependencyUpgrader::new();
        upgrader.set_config(config);
    }

    #[test]
    fn test_available_upgrade_conversion() {
        // Create a WsAvailableUpgrade
        let ws_upgrade = ws_pkg::upgrader::AvailableUpgrade {
            package_name: "test-pkg".to_string(),
            dependency_name: "dep1".to_string(),
            current_version: "^1.0.0".to_string(),
            compatible_version: Some("1.1.0".to_string()),
            latest_version: Some("2.0.0".to_string()),
            status: ws_pkg::upgrader::UpgradeStatus::MinorAvailable("1.1.0".to_string()),
        };

        // Convert to our AvailableUpgrade
        let upgrade = AvailableUpgrade::from(ws_upgrade);

        // Verify the conversion
        assert_eq!(upgrade.package_name, "test-pkg");
        assert_eq!(upgrade.dependency_name, "dep1");
        assert_eq!(upgrade.current_version, "^1.0.0");
        assert_eq!(upgrade.compatible_version, Some("1.1.0".to_string()));
        assert_eq!(upgrade.latest_version, Some("2.0.0".to_string()));
        assert!(matches!(upgrade.status, UpgradeStatus::MinorAvailable));
    }
}
