//! Hook installer type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use std::path::PathBuf;

/// Installer for Git hooks that manages hook files and permissions
///
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct HookInstaller {
    /// Path to the Git hooks directory
    pub(crate) hooks_dir: PathBuf,

    /// Template for the hook script wrapper
    pub(crate) hook_template: String,
}
