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

pub mod types;
mod builder;
mod manager;
mod registry;
mod executor;
mod checker;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use types::{
    TaskDefinition, TaskCommand, PackageScript, TaskPriority,
    TaskCondition, TaskScope, TaskTrigger, FilePattern, FilePatternType,
    EnvironmentCondition, BranchCondition,
    TaskExecutionResult, TaskStatus, TaskOutput, TaskError,
    TaskErrorCode, TaskExecutionStats, TaskExecutionLog, TaskLogLevel,
    // Implementation structs
    TaskManager, TaskRegistry, TaskExecutor, ConditionChecker, ExecutionContext,
};