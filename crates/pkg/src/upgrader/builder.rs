//! Dependency upgrader implementation
//!
//! Provides the main implementation for checking and applying dependency upgrades
//! across packages in a workspace.

use crate::{
    errors::{DependencyResolutionError, PackageRegistryError},
    AvailableUpgrade, Dependency, ExecutionMode, Package, RegistryManager, UpgradeConfig,
    UpgradeStatus, VersionStability, VersionUpdateStrategy,
};
use semver::{Version, VersionReq};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Package dependency upgrader
///
/// Checks for available dependency upgrades and applies them according to
/// configurable strategies and filters.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{Upgrader, UpgradeConfig, RegistryManager};
///
/// // Create with default configuration
/// let upgrader = Upgrader::new();
///
/// // Create with custom configuration
/// let config = UpgradeConfig::default();
/// let registry_manager = RegistryManager::new();
/// let custom_upgrader = Upgrader::create(config, registry_manager);
/// ```
#[derive(Clone)]
pub struct Upgrader {
    registry_manager: RegistryManager,
    config: UpgradeConfig,
    cache: HashMap<String, Vec<String>>,
}

impl Upgrader {
    /// Create a new dependency upgrader with the given registry and default configuration
    ///
    /// # Returns
    ///
    /// A new `Upgrader` instance with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Upgrader;
    ///
    /// let upgrader = Upgrader::new();
    /// ```
    pub fn new() -> Self {
        Self {
            registry_manager: RegistryManager::new(),
            config: UpgradeConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Create an upgrader with custom configuration and registry manager
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the upgrader
    /// * `registry_manager` - Registry manager for package lookups
    ///
    /// # Returns
    ///
    /// A new `Upgrader` instance with the specified configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Upgrader, UpgradeConfig, RegistryManager};
    ///
    /// let config = UpgradeConfig::default();
    /// let registry_manager = RegistryManager::new();
    /// let upgrader = Upgrader::create(config, registry_manager);
    /// ```
    pub fn create(config: UpgradeConfig, registry_manager: RegistryManager) -> Self {
        Self { registry_manager, config, cache: HashMap::new() }
    }

    /// Create a new dependency upgrader with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the upgrader
    ///
    /// # Returns
    ///
    /// A new `Upgrader` instance with the specified configuration and default registry
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Upgrader, UpgradeConfig, VersionUpdateStrategy};
    ///
    /// let config = UpgradeConfig {
    ///     update_strategy: VersionUpdateStrategy::MinorAndPatch,
    ///     ..UpgradeConfig::default()
    /// };
    /// let upgrader = Upgrader::with_config(config);
    /// ```
    pub fn with_config(config: UpgradeConfig) -> Self {
        Self { registry_manager: RegistryManager::new(), config, cache: HashMap::new() }
    }

    /// Create with a specific registry manager
    ///
    /// # Arguments
    ///
    /// * `registry_manager` - Registry manager for package lookups
    ///
    /// # Returns
    ///
    /// A new `Upgrader` instance with default configuration and the specified registry manager
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Upgrader, RegistryManager};
    ///
    /// let registry_manager = RegistryManager::new();
    /// let upgrader = Upgrader::with_registry_manager(registry_manager);
    /// ```
    pub fn with_registry_manager(registry_manager: RegistryManager) -> Self {
        Self { registry_manager, config: UpgradeConfig::default(), cache: HashMap::new() }
    }

    /// Get the registry manager
    ///
    /// # Returns
    ///
    /// Reference to the upgrader's registry manager
    #[must_use]
    pub fn registry_manager(&self) -> &RegistryManager {
        &self.registry_manager
    }

    /// Get a mutable reference to the registry manager
    ///
    /// # Returns
    ///
    /// Mutable reference to the upgrader's registry manager
    pub fn registry_manager_mut(&mut self) -> &mut RegistryManager {
        &mut self.registry_manager
    }

    /// Set the configuration for the upgrader
    ///
    /// # Arguments
    ///
    /// * `config` - New configuration to use
    pub fn set_config(&mut self, config: UpgradeConfig) {
        self.config = config;
    }

