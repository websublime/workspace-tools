//! Task management system
//!
//! This module provides comprehensive task management capabilities for monorepos,
//! including task definition, execution, condition checking, and result tracking.
//!
//! # What
//! Manages task execution based on package.json scripts and custom commands, with
//! support for conditional execution, scoping, and parallel execution.
//!
//! # How
//! Leverages the `CommandQueue` from standard-tools for execution, integrates with
//! change detection to run tasks on affected packages, and provides sophisticated
//! condition checking for when tasks should run.
//!
//! # Why
//! Essential for automated workflows where different tasks need to run based on
//! what has changed, enabling efficient CI/CD and development workflows.

mod async_adapter;
mod builder;
mod checker;
mod executor;
mod manager;
mod registry;
#[cfg(test)]
mod tests;
pub mod types;

// Re-export main types for convenience
pub use types::{
    BranchCondition, EnvironmentCondition, ExecutionContext, FilePattern, FilePatternType,
    PackageScript, TaskCommand, TaskCondition, TaskDefinition, TaskError, TaskErrorCode,
    TaskExecutionLog, TaskExecutionResult, TaskExecutionStats, TaskLogLevel, TaskManager,
    TaskOutput, TaskPriority, TaskScope, TaskStatus, TaskTrigger,
};

// Re-export async boundary adapter
pub use async_adapter::AsyncConditionAdapter;
