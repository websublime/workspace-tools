//! Change detector type definitions

use super::engine::ChangeDetectionEngine;

/// Analyzes changes in the repository to determine affected packages
pub struct ChangeDetector {
    /// Root path of the monorepo
    #[allow(dead_code)]
    pub(crate) root_path: std::path::PathBuf,

    /// Configurable detection engine
    pub(crate) engine: ChangeDetectionEngine,
}

// PackageChange is now imported from super (core.rs) - no duplication
