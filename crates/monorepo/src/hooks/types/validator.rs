//! Hook validator type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use super::HookValidationResult;
use crate::changesets::Changeset;
use crate::core::MonorepoPackageInfo;
use crate::config::MonorepoConfig;
use sublime_git_tools::Repo;
use std::path::Path;

/// Validator for hook conditions and requirements
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct HookValidator<'a> {
    /// Direct reference to git repository
    pub(crate) repository: &'a Repo,
    
    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],
    
    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,

    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
}

/// Result of changeset validation
#[derive(Debug, Clone)]
pub struct ChangesetValidationResult {
    /// Whether a changeset exists for the changes
    pub changeset_exists: bool,

    /// The changeset if found
    pub changeset: Option<Changeset>,

    /// Detailed validation information
    pub validation_details: HookValidationResult,
}

impl ChangesetValidationResult {
    /// Create a new changeset validation result
    #[must_use]
    pub fn new() -> Self {
        Self {
            changeset_exists: false,
            changeset: None,
            validation_details: HookValidationResult::new(),
        }
    }

    /// Set changeset exists status
    #[must_use]
    pub fn with_changeset_exists(mut self, exists: bool) -> Self {
        self.changeset_exists = exists;
        self
    }

    /// Set the changeset
    #[must_use]
    pub fn with_changeset(mut self, changeset: Changeset) -> Self {
        self.changeset = Some(changeset);
        self.changeset_exists = true;
        self
    }

    /// Set validation details
    #[must_use]
    pub fn with_validation_details(mut self, details: HookValidationResult) -> Self {
        self.validation_details = details;
        self
    }
}

impl Default for ChangesetValidationResult {
    fn default() -> Self {
        Self::new()
    }
}