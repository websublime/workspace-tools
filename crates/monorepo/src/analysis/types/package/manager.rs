//! Package manager analysis types

use serde_json::Value;
use std::path::PathBuf;
use sublime_standard_tools::monorepo::PackageManagerKind;

/// Analysis of the package manager
#[derive(Debug, Clone)]
pub struct PackageManagerAnalysis {
    /// Type of package manager
    pub kind: PackageManagerKind,

    /// Version of the package manager
    pub version: String,

    /// Path to lock file
    pub lock_file: PathBuf,

    /// Configuration files found
    pub config_files: Vec<PathBuf>,

    /// Workspace configuration (raw JSON)
    pub workspaces_config: Value,
}
