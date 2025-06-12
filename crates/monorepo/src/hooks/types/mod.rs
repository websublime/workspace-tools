//! Hook type definitions
//!
//! This module contains all the type definitions for the hook management system,
//! including hook types, definitions, execution contexts, and results.

mod definitions;
mod results;
mod context;

// Implementation structs (moved from main modules)
mod installer;
mod manager;
mod validator;

pub use definitions::{
    HookType, HookDefinition, HookScript, HookCondition, DependencyType,
};

pub use results::{
    HookExecutionResult, PreCommitResult, PrePushResult, PostCommitResult,
    HookStatus, HookError, HookErrorCode, HookValidationResult, ValidationCheck,
};

pub use context::{
    HookExecutionContext, GitOperationType, RemoteInfo, CommitInfo,
};

pub use installer::HookInstaller;
pub use manager::HookManager;
pub use validator::{HookValidator, ChangesetValidationResult};