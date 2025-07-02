//! Changeset hook integration workflow type definitions

use crate::changesets::ChangesetManager;

/// Handles integration between changesets and Git hooks
///
/// This workflow manages the seamless integration of changeset validation
/// with Git hooks, ensuring that changesets are properly validated during
/// Git operations like commits and pushes.
///
/// # Features
///
/// - Pre-commit validation of changeset requirements
/// - Pre-push validation of changeset application
/// - Automatic changeset dependency validation
/// - Integration with existing Git workflow
///
/// This workflow ensures that changesets are properly validated during
/// Git operations and provides seamless integration between the changeset
/// system and Git hooks.
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct ChangesetHookIntegration<'a> {
    /// Changeset manager for changeset operations
    pub(crate) changeset_manager: ChangesetManager<'a>,

    /// Task manager for validation tasks
    pub(crate) task_manager: crate::tasks::TaskManager<'a>,

    /// Plugin manager for extensible integration functionality
    pub(crate) plugin_manager: crate::plugins::PluginManager<'a>,

    /// Direct reference to configuration
    pub(crate) config: &'a crate::config::MonorepoConfig,

    /// Direct reference to packages
    pub(crate) packages: &'a [crate::core::MonorepoPackageInfo],

    /// Direct reference to git repository
    pub(crate) repository: &'a sublime_git_tools::Repo,

    /// Direct reference to file system manager
    pub(crate) file_system: &'a sublime_standard_tools::filesystem::FileSystemManager,

    /// Direct reference to root path
    pub(crate) root_path: &'a std::path::Path,
}
