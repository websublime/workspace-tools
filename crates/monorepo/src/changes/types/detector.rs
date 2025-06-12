//! Change detector type definitions

use super::{PackageChangeType, ChangeSignificance, VersionBumpType};
use super::engine::ChangeDetectionEngine;
use sublime_git_tools::GitChangedFile;

/// Analyzes changes in the repository to determine affected packages
pub struct ChangeDetector {
    /// Root path of the monorepo
    pub(crate) root_path: std::path::PathBuf,
    
    /// Configurable detection engine
    pub(crate) engine: ChangeDetectionEngine,
}

/// Represents a change to a package
#[derive(Debug, Clone)]
pub struct PackageChange {
    /// Name of the affected package
    pub package_name: String,

    /// Files changed in this package
    pub changed_files: Vec<GitChangedFile>,

    /// Type of changes
    pub change_type: PackageChangeType,

    /// Significance of the changes
    pub significance: ChangeSignificance,

    /// Suggested version bump
    pub suggested_version_bump: VersionBumpType,
}