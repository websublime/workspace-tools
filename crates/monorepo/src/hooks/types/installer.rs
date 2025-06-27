//! Hook installer type definitions

use crate::core::{GitProvider, FileSystemProvider};
use std::path::PathBuf;

/// Installer for Git hooks that manages hook files and permissions
pub struct HookInstaller {
    /// Git operations provider
    pub(crate) git_provider: Box<dyn GitProvider>,

    /// File system operations provider
    pub(crate) file_system_provider: Box<dyn FileSystemProvider>,

    /// Path to the Git hooks directory
    pub(crate) hooks_dir: PathBuf,

    /// Template for the hook script wrapper
    pub(crate) hook_template: String,
}