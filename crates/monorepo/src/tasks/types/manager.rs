//! Task manager type definitions

use super::{ConditionChecker, TaskExecutor, TaskRegistry};
use crate::core::MonorepoProject;
use std::collections::HashMap;
use std::sync::Arc;

/// Central manager for task execution and coordination
pub struct TaskManager {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,

    /// Task registry for storing and managing task definitions
    pub(crate) registry: TaskRegistry,

    /// Task executor for running tasks
    pub(crate) executor: TaskExecutor,

    /// Condition checker for evaluating task conditions
    pub(crate) condition_checker: ConditionChecker,

    /// Current execution context
    pub(crate) execution_context: ExecutionContext,
}

/// Execution context for tasks
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Packages that are affected by changes
    pub affected_packages: Vec<String>,

    /// Files that have changed
    pub changed_files: Vec<sublime_git_tools::GitChangedFile>,

    /// Current branch
    pub current_branch: Option<String>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Working directory
    pub working_directory: Option<std::path::PathBuf>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    /// Create new execution context
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set affected packages
    #[must_use]
    pub fn with_affected_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    /// Set changed files
    #[must_use]
    pub fn with_changed_files(mut self, files: Vec<sublime_git_tools::GitChangedFile>) -> Self {
        self.changed_files = files;
        self
    }

    /// Set current branch
    #[must_use]
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.current_branch = Some(branch.into());
        self
    }
}