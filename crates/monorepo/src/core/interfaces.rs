//! Core interfaces for dependency injection and separation of concerns
//!
//! This module provides focused traits that allow components to depend only on 
//! specific functionality instead of the entire MonorepoProject structure.
//! This follows the Interface Segregation Principle and enables better testing
//! and decoupling.

use std::path::Path;
use std::sync::Arc;
use sublime_git_tools::{GitChangedFile, Repo};
use sublime_package_tools::RegistryManager;
use sublime_standard_tools::filesystem::FileSystem;
use sublime_standard_tools::monorepo::MonorepoDescriptor;
use crate::config::MonorepoConfig;
use crate::core::{MonorepoPackageInfo, MonorepoProject};
use crate::error::Result;

/// Provider for package-related operations
/// 
/// Components that need access to package information, dependency relationships,
/// and package metadata should depend on this trait instead of the full project.
pub trait PackageProvider {
    /// Get all packages in the monorepo
    fn packages(&self) -> &[MonorepoPackageInfo];
    
    /// Get a specific package by name
    fn get_package(&self, name: &str) -> Option<&MonorepoPackageInfo>;
    
    /// Get packages that depend on the given package
    fn get_dependents(&self, package_name: &str) -> Vec<&MonorepoPackageInfo>;
    
    /// Get the root path of the monorepo
    fn root_path(&self) -> &Path;
}

/// Provider for configuration access
/// 
/// Components that need configuration data should depend on this trait
/// for clean separation of concerns.
pub trait ConfigProvider {
    /// Get the current monorepo configuration
    fn config(&self) -> &MonorepoConfig;
}

/// Provider for file system operations
/// 
/// Components that need to read/write files should use this interface
/// instead of accessing the file system manager directly.
pub trait FileSystemProvider {
    /// Read a file as a string
    fn read_file_string(&self, path: &Path) -> Result<String>;
    
    /// Write a string to a file
    fn write_file_string(&self, path: &Path, content: &str) -> Result<()>;
    
    /// Check if a path exists
    fn path_exists(&self, path: &Path) -> bool;
    
    /// Create a directory
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    
    /// Remove a file
    fn remove_file(&self, path: &Path) -> Result<()>;
    
    /// Walk directory and list all files
    fn walk_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>>;
}

/// Provider for Git operations
/// 
/// Components that need Git functionality should depend on this trait
/// for repository operations.
pub trait GitProvider {
    /// Get the Git repository
    fn repository(&self) -> &Repo;
    
    /// Get the repository root path
    fn repository_root(&self) -> &Path;
    
    /// Get files changed since a specific commit
    fn get_changed_files_since(&self, since: &str) -> Result<Vec<GitChangedFile>>;
    
    /// Get the current branch name
    fn current_branch(&self) -> Result<String>;
    
    /// Get current commit SHA
    fn current_sha(&self) -> Result<String>;
    
    /// Get the diverged commit between branches
    fn get_diverged_commit(&self, base_branch: &str) -> Result<String>;
    
    /// Get all files changed since a specific SHA with their status
    fn get_all_files_changed_since_sha_with_status(&self, sha: &str) -> Result<Vec<GitChangedFile>>;
    
    /// Get all files changed since a specific SHA  
    fn get_all_files_changed_since_sha(&self, sha: &str) -> Result<Vec<String>>;
}

/// Provider for package registry operations
/// 
/// Components that need registry access should use this interface.
pub trait RegistryProvider {
    /// Get the registry manager
    fn registry_manager(&self) -> &RegistryManager;
    
    /// Get registry type for a URL
    fn get_registry_type(&self, url: &str) -> String;
}

/// Provider for workspace operations and patterns
/// 
/// Components that need workspace configuration, patterns, and package manager
/// metadata should use this interface for complex workspace analysis.
pub trait WorkspaceProvider {
    /// Get the monorepo root path
    fn root_path(&self) -> &Path;
    
    /// Get workspace patterns for the current package manager
    fn get_workspace_patterns(&self) -> Result<Vec<String>>;
    
    /// Get effective workspace patterns after validation
    fn get_effective_workspace_patterns(&self) -> Result<Vec<String>>;
    
    /// Get package manager specific patterns
    fn get_package_manager_patterns(&self) -> Result<Vec<String>>;
    
