//! Changeset manager type definitions

use super::ChangesetStorage;
use crate::tasks::TaskManager;

/// Manager for changeset operations
///
/// The `ChangesetManager` provides the main interface for working with changesets.
/// It handles creation, validation, storage, and deployment of changesets across
/// different environments during the development workflow.
pub struct ChangesetManager {
    /// Storage for changeset persistence
    pub(crate) storage: ChangesetStorage,

    /// Task manager for executing deployment tasks
    pub(crate) task_manager: TaskManager,

    /// Configuration provider for accessing configuration settings
    pub(crate) config_provider: Box<dyn crate::core::ConfigProvider>,

    /// File system provider for file operations
    pub(crate) file_system_provider: Box<dyn crate::core::FileSystemProvider>,

    /// Package provider for accessing package information
    pub(crate) package_provider: Box<dyn crate::core::PackageProvider>,

    /// Git provider for repository operations
    pub(crate) git_provider: Box<dyn crate::core::GitProvider>,
}