//! The main dependency upgrader implementation.

use crate::error::Result;
use crate::registry::RegistryManager;
use crate::types::dependency::Dependency;
use crate::types::package::Package;
use crate::types::version::VersionStability;
use crate::upgrader::config::{ExecutionMode, UpgradeConfig};
use crate::upgrader::status::UpgradeStatus;
use crate::VersionUpdateStrategy;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Represents an available upgrade for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(skip)]
    pub status: UpgradeStatus,
}

impl fmt::Display for AvailableUpgrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} in {}: {} -> {}",
            self.dependency_name,
            self.package_name,
            self.current_version,
            match &self.compatible_version {
                Some(v) => v,
                None => "no compatible version",
            }
        )
    }
}

/// The DependencyUpgrader provides methods to check and upgrade dependencies
pub struct DependencyUpgrader {
    registry_manager: RegistryManager,
    config: UpgradeConfig,
    cache: HashMap<String, Vec<String>>,
}

impl DependencyUpgrader {
    /// Create a new dependency upgrader with the given registry and default configuration
    pub fn new() -> Self {
        Self {
            registry_manager: RegistryManager::new(),
            config: UpgradeConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Create a new dependency upgrader with the given registry and configuration
    pub fn with_config(config: UpgradeConfig) -> Self {
        Self { registry_manager: RegistryManager::new(), config, cache: HashMap::new() }
    }

    /// Create with a specific registry manager
    pub fn with_registry_manager(registry_manager: RegistryManager) -> Self {
        Self { registry_manager, config: UpgradeConfig::default(), cache: HashMap::new() }
    }

    /// Get the registry manager
    pub fn registry_manager(&self) -> &RegistryManager {
        &self.registry_manager
    }

    /// Get a mutable reference to the registry manager
    pub fn registry_manager_mut(&mut self) -> &mut RegistryManager {
        &mut self.registry_manager
    }

    /// Set the configuration for the upgrader
    pub fn set_config(&mut self, config: UpgradeConfig) {
        self.config = config;
    }

    /// Get the current configuration
    pub fn config(&self) -> &UpgradeConfig {
        &self.config
    }

    /// Get all available versions of a package, using the cache if available
    fn get_cached_versions(&mut self, package_name: &str) -> Result<Vec<String>> {
        if !self.cache.contains_key(package_name) {
            let versions = self.registry_manager.get_all_versions(package_name)?;
            self.cache.insert(package_name.to_string(), versions);
        }

        Ok(self.cache.get(package_name).unwrap_or(&Vec::new()).clone())
    }

    /// Find the highest version that satisfies the given requirement
    fn find_highest_satisfying_version(
        &mut self,
        package_name: &str,
        requirement: &VersionReq,
        include_prereleases: bool,
    ) -> Result<Option<String>> {
        let all_versions = self.get_cached_versions(package_name)?;

        if all_versions.is_empty() {
            return Ok(None);
        }

        let mut highest: Option<Version> = None;

        // Get the strategy
        let allow_major = matches!(self.config.update_strategy, VersionUpdateStrategy::AllUpdates);

        // Parse the base version from requirement
        let req_str = requirement.to_string();
        let base_version = req_str
            .trim_start_matches('^')
            .trim_start_matches('~')
            .split('.')
            .collect::<Vec<&str>>();

        let base_major = base_version.first().and_then(|v| v.parse::<u64>().ok());

        for version_str in all_versions {
            let Ok(version) = Version::parse(&version_str) else { continue };

            // Skip prereleases unless explicitly included
            if !include_prereleases && !version.pre.is_empty() {
                continue;
            }

            // For MinorAndPatch strategy, skip versions with different major number
            if !allow_major {
                if let Some(base_maj) = base_major {
                    if version.major != base_maj {
                        continue;
                    }
                }
            }

            // Check if this version satisfies the requirement
            let satisfies = if allow_major {
                // When allowing major updates, we're more permissive
                requirement.matches(&version)
                    || version > Version::new(base_major.unwrap_or(0), 0, 0)
            } else {
                requirement.matches(&version)
            };

            if satisfies {
                if let Some(ref current_highest) = highest {
                    if version > *current_highest {
                        highest = Some(version);
                    }
                } else {
                    highest = Some(version);
                }
            }
        }

        Ok(highest.map(|v| v.to_string()))
    }

    /// Find the highest version available, regardless of requirements
    fn find_latest_version(
        &mut self,
        package_name: &str,
        include_prereleases: bool,
    ) -> Result<Option<String>> {
        let all_versions = self.get_cached_versions(package_name)?;

        if all_versions.is_empty() {
            return Ok(None);
        }

        let mut highest: Option<Version> = None;

        // Get the current update strategy
        let allow_major = matches!(self.config.update_strategy, VersionUpdateStrategy::AllUpdates);

        // If we're not allowing major updates, we need to know the current major version
        let current_major = if allow_major {
            None
        } else {
            // Try to find the lowest major version available
            all_versions.iter().filter_map(|v| Version::parse(v).ok()).map(|v| v.major).min()
        };

        for version_str in all_versions {
            let Ok(version) = Version::parse(&version_str) else { continue };

            // Skip prereleases unless explicitly included
            if !include_prereleases && !version.pre.is_empty() {
                continue;
            }

            // If we're not allowing major updates, skip versions with different major numbers
            if !allow_major {
                if let Some(base_major) = current_major {
                    if version.major != base_major {
                        continue;
                    }
                }
            }

            // Update highest version if this one is higher
            if let Some(ref current_highest) = highest {
                if version > *current_highest {
                    highest = Some(version);
                }
            } else {
                highest = Some(version);
            }
        }

        Ok(highest.map(|v| v.to_string()))
    }

    #[allow(clippy::unused_self)]
    /// Determine the upgrade status by comparing versions
    fn determine_upgrade_status(
        &self,
        current_version: &str,
        compatible_version: Option<&str>,
        latest_version: Option<&str>,
    ) -> UpgradeStatus {
        // If there's no compatible version, we're up to date
        let Some(compatible) = compatible_version else {
            return UpgradeStatus::UpToDate;
        };

        // Parse current version
        let Ok(current) =
            Version::parse(current_version.trim_start_matches('^').trim_start_matches('~'))
        else {
            return UpgradeStatus::CheckFailed("Invalid current version".to_string());
        };

        // Parse compatible version
        let Ok(compatible_ver) = Version::parse(compatible) else {
            return UpgradeStatus::CheckFailed("Invalid compatible version".to_string());
        };

        // If compatible version is the same as current, we're up to date
        if compatible_ver == current {
            return UpgradeStatus::UpToDate;
        }

        // Check if there's a latest version that's not compatible
        if let Some(latest) = latest_version {
            if latest != compatible
                && !matches!(self.config.update_strategy, VersionUpdateStrategy::MinorAndPatch)
            {
                return UpgradeStatus::Constrained(latest.to_string());
            }
        }

        // Determine update type
        if compatible_ver.major > current.major {
            UpgradeStatus::MajorAvailable(compatible.to_string())
        } else if compatible_ver.minor > current.minor {
            UpgradeStatus::MinorAvailable(compatible.to_string())
        } else if compatible_ver.patch > current.patch {
            UpgradeStatus::PatchAvailable(compatible.to_string())
        } else if !current.pre.is_empty() && compatible_ver.pre.is_empty() {
            // Handle prerelease to stable upgrade
            UpgradeStatus::PatchAvailable(compatible.to_string())
        } else {
            UpgradeStatus::UpToDate
        }
    }

    /// Check for upgrades for a single dependency
    pub fn check_dependency_upgrade(
        &mut self,
        package_name: &str,
        dependency: &Dependency,
    ) -> Result<AvailableUpgrade> {
        let dependency_name = dependency.name();
        let current_version = dependency.version_str();

        // Skip if this dependency is not in our target list (if specified)
        if !self.config.target_dependencies.is_empty()
            && !self.config.target_dependencies.iter().any(|d| d == dependency_name)
        {
            return Ok(AvailableUpgrade {
                package_name: package_name.to_string(),
                dependency_name: dependency_name.to_string(),
                current_version: current_version.clone(),
                compatible_version: None,
                latest_version: None,
                status: UpgradeStatus::UpToDate,
            });
        }

        // Parse the current version requirement
        let version_req = dependency.version();

        // Find the highest version that satisfies the requirement
        let include_prereleases =
            matches!(self.config.version_stability, VersionStability::IncludePrerelease);

        let compatible_version = self.find_highest_satisfying_version(
            dependency_name,
            &version_req,
            include_prereleases,
        )?;

        // Find the latest version overall (this will include major versions)
        let latest_version = self.find_latest_version(dependency_name, include_prereleases)?;

        // Clean the current version for comparison
        let clean_version = current_version.trim_start_matches('^').trim_start_matches('~');

        // Determine the upgrade status
        let status = self.determine_upgrade_status(
            clean_version,
            compatible_version.as_deref(),
            latest_version.as_deref(),
        );

        Ok(AvailableUpgrade {
            package_name: package_name.to_string(),
            dependency_name: dependency_name.to_string(),
            current_version,
            compatible_version,
            latest_version,
            status,
        })
    }

    /// Check all dependencies in a package for available upgrades
    pub fn check_package_upgrades(&mut self, package: &Package) -> Result<Vec<AvailableUpgrade>> {
        // Skip if this package is not in our target list (if specified)
        if !self.config.target_packages.is_empty() {
            let package_in_target = self.config.target_packages.iter().any(|p| p == package.name());

            if !package_in_target {
                return Ok(Vec::new());
            }
        }

        let mut upgrades = Vec::new();

        for dep_rc in package.dependencies() {
            let dep = dep_rc.borrow();

            // Skip if this dependency is not in our target list (if specified)
            if !self.config.target_dependencies.is_empty()
                && !self.config.target_dependencies.iter().any(|d| d == dep.name())
            {
                continue;
            }

            let upgrade = self.check_dependency_upgrade(package.name(), &dep)?;

            // Only include upgrades where there's an actual upgrade available
            match &upgrade.status {
                UpgradeStatus::UpToDate | UpgradeStatus::CheckFailed(_) => {
                    // Skip upgrades that are up to date or failed to check
                }
                UpgradeStatus::MajorAvailable(_) => {
                    // Include major updates only if configured to do so
                    if matches!(
                        self.config.update_strategy,
                        crate::types::version::VersionUpdateStrategy::AllUpdates
                    ) {
                        upgrades.push(upgrade);
                    }
                }
                _ => {
                    upgrades.push(upgrade);
                }
            }
        }

        Ok(upgrades)
    }

    /// Check all packages in a collection for available upgrades
    pub fn check_all_upgrades(&mut self, packages: &[Package]) -> Result<Vec<AvailableUpgrade>> {
        let mut all_upgrades = Vec::new();

        for package in packages {
            let package_upgrades = self.check_package_upgrades(package)?;
            all_upgrades.extend(package_upgrades);
        }

        Ok(all_upgrades)
    }

    /// Apply upgrades to packages based on what was found
    pub fn apply_upgrades(
        &self,
        packages: &[Rc<RefCell<Package>>],
        upgrades: &[AvailableUpgrade],
    ) -> Result<Vec<AvailableUpgrade>> {
        // Skip actually applying anything if this is a dry run
        if matches!(self.config.execution_mode, ExecutionMode::DryRun) {
            return Ok(upgrades.to_vec());
        }

        let mut applied_upgrades = Vec::new();

        for upgrade in upgrades {
            if let Some(new_version) = &upgrade.compatible_version {
                // Find the package this upgrade applies to
                if let Some(package_rc) =
                    packages.iter().find(|p| p.borrow().name() == upgrade.package_name)
                {
                    let package = package_rc.borrow();

                    // Apply the upgrade
                    package.update_dependency_version(&upgrade.dependency_name, new_version)?;

                    applied_upgrades.push(upgrade.clone());
                }
            }
        }

        Ok(applied_upgrades)
    }

    /// Generate a report of upgrades in a human-readable format
    pub fn generate_upgrade_report(&self, upgrades: &[AvailableUpgrade]) -> String {
        if upgrades.is_empty() {
            return "All dependencies are up to date.".to_string();
        }

        let mut report = String::new();
        report.push_str("Available dependency upgrades:\n");

        // Group upgrades by package
        let mut by_package: HashMap<&str, Vec<&AvailableUpgrade>> = HashMap::new();
        for upgrade in upgrades {
            by_package.entry(&upgrade.package_name).or_default().push(upgrade);
        }

        // Generate report for each package
        for (package, pkg_upgrades) in by_package {
            report.push_str(&format!("\nPackage: {package}\n"));

            for upgrade in pkg_upgrades {
                let status_str = match &upgrade.status {
                    UpgradeStatus::PatchAvailable(v) => format!("patch: {v}"),
                    UpgradeStatus::MinorAvailable(v) => format!("minor: {v}"),
                    UpgradeStatus::MajorAvailable(v) => format!("major: {v}"),
                    UpgradeStatus::Constrained(v) => format!("constrained (latest: {v})"),
                    UpgradeStatus::UpToDate => "up to date".to_string(),
                    UpgradeStatus::CheckFailed(msg) => format!("check failed: {msg}"),
                };

                report.push_str(&format!(
                    "  {} {} -> {} ({})\n",
                    upgrade.dependency_name,
                    upgrade.current_version.trim_start_matches('^').trim_start_matches('~'),
                    upgrade.compatible_version.as_ref().unwrap_or(&"N/A".to_string()),
                    status_str
                ));
            }
        }

        report
    }
}
