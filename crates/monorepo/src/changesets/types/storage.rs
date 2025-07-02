//! Changeset storage type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use crate::config::types::ChangesetsConfig;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

/// Storage interface for changesets
///
/// Provides methods for persisting and retrieving changesets from the filesystem.
/// Uses the `FileSystemManager` for all file operations to ensure consistency
/// with the rest of the monorepo tooling.
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct ChangesetStorage<'a> {
    /// Changeset configuration
    pub(crate) config: ChangesetsConfig,

    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,
    
    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
}