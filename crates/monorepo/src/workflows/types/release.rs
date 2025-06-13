//! Release workflow type definitions

use crate::analysis::MonorepoAnalyzer;
use crate::core::MonorepoProject;
use std::sync::Arc;

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
/// let workflow = ReleaseWorkflow::new(project);
///
/// // Run release workflow - applies changesets and bumps versions
/// let result = workflow.run().await?;
/// println!("Released packages: {}", result.released_packages.len());
/// # Ok(())
/// # }
/// ```
pub struct ReleaseWorkflow {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,

    /// Analyzer for detecting changes and affected packages
    pub(crate) analyzer: MonorepoAnalyzer,

    /// Version manager for handling version bumps
    pub(crate) version_manager: crate::core::VersionManager,

    /// Changeset manager for applying production changesets
    pub(crate) changeset_manager: crate::changesets::ChangesetManager,

    /// Task manager for executing release tasks
    pub(crate) task_manager: crate::tasks::TaskManager,
}
