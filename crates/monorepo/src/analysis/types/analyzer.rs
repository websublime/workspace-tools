//! Analyzer types for monorepo analysis
//!
//! This module contains type definitions for the monorepo analyzer.
//! Follows direct borrowing patterns instead of trait objects.

use crate::core::MonorepoPackageInfo;
use crate::config::MonorepoConfig;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_package_tools::RegistryManager;
use sublime_standard_tools::monorepo::MonorepoDescriptor;
use std::path::Path;

/// Analyzer for comprehensive monorepo analysis
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct MonorepoAnalyzer<'a> {
    /// Direct reference to packages for analysis
    pub(crate) packages: &'a [MonorepoPackageInfo],
    
    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,
    
    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,
    
    /// Direct reference to git repository
    pub(crate) repository: &'a Repo,
    
    /// Direct reference to registry manager
    pub(crate) registry_manager: &'a RegistryManager,
    
    /// Direct reference to monorepo descriptor
    pub(crate) descriptor: &'a MonorepoDescriptor,
    
    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
}