//! Task definition types
//!
//! Core types that define tasks, their commands, and execution parameters.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Complete definition of a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    /// Unique name of the task
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Commands to execute
    pub commands: Vec<TaskCommand>,
    
    /// Package.json scripts to run
    pub package_scripts: Vec<PackageScript>,
    
    /// Tasks that must run before this one
    pub dependencies: Vec<String>,
    
    /// Conditions that must be met for task to run
    pub conditions: Vec<super::TaskCondition>,
    
    /// Execution priority
    pub priority: TaskPriority,
    
    /// Scope of task execution
    pub scope: super::TaskScope,
    
    /// Whether to continue on error
    pub continue_on_error: bool,
    
    /// Maximum execution time
    pub timeout: Option<TaskTimeout>,
    
    /// Environment variables
    pub environment: TaskEnvironment,
}

/// Task command with standard Command integration and task-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommand {
    /// The underlying command (using standard crate)
    #[serde(flatten)]
    pub command: TaskCommandCore,
    
    /// Whether to run in shell
    pub shell: bool,
    
    /// Expected exit codes (default: [0])
    pub expected_exit_codes: Vec<i32>,
}

/// Core command data that maps to standard Command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommandCore {
    /// Program to execute
    pub program: String,
    
    /// Arguments to pass
    pub args: Vec<String>,
    
    /// Working directory (relative to package or absolute)
    pub current_dir: Option<PathBuf>,
    
    /// Environment variables for this command
    pub env: HashMap<String, String>,
    
    /// Command-specific timeout
    pub timeout: Option<Duration>,
}

/// Reference to a package.json script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageScript {
    /// Package name (if None, runs in all affected packages)
    pub package_name: Option<String>,
    
    /// Script name from package.json
    pub script_name: String,
    
    /// Working directory override
    pub working_directory: Option<PathBuf>,
    
    /// Additional arguments to pass
    pub extra_args: Vec<String>,
    
    /// Package manager to use (npm, yarn, pnpm)
    pub package_manager: Option<String>,
}

/// Task execution priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum TaskPriority {
    /// Lowest priority
    Low = 0,
    /// Normal priority (default)
    Normal = 50,
    /// High priority
    High = 100,
    /// Critical priority
    Critical = 200,
    /// Custom priority value
    Custom(u32),
}

impl TaskPriority {
    /// Create a TaskPriority from a configured value
    #[must_use]
    pub fn from_config_value(value: u32) -> Self {
        match value {
            0 => Self::Low,
            50 => Self::Normal,
            100 => Self::High,
            200 => Self::Critical,
            custom => Self::Custom(custom),
        }
    }

    /// Create TaskPriority from configuration using priority name
    #[must_use]
    pub fn from_config(config: &crate::config::types::ValidationConfig, priority_name: &str) -> Self {
        let value = config.get_task_priority(priority_name);
        Self::from_config_value(value)
    }
}

/// Task timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskTimeout {
    /// Fixed timeout duration
    Fixed(Duration),
    /// Timeout per package (for multi-package tasks)
    PerPackage(Duration),
    /// Dynamic timeout based on package count
    Dynamic {
        /// Base timeout
        base: Duration,
        /// Additional time per package
        per_package: Duration,
    },
}

/// Task environment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskEnvironment {
    /// Environment variables to set
    pub variables: HashMap<String, String>,
    
    /// Variables to inherit from parent process
    pub inherit: Vec<String>,
    
    /// Variables to explicitly unset
    pub unset: Vec<String>,
}

