//! Task execution results and output types
//!
//! Types that represent the results of task execution, including
//! success/failure status, output capture, and execution statistics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Result of task execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    /// Task that was executed
    pub task_name: String,
    
    /// Execution status
    pub status: TaskStatus,
    
    /// Start time of execution
    pub started_at: SystemTime,
    
    /// End time of execution
    pub ended_at: SystemTime,
    
    /// Total duration
    pub duration: Duration,
    
    /// Output from commands
    pub outputs: Vec<TaskOutput>,
    
    /// Execution statistics
    pub stats: TaskExecutionStats,
    
    /// Packages that were affected
    pub affected_packages: Vec<String>,
    
    /// Any errors that occurred
    pub errors: Vec<TaskError>,
    
    /// Execution logs
    pub logs: Vec<TaskExecutionLog>,
    
    /// Artifacts produced
    pub artifacts: Vec<TaskArtifact>,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,
    
    /// Task is currently running
    Running,
    
    /// Task completed successfully
    Success,
    
    /// Task failed
    Failed {
        /// Failure reason
        reason: String,
    },
    
    /// Task was skipped
    Skipped {
        /// Skip reason
        reason: String,
    },
    
    /// Task was cancelled
    Cancelled,
    
    /// Task timed out
    TimedOut {
        /// Timeout duration
        after: Duration,
    },
}

/// Output from a single command execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskOutput {
    /// Command that was executed
    pub command: String,
    
    /// Working directory
    pub working_dir: PathBuf,
    
    /// Exit code
    pub exit_code: Option<i32>,
    
    /// Standard output
    pub stdout: String,
    
    /// Standard error
    pub stderr: String,
    
    /// Execution duration
    pub duration: Duration,
    
    /// Environment variables used
    pub environment: HashMap<String, String>,
}

/// Task execution error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskError {
    /// Error code
    pub code: TaskErrorCode,
    
    /// Error message
    pub message: String,
    
    /// Additional context
    pub context: HashMap<String, String>,
    
    /// When the error occurred
    pub occurred_at: SystemTime,
    
    /// Related package (if any)
    pub package: Option<String>,
    
    /// Related command (if any)
    pub command: Option<String>,
}

/// Task error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskErrorCode {
    /// Command not found
    CommandNotFound,
    
    /// Command execution failed
    ExecutionFailed,
    
    /// Task timed out
    Timeout,
    
    /// Dependency task failed
    DependencyFailed,
    
    /// Condition check failed
    ConditionFailed,
    
    /// Invalid configuration
    InvalidConfiguration,
    
    /// Permission denied
    PermissionDenied,
    
    /// Resource not available
    ResourceUnavailable,
    
    /// Unknown error
    Unknown,
}

/// Task execution statistics
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskExecutionStats {
    /// Number of commands executed
    pub commands_executed: usize,
    
    /// Number of successful commands
    pub commands_succeeded: usize,
    
    /// Number of failed commands
    pub commands_failed: usize,
    
    /// Number of packages processed
    pub packages_processed: usize,
    
    /// Total bytes of stdout
    pub stdout_bytes: usize,
    
    /// Total bytes of stderr
    pub stderr_bytes: usize,
    
    /// Peak memory usage (if available)
    pub peak_memory_bytes: Option<usize>,
    
    /// CPU time used (if available)
    pub cpu_time: Option<Duration>,
}

/// Task execution log entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskExecutionLog {
    /// Timestamp
    pub timestamp: SystemTime,
    
    /// Log level
    pub level: TaskLogLevel,
    
    /// Log message
    pub message: String,
    
    /// Related package (if any)
    pub package: Option<String>,
    
    /// Additional data
    pub data: HashMap<String, serde_json::Value>,
}

/// Task log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskLogLevel {
    /// Debug information
    Debug,
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

/// Artifact produced by task execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskArtifact {
    /// Artifact name
    pub name: String,
    
    /// File path
    pub path: PathBuf,
    
    /// Artifact type
    pub artifact_type: String,
    
    /// Size in bytes
    pub size_bytes: u64,
    
    /// Related package
    pub package: Option<String>,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}