//! Hook installer type definitions

use crate::core::MonorepoProject;
use std::path::PathBuf;
use std::sync::Arc;

/// Installer for Git hooks that manages hook files and permissions
pub struct HookInstaller {
    /// Reference to the monorepo project
    #[allow(dead_code)] // Will be used when full integration is implemented
    pub(crate) project: Arc<MonorepoProject>,

    /// Path to the Git hooks directory
    pub(crate) hooks_dir: PathBuf,

    /// Template for the hook script wrapper
    pub(crate) hook_template: String,
}