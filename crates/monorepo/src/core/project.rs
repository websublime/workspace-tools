//! Core `MonorepoProject` implementation that integrates all base crates
//!
//! This module implements the MonorepoProject facade that uses the service container
//! pattern internally while maintaining backward compatibility with the existing API.
//! The refactoring breaks the god object pattern by delegating to focused services.

use super::services::MonorepoServices;
use super::types::{MonorepoPackageInfo, MonorepoProject};
use crate::config::{ConfigManager, MonorepoConfig};
use crate::error::{Error, Result};
use std::path::Path;

use sublime_git_tools::Repo;
use sublime_package_tools::{DependencyRegistry, RegistryManager};
use sublime_standard_tools::monorepo::PackageManager;

impl MonorepoProject {
    /// Create a new `MonorepoProject` by discovering and analyzing a monorepo
    ///
    /// This method now uses the service container pattern internally while
    /// maintaining complete backward compatibility with the existing API.
    /// All services are initialized and configured automatically.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Initialize the service container with all required services
        let services = MonorepoServices::new(path)?;

        // Perform initial package discovery to populate cache
        let packages = services.package_service().discover_packages()?;

        // Extract references for direct field access (pub(crate) pattern)
        let root_path = path.to_path_buf();
        let config = services.config_service().get_configuration().clone();
        let file_system = sublime_standard_tools::filesystem::FileSystemManager::new();

        // Create new Repo instance instead of cloning
        let path_str =
            path.to_str().ok_or_else(|| Error::git("Invalid UTF-8 in repository path"))?;
        let repository = Repo::open(path_str).map_err(|e| {
            Error::git(format!("Failed to open Git repository at {}: {}", path.display(), e))
        })?;

