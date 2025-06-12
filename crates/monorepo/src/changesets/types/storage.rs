//! Changeset storage type definitions

use std::sync::Arc;
use crate::config::types::ChangesetsConfig;
use crate::core::MonorepoProject;

/// Storage interface for changesets
///
/// Provides methods for persisting and retrieving changesets from the filesystem.
/// Uses the `FileSystemManager` for all file operations to ensure consistency
/// with the rest of the monorepo tooling.
pub struct ChangesetStorage {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,

    /// Changeset configuration
    pub(crate) config: ChangesetsConfig,
}