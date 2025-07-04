//! Task management system for CLI/daemon consumption
//!
//! This module provides streamlined task management optimized for CLI and daemon usage,
//! including task definition, synchronous execution, condition checking, and result tracking.
//!
//! # What
//! Manages task execution based on package.json scripts and custom commands, with
//! support for conditional execution, scoping, and parallel execution using direct
//! base crate integration.
//!
//! # How
//! Uses `SharedSyncExecutor` from sublime-standard-tools for direct command execution,
//! integrates with change detection to run tasks on affected packages, and provides
//! efficient synchronous condition checking optimized for CLI responsiveness.
//!
//! # Why
//! Essential for CLI and daemon workflows where tasks need to run based on changes,
//! providing sub-second performance and clean ownership patterns for real-time usage.

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
