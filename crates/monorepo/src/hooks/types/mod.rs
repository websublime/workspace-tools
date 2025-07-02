//! Hook type definitions
//!
//! This module contains all the type definitions for the hook management system,
//! including hook types, definitions, execution contexts, and results.

mod context;
mod definitions;
mod results;

// Implementation structs (moved from main modules)
mod installer;
mod manager;
mod validator;

pub use definitions::{DependencyType, HookCondition, HookDefinition, HookScript, HookType};

pub use results::{
    HookError, HookErrorCode, HookExecutionResult, HookStatus, HookValidationResult,
    PostCommitResult, PreCommitResult, PrePushResult, ValidationCheck,
};

pub use context::{CommitInfo, GitOperationType, HookExecutionContext, RemoteInfo};

pub use installer::HookInstaller;
pub use manager::HookManager;
pub use validator::{ChangesetValidationResult, HookValidator};
