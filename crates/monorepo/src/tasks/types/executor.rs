//! Task executor type definitions

use crate::core::MonorepoProject;
use std::sync::Arc;

/// Executor for running tasks with various scopes and configurations
pub struct TaskExecutor {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,
}