    /// Get package manager type and metadata
    fn get_package_manager_info(&self) -> Result<(String, std::collections::HashMap<String, String>)>;
    
    /// Get workspace configuration section
    fn get_workspace_config(&self) -> Result<MonorepoDescriptor>;
    
    /// Validate workspace configuration
    fn validate_workspace_config(&self) -> Result<Vec<String>>;
    
    /// Get package manager commands configuration
    fn get_package_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>>;
}

/// Provider for comprehensive package discovery and metadata
/// 
/// Components that need complete package enumeration with full metadata
/// should use this interface instead of the basic PackageProvider.
pub trait PackageDiscoveryProvider {
    /// Get all packages with complete metadata
    fn get_all_packages_with_metadata(&self) -> &[MonorepoPackageInfo];
    
    /// Get package descriptor with full package information
    fn get_package_descriptor(&self) -> &MonorepoDescriptor;
    
    /// Find packages matching a specific pattern
    fn find_packages_by_pattern(&self, pattern: &str) -> Result<Vec<&MonorepoPackageInfo>>;
    
    /// Get package locations and paths
    fn get_package_locations(&self) -> Vec<(String, &Path)>;
    
    /// Get orphaned packages (packages not matching workspace patterns)
    fn find_orphaned_packages(&self, patterns: &[String]) -> Vec<&MonorepoPackageInfo>;
}

/// Enhanced configuration provider for workspace management
/// 
/// Extension of ConfigProvider that adds workspace-specific configuration
/// methods for complex analysis operations.
pub trait EnhancedConfigProvider: ConfigProvider {
    /// Get effective workspace patterns after resolution
    fn get_effective_patterns(&self) -> Result<Vec<String>>;
    
    /// Validate workspace configuration and return issues  
    fn validate_config(&self) -> Result<Vec<String>>;
    
    /// Get package manager command mappings
    fn get_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>>;
    
    /// Get workspace section from configuration
    fn get_workspace_section(&self) -> Result<&crate::config::WorkspaceConfig>;
}

/// Composite interface for components that need multiple capabilities
/// 
/// Some components may need access to multiple providers. This trait
/// composes the individual traits for convenience while still maintaining
/// clear interface boundaries.
pub trait MonorepoContext: PackageProvider + ConfigProvider + FileSystemProvider + GitProvider + RegistryProvider {
    // Marker trait - no additional methods needed
}

// Implement all traits for MonorepoProject to maintain compatibility
impl PackageProvider for MonorepoProject {
    fn packages(&self) -> &[MonorepoPackageInfo] {
        &self.packages
    }
    
    fn get_package(&self, name: &str) -> Option<&MonorepoPackageInfo> {
        self.packages.iter().find(|pkg| pkg.package_info.package.borrow().name() == name)
    }
    
    fn get_dependents(&self, package_name: &str) -> Vec<&MonorepoPackageInfo> {
        // Use the get_dependents method from MonorepoProject
        self.get_dependents(package_name)
    }
    
    fn root_path(&self) -> &Path {
        &self.root_path
    }
}

impl ConfigProvider for MonorepoProject {
    fn config(&self) -> &MonorepoConfig {
        &self.config
    }
}

impl FileSystemProvider for MonorepoProject {
    fn read_file_string(&self, path: &Path) -> Result<String> {
        self.file_system.read_file_string(path).map_err(Into::into)
    }
    
    fn write_file_string(&self, path: &Path, content: &str) -> Result<()> {
        self.file_system.write_file_string(path, content).map_err(Into::into)
    }
    
    fn path_exists(&self, path: &Path) -> bool {
        self.file_system.exists(path)
    }
    
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.file_system.create_dir_all(path).map_err(Into::into)
    }
    
    fn remove_file(&self, path: &Path) -> Result<()> {
        self.file_system.remove(path).map_err(Into::into)
    }
    
    fn walk_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>> {
        self.file_system.walk_dir(path).map_err(Into::into)
    }
}

impl GitProvider for MonorepoProject {
    fn repository(&self) -> &Repo {
        &self.repository
    }
    
    fn repository_root(&self) -> &Path {
        self.root_path()
    }
    
