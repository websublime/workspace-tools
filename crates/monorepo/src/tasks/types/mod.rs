//! Task system type definitions
//!
//! This module provides all the type definitions for the task management system,
//! including task definitions, execution contexts, and results.

mod definitions;
mod conditions;
mod conversions;
pub mod results;

// Implementation structs (moved from main modules)
mod checker;
mod executor;
mod manager;
mod registry;

pub use definitions::{
    TaskDefinition, TaskCommand, TaskCommandCore, PackageScript, TaskPriority,
    TaskEnvironment, TaskTimeout,
};
pub use conditions::{
    TaskCondition, TaskScope, TaskTrigger, FilePattern, FilePatternType,
    EnvironmentCondition, BranchCondition, DependencyFilter, VersionChangeThreshold,
};
pub use results::{
    TaskExecutionResult, TaskStatus, TaskOutput, TaskError,
    TaskErrorCode, TaskExecutionStats, TaskExecutionLog, TaskLogLevel,
};
pub(crate) use checker::ConditionChecker;
pub(crate) use executor::TaskExecutor;
pub use manager::{TaskManager, ExecutionContext};
pub(crate) use registry::TaskRegistry;