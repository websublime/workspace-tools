//! Implementation of workflow status and progress types

use super::types::{WorkflowProgress, WorkflowStatus, WorkflowStep};

impl WorkflowProgress {
    /// Create a new workflow progress tracker
    #[must_use]
    pub fn new(total_steps: usize, description: String) -> Self {
        Self {
            current_step: 0,
            total_steps,
            current_step_description: "Starting workflow".to_string(),
            status: WorkflowStatus::Pending,
            started_at: None,
            finished_at: None,
            error_message: None,
            description,
            steps: Vec::new(),
        }
    }

    /// Advance to the next step
    pub fn advance_step(&mut self, step_description: String) {
        if self.is_completed() || self.is_failed() {
            return;
        }

        if self.started_at.is_none() {
            self.started_at = Some(chrono::Utc::now());
            self.status = WorkflowStatus::Running;
        }

        if self.current_step > 0 {
            // Mark previous step as completed
            if let Some(step) = self.steps.last_mut() {
                step.status = WorkflowStatus::Completed;
                step.finished_at = Some(chrono::Utc::now());
            }
        }

        // Add new step
        let step = WorkflowStep {
            name: step_description.clone(),
            description: step_description.clone(),
            status: WorkflowStatus::Running,
            started_at: Some(chrono::Utc::now()),
            finished_at: None,
            error_message: None,
        };
        self.steps.push(step);

        self.current_step += 1;
        self.current_step_description = step_description;

        // Check if completed
        if self.current_step >= self.total_steps {
            self.status = WorkflowStatus::Completed;
            self.finished_at = Some(chrono::Utc::now());
            
            // Mark last step as completed
            if let Some(step) = self.steps.last_mut() {
                step.status = WorkflowStatus::Completed;
                step.finished_at = Some(chrono::Utc::now());
            }
        }
    }

    /// Mark the workflow as failed
    pub fn fail(&mut self, error: String) {
        self.status = WorkflowStatus::Failed;
        self.finished_at = Some(chrono::Utc::now());
        self.error_message = Some(error.clone());

        // Mark current step as failed
        if let Some(step) = self.steps.last_mut() {
            step.status = WorkflowStatus::Failed;
            step.finished_at = Some(chrono::Utc::now());
            step.error_message = Some(error);
        }
    }

    /// Get completion percentage (0.0 to 100.0)
    #[must_use]
    pub fn completion_percentage(&self) -> f64 {
        if self.total_steps == 0 {
            return 100.0;
        }
        
        let completed_steps = if self.is_completed() {
            self.total_steps
        } else {
            self.current_step.saturating_sub(1) // Don't count current running step as completed
        };
        
        (completed_steps as f64 / self.total_steps as f64) * 100.0
    }

    /// Check if workflow is currently running
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self.status, WorkflowStatus::Running)
    }

    /// Check if workflow is completed
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self.status, WorkflowStatus::Completed)
    }

    /// Check if workflow has failed
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self.status, WorkflowStatus::Failed)
    }

    /// Get estimated time remaining
    #[must_use]
    pub fn estimated_time_remaining(&self) -> Option<chrono::Duration> {
        if self.is_completed() || self.is_failed() || self.started_at.is_none() {
            return None;
        }

        let started_at = self.started_at?;
        let elapsed = chrono::Utc::now().signed_duration_since(started_at);
        
        if self.current_step == 0 {
            return None;
        }

        let avg_time_per_step = elapsed.num_milliseconds() / self.current_step as i64;
        let remaining_steps = self.total_steps.saturating_sub(self.current_step);
        let estimated_remaining_ms = avg_time_per_step * remaining_steps as i64;
        
        chrono::Duration::milliseconds(estimated_remaining_ms).into()
    }

    /// Get workflow duration
    #[must_use]
    pub fn duration(&self) -> Option<chrono::Duration> {
        let started_at = self.started_at?;
        let end_time = self.finished_at.unwrap_or_else(chrono::Utc::now);
        Some(end_time.signed_duration_since(started_at))
    }

    /// Get current step information
    #[must_use]
    pub fn current_step_info(&self) -> Option<&WorkflowStep> {
        self.steps.last()
    }

    /// Add a substep to the current step
    pub fn add_substep(&mut self, description: String) {
        // For now, just update the current step description
        // In a more complex implementation, we could track substeps
        if let Some(step) = self.steps.last_mut() {
            step.description = format!("{}: {}", step.description, description);
        }
        self.current_step_description = description;
    }
}