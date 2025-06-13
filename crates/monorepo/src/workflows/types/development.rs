//! Development workflow type definitions

use crate::analysis::MonorepoAnalyzer;
use crate::core::MonorepoProject;
use std::sync::Arc;

/// Implements development workflow for monorepo projects
///
/// This workflow handles the development phase of monorepo management,
/// including change detection, affected package identification, and task execution.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_monorepo_tools::{DevelopmentWorkflow, MonorepoProject};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
/// let workflow = DevelopmentWorkflow::new(project);
///
/// // Run development workflow - detects changes and runs affected tasks
/// let result = workflow.run(None).await?;
/// println!("Affected packages: {}", result.affected_packages.len());
/// println!("Executed tasks: {}", result.affected_tasks.len());
/// # Ok(())
/// # }
/// ```
pub struct DevelopmentWorkflow {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,

    /// Analyzer for detecting changes and affected packages
    pub(crate) analyzer: MonorepoAnalyzer,

    /// Task manager for executing development tasks
    pub(crate) task_manager: crate::tasks::TaskManager,

    /// Changeset manager for handling development changesets
    pub(crate) changeset_manager: crate::changesets::ChangesetManager,
}
