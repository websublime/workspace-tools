//! The main dependency upgrader implementation.

use crate::error::Result;
use crate::registry::package::PackageRegistry;
use crate::types::dependency::Dependency;
use crate::types::package::Package;
use crate::types::version::VersionStability;
use crate::upgrader::config::{ExecutionMode, UpgradeConfig};
use crate::upgrader::status::UpgradeStatus;
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
pub struct DependencyUpgrader<R: PackageRegistry> {
    registry: R,
    config: UpgradeConfig,
    cache: HashMap<String, Vec<String>>,
}

impl<R: PackageRegistry> DependencyUpgrader<R> {
    /// Create a new dependency upgrader with the given registry and default configuration
    pub fn new(registry: R) -> Self {
        Self { registry, config: UpgradeConfig::default(), cache: HashMap::new() }
    }

    /// Create a new dependency upgrader with the given registry and configuration
    pub fn with_config(registry: R, config: UpgradeConfig) -> Self {
        Self { registry, config, cache: HashMap::new() }
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
            let versions = self.registry.get_all_versions(package_name)?;
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
        let consider_major = matches!(
            self.config.update_strategy,
            crate::types::version::VersionUpdateStrategy::AllUpdates
        );

        // Parse the base version from the requirement to properly handle prereleases
        // We'll consider a prerelease valid if it's for the same major.minor as what's allowed by the requirement
        let binding = requirement.to_string();
        let req_components = binding.trim_start_matches('^').trim_start_matches('~').split('.');
        let req_base = req_components.collect::<Vec<&str>>();

        for version_str in all_versions {
            // Parse the version
            let Ok(version) = Version::parse(&version_str) else { continue };

            // Handle prereleases specially if we're including them
            let satisfies_requirement = if !version.pre.is_empty() && include_prereleases {
                // For prereleases, we need special handling
                // A prerelease version satisfies a non-prerelease requirement if:
                // 1. The major.minor.patch version satisfies the requirement
                // 2. We've explicitly enabled prerelease versions

                // Create a non-prerelease version for checking
                let mut clean_version = version.clone();
                clean_version.pre = semver::Prerelease::EMPTY;

                // If the clean version satisfies the requirement, the prerelease is acceptable
                requirement.matches(&clean_version)
            } else {
                // For non-prerelease versions or when not including prereleases, use standard matching
                requirement.matches(&version)
            };

            // Apply additional filters for major upgrades if configured
            let passes_filters = if consider_major && !satisfies_requirement {
                // When allowing major upgrades, we might still want to consider this version
                // if it's higher than the requirement base

                // Extract the base version from the requirement
                match Version::parse(req_base.join(".").as_str()) {
                    Ok(base_version) => version >= base_version,
                    Err(_) => false,
                }
            } else {
                satisfies_requirement
            };

            if passes_filters {
                // If it's higher than our current highest, update it
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

        for version_str in all_versions {
            // Parse the version
            let Ok(version) = Version::parse(&version_str) else { continue };

            // Skip prereleases if not including them
            if !include_prereleases && !version.pre.is_empty() {
                continue;
            }

            // If it's higher than our current highest, update it
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
        // If there's no compatible version, we can't upgrade
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
            if latest != compatible {
                return UpgradeStatus::Constrained(latest.to_string());
            }
        }

        // Determine update type (patch, minor, major)
        if compatible_ver.major > current.major {
            UpgradeStatus::MajorAvailable(compatible.to_string())
        } else if compatible_ver.minor > current.minor {
            UpgradeStatus::MinorAvailable(compatible.to_string())
        } else if compatible_ver.patch > current.patch {
            UpgradeStatus::PatchAvailable(compatible.to_string())
        } else {
            // This shouldn't happen if our version comparison logic is correct
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

        // Find the latest version overall
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
