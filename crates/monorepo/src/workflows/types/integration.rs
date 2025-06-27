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
pub struct ChangesetHookIntegration {
    /// Changeset manager for changeset operations
    pub(crate) changeset_manager: ChangesetManager,

    /// Hook manager for Git hook operations
    pub(crate) hook_manager: crate::hooks::HookManager,

    /// Task manager for validation tasks
    pub(crate) task_manager: crate::tasks::TaskManager,

    /// Configuration provider for accessing configuration settings
    pub(crate) config_provider: Box<dyn crate::core::ConfigProvider>,

    /// Git provider for repository operations
    pub(crate) git_provider: Box<dyn crate::core::GitProvider>,

    /// File system provider for file operations
    pub(crate) file_system_provider: Box<dyn crate::core::FileSystemProvider>,

    /// Package provider for accessing package information
    pub(crate) package_provider: Box<dyn crate::core::PackageProvider>,
}
