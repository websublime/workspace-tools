//! Changeset storage type definitions

use crate::config::types::ChangesetsConfig;

/// Storage interface for changesets
///
/// Provides methods for persisting and retrieving changesets from the filesystem.
/// Uses the `FileSystemManager` for all file operations to ensure consistency
/// with the rest of the monorepo tooling.
pub struct ChangesetStorage {
    /// Changeset configuration
    pub(crate) config: ChangesetsConfig,

    /// File system provider for file operations
    pub(crate) file_system_provider: Box<dyn crate::core::FileSystemProvider>,

    /// Package provider for accessing root path
    pub(crate) package_provider: Box<dyn crate::core::PackageProvider>,
}