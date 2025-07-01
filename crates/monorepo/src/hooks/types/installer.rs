//! Hook installer type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::{Path, PathBuf};

/// Installer for Git hooks that manages hook files and permissions
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct HookInstaller<'a> {
    /// Direct reference to git repository
    pub(crate) repository: &'a Repo,

    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,

    /// Direct reference to root path
    pub(crate) root_path: &'a Path,

    /// Path to the Git hooks directory
    pub(crate) hooks_dir: PathBuf,

    /// Template for the hook script wrapper
    pub(crate) hook_template: String,
}