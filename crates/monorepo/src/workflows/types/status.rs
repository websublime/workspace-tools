//! Workflow status types
//!
//! Status tracking and progress monitoring types for workflow executions.

use serde::{Deserialize, Serialize};
use std::time::Duration;

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
/// let failed_status = WorkflowStatus::Failed {
///     error: "Build failed".to_string()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is running
    Running,

    /// Workflow completed successfully
    Completed,

    /// Workflow failed with errors
    Failed {
        /// Error message describing the failure
        error: String,
    },

    /// Workflow was cancelled
    Cancelled,
}

/// Progress information for long-running workflows
///
/// Provides detailed progress tracking for workflows that take time to execute,
/// including step-by-step progress and time estimates.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use sublime_monorepo_tools::{WorkflowProgress, WorkflowStatus};
///
/// let progress = WorkflowProgress {
///     current_step: "Running tests".to_string(),
///     completed_steps: 3,
///     total_steps: 5,
///     status: WorkflowStatus::Running,
///     estimated_remaining: Some(Duration::from_secs(120)),
/// };
///
/// let percentage = (progress.completed_steps as f64 / progress.total_steps as f64) * 100.0;
/// println!("Progress: {:.1}%", percentage);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowProgress {
    /// Current step being executed
    pub current_step: String,

    /// Number of completed steps
    pub completed_steps: usize,

    /// Total number of steps
    pub total_steps: usize,

    /// Current workflow status
    pub status: WorkflowStatus,

    /// Estimated time remaining
    pub estimated_remaining: Option<Duration>,
}

impl WorkflowProgress {
    /// Creates a new workflow progress tracker
    ///
    /// # Arguments
    ///
    /// * `total_steps` - Total number of steps in the workflow
    /// * `initial_step` - Description of the first step
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::WorkflowProgress;
    ///
    /// let progress = WorkflowProgress::new(5, "Initializing".to_string());
    /// assert_eq!(progress.completed_steps, 0);
    /// assert_eq!(progress.total_steps, 5);
    /// ```
    #[must_use]
    pub fn new(total_steps: usize, initial_step: String) -> Self {
        Self {
            current_step: initial_step,
            completed_steps: 0,
            total_steps,
            status: WorkflowStatus::Running,
            estimated_remaining: None,
        }
    }

    /// Advances to the next step
    ///
    /// # Arguments
    ///
    /// * `next_step` - Description of the next step
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::WorkflowProgress;
    ///
    /// let mut progress = WorkflowProgress::new(3, "Starting".to_string());
    /// progress.advance_step("Building".to_string());
    /// assert_eq!(progress.completed_steps, 1);
    /// assert_eq!(progress.current_step, "Building");
    /// ```
    pub fn advance_step(&mut self, next_step: String) {
        self.completed_steps += 1;
        self.current_step = next_step;

        if self.completed_steps >= self.total_steps {
            self.status = WorkflowStatus::Completed;
        }
    }

    /// Marks the workflow as failed
    ///
    /// # Arguments
    ///
    /// * `error` - Error message describing the failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{WorkflowProgress, WorkflowStatus};
    ///
    /// let mut progress = WorkflowProgress::new(3, "Starting".to_string());
    /// progress.fail("Build error".to_string());
    /// assert!(matches!(progress.status, WorkflowStatus::Failed { .. }));
    /// ```
    pub fn fail(&mut self, error: String) {
        self.status = WorkflowStatus::Failed { error };
    }

    /// Calculates the completion percentage
    ///
    /// # Returns
    ///
    /// Percentage of completion as a float between 0.0 and 100.0
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::WorkflowProgress;
    ///
    /// let mut progress = WorkflowProgress::new(4, "Starting".to_string());
    /// progress.advance_step("Step 1".to_string());
    /// progress.advance_step("Step 2".to_string());
    /// assert_eq!(progress.completion_percentage(), 50.0);
    /// ```
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn completion_percentage(&self) -> f64 {
        if self.total_steps == 0 {
            return 100.0;
        }
        (self.completed_steps as f64 / self.total_steps as f64) * 100.0
    }

    /// Checks if the workflow is still running
    ///
    /// # Returns
    ///
    /// True if the workflow is still in progress
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::WorkflowProgress;
    ///
    /// let progress = WorkflowProgress::new(3, "Starting".to_string());
    /// assert!(progress.is_running());
    /// ```
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self.status, WorkflowStatus::Running)
    }

    /// Checks if the workflow completed successfully
    ///
    /// # Returns
    ///
    /// True if the workflow completed without errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::WorkflowProgress;
    ///
    /// let mut progress = WorkflowProgress::new(1, "Starting".to_string());
    /// progress.advance_step("Done".to_string());
    /// assert!(progress.is_completed());
    /// ```
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self.status, WorkflowStatus::Completed)
    }

    /// Checks if the workflow failed
    ///
    /// # Returns
    ///
    /// True if the workflow failed with an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::WorkflowProgress;
    ///
    /// let mut progress = WorkflowProgress::new(3, "Starting".to_string());
    /// progress.fail("Error occurred".to_string());
    /// assert!(progress.is_failed());
    /// ```
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self.status, WorkflowStatus::Failed { .. })
    }
}
