//! Hook execution results implementation
//!
//! This module provides implementation methods for hook execution results,
//! following the project's pattern of separating declarations from implementations.

use super::{
    HookError, HookErrorCode, HookExecutionResult, HookStatus, HookType, HookValidationResult,
    PostCommitResult, PreCommitResult, PrePushResult, ValidationCheck,
};
use crate::Changeset;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

impl HookExecutionResult {
    /// Create a new hook execution result
    #[must_use]
    pub fn new(hook_type: HookType) -> Self {
        let now = Utc::now();
        Self {
            hook_type,
            status: HookStatus::Pending,
            duration: Duration::ZERO,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            error: None,
            validation_result: None,
            started_at: now,
            completed_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Mark the hook as successful
    #[must_use]
    pub fn with_success(mut self) -> Self {
        self.status = HookStatus::Success;
        self.completed_at = Utc::now();
        self.duration = self
            .completed_at
            .signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or(Duration::ZERO);
        self
    }

    /// Mark the hook as failed
    #[must_use]
    pub fn with_failure(mut self, error: HookError) -> Self {
        self.status = HookStatus::Failed;
        self.error = Some(error);
        self.completed_at = Utc::now();
        self.duration = self
            .completed_at
            .signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or(Duration::ZERO);
        self
    }

    /// Mark the hook as skipped
    #[must_use]
    pub fn with_skipped(mut self, reason: impl Into<String>) -> Self {
        self.status = HookStatus::Skipped;
        self.completed_at = Utc::now();
        self.duration = self
            .completed_at
            .signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or(Duration::ZERO);
        self.metadata.insert("skip_reason".to_string(), reason.into());
        self
    }

    /// Set stdout output
    #[must_use]
    pub fn with_stdout(mut self, output: impl Into<String>) -> Self {
        self.stdout = output.into();
        self
    }

    /// Set stderr output
    #[must_use]
    pub fn with_stderr(mut self, output: impl Into<String>) -> Self {
        self.stderr = output.into();
        self
    }

    /// Set exit code
    #[must_use]
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Set validation result
    #[must_use]
    pub fn with_validation_result(mut self, result: HookValidationResult) -> Self {
        self.validation_result = Some(result);
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if the hook execution was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self.status, HookStatus::Success)
    }

    /// Check if the hook execution failed
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(self.status, HookStatus::Failed)
    }

    /// Check if the hook was skipped
    #[must_use]
    pub fn is_skipped(&self) -> bool {
        matches!(self.status, HookStatus::Skipped)
    }

    /// Get the duration in milliseconds
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn duration_ms(&self) -> u64 {
        self.duration.as_millis() as u64
    }

    /// Get error message if failed
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_ref().map(|e| e.message.as_str())
    }
}

impl PreCommitResult {
    /// Create a new pre-commit result
    #[must_use]
    pub fn new() -> Self {
        Self {
            validation_passed: false,
            affected_packages: Vec::new(),
            changeset: None,
            required_actions: Vec::new(),
            validation_details: HookValidationResult::new(),
        }
    }

    /// Mark validation as passed
    #[must_use]
    pub fn with_validation_passed(mut self, passed: bool) -> Self {
        self.validation_passed = passed;
        self
    }

    /// Set affected packages
    #[must_use]
    pub fn with_affected_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    /// Set changeset
    #[must_use]
    pub fn with_changeset(mut self, changeset: Changeset) -> Self {
        self.changeset = Some(changeset);
        self
    }

    /// Add required action
    #[must_use]
    pub fn with_required_action(mut self, action: impl Into<String>) -> Self {
        self.required_actions.push(action.into());
        self
    }

    /// Set validation details
    #[must_use]
    pub fn with_validation_details(mut self, details: HookValidationResult) -> Self {
        self.validation_details = details;
        self
    }
}

impl Default for PreCommitResult {
    fn default() -> Self {
        Self::new()
    }
}

impl PrePushResult {
    /// Create a new pre-push result
    #[must_use]
    pub fn new() -> Self {
        Self {
            validation_passed: false,
            commit_count: 0,
            affected_packages: Vec::new(),
            task_results: HashMap::new(),
            required_actions: Vec::new(),
            validation_details: HookValidationResult::new(),
        }
    }

    /// Set commit count
    #[must_use]
    pub fn with_commit_count(mut self, count: usize) -> Self {
        self.commit_count = count;
        self
    }

    /// Mark validation as passed
    #[must_use]
    pub fn with_validation_passed(mut self, passed: bool) -> Self {
        self.validation_passed = passed;
        self
    }

    /// Set affected packages
    #[must_use]
    pub fn with_affected_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    /// Add task result
    #[must_use]
    pub fn with_task_result(mut self, task_name: impl Into<String>, success: bool) -> Self {
        self.task_results.insert(task_name.into(), success);
        self
    }

    /// Add required action
    #[must_use]
    pub fn with_required_action(mut self, action: impl Into<String>) -> Self {
        self.required_actions.push(action.into());
        self
    }

    /// Set validation details
    #[must_use]
    pub fn with_validation_details(mut self, details: HookValidationResult) -> Self {
        self.validation_details = details;
        self
    }
}

impl Default for PrePushResult {
    fn default() -> Self {
        Self::new()
    }
}

impl PostCommitResult {
    /// Create a new post-commit result
    #[must_use]
    pub fn new() -> Self {
        Self { notifications_sent: Vec::new(), metadata: HashMap::new() }
    }

    /// Add notification
    #[must_use]
    pub fn with_notification(mut self, notification: impl Into<String>) -> Self {
        self.notifications_sent.push(notification.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for PostCommitResult {
    fn default() -> Self {
        Self::new()
    }
}

impl HookValidationResult {
    /// Create a new validation result
    #[must_use]
    pub fn new() -> Self {
        Self { checks: HashMap::new(), overall_passed: true, required_actions: Vec::new() }
    }

    /// Add validation check
    #[must_use]
    pub fn with_check(mut self, name: impl Into<String>, check: ValidationCheck) -> Self {
        let name = name.into();
        let passed = check.passed;
        self.checks.insert(name, check);
        if !passed {
            self.overall_passed = false;
        }
        self
    }

    /// Add required action
    #[must_use]
    pub fn with_required_action(mut self, action: impl Into<String>) -> Self {
        self.required_actions.push(action.into());
        self
    }

    /// Check if all validations passed
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.overall_passed
    }

    /// Get failed checks
    #[must_use]
    pub fn failed_checks(&self) -> Vec<(&String, &ValidationCheck)> {
        self.checks.iter().filter(|(_, check)| !check.passed).collect()
    }
}

impl Default for HookValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl HookError {
    /// Create a new hook error
    #[must_use]
    pub fn new(code: HookErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), context: HashMap::new(), cause: None }
    }

    /// Add context information
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Set the root cause
    #[must_use]
    pub fn with_cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }
}

impl ValidationCheck {
    /// Create a passing validation check
    #[must_use]
    pub fn passed(message: impl Into<String>) -> Self {
        Self { passed: true, message: message.into(), details: None }
    }

    /// Create a failing validation check
    #[must_use]
    pub fn failed(message: impl Into<String>) -> Self {
        Self { passed: false, message: message.into(), details: None }
    }

    /// Add details to the check
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}
