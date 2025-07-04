//! Changeset manager type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.
//! Focused on CRUD operations for CLI consumption.

use super::ChangesetStorage;
use crate::config::MonorepoConfig;
use crate::core::MonorepoPackageInfo;
use std::path::Path;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;

/// Manager for changeset operations
///
/// The `ChangesetManager` provides the main interface for working with changesets.
/// It handles creation, validation, storage, and application of changesets for
/// CI/CD integration and version bump indicators. Designed for CLI and daemon consumption.
///
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct ChangesetManager<'a> {
    /// Storage for changeset persistence
    pub(crate) storage: ChangesetStorage<'a>,

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
