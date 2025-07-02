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
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct DevelopmentWorkflow<'a> {
    /// Analyzer for detecting changes and affected packages
    pub(crate) analyzer: MonorepoAnalyzer<'a>,

    /// Task manager for executing development tasks
    pub(crate) task_manager: crate::tasks::TaskManager<'a>,

    /// Changeset manager for handling development changesets
    pub(crate) changeset_manager: crate::changesets::ChangesetManager<'a>,

    /// Direct reference to configuration
    pub(crate) config: &'a crate::config::MonorepoConfig,

    /// Direct reference to packages
    pub(crate) packages: &'a [crate::core::MonorepoPackageInfo],

    /// Direct reference to git repository
    pub(crate) repository: &'a sublime_git_tools::Repo,

    /// Direct reference to root path
    pub(crate) root_path: &'a std::path::Path,
}
