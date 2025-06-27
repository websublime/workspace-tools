//! Development workflow type definitions

use crate::analysis::MonorepoAnalyzer;
// Removed unused imports - now using dependency injection

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
/// let workflow = DevelopmentWorkflow::from_project(project)?;
///
/// // Run development workflow - detects changes and runs affected tasks
/// let result = workflow.run(None).await?;
/// println!("Affected packages: {}", result.affected_packages.len());
/// println!("Executed tasks: {}", result.affected_tasks.len());
/// # Ok(())
/// # }
/// ```
pub struct DevelopmentWorkflow {
    /// Analyzer for detecting changes and affected packages
    pub(crate) analyzer: MonorepoAnalyzer,

    /// Task manager for executing development tasks
    pub(crate) task_manager: crate::tasks::TaskManager,

    /// Changeset manager for handling development changesets
    pub(crate) changeset_manager: crate::changesets::ChangesetManager,

    /// Configuration provider for accessing configuration settings
    pub(crate) config_provider: Box<dyn crate::core::ConfigProvider>,

    /// Package provider for accessing package information
    pub(crate) package_provider: Box<dyn crate::core::PackageProvider>,

    /// Git provider for repository operations
    pub(crate) git_provider: Box<dyn crate::core::GitProvider>,
}
