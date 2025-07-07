//! Task manager type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use super::{ConditionChecker, TaskExecutor, TaskRegistry};
use crate::core::MonorepoPackageInfo;
use std::collections::HashMap;
use sublime_standard_tools::filesystem::FileSystemManager;

/// Central manager for task execution and coordination
///
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct TaskManager<'a> {
    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,

    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],

    /// Task registry for storing and managing task definitions
    pub(crate) registry: TaskRegistry,

    /// Task executor for running tasks
    pub(crate) executor: TaskExecutor<'a>,

    /// Condition checker for evaluating task conditions
    pub(crate) condition_checker: ConditionChecker<'a>,

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
