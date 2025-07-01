//! Implementation of workflow status and progress types

use super::types::{WorkflowProgress, WorkflowStatus, WorkflowStep, SubStep};

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
            substeps: Vec::new(),
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
    #[allow(clippy::cast_precision_loss)]
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
    #[allow(clippy::cast_possible_wrap)]
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
    ///
    /// This method adds a new substep to the currently running workflow step.
    /// Substeps provide fine-grained progress tracking within major workflow steps.
    ///
    /// # Arguments
    ///
    /// * `description` - Description of the substep being performed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::workflows::WorkflowProgress;
    ///
    /// let mut progress = WorkflowProgress::new(3, "Release workflow".to_string());
    /// progress.advance_step("Building packages".to_string());
    /// progress.add_substep("Compiling TypeScript".to_string());
    /// progress.add_substep("Running tests".to_string());
    /// progress.add_substep("Creating bundles".to_string());
    /// ```
    pub fn add_substep(&mut self, description: &str) {
        if let Some(current_step) = self.steps.last_mut() {
            // Only add substeps to running steps
            if matches!(current_step.status, WorkflowStatus::Running) {
                let substep = SubStep {
                    description: description.to_string(),
                    timestamp: chrono::Utc::now(),
                    completed: false,
                };
                current_step.substeps.push(substep);
                
                log::debug!(
                    "Added substep '{}' to step '{}' (total substeps: {})",
                    description,
                    current_step.name,
                    current_step.substeps.len()
                );
            } else {
                log::warn!(
                    "Cannot add substep '{}' to step '{}' - step is not running (status: {:?})",
                    description,
                    current_step.name,
                    current_step.status
                );
            }
        } else {
            log::warn!("Cannot add substep '{}' - no current step available", description);
        }
        
        // Update the current step description to reflect the latest substep
        self.current_step_description = format!("{current}: {description}", current = self.current_step_description);
    }

    /// Mark the current substep as completed
    ///
    /// This method marks the most recently added substep as completed.
    /// Useful for tracking which substeps within a workflow step are finished.
    pub fn complete_current_substep(&mut self) {
        if let Some(current_step) = self.steps.last_mut() {
            if let Some(last_substep) = current_step.substeps.last_mut() {
                if !last_substep.completed {
                    last_substep.completed = true;
                    log::debug!(
                        "Marked substep '{}' as completed in step '{}'",
                        last_substep.description,
                        current_step.name
                    );
                }
            }
        }
    }

    /// Get the number of completed substeps for the current step
    #[must_use]
    pub fn current_step_completed_substeps(&self) -> usize {
        self.steps
            .last()
            .map_or(0, |step| step.substeps.iter().filter(|substep| substep.completed).count())
    }

    /// Get the total number of substeps for the current step
    #[must_use]
    pub fn current_step_total_substeps(&self) -> usize {
        self.steps
            .last()
            .map_or(0, |step| step.substeps.len())
    }

    /// Get substep completion percentage for the current step (0.0 to 100.0)
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn current_step_substep_percentage(&self) -> f64 {
        let total = self.current_step_total_substeps();
        if total == 0 {
            return 100.0;
        }
        
        let completed = self.current_step_completed_substeps();
        (completed as f64 / total as f64) * 100.0
    }
}
