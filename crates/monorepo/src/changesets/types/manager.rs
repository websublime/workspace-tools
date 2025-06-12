//! Changeset manager type definitions

use std::sync::Arc;
use super::ChangesetStorage;
use crate::core::MonorepoProject;
use crate::tasks::TaskManager;

/// Manager for changeset operations
///
/// The `ChangesetManager` provides the main interface for working with changesets.
/// It handles creation, validation, storage, and deployment of changesets across
/// different environments during the development workflow.
pub struct ChangesetManager {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,

    /// Storage for changeset persistence
    pub(crate) storage: ChangesetStorage,

    /// Task manager for executing deployment tasks
    pub(crate) task_manager: TaskManager,
}