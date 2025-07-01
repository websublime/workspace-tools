//! Version manager type definitions

use super::VersioningStrategy;
use crate::config::MonorepoConfig;
use crate::core::MonorepoPackageInfo;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

/// Manager for package versioning with dependency propagation
/// 
/// Uses direct borrowing from MonorepoProject components instead of Arc.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct VersionManager<'a> {
    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,
    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],
    /// Direct reference to repository
    pub(crate) repository: &'a Repo,
    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,
    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
    /// Versioning strategy to use
    pub(crate) strategy: Box<dyn VersioningStrategy + 'a>,
}