    fn get_changed_files_since(&self, since: &str) -> Result<Vec<GitChangedFile>> {
        self.repository.get_all_files_changed_since_sha_with_status(since).map_err(Into::into)
    }
    
    fn current_branch(&self) -> Result<String> {
        self.repository.get_current_branch().map_err(Into::into)
    }
    
    fn current_sha(&self) -> Result<String> {
        self.repository.get_current_sha().map_err(Into::into)
    }
    
    fn get_diverged_commit(&self, base_branch: &str) -> Result<String> {
        self.repository.get_diverged_commit(base_branch).map_err(Into::into)
    }
    
    fn get_all_files_changed_since_sha_with_status(&self, sha: &str) -> Result<Vec<GitChangedFile>> {
        self.repository.get_all_files_changed_since_sha_with_status(sha).map_err(Into::into)
    }
    
    fn get_all_files_changed_since_sha(&self, sha: &str) -> Result<Vec<String>> {
        self.repository.get_all_files_changed_since_sha(sha).map_err(Into::into)
    }
}

impl RegistryProvider for MonorepoProject {
    fn registry_manager(&self) -> &RegistryManager {
        &self.registry_manager
    }
    
    fn get_registry_type(&self, url: &str) -> String {
        self.config.workspace.tool_configs.get_registry_type(url).to_string()
    }
}

impl WorkspaceProvider for MonorepoProject {
    fn root_path(&self) -> &Path {
        &self.root_path
    }
    
    fn get_workspace_patterns(&self) -> Result<Vec<String>> {
        Ok(self.config_manager.get_effective_workspace_patterns(vec![], None, None))
    }
    
    fn get_effective_workspace_patterns(&self) -> Result<Vec<String>> {
        Ok(self.config_manager.get_effective_workspace_patterns(vec![], None, None))
    }
    
    fn get_package_manager_patterns(&self) -> Result<Vec<String>> {
        // Convert PackageManagerKind to appropriate patterns
        // For now, return default patterns based on package manager type
        // All package managers currently use the same pattern
        Ok(vec!["packages/*".to_string()])
    }
    
    fn get_package_manager_info(&self) -> Result<(String, std::collections::HashMap<String, String>)> {
        let kind = format!("{:?}", self.package_manager.kind());
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("root".to_string(), self.package_manager.root().to_string_lossy().to_string());
        Ok((kind, metadata))
    }
    
    fn get_workspace_config(&self) -> Result<MonorepoDescriptor> {
        Ok(self.descriptor.clone())
    }
    
    fn validate_workspace_config(&self) -> Result<Vec<String>> {
        let package_names: Vec<String> = self.packages.iter()
            .map(|pkg| pkg.package_info.package.borrow().name().to_string())
            .collect();
        Ok(self.config_manager.validate_workspace_config(&package_names))
    }
    
    fn get_package_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>> {
        // Convert PackageManagerCommandConfig to HashMap<String, Vec<String>>
        // For now, return empty HashMap as the conversion would require understanding the internal structure
        std::collections::HashMap::new()
    }
}

impl PackageDiscoveryProvider for MonorepoProject {
    fn get_all_packages_with_metadata(&self) -> &[MonorepoPackageInfo] {
        &self.packages
    }
    
    fn get_package_descriptor(&self) -> &MonorepoDescriptor {
        &self.descriptor
    }
    
    fn find_packages_by_pattern(&self, pattern: &str) -> Result<Vec<&MonorepoPackageInfo>> {
        let packages = self.packages.iter()
            .filter(|pkg| {
                let binding = pkg.package_info.package.borrow();
                let pkg_name = binding.name();
                pkg_name.contains(pattern) || 
                pkg.workspace_package.absolute_path.to_string_lossy().contains(pattern)
            })
            .collect();
        Ok(packages)
    }
    
    fn get_package_locations(&self) -> Vec<(String, &Path)> {
        self.packages.iter()
            .map(|pkg| {
                let name = pkg.package_info.package.borrow().name().to_string();
                let path = pkg.workspace_package.absolute_path.as_path();
                (name, path)
            })
            .collect()
    }
    
    fn find_orphaned_packages(&self, _patterns: &[String]) -> Vec<&MonorepoPackageInfo> {
        // Implementation would check which packages don't match any workspace patterns
        // For now, return empty vector as this requires pattern matching logic
        Vec::new()
    }
}

