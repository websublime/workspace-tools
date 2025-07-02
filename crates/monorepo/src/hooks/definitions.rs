//! Hook definitions implementation
//!
//! This module provides implementation methods for hook definitions, scripts, and conditions,
//! following the project's pattern of separating declarations from implementations.

use super::{DependencyType, HookCondition, HookDefinition, HookScript, HookType};
use crate::Environment;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

impl HookType {
    /// Get the Git hook file name for this hook type
    #[must_use]
    pub fn git_hook_filename(&self) -> &'static str {
        match self {
            Self::PreCommit => "pre-commit",
            Self::PrePush => "pre-push",
            Self::PostCommit => "post-commit",
            Self::PostMerge => "post-merge",
            Self::PostCheckout => "post-checkout",
        }
    }

    /// Check if this hook type should block the Git operation on failure
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::PreCommit | Self::PrePush)
    }

    /// Get all supported hook types
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![Self::PreCommit, Self::PrePush, Self::PostCommit, Self::PostMerge, Self::PostCheckout]
    }
}

impl HookDefinition {
    /// Create a new hook definition with a script
    #[must_use]
    pub fn new(script: HookScript, description: impl Into<String>) -> Self {
        Self {
            script,
            conditions: Vec::new(),
            fail_on_error: true,
            timeout: Some(Self::default_timeout()),
            environment: HashMap::new(),
            working_directory: None,
            description: description.into(),
            enabled: true,
        }
    }

    /// Get the default timeout for hook execution
    ///
    /// This can be overridden by configuration, but provides a sensible default
    /// when no configuration is available.
    #[must_use]
    pub fn default_timeout() -> Duration {
        Duration::from_secs(300) // 5 minutes default
    }

    /// Add a condition to the hook
    #[must_use]
    pub fn with_condition(mut self, condition: HookCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set whether to fail on error
    #[must_use]
    pub fn with_fail_on_error(mut self, fail_on_error: bool) -> Self {
        self.fail_on_error = fail_on_error;
        self
    }

    /// Set timeout for execution
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add environment variable
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Set working directory
    #[must_use]
    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = Some(dir);
        self
    }

    /// Set enabled status
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl HookScript {
    /// Create a task execution script
    #[must_use]
    pub fn tasks(tasks: Vec<String>) -> Self {
        Self::TaskExecution { tasks, parallel: false }
    }

    /// Create a parallel task execution script
    #[must_use]
    pub fn parallel_tasks(tasks: Vec<String>) -> Self {
        Self::TaskExecution { tasks, parallel: true }
    }

    /// Create a command script
    #[must_use]
    pub fn command(cmd: impl Into<String>, args: Vec<String>) -> Self {
        Self::Command { cmd: cmd.into(), args }
    }

    /// Create a script file execution
    #[must_use]
    pub fn script_file(path: PathBuf) -> Self {
        Self::ScriptFile { path, args: Vec::new() }
    }

    /// Create a script file execution with arguments
    #[must_use]
    pub fn script_file_with_args(path: PathBuf, args: Vec<String>) -> Self {
        Self::ScriptFile { path, args }
    }

    /// Create a sequence of scripts
    #[must_use]
    pub fn sequence(scripts: Vec<HookScript>) -> Self {
        Self::Sequence { scripts, stop_on_failure: true }
    }
}

impl HookCondition {
    /// Create a files changed condition
    #[must_use]
    pub fn files_changed(patterns: Vec<String>) -> Self {
        Self::FilesChanged { patterns, match_any: true }
    }

    /// Create a packages changed condition
    #[must_use]
    pub fn packages_changed(packages: Vec<String>) -> Self {
        Self::PackagesChanged { packages, match_any: true }
    }

    /// Create a dependencies changed condition
    #[must_use]
    pub fn dependencies_changed() -> Self {
        Self::DependenciesChanged { dependency_types: vec![DependencyType::All] }
    }

    /// Create a branch condition
    #[must_use]
    pub fn on_branch(pattern: impl Into<String>) -> Self {
        Self::OnBranch { pattern: pattern.into() }
    }

    /// Create an environment condition
    #[must_use]
    pub fn environment(env: Environment) -> Self {
        Self::Environment { env }
    }

    /// Create a changeset exists condition
    #[must_use]
    pub fn changeset_exists() -> Self {
        Self::ChangesetExists { require_for_all: true }
    }

    /// Create an environment variable condition
    #[must_use]
    pub fn env_var(name: impl Into<String>, value: Option<String>) -> Self {
        Self::EnvironmentVariable { name: name.into(), value }
    }

    /// Create a Git ref exists condition
    #[must_use]
    pub fn git_ref_exists(ref_pattern: impl Into<String>) -> Self {
        Self::GitRefExists { ref_pattern: ref_pattern.into() }
    }
}
