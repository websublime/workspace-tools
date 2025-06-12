//! Core `MonorepoProject` implementation that integrates all base crates

use super::types::{MonorepoPackageInfo, MonorepoProject};
use crate::config::{ConfigManager, MonorepoConfig};
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

use sublime_git_tools::Repo;
use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package, RegistryManager};
use sublime_standard_tools::{
    filesystem::FileSystemManager,
    monorepo::{MonorepoDescriptor, PackageManager},
};

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
        // This will be implemented when we have the analysis module
        Ok(())
    }

    /// Build or rebuild the dependency graph
    pub fn build_dependency_graph(&mut self) -> Result<()> {
        // This will be implemented when we have the full package analysis
        Ok(())
    }
}
