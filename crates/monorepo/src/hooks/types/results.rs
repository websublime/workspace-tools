//! Hook execution results and status types
//!
//! This module defines the result types returned by hook executions,
//! including success/failure status, validation results, and error information.

use crate::Changeset;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Result of a hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecutionResult {
    /// The hook type that was executed
    pub hook_type: super::HookType,
    
    /// Overall status of the hook execution
    pub status: HookStatus,
    
    /// Duration of the hook execution
    pub duration: Duration,
    
    /// Standard output from the hook
    pub stdout: String,
    
    /// Standard error from the hook
    pub stderr: String,
    
    /// Exit code of the hook execution
    pub exit_code: Option<i32>,
    
    /// Detailed error information if the hook failed
    pub error: Option<HookError>,
    
    /// Validation results from pre-commit/pre-push checks
    pub validation_result: Option<HookValidationResult>,
    
    /// Timestamp when the hook started
    pub started_at: DateTime<Utc>,
    
    /// Timestamp when the hook completed
    pub completed_at: DateTime<Utc>,
    
    /// Additional metadata about the execution
    pub metadata: HashMap<String, String>,
}

/// Status of a hook execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookStatus {
    /// Hook is waiting to be executed
    Pending,
    /// Hook is currently running
    Running,
    /// Hook completed successfully
    Success,
    /// Hook failed
    Failed,
    /// Hook was skipped due to conditions
    Skipped,
}

/// Error information for failed hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookError {
    /// Error code for categorization
    pub code: HookErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional context information
    pub context: HashMap<String, String>,
    /// Root cause of the error
    pub cause: Option<String>,
}

/// Categories of hook errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookErrorCode {
    /// Hook script execution failed
    ExecutionFailed,
    /// Hook validation checks failed
    ValidationFailed,
    /// Required changeset is missing
    ChangesetMissing,
    /// Task execution failed
    TaskFailed,
    /// Hook installation failed
    InstallationFailed,
    /// Hook configuration error
    ConfigurationError,
    /// System error (file system, permissions, etc.)
    SystemError,
}

/// Detailed validation results from hook checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookValidationResult {
    /// Individual validation checks and their results
    pub checks: HashMap<String, ValidationCheck>,
    /// Whether all validations passed
    pub overall_passed: bool,
    /// Actions required to fix validation failures
    pub required_actions: Vec<String>,
}

/// Individual validation check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    /// Whether the check passed
    pub passed: bool,
    /// Description of the check result
    pub message: String,
    /// Additional details about the check
    pub details: Option<String>,
}

/// Result of pre-commit hook validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCommitResult {
    /// Whether pre-commit validation passed
    pub validation_passed: bool,
    /// Packages affected by the changes
    pub affected_packages: Vec<String>,
    /// Changeset information if found
    pub changeset: Option<Changeset>,
    /// Actions required to fix validation failures
    pub required_actions: Vec<String>,
    /// Detailed validation results
    pub validation_details: HookValidationResult,
}

/// Result of pre-push hook validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrePushResult {
    /// Whether pre-push validation passed
    pub validation_passed: bool,
    /// Number of commits being pushed
    pub commit_count: usize,
    /// Packages affected by the changes
    pub affected_packages: Vec<String>,
    /// Results of individual task executions
    pub task_results: HashMap<String, bool>,
    /// Actions required to fix validation failures
    pub required_actions: Vec<String>,
    /// Detailed validation results
    pub validation_details: HookValidationResult,
}

/// Result of post-commit hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommitResult {
    /// Notifications that were sent
    pub notifications_sent: Vec<String>,
    /// Additional metadata from post-commit actions
    pub metadata: HashMap<String, String>,
}

impl Default for HookStatus {
    fn default() -> Self {
        Self::Pending
    }
}