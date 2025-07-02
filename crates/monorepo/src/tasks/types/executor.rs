//! Task executor type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use crate::config::MonorepoConfig;
use crate::core::MonorepoPackageInfo;
use std::path::Path;

/// Executor for running tasks with various scopes and configurations
///
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub(crate) struct TaskExecutor<'a> {
    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],

    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,

    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
}
