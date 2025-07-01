//! Task condition checker type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use crate::core::MonorepoPackageInfo;
use crate::config::MonorepoConfig;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

/// Checker for evaluating task execution conditions
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct ConditionChecker<'a> {
    /// Direct reference to git repository
    pub(crate) repository: &'a Repo,
    
    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,
    
    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],
    
    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,
    
    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
}