impl EnhancedConfigProvider for MonorepoProject {
    fn get_effective_patterns(&self) -> Result<Vec<String>> {
        Ok(self.config_manager.get_effective_workspace_patterns(vec![], None, None))
    }
    
    fn validate_config(&self) -> Result<Vec<String>> {
        let package_names: Vec<String> = self.packages.iter()
            .map(|pkg| pkg.package_info.package.borrow().name().to_string())
            .collect();
        Ok(self.config_manager.validate_workspace_config(&package_names))
    }
    
    fn get_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>> {
        // Convert PackageManagerCommandConfig to HashMap<String, Vec<String>>
        // For now, return empty HashMap as the conversion would require understanding the internal structure
        std::collections::HashMap::new()
    }
    
    fn get_workspace_section(&self) -> Result<&crate::config::WorkspaceConfig> {
        Ok(&self.config.workspace)
    }
}

impl MonorepoContext for MonorepoProject {}

// Implement all traits for Arc<MonorepoProject> to enable shared ownership
impl PackageProvider for Arc<MonorepoProject> {
    fn packages(&self) -> &[MonorepoPackageInfo] {
        self.as_ref().packages()
    }
    
    fn get_package(&self, name: &str) -> Option<&MonorepoPackageInfo> {
        self.as_ref().get_package(name)
    }
    
    fn get_dependents(&self, package_name: &str) -> Vec<&MonorepoPackageInfo> {
        self.as_ref().get_dependents(package_name)
    }
    
    fn root_path(&self) -> &Path {
        self.as_ref().root_path()
    }
}

impl ConfigProvider for Arc<MonorepoProject> {
    fn config(&self) -> &MonorepoConfig {
        self.as_ref().config()
    }
}

impl FileSystemProvider for Arc<MonorepoProject> {
    fn read_file_string(&self, path: &Path) -> Result<String> {
        self.as_ref().read_file_string(path)
    }
    
    fn write_file_string(&self, path: &Path, content: &str) -> Result<()> {
        self.as_ref().write_file_string(path, content)
    }
    
    fn path_exists(&self, path: &Path) -> bool {
        self.as_ref().path_exists(path)
    }
    
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.as_ref().create_dir_all(path)
    }
    
    fn remove_file(&self, path: &Path) -> Result<()> {
        self.as_ref().remove_file(path)
    }
    
    fn walk_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>> {
        self.as_ref().walk_dir(path)
    }
}

impl GitProvider for Arc<MonorepoProject> {
    fn repository(&self) -> &Repo {
        self.as_ref().repository()
    }
    
    fn repository_root(&self) -> &Path {
        self.as_ref().root_path()
    }
    
    fn get_changed_files_since(&self, since: &str) -> Result<Vec<GitChangedFile>> {
        self.as_ref().get_changed_files_since(since)
    }
    
    fn current_branch(&self) -> Result<String> {
        self.as_ref().current_branch()
    }
    
    fn current_sha(&self) -> Result<String> {
        self.as_ref().current_sha()
    }
    
    fn get_diverged_commit(&self, base_branch: &str) -> Result<String> {
        self.as_ref().get_diverged_commit(base_branch)
    }
    
    fn get_all_files_changed_since_sha_with_status(&self, sha: &str) -> Result<Vec<GitChangedFile>> {
        self.as_ref().get_all_files_changed_since_sha_with_status(sha)
    }
    
    fn get_all_files_changed_since_sha(&self, sha: &str) -> Result<Vec<String>> {
        self.as_ref().get_all_files_changed_since_sha(sha)
    }
}

impl RegistryProvider for Arc<MonorepoProject> {
    fn registry_manager(&self) -> &RegistryManager {
        self.as_ref().registry_manager()
    }
    
    fn get_registry_type(&self, url: &str) -> String {
        self.as_ref().get_registry_type(url)
    }
}

impl WorkspaceProvider for Arc<MonorepoProject> {
    fn root_path(&self) -> &Path {
        self.as_ref().root_path()
    }
    
    fn get_workspace_patterns(&self) -> Result<Vec<String>> {
        self.as_ref().get_workspace_patterns()
    }
    
