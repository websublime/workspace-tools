//! Core `MonorepoProject` implementation that integrates all base crates

use super::types::{MonorepoPackageInfo, MonorepoProject};
use crate::config::{ConfigManager, MonorepoConfig};
use crate::error::{Error, Result};
use std::path::Path;

use sublime_git_tools::Repo;
use sublime_package_tools::{DependencyGraph, DependencyRegistry, RegistryManager};
use sublime_standard_tools::{filesystem::FileSystemManager, monorepo::PackageManager};

impl MonorepoProject {
    /// Create a new `MonorepoProject` by discovering and analyzing a monorepo
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Open git repository
        let repository =
            Repo::open(path.to_str().ok_or_else(|| Error::project_init("Invalid path encoding"))?)?;

        // Detect monorepo using standard-tools
        let detector = sublime_standard_tools::monorepo::MonorepoDetector::new();
        let descriptor = detector.detect_monorepo(path)?;

        // Detect package manager
        let package_manager = PackageManager::detect(path)?;

        // Create file system manager
        let file_system = FileSystemManager::new();

        // Load or create configuration
        let (config_manager, config) = Self::load_configuration(path)?;

        // Create registry manager
        let mut registry_manager = RegistryManager::new();
        if registry_manager.load_from_npmrc(None).is_ok() {
            // Successfully loaded npmrc configuration
        }

        // Create dependency registry with package registry
        let dependency_registry = DependencyRegistry::with_package_registry(Box::new(
            sublime_package_tools::LocalRegistry::default(),
        ));

        // Initialize empty packages list - will be populated by analyze_packages
        let packages = Vec::new();

        Ok(Self {
            repository,
            descriptor,
            package_manager,
            dependency_registry,
            registry_manager,
            config_manager,
            file_system,
            packages,
            dependency_graph: None,
            config,
            root_path: path.to_path_buf(),
        })
    }

    /// Load configuration from file or create default
    fn load_configuration(path: &Path) -> Result<(ConfigManager, MonorepoConfig)> {
        if let Some(config_path) = ConfigManager::find_config_file(path) {
            let manager = ConfigManager::load_from_file(config_path)?;
            let config = manager.get()?;
            Ok((manager, config))
        } else {
            // Create default configuration
            let config = MonorepoConfig::default();
            let manager = ConfigManager::with_config(config.clone());
            Ok((manager, config))
        }
    }

    /// Get the root path of the monorepo
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get a package by name
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<&MonorepoPackageInfo> {
        self.packages.iter().find(|p| p.name() == name)
    }

    /// Get mutable reference to a package by name
    pub fn get_package_mut(&mut self, name: &str) -> Option<&mut MonorepoPackageInfo> {
        self.packages.iter_mut().find(|p| p.name() == name)
    }

    /// Get all internal packages (part of the monorepo)
    #[must_use]
    pub fn internal_packages(&self) -> Vec<&MonorepoPackageInfo> {
        self.packages.iter().filter(|p| p.is_internal).collect()
    }

    /// Get all external dependencies across all packages
    #[must_use]
    pub fn external_dependencies(&self) -> Vec<String> {
        let mut deps = Vec::new();
        for package in &self.packages {
            deps.extend(package.dependencies_external.clone());
        }
        deps.sort();
        deps.dedup();
        deps
    }

    /// Check if a package name is internal to the monorepo
    #[must_use]
    pub fn is_internal_package(&self, name: &str) -> bool {
        self.packages.iter().any(|p| p.name() == name && p.is_internal)
    }

    /// Get packages that depend on a given package
    #[must_use]
    pub fn get_dependents(&self, package_name: &str) -> Vec<&MonorepoPackageInfo> {
        self.packages.iter().filter(|p| p.dependents.contains(&package_name.to_string())).collect()
    }

    /// Update configuration
    pub fn update_config<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        self.config_manager.update(updater)
    }

    /// Save configuration to file
    pub fn save_config(&self) -> Result<()> {
        self.config_manager.save()
    }

    /// Create default configuration files if they don't exist
    pub fn create_default_config_files(&self) -> Result<()> {
        ConfigManager::create_default_config_files(&self.root_path)
    }

    /// Refresh packages information from disk
    pub fn refresh_packages(&mut self) -> Result<()> {
        log::info!("Refreshing packages for project at: {}", self.root_path.display());

        // Use the detector to get fresh monorepo information
        let detector = sublime_standard_tools::monorepo::MonorepoDetector::new();
        let fresh_descriptor = detector.detect_monorepo(&self.root_path)?;

        log::debug!("Detected {} packages from fresh scan", fresh_descriptor.packages().len());

        // Convert discovered packages to MonorepoPackageInfo
        let mut refreshed_packages = Vec::new();

        for package in fresh_descriptor.packages() {
            log::debug!("Processing package: {} at {}", package.name, package.location.display());

            // Create PackageInfo using the workspace package information
            let package_info = sublime_package_tools::PackageInfo::new(
                sublime_package_tools::Package::new(&package.name, &package.version, None)?,
                package.location.join("package.json").to_string_lossy().to_string(),
                package.location.to_string_lossy().to_string(),
                package.absolute_path.to_string_lossy().to_string(),
                serde_json::Value::Object(serde_json::Map::new()), // Will be loaded from actual package.json
            );

            // Create MonorepoPackageInfo with workspace information
            let monorepo_package = MonorepoPackageInfo::new(
                package_info,
                package.clone(),
                true, // Assume internal packages for now
            );

            refreshed_packages.push(monorepo_package);
        }

        // Update the descriptor with fresh information
        self.descriptor = fresh_descriptor;

        // Update the packages list
        self.packages = refreshed_packages;

        log::info!("Successfully refreshed {} packages", self.packages.len());
        Ok(())
    }

    /// Build or rebuild the dependency graph
    pub fn build_dependency_graph(&mut self) -> Result<()> {
        log::info!("Building dependency graph for {} packages", self.packages.len());

        if self.packages.is_empty() {
            log::warn!("No packages found. Consider calling refresh_packages() first.");
            return Ok(());
        }

        // Extract Package instances for the dependency graph
        let mut packages_for_graph = Vec::new();

        for pkg_info in &self.packages {
            // Clone the Package from the Rc<RefCell<Package>>
            let package = pkg_info.package_info.package.borrow().clone();
            packages_for_graph.push(package);
        }

        log::debug!("Creating dependency graph from {} packages", packages_for_graph.len());

        // Create dependency graph from packages slice and detect cycles
        let graph = DependencyGraph::from(packages_for_graph.as_slice());

        // The detect_circular_dependencies method returns a reference for method chaining
        let _graph_ref = graph.detect_circular_dependencies();

        // Log workspace dependencies for debugging
        for package_info in &self.packages {
            let package_name = package_info.name();

            // Log workspace dependencies for visibility
            for workspace_dep in &package_info.workspace_package.workspace_dependencies {
                log::debug!("Workspace dependency: {} -> {}", package_name, workspace_dep);
            }
        }

        // Check for cycles and log results
        if graph.has_cycles() {
            log::warn!(
                "Dependency graph contains cycles - this may cause issues in package resolution"
            );
            let cycles = graph.get_cycle_strings();
            for cycle in cycles {
                log::warn!("Circular dependency detected: {}", cycle.join(" -> "));
            }
        } else {
            log::info!("Dependency graph is acyclic - no circular dependencies detected");
        }

        // Mark that dependency graph analysis has been completed
        // In a future iteration, we could store summary information about the graph
        self.dependency_graph = Some(());

        log::info!("Successfully built dependency graph with {} packages", self.packages.len());

        Ok(())
    }
}
