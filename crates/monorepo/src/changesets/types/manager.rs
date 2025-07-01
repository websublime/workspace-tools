//! Changeset manager type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use super::ChangesetStorage;
use crate::tasks::TaskManager;
use crate::core::MonorepoPackageInfo;
use crate::config::MonorepoConfig;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

/// Manager for changeset operations
///
/// The `ChangesetManager` provides the main interface for working with changesets.
/// It handles creation, validation, storage, and deployment of changesets across
/// different environments during the development workflow.
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct ChangesetManager<'a> {
    /// Storage for changeset persistence
    pub(crate) storage: ChangesetStorage<'a>,

    /// Task manager for executing deployment tasks
    pub(crate) task_manager: TaskManager<'a>,

    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,

    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,

    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],

    /// Direct reference to git repository
    pub(crate) repository: &'a Repo,
    
    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
}