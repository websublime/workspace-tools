//! Task builders implementation
//!
//! Builder pattern implementations for creating tasks, commands, and package scripts.

use super::types::{
    TaskDefinition, TaskCommand, TaskCommandCore, PackageScript, TaskPriority, TaskScope, 
    TaskCondition, TaskTimeout, TaskEnvironment, TaskExecutionResult,
    TaskStatus, TaskError, TaskErrorCode, TaskExecutionLog, TaskLogLevel,
    TaskOutput, TaskExecutionStats,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

impl TaskDefinition {
    /// Create a new task definition with minimal requirements
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            commands: Vec::new(),
            package_scripts: Vec::new(),
            dependencies: Vec::new(),
            conditions: Vec::new(),
            priority: TaskPriority::Normal,
            scope: TaskScope::AffectedPackages,
            continue_on_error: false,
            timeout: None,
            environment: TaskEnvironment::default(),
        }
    }
    
    /// Set priority
    #[must_use]
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Add a dependency
    #[must_use]
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }
    
    /// Set continue on error behavior
    #[must_use]
    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }
    
    /// Add a command to the task
    #[must_use]
    pub fn with_command(mut self, command: TaskCommand) -> Self {
        self.commands.push(command);
        self
    }
    
    /// Add a package script to the task
    #[must_use]
    pub fn with_package_script(mut self, script: PackageScript) -> Self {
        self.package_scripts.push(script);
        self
    }
    
    /// Set task scope
    #[must_use]
    pub fn with_scope(mut self, scope: TaskScope) -> Self {
        self.scope = scope;
        self
    }
    
    /// Add a condition
    #[must_use]
    pub fn with_condition(mut self, condition: TaskCondition) -> Self {
        self.conditions.push(condition);
        self
    }
    
    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(TaskTimeout::Fixed(timeout));
        self
    }
}

impl TaskCommandCore {
    /// Create a new command core
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            env: HashMap::new(),
            timeout: None,
        }
    }
    
    /// Add arguments
    #[must_use]
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
    
    /// Set working directory
    #[must_use]
    pub fn with_current_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(dir.into());
        self
    }
    
    /// Add environment variable
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
    
    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl TaskCommand {
    /// Create a new task command
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            command: TaskCommandCore::new(program),
            shell: false,
            expected_exit_codes: vec![0],
        }
    }
    
    /// Create from command core
    #[must_use] pub fn from_core(command: TaskCommandCore) -> Self {
        Self {
            command,
            shell: false,
            expected_exit_codes: vec![0],
        }
    }
    
    /// Set to run in shell
    #[must_use]
    pub fn in_shell(mut self) -> Self {
        self.shell = true;
        self
    }
    
    /// Set expected exit codes
    #[must_use]
    pub fn with_expected_exit_codes(mut self, codes: Vec<i32>) -> Self {
        self.expected_exit_codes = codes;
        self
    }
    
    /// Add arguments (convenience method)
    #[must_use]
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command = self.command.with_args(args);
        self
    }
    
    /// Set working directory (convenience method)
    #[must_use]
    pub fn with_current_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.command = self.command.with_current_dir(dir);
        self
    }
    
    /// Add environment variable (convenience method)
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.command = self.command.with_env(key, value);
        self
    }
    
    /// Set timeout (convenience method)
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.command = self.command.with_timeout(timeout);
        self
    }
}

impl PackageScript {
    /// Create a new package script reference
    pub fn new(script_name: impl Into<String>) -> Self {
        Self {
            package_name: None,
            script_name: script_name.into(),
            working_directory: None,
            extra_args: Vec::new(),
            package_manager: None,
        }
    }
    
    /// Target specific package
    #[must_use]
    pub fn for_package(mut self, package_name: impl Into<String>) -> Self {
        self.package_name = Some(package_name.into());
        self
    }
    
    /// Add extra arguments
    #[must_use]
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extra_args.extend(args.into_iter().map(Into::into));
        self
    }
    
    /// Set package manager
    #[must_use]
    pub fn with_package_manager(mut self, pm: impl Into<String>) -> Self {
        self.package_manager = Some(pm.into());
        self
    }
    
    /// Set working directory
    #[must_use]
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }
}

impl TaskEnvironment {
    /// Create new environment configuration
    #[must_use] pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an environment variable
    #[must_use]
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }
    
    /// Inherit a variable from parent process
    #[must_use]
    pub fn inherit(mut self, var: impl Into<String>) -> Self {
        self.inherit.push(var.into());
        self
    }
    
    /// Unset a variable
    #[must_use]
    pub fn unset(mut self, var: impl Into<String>) -> Self {
        self.unset.push(var.into());
        self
    }
}

impl TaskExecutionResult {
    /// Create a new task execution result
    pub fn new(task_name: impl Into<String>) -> Self {
        let now = SystemTime::now();
        Self {
            task_name: task_name.into(),
            status: TaskStatus::Pending,
            started_at: now,
            ended_at: now,
            duration: Duration::default(),
            outputs: Vec::new(),
            stats: TaskExecutionStats::default(),
            affected_packages: Vec::new(),
            errors: Vec::new(),
            logs: Vec::new(),
            artifacts: Vec::new(),
        }
    }
    
    /// Mark task as started
    pub fn mark_started(&mut self) {
        self.started_at = SystemTime::now();
        self.status = TaskStatus::Running;
    }
    
    /// Mark task as completed
    pub fn mark_completed(&mut self, success: bool) {
        let now = SystemTime::now();
        self.ended_at = now;
        self.duration = now.duration_since(self.started_at).unwrap_or_default();
        self.status = if success {
            TaskStatus::Success
        } else {
            TaskStatus::Failed {
                reason: "Task execution failed".to_string(),
            }
        };
    }
    
    /// Add an error
    pub fn add_error(&mut self, error: TaskError) {
        self.errors.push(error);
    }
    
    /// Add a log entry
    pub fn add_log(&mut self, log: TaskExecutionLog) {
        self.logs.push(log);
    }
    
    /// Check if execution was successful
    #[must_use] pub fn is_success(&self) -> bool {
        matches!(self.status, TaskStatus::Success)
    }
    
    /// Check if execution failed
    #[must_use] pub fn is_failure(&self) -> bool {
        matches!(self.status, TaskStatus::Failed { .. })
    }
}

impl TaskError {
    /// Create a new task error
    pub fn new(code: TaskErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: HashMap::new(),
            occurred_at: SystemTime::now(),
            package: None,
            command: None,
        }
    }
    
    /// Set the package name
    #[must_use]
    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }
    
    /// Set the command
    #[must_use]
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }
    
    /// Add context information
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

impl TaskExecutionLog {
    /// Create a new log entry
    pub fn new(level: TaskLogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.into(),
            package: None,
            data: HashMap::new(),
        }
    }
    
    /// Create an info log
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(TaskLogLevel::Info, message)
    }
    
    /// Create a warning log
    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(TaskLogLevel::Warning, message)
    }
    
    /// Create an error log
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(TaskLogLevel::Error, message)
    }
    
    /// Create a debug log
    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(TaskLogLevel::Debug, message)
    }
}

impl TaskOutput {
    /// Check if the command execution was successful
    #[must_use] pub fn is_success(&self) -> bool {
        matches!(self.exit_code, Some(0))
    }
}