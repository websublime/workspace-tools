//! Release workflow type definitions

use crate::analysis::MonorepoAnalyzer;

/// Implements release workflow for monorepo projects
///
/// This workflow handles the release phase of monorepo management,
/// including changeset application, version bumping, and production deployment.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_monorepo_tools::{ReleaseWorkflow, MonorepoProject};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
/// let workflow = ReleaseWorkflow::from_project(project)?;
///
/// // Run release workflow - applies changesets and bumps versions
/// let result = workflow.run().await?;
/// println!("Released packages: {}", result.released_packages.len());
/// # Ok(())
/// # }
/// ```
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct ReleaseWorkflow<'a> {
    /// Analyzer for detecting changes and affected packages
    pub(crate) analyzer: MonorepoAnalyzer<'a>,

    /// Changeset manager for applying production changesets
    pub(crate) changeset_manager: crate::changesets::ChangesetManager<'a>,

    /// Task manager for executing release tasks
    pub(crate) task_manager: crate::tasks::TaskManager<'a>,

    /// Direct reference to configuration
    pub(crate) config: &'a crate::config::MonorepoConfig,

    /// Direct reference to packages
    pub(crate) packages: &'a [crate::core::MonorepoPackageInfo],

    /// Direct reference to git repository
    pub(crate) repository: &'a sublime_git_tools::Repo,

    /// Direct reference to root path
    pub(crate) root_path: &'a std::path::Path,
}
