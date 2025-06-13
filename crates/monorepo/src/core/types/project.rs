//! Monorepo project type definitions

use crate::config::{ConfigManager, MonorepoConfig};
use std::path::PathBuf;
use sublime_git_tools::Repo;
use sublime_package_tools::{DependencyRegistry, RegistryManager};
use sublime_standard_tools::{
    filesystem::FileSystemManager,
    monorepo::{MonorepoDescriptor, PackageManager},
};

/// Main project structure that aggregates all monorepo information
pub struct MonorepoProject {
    /// Git repository
    pub(crate) repository: Repo,

    /// Monorepo descriptor from standard-tools
    pub(crate) descriptor: MonorepoDescriptor,

    /// Package manager information
    pub(crate) package_manager: PackageManager,

    /// Dependency registry for package management
    pub(crate) dependency_registry: DependencyRegistry,

    /// Registry manager for package lookups
    pub(crate) registry_manager: RegistryManager,

    /// Configuration manager
    pub(crate) config_manager: ConfigManager,

    /// File system manager
    pub(crate) file_system: FileSystemManager,

    /// All packages in the monorepo with enhanced information
    pub(crate) packages: Vec<super::MonorepoPackageInfo>,

    /// Dependency graph of all packages (built on-demand)
    pub(crate) dependency_graph: Option<()>, // TODO: Store graph metadata instead of the graph itself

    /// Current configuration
    pub(crate) config: MonorepoConfig,

    /// Root path of the monorepo
    pub(crate) root_path: PathBuf,
}
