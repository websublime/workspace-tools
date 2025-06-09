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
//! Leverages FileSystemManager from standard-tools for hook installation, integrates with
//! TaskManager for executing tasks on affected packages, and provides changeset validation
//! through ChangesetManager integration.
//!
//! # Why
//! Essential for maintaining code quality and ensuring proper changeset management in
//! monorepo workflows, preventing commits without proper changesets and running validation
//! tasks before push operations.

pub mod types;
mod context;
mod results;
mod definitions;
mod manager;
mod installer;
mod validator;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use types::{
    HookType, HookDefinition, HookScript, HookCondition, DependencyType, HookExecutionContext,
    HookExecutionResult, PreCommitResult, PrePushResult, PostCommitResult,
    HookStatus, HookError, HookErrorCode, HookValidationResult, ValidationCheck,
    GitOperationType, RemoteInfo, CommitInfo,
};

pub use manager::HookManager;
pub use installer::HookInstaller;
pub use validator::HookValidator;