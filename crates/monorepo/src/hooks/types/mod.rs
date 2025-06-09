//! Hook type definitions
//!
//! This module contains all the type definitions for the hook management system,
//! including hook types, definitions, execution contexts, and results.

mod definitions;
mod results;
mod context;

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