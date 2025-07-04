//! Task system type definitions
//!
//! This module provides all the type definitions for the task management system,
//! including task definitions, execution contexts, and results.

mod conditions;
mod conversions;
mod definitions;
pub mod results;

// Implementation structs (moved from main modules)
mod checker;
mod executor;
mod manager;
mod registry;

pub(crate) use checker::ConditionChecker;
pub use conditions::{
    BranchCondition, DependencyFilter, EnvironmentCondition, FilePattern, FilePatternType,
    TaskCondition, TaskScope, TaskTrigger, VersionChangeThreshold,
};
pub use definitions::{
    PackageScript, TaskCommand, TaskCommandCore, TaskDefinition, TaskEnvironment, TaskPriority,
    TaskTimeout,
};
pub(crate) use executor::TaskExecutor;
pub use manager::{ExecutionContext, TaskManager};
pub use registry::TaskRegistry;
pub use results::{
    TaskArtifact, TaskError, TaskErrorCode, TaskExecutionLog, TaskExecutionResult, TaskExecutionStats,
    TaskLogLevel, TaskOutput, TaskStatus,
};
