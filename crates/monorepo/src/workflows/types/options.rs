//! Workflow option types
//!
//! Configuration options for different types of workflows.

use serde::{Deserialize, Serialize};

/// Options for release workflow
///
/// Configures how a release should be executed, including which environments
/// to deploy to and what validation steps to perform.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::ReleaseOptions;
///
/// let options = ReleaseOptions {
///     dry_run: false,
///     skip_tests: false,
///     skip_changelogs: false,
///     target_environments: vec!["production".to_string()],
///     force: false,
/// };
/// ```
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseOptions {
    /// Perform a dry run without making actual changes
    pub dry_run: bool,

    /// Skip running tests during release
    pub skip_tests: bool,

    /// Skip generating changelogs
    pub skip_changelogs: bool,

    /// Target environments for deployment
    pub target_environments: Vec<String>,

    /// Force release even if validation fails
    pub force: bool,
}

impl Default for ReleaseOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            skip_tests: false,
            skip_changelogs: false,
            target_environments: vec!["production".to_string()],
            force: false,
        }
    }
}
