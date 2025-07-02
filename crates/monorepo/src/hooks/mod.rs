//! Git hooks management system
//!
//! This module provides comprehensive Git hooks management capabilities for monorepos,
//! including hook installation, execution, validation, and integration with changesets.
//!
//! # What
//! Manages Git hooks (pre-commit, pre-push, etc.) with sophisticated validation logic
//! including changeset verification, affected package task execution, and custom hook scripts.
//!
//! # How
//! Leverages `FileSystemManager` from standard-tools for hook installation, integrates with
//! `TaskManager` for executing tasks on affected packages, and provides changeset validation
//! through `ChangesetManager` integration.
//!
//! # Why
//! Essential for maintaining code quality and ensuring proper changeset management in
//! monorepo workflows, preventing commits without proper changesets and running validation
//! tasks before push operations.

mod context;
mod definitions;
mod installer;
mod manager;
mod results;
mod sync_task_executor;
pub mod types;
mod validator;

// Re-export main types for convenience
pub use types::{
    ChangesetValidationResult,
    CommitInfo,
    DependencyType,
    GitOperationType,
    HookCondition,
    HookDefinition,
    HookError,
    HookErrorCode,
    HookExecutionContext,
    HookExecutionResult,
    HookInstaller,
    // Implementation structs
    HookManager,
    HookScript,
    HookStatus,
    HookType,
    HookValidationResult,
    HookValidator,
    PostCommitResult,
    PreCommitResult,
    PrePushResult,
    RemoteInfo,
    ValidationCheck,
};

// Re-export SyncTaskExecutor for testing and integration purposes
pub use sync_task_executor::SyncTaskExecutor;
