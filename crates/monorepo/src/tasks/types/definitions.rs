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

impl TaskDefinition {
    /// Validate the task definition
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.name.is_empty() {
            return Err(crate::error::Error::Task(
                "Task name cannot be empty".to_string(),
            ));
        }

        if self.description.is_empty() {
            return Err(crate::error::Error::Task(
                "Task description cannot be empty".to_string(),
            ));
        }

        if self.commands.is_empty() && self.package_scripts.is_empty() {
            return Err(crate::error::Error::Task(
                "Task must have at least one command or package script".to_string(),
            ));
        }

        // Validate commands
        for command in &self.commands {
            if command.command.program.is_empty() {
                return Err(crate::error::Error::Task(
                    "Command program cannot be empty".to_string(),
                ));
            }
        }

        // Validate package scripts
        for script in &self.package_scripts {
            if script.script_name.is_empty() {
                return Err(crate::error::Error::Task(
                    "Package script name cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Task command with standard Command integration and task-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommand {
    /// The underlying command (using standard crate)
    #[serde(flatten)]
    pub command: TaskCommandCore,

    /// Whether to run in shell
    pub shell: bool,

    /// Expected exit codes (default: \[0\])
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Lowest priority
    Low,
    /// Normal priority (default)
    Normal,
    /// High priority
    High,
    /// Critical priority
    Critical,
    /// Custom priority value
    Custom(u32),
}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_value().cmp(&other.to_value())
    }
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

    /// Get the numeric value for priority comparison
    #[must_use]
    pub fn to_value(&self) -> u32 {
        match self {
            Self::Low => 0,
            Self::Normal => 50,
            Self::High => 100,
            Self::Critical => 200,
            Self::Custom(value) => *value,
        }
    }

    /// Create TaskPriority from configuration using priority name
    #[must_use]
    pub fn from_config(
        config: &crate::config::types::ValidationConfig,
        priority_name: &str,
    ) -> Self {
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

impl TaskTimeout {
    /// Calculate the actual timeout duration based on package count
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn calculate_timeout(&self, package_count: usize) -> Duration {
        match self {
            Self::Fixed(duration) => *duration,
            Self::PerPackage(duration) => *duration * package_count as u32,
            Self::Dynamic { base, per_package } => {
                *base + (*per_package * package_count as u32)
            }
        }
    }
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

impl TaskEnvironment {
    /// Merge this environment with another, with the other taking precedence
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let mut variables = self.variables.clone();
        for (key, value) in &other.variables {
            variables.insert(key.clone(), value.clone());
        }

        let mut inherit = self.inherit.clone();
        for var in &other.inherit {
            if !inherit.contains(var) {
                inherit.push(var.clone());
            }
        }

        let mut unset = self.unset.clone();
        for var in &other.unset {
            if !unset.contains(var) {
                unset.push(var.clone());
            }
        }

        Self {
            variables,
            inherit,
            unset,
        }
    }
}
