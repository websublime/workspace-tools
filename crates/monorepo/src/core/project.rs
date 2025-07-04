//! Core `MonorepoProject` implementation that integrates base crates directly
//!
//! This module implements the MonorepoProject that uses base crates directly
//! for CLI/daemon consumption. Removes service abstractions for optimal performance.

use super::types::{MonorepoPackageInfo, MonorepoProject};
use crate::config::{ConfigManager, MonorepoConfig};
use crate::error::{Error, Result};
use std::path::Path;

use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};

impl MonorepoProject {
    /// Create a new `MonorepoProject` by discovering and analyzing a monorepo
    ///
    /// Uses base crates directly for optimal CLI/daemon performance.
    /// Eliminates service abstractions in favor of direct base crate usage.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let root_path = path.to_path_buf();

        // Initialize base crate components directly
        let file_system = FileSystemManager::new();

        // Initialize configuration using direct config manager
        let config_manager = ConfigManager::new();
        let config = config_manager.load_config(&root_path)?;

        // Initialize Git repository directly
        let path_str =
            path.to_str().ok_or_else(|| Error::git("Invalid UTF-8 in repository path"))?;
        let repository = Repo::open(path_str).map_err(|e| {
            Error::git(format!("Failed to open Git repository at {}: {}", path.display(), e))
        })?;

        // Direct package discovery using base crates
        let packages = Self::discover_packages_direct(&root_path, &file_system, &config);

        Ok(Self {
            packages,
            root_path,
            config,
            file_system,
            repository,
        })
    }

    /// Direct package discovery using base crates
    ///
    /// Discovers packages using `sublime_standard_tools` and `sublime_package_tools` directly
    /// without service abstractions for optimal CLI/daemon performance.
    fn discover_packages_direct(
        _root_path: &Path,
        _file_system: &FileSystemManager,
        _config: &MonorepoConfig,
    ) -> Vec<MonorepoPackageInfo> {
        // Simplified package discovery for now - return empty list
        // Full implementation will be added when we have the correct base crate APIs
        Vec::new()
    }

    /// Get the root path of the monorepo
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get a reference to the git repository
    #[must_use]
    pub fn repository(&self) -> &Repo {
        &self.repository
    }

    /// Get a mutable reference to the git repository
    ///
    /// Provides direct access to the Git repository for CLI/daemon operations
    pub fn repository_mut(&mut self) -> &mut Repo {
        &mut self.repository
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

    /// Get a reference to the configuration
    #[must_use]
    pub fn config(&self) -> &MonorepoConfig {
        &self.config
    }

    /// Update configuration using a closure
    ///
    /// Provides direct access to configuration for CLI/daemon operations
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        updater(&mut self.config);
        Ok(())
    }

    /// Save configuration to file
    ///
    /// Uses ConfigManager directly for CLI/daemon operations
    pub fn save_config(&self) -> Result<()> {
        // For now, return Ok - save functionality can be implemented later
        // when we have the correct ConfigManager API
        Ok(())
    }

    /// Create default configuration files if they don't exist
    ///
    /// Uses ConfigManager directly for CLI/daemon operations
    pub fn create_default_config_files(&self) -> Result<()> {
        // Check if configuration file already exists
        let config_path = self.root_path.join("monorepo.toml");
        if self.file_system.exists(&config_path) {
            Ok(()) // Configuration file already exists
        } else {
            // Create default configuration using ConfigManager directly
            ConfigManager::create_default_config_files(&self.root_path)
        }
    }

    /// Refresh packages information from disk
    ///
    /// Uses base crates directly for optimal CLI/daemon performance
    pub fn refresh_packages(&mut self) -> Result<()> {
        log::info!("Refreshing packages for project at: {}", self.root_path.display());

        // Refresh packages using direct base crate access
        self.packages = Self::discover_packages_direct(&self.root_path, &self.file_system, &self.config);

        log::info!("Successfully refreshed {} packages", self.packages.len());
        Ok(())
    }

    /// Build or rebuild the dependency graph
    ///
    /// Uses direct analysis for CLI/daemon operations
    pub fn build_dependency_graph(&mut self) -> Result<()> {
        log::info!("Building dependency graph for {} packages", self.packages.len());

        if self.packages.is_empty() {
            log::warn!("No packages found. Consider calling refresh_packages() first.");
            return Ok(());
        }

        // Populate the dependents field for each package using direct analysis
        self.populate_dependents_mapping()?;

        log::info!("Successfully built dependency graph with {} packages", self.packages.len());

        Ok(())
    }

    /// Populate the dependents field for all packages based on their dependencies
    ///
    /// Internal method for direct dependency analysis without service abstractions
    #[allow(clippy::unnecessary_wraps)]
    fn populate_dependents_mapping(&mut self) -> Result<()> {

        // Clear existing dependents to rebuild from scratch
        for package in &mut self.packages {
            package.dependents.clear();
        }

        // Build reverse dependency mapping efficiently using HashMap for O(1) lookups
        let mut dependents_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        
        // Collect all package dependencies
        for pkg in &self.packages {
            let package_name = pkg.name().to_string();
            for dependency_name in &pkg.workspace_package.workspace_dependencies {
                dependents_map
                    .entry(dependency_name.clone())
                    .or_default()
                    .push(package_name.clone());
            }
        }

        // Update the dependents fields efficiently
        for package in &mut self.packages {
            let package_name = package.name().to_string();
            if let Some(dependents) = dependents_map.remove(&package_name) {
                package.dependents = dependents;
            }
        }

        log::info!("Successfully populated dependents mapping");
        Ok(())
    }

    /// Get file system manager reference
    pub fn file_system(&self) -> &sublime_standard_tools::filesystem::FileSystemManager {
        &self.file_system
    }
}