    fn get_effective_workspace_patterns(&self) -> Result<Vec<String>> {
        self.as_ref().get_effective_workspace_patterns()
    }
    
    fn get_package_manager_patterns(&self) -> Result<Vec<String>> {
        self.as_ref().get_package_manager_patterns()
    }
    
    fn get_package_manager_info(&self) -> Result<(String, std::collections::HashMap<String, String>)> {
        self.as_ref().get_package_manager_info()
    }
    
    fn get_workspace_config(&self) -> Result<MonorepoDescriptor> {
        self.as_ref().get_workspace_config()
    }
    
    fn validate_workspace_config(&self) -> Result<Vec<String>> {
        self.as_ref().validate_workspace_config()
    }
    
    fn get_package_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>> {
        self.as_ref().get_package_manager_commands()
    }
}

impl PackageDiscoveryProvider for Arc<MonorepoProject> {
    fn get_all_packages_with_metadata(&self) -> &[MonorepoPackageInfo] {
        self.as_ref().get_all_packages_with_metadata()
    }
    
    fn get_package_descriptor(&self) -> &MonorepoDescriptor {
        self.as_ref().get_package_descriptor()
    }
    
    fn find_packages_by_pattern(&self, pattern: &str) -> Result<Vec<&MonorepoPackageInfo>> {
        self.as_ref().find_packages_by_pattern(pattern)
    }
    
    fn get_package_locations(&self) -> Vec<(String, &Path)> {
        self.as_ref().get_package_locations()
    }
    
    fn find_orphaned_packages(&self, patterns: &[String]) -> Vec<&MonorepoPackageInfo> {
        self.as_ref().find_orphaned_packages(patterns)
    }
}

impl EnhancedConfigProvider for Arc<MonorepoProject> {
    fn get_effective_patterns(&self) -> Result<Vec<String>> {
        self.as_ref().get_effective_patterns()
    }
    
    fn validate_config(&self) -> Result<Vec<String>> {
        self.as_ref().validate_config()
    }
    
    fn get_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>> {
        self.as_ref().get_manager_commands()
    }
    
    fn get_workspace_section(&self) -> Result<&crate::config::WorkspaceConfig> {
        self.as_ref().get_workspace_section()
    }
}

impl MonorepoContext for Arc<MonorepoProject> {}

/// Factory for creating focused component dependencies
/// 
/// This allows components to receive only the interfaces they need,
/// reducing coupling and improving testability.
pub struct DependencyFactory;

impl DependencyFactory {
    /// Create a package provider from any source implementing the interface
    pub fn package_provider(provider: impl PackageProvider + 'static) -> Box<dyn PackageProvider> {
        Box::new(provider)
    }
    
    /// Create a config provider from any source implementing the interface  
    pub fn config_provider(provider: impl ConfigProvider + 'static) -> Box<dyn ConfigProvider> {
        Box::new(provider)
    }
    
    /// Create a file system provider from any source implementing the interface
    pub fn file_system_provider(provider: impl FileSystemProvider + 'static) -> Box<dyn FileSystemProvider> {
        Box::new(provider)
    }
    
    /// Create a git provider from any source implementing the interface
    pub fn git_provider(provider: impl GitProvider + 'static) -> Box<dyn GitProvider> {
        Box::new(provider)
    }
    
    /// Create a registry provider from any source implementing the interface
    pub fn registry_provider(provider: impl RegistryProvider + 'static) -> Box<dyn RegistryProvider> {
        Box::new(provider)
    }
    
    /// Create a workspace provider from any source implementing the interface
    pub fn workspace_provider(provider: impl WorkspaceProvider + 'static) -> Box<dyn WorkspaceProvider> {
        Box::new(provider)
    }
    
    /// Create a package discovery provider from any source implementing the interface
    pub fn package_discovery_provider(provider: impl PackageDiscoveryProvider + 'static) -> Box<dyn PackageDiscoveryProvider> {
        Box::new(provider)
    }
    
    /// Create an enhanced config provider from any source implementing the interface
    pub fn enhanced_config_provider(provider: impl EnhancedConfigProvider + 'static) -> Box<dyn EnhancedConfigProvider> {
        Box::new(provider)
    }
}