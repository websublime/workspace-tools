//! Core change detection types

use serde::{Deserialize, Serialize};

/// Type of changes in a package
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageChangeType {
    /// Source code changes
    SourceCode,
    /// Dependency changes
    Dependencies,
    /// Configuration changes
    Configuration,
    /// Documentation changes
    Documentation,
    /// Test changes
    Tests,
}

/// Significance level of changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChangeSignificance {
    /// Low impact changes (docs, tests)
    Low,
    /// Medium impact changes (features, bug fixes)
    Medium,
    /// High impact changes (breaking changes, major features)
    High,
}

impl ChangeSignificance {
    /// Elevate significance to the next level
    #[must_use]
    pub fn elevate(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium | Self::High => Self::High,
        }
    }
}

// Re-export the VersionBumpType from config to avoid duplication
pub use crate::config::VersionBumpType;

/// Information about a package change detected by analysis
#[derive(Debug, Clone)]
pub struct PackageChange {
    /// Name of the changed package
    pub package_name: String,
    /// Type of change detected
    pub change_type: PackageChangeType,
    /// Significance level of the change
    pub significance: ChangeSignificance,
    /// Git changed files in this package (structured change information)
    pub changed_files: Vec<sublime_git_tools::GitChangedFile>,
    /// Suggested version bump based on change analysis
    pub suggested_version_bump: VersionBumpType,
    /// Additional metadata about the change
    pub metadata: std::collections::HashMap<String, String>,
}