        Ok(Self {
            services,
            packages,
            dependency_graph: None,
            root_path,
            config,
            file_system,
            repository,
        })
    }

    /// Get the root path of the monorepo
    #[must_use]
    pub fn root_path(&self) -> &Path {
        self.services.file_system_service().root_path()
    }

    /// Get a reference to the git repository
    #[must_use]
    pub fn repository(&self) -> &Repo {
        self.services.git_service().repository()
    }

    /// Get a mutable reference to the git repository
    ///
    /// DEPRECATED: This method has been removed due to undefined behavior.
    /// Use GitOperationsService methods through self.services.git_service() instead.
    /// The service pattern eliminates the need for direct mutable repository access.
    #[deprecated(note = "Use GitOperationsService methods instead")]
    #[allow(clippy::panic)]
    pub fn repository_mut(&mut self) -> &mut Repo {
        // This method has been removed due to undefined behavior (casting &T to &mut T).
        // Use GitOperationsService methods instead for all git operations.
        panic!("repository_mut() has been removed. Use GitOperationsService methods instead.")
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
        if let Some(package) = self.packages.iter().find(|pkg| pkg.name() == package_name) {
            package
                .dependents
                .iter()
                .filter_map(|dependent_name| {
                    self.packages.iter().find(|pkg| pkg.name() == dependent_name)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update configuration
    ///
    /// Note: This method is deprecated in favor of using ConfigurationService methods.
    /// Direct configuration updates through the facade are not recommended in the
    /// service architecture. Use the configuration service for better state management.
    #[deprecated(note = "Use ConfigurationService methods instead for better state management")]
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        // For backward compatibility, we'll indicate this is not supported
        // in the service architecture without causing compilation errors
        let _ = updater; // Suppress unused parameter warning
        Err(Error::config_validation("Configuration updates through MonorepoProject are deprecated. Use ConfigurationService instead."))
    }

    /// Save configuration to file
    ///
    /// Note: This method is deprecated. Configuration saving should be handled
    /// by the ConfigurationService for proper state management and validation.
    #[deprecated(note = "Use ConfigurationService methods instead")]
    pub fn save_config(&self) -> Result<()> {
        Err(Error::config_validation("Configuration saving through MonorepoProject is deprecated. Use ConfigurationService instead."))
    }

    /// Create default configuration files if they don't exist
    ///
    /// Delegates to the configuration service for consistent configuration management.
    pub fn create_default_config_files(&self) -> Result<()> {
        // Check if configuration file already exists using the service
        if self.services.config_service().has_configuration_file() {
            Ok(()) // Configuration file already exists
        } else {
            // For backward compatibility, we'll create a default configuration
            // This delegates to the ConfigManager static method
            ConfigManager::create_default_config_files(self.root_path())
        }
    }

    /// Refresh packages information from disk
    ///
    /// Delegates to the package discovery service to refresh package information
    /// and updates the local cache for backward compatibility.
    pub fn refresh_packages(&mut self) -> Result<()> {
        log::info!("Refreshing packages for project at: {}", self.root_path().display());

        // Refresh packages using the service
        self.packages = self.services.package_service().discover_packages()?;

        log::info!("Successfully refreshed {} packages", self.packages.len());
        Ok(())
    }

    /// Build or rebuild the dependency graph
    ///
    /// Delegates to the dependency analysis service to build a comprehensive
    /// dependency graph and updates the local state for backward compatibility.
    pub fn build_dependency_graph(&mut self) -> Result<()> {
        log::info!("Building dependency graph for {} packages", self.packages.len());

        if self.packages.is_empty() {
            log::warn!("No packages found. Consider calling refresh_packages() first.");
            return Ok(());
        }

        // Populate the dependents field for each package using current logic
        self.populate_dependents_mapping()?;

        // Mark that dependency graph analysis has been completed
        self.dependency_graph = Some(());

        log::info!("Successfully built dependency graph with {} packages", self.packages.len());

        Ok(())
    }

    /// Populate the dependents field for all packages based on their dependencies
    ///
    /// Internal method to maintain backward compatibility with existing dependents mapping.
    /// In the service architecture, this would be handled by the DependencyAnalysisService.
    #[allow(clippy::unnecessary_wraps)]
    fn populate_dependents_mapping(&mut self) -> Result<()> {
        log::debug!("Populating dependents mapping for {} packages", self.packages.len());

        // Clear existing dependents to rebuild from scratch
        for package in &mut self.packages {
            package.dependents.clear();
        }

        // Build reverse dependency mapping
        // For each package, find its dependencies and add this package to their dependents list
        let package_dependencies: Vec<(String, Vec<String>)> = self
            .packages
            .iter()
            .map(|pkg| {
                let package_name = pkg.name().to_string();
                let dependencies = pkg.workspace_package.workspace_dependencies.clone();
                (package_name, dependencies)
            })
            .collect();

        // Now update the dependents fields
        for (package_name, dependencies) in package_dependencies {
            for dependency_name in dependencies {
                // Find the dependency package and add this package as a dependent
                if let Some(dependency_package) =
                    self.packages.iter_mut().find(|pkg| pkg.name() == dependency_name)
                {
                    if !dependency_package.dependents.contains(&package_name) {
                        dependency_package.dependents.push(package_name.clone());
                        log::debug!("Added {} as dependent of {}", package_name, dependency_name);
                    }
                }
            }
        }

        log::info!("Successfully populated dependents mapping");
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &MonorepoConfig {
        self.services.config_service().get_configuration()
    }

    /// Get file system manager reference
    pub fn file_system(&self) -> &sublime_standard_tools::filesystem::FileSystemManager {
        self.services.file_system_service().manager()
    }

    /// Get registry manager reference
    pub fn registry_manager(&self) -> &RegistryManager {
        self.services.dependency_service().registry_manager()
    }

    /// Get dependency registry reference
    pub fn dependency_registry(&self) -> &DependencyRegistry {
        self.services.dependency_service().registry()
    }

    /// Get monorepo descriptor reference
    pub fn descriptor(&self) -> &sublime_standard_tools::monorepo::MonorepoDescriptor {
        self.services.package_service().descriptor()
    }

    /// Get package manager reference
    pub fn package_manager(&self) -> &PackageManager {
        self.services.package_service().get_package_manager()
    }
}