    /// Get the current configuration
    ///
    /// # Returns
    ///
    /// Reference to the upgrader's configuration
    #[must_use]
    pub fn config(&self) -> &UpgradeConfig {
        &self.config
    }

    /// Get all available versions of a package, using the cache if available
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to look up
    ///
    /// # Returns
    ///
    /// List of available versions, or a `PackageRegistryError` if the lookup fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    fn get_cached_versions(
        &mut self,
        package_name: &str,
    ) -> Result<Vec<String>, PackageRegistryError> {
        if !self.cache.contains_key(package_name) {
            let versions = self.registry_manager.get_all_versions(package_name)?;
            self.cache.insert(package_name.to_string(), versions);
        }

        Ok(self.cache.get(package_name).unwrap_or(&Vec::new()).clone())
    }

    /// Find the highest version that satisfies the given requirement
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    /// * `requirement` - Version requirement to satisfy
    /// * `include_prereleases` - Whether to include prerelease versions
    ///
    /// # Returns
    ///
    /// The highest satisfying version string, or `None` if no satisfying version exists,
    /// or a `PackageRegistryError` if the lookup fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    fn find_highest_satisfying_version(
        &mut self,
        package_name: &str,
        requirement: &VersionReq,
        include_prereleases: bool,
    ) -> Result<Option<String>, PackageRegistryError> {
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
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    /// * `include_prereleases` - Whether to include prerelease versions
    ///
    /// # Returns
    ///
    /// The highest version string, or `None` if no versions exist,
    /// or a `PackageRegistryError` if the lookup fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    fn find_latest_version(
        &mut self,
        package_name: &str,
        include_prereleases: bool,
    ) -> Result<Option<String>, PackageRegistryError> {
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

    /// Determine the upgrade status by comparing versions
    ///
    /// # Arguments
    ///
    /// * `current_version` - Current version string
    /// * `compatible_version` - Compatible version string (if available)
    /// * `latest_version` - Latest version string (if available)
    ///
    /// # Returns
    ///
    /// Status indicating what kind of upgrade is available
    #[allow(clippy::unused_self)]
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
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package containing the dependency
    /// * `dependency` - The dependency to check
    ///
    /// # Returns
    ///
    /// Information about available upgrades, or a `PackageRegistryError` if the check fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    /// - Package versions cannot be compared due to invalid format
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Upgrader, Dependency, Package};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut upgrader = Upgrader::new();
    ///
    /// // Create a dependency to check
    /// let dep = Dependency::new("react", "^16.0.0")?;
    ///
    /// // Check for upgrades
    /// let upgrade = upgrader.check_dependency_upgrade("my-app", &dep)?;
    /// println!("Upgrade status: {}", upgrade.status);
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::needless_borrow)]
    pub fn check_dependency_upgrade(
        &mut self,
        package_name: &str,
        dependency: &Dependency,
    ) -> Result<AvailableUpgrade, PackageRegistryError> {
        let dependency_name = dependency.name();
        let current_version = dependency.version().to_string();

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
    ///
    /// # Arguments
    ///
    /// * `package` - The package to check
    ///
    /// # Returns
    ///
    /// List of available upgrades, or a `PackageRegistryError` if any check fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    /// - Package versions cannot be compared due to invalid format
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Upgrader, Package, Dependency};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut upgrader = Upgrader::new();
    ///
    /// // Create a package with dependencies
    /// let dep1 = Rc::new(RefCell::new(Dependency::new("react", "^16.0.0")?));
    /// let dep2 = Rc::new(RefCell::new(Dependency::new("lodash", "^4.0.0")?));
    /// let package = Package::new("my-app", "1.0.0", Some(vec![dep1, dep2]))?;
    ///
    /// // Check for upgrades
    /// let upgrades = upgrader.check_package_upgrades(&package)?;
    /// println!("Found {} possible upgrades", upgrades.len());
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::needless_borrow)]
    pub fn check_package_upgrades(
        &mut self,
        package: &Package,
    ) -> Result<Vec<AvailableUpgrade>, PackageRegistryError> {
        // Skip if this package is not in our target list (if specified)
        if !self.config.target_packages.is_empty() {
            let package_in_target = self.config.target_packages.iter().any(|p| p == package.name());

            if !package_in_target {
                return Ok(Vec::new());
            }
        }

        let mut upgrades = Vec::new();

        for dep in package.dependencies() {
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
                    if matches!(self.config.update_strategy, VersionUpdateStrategy::AllUpdates) {
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
    ///
    /// # Arguments
    ///
    /// * `packages` - Collection of packages to check
    ///
    /// # Returns
    ///
    /// List of available upgrades across all packages, or a `PackageRegistryError` if any check fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    /// - Package versions cannot be compared due to invalid format
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Upgrader, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut upgrader = Upgrader::new();
    /// let packages = vec![]; // Add your packages here
    ///
    /// // Check for upgrades across all packages
    /// let upgrades = upgrader.check_all_upgrades(&packages)?;
    /// println!("Found {} possible upgrades in total", upgrades.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn check_all_upgrades(
        &mut self,
        packages: &[Package],
    ) -> Result<Vec<AvailableUpgrade>, PackageRegistryError> {
        let mut all_upgrades = Vec::new();

        for package in packages {
            let package_upgrades = self.check_package_upgrades(package)?;
            all_upgrades.extend(package_upgrades);
        }

        Ok(all_upgrades)
    }

    /// Apply upgrades to packages based on what was found
    ///
    /// # Arguments
    ///
    /// * `packages` - Collection of packages to update
    /// * `upgrades` - List of upgrades to apply
    ///
    /// # Returns
    ///
    /// List of upgrades that were actually applied, or a `DependencyResolutionError` if any update fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The dependency with the specified name is not found in the package
    /// - The new version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Upgrader, Package, UpgradeConfig, ExecutionMode};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create an upgrader in Apply mode
    /// let config = UpgradeConfig {
    ///     execution_mode: ExecutionMode::Apply,
    ///     ..UpgradeConfig::default()
    /// };
    /// let upgrader = Upgrader::with_config(config);
    ///
    /// // Prepare packages (as Rc<RefCell<>> for mutability)
    /// let packages = vec![]; // Add your Rc<RefCell<Package>> references here
    ///
    /// // Find upgrades
    /// let upgrades = vec![]; // From check_all_upgrades or similar
    ///
    /// // Apply upgrades
    /// let applied = upgrader.apply_upgrades(&packages, &upgrades)?;
    /// println!("Applied {} upgrades", applied.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn apply_upgrades(
        &self,
        packages: &[Rc<RefCell<Package>>],
        upgrades: &[AvailableUpgrade],
    ) -> Result<Vec<AvailableUpgrade>, DependencyResolutionError> {
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
                    let mut package = package_rc.borrow_mut();

                    // Apply the upgrade
                    package.update_dependency_version(&upgrade.dependency_name, new_version)?;

                    applied_upgrades.push(upgrade.clone());
                }
            }
        }

        Ok(applied_upgrades)
    }

    /// Generate a report of upgrades in a human-readable format
    ///
    /// # Arguments
    ///
    /// * `upgrades` - List of available upgrades
    ///
    /// # Returns
    ///
    /// A formatted string report of the upgrades
    #[must_use]
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Upgrader, AvailableUpgrade, UpgradeStatus};
    ///
    /// // Prepare some upgrades (normally from check_all_upgrades)
    /// let upgrades = vec![
    ///     AvailableUpgrade {
    ///         package_name: "my-app".to_string(),
    ///         dependency_name: "react".to_string(),
    ///         current_version: "^16.0.0".to_string(),
    ///         compatible_version: Some("^17.0.0".to_string()),
    ///         latest_version: Some("^17.0.0".to_string()),
    ///         status: UpgradeStatus::MajorAvailable("^17.0.0".to_string()),
    ///     }
    /// ];
    ///
    /// // Generate report
    /// let report = Upgrader::generate_upgrade_report(&upgrades);
    /// println!("{}", report);
    /// ```
    pub fn generate_upgrade_report(upgrades: &[AvailableUpgrade]) -> String {
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
