//! Hook type definitions and configurations
//!
//! This module defines the core types for Git hooks, including hook types,
//! definitions, scripts, and conditions for execution.

use crate::Environment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

/// Git hook types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookType {
    /// Pre-commit hook - runs before commits are created
    PreCommit,
    /// Pre-push hook - runs before pushes are executed
    PrePush,
    /// Post-commit hook - runs after commits are created
    PostCommit,
    /// Post-merge hook - runs after merges are completed
    PostMerge,
    /// Post-checkout hook - runs after checkouts are performed
    PostCheckout,
}

/// Definition of a Git hook with its script and execution conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    /// The script or command to execute
    pub script: HookScript,
    
    /// Conditions that must be met for the hook to execute
    pub conditions: Vec<HookCondition>,
    
    /// Whether to fail the Git operation if the hook fails
    pub fail_on_error: bool,
    
    /// Timeout for hook execution
    pub timeout: Option<Duration>,
    
    /// Environment variables to set during execution
    pub environment: HashMap<String, String>,
    
    /// Working directory for hook execution
    pub working_directory: Option<PathBuf>,
    
    /// Description of what this hook does
    pub description: String,
    
    /// Whether this hook is enabled
    pub enabled: bool,
}


/// Script types that can be executed by hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookScript {
    /// Execute registered tasks by name
    TaskExecution { 
        /// Names of tasks to execute
        tasks: Vec<String>,
        /// Whether to run tasks in parallel
        parallel: bool,
    },
    
    /// Execute a shell command
    Command { 
        /// Command to execute
        cmd: String,
        /// Command arguments
        args: Vec<String>,
    },
    
    /// Execute a script file
    ScriptFile { 
        /// Path to the script file
        path: PathBuf,
        /// Arguments to pass to the script
        args: Vec<String>,
    },
    
    /// Execute multiple scripts in sequence
    Sequence {
        /// Scripts to execute in order
        scripts: Vec<HookScript>,
        /// Whether to stop on first failure
        stop_on_failure: bool,
    },
}


/// Conditions that determine when a hook should execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookCondition {
    /// Only execute if specific files have changed
    FilesChanged {
        /// File patterns to match
        patterns: Vec<String>,
        /// Whether to match any pattern (true) or all patterns (false)
        match_any: bool,
    },
    
    /// Only execute if specific packages have changed
    PackagesChanged {
        /// Package names to check
        packages: Vec<String>,
        /// Whether to match any package (true) or all packages (false)
        match_any: bool,
    },
    
    /// Only execute if dependencies have changed
    DependenciesChanged {
        /// Specific dependency types to check
        dependency_types: Vec<DependencyType>,
    },
    
    /// Only execute on specific branches
    OnBranch {
        /// Branch name pattern (supports wildcards)
        pattern: String,
    },
    
    /// Only execute in specific environments
    Environment {
        /// Environment to check
        env: Environment,
    },
    
    /// Only execute if changeset exists for changes
    ChangesetExists {
        /// Whether to require changeset for all changes
        require_for_all: bool,
    },
    
    /// Custom condition based on environment variables
    EnvironmentVariable {
        /// Environment variable name
        name: String,
        /// Expected value (optional)
        value: Option<String>,
    },
    
    /// Only execute if specific Git refs exist
    GitRefExists {
        /// Git reference pattern
        ref_pattern: String,
    },
}

/// Types of dependencies that can trigger hook conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Production dependencies
    Production,
    /// Development dependencies
    Development,
    /// Peer dependencies
    Peer,
    /// Optional dependencies
    Optional,
    /// All dependency types
    All,
}

impl fmt::Display for HookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PreCommit => write!(f, "pre-commit"),
            Self::PrePush => write!(f, "pre-push"),
            Self::PostCommit => write!(f, "post-commit"),
            Self::PostMerge => write!(f, "post-merge"),
            Self::PostCheckout => write!(f, "post-checkout"),
        }
    }
}

