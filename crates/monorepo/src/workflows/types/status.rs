//! Workflow status types
//!
//! Status tracking and progress monitoring types for workflow executions.

use serde::{Deserialize, Serialize};

/// Status of a workflow execution
///
/// Tracks the current state of a workflow execution, including
/// completion status and any errors that occurred.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::WorkflowStatus;
///
/// let status = WorkflowStatus::Running;
/// assert!(matches!(status, WorkflowStatus::Running));
///
/// let failed_status = WorkflowStatus::Failed;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is pending start
    Pending,
    /// Workflow is running
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with errors
    Failed,
    /// Workflow was cancelled
    Cancelled,
}

/// Individual step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Current status
    pub status: WorkflowStatus,
    /// When step started
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When step finished
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Progress information for long-running workflows
///
/// Provides detailed progress tracking for workflows that take time to execute,
/// including step-by-step progress and time estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    /// Current step number (0-based)
    pub current_step: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Description of current step
    pub current_step_description: String,
    /// Current workflow status
    pub status: WorkflowStatus,
    /// When workflow started
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When workflow finished
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Workflow description
    pub description: String,
    /// Individual workflow steps
    pub steps: Vec<WorkflowStep>,
}
