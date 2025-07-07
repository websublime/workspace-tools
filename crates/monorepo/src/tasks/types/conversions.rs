//! Type conversions for task definitions
//!
//! Implementation of conversion traits between task types and standard library types.
//! Separated from type definitions for better architecture organization.

use super::definitions::{PackageScript, TaskCommand, TaskCommandCore, TaskPriority};
use sublime_standard_tools::command::{Command, CommandBuilder, CommandPriority};

/// Convert TaskCommandCore to standard Command
impl From<TaskCommandCore> for Command {
    fn from(task_cmd: TaskCommandCore) -> Self {
        let mut builder = CommandBuilder::new(task_cmd.program);

        for arg in task_cmd.args {
            builder = builder.arg(arg);
        }

        for (key, value) in task_cmd.env {
            builder = builder.env(key, value);
        }

        if let Some(dir) = task_cmd.current_dir {
            builder = builder.current_dir(dir);
        }

        if let Some(timeout) = task_cmd.timeout {
            builder = builder.timeout(timeout);
        }

        builder.build()
    }
}

/// Convert TaskCommand to standard Command
impl From<TaskCommand> for Command {
    fn from(task_cmd: TaskCommand) -> Self {
        task_cmd.command.into()
    }
}

/// Convert TaskPriority to standard CommandPriority
impl From<TaskPriority> for CommandPriority {
    fn from(task_priority: TaskPriority) -> Self {
        match task_priority {
            TaskPriority::Low => CommandPriority::Low,
            TaskPriority::Normal => CommandPriority::Normal,
            TaskPriority::High => CommandPriority::High,
            TaskPriority::Critical => CommandPriority::Critical,
            TaskPriority::Custom(value) => {
                // Map custom values to closest standard priority
                match value {
                    0..=25 => CommandPriority::Low,
                    26..=75 => CommandPriority::Normal,
                    76..=150 => CommandPriority::High,
                    _ => CommandPriority::Critical,
                }
            }
        }
    }
}

/// Convert PackageScript to standard Command
impl From<PackageScript> for Command {
    fn from(script: PackageScript) -> Self {
        // Determine package manager command
        let manager = script.package_manager.as_deref().unwrap_or("npm");

        let mut builder = CommandBuilder::new(manager);

        // Add run command for script execution
        builder = builder.arg("run").arg(script.script_name);

        // Add extra arguments
        if !script.extra_args.is_empty() {
            builder = builder.arg("--");
            for arg in script.extra_args {
                builder = builder.arg(arg);
            }
        }

        // Set working directory if specified
        if let Some(dir) = script.working_directory {
            builder = builder.current_dir(dir);
        }

        builder.build()
    }
}

/// Default implementation for TaskPriority
impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// TaskPriority utility methods
impl TaskPriority {
    /// Get numeric value for priority comparison
    #[must_use]
    pub fn value(&self) -> u32 {
        match self {
            TaskPriority::Low => 25,
            TaskPriority::Normal => 50,
            TaskPriority::High => 100,
            TaskPriority::Critical => 200,
            TaskPriority::Custom(v) => *v,
        }
    }
}
