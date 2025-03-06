//! Status of dependency upgrades.

use std::fmt;

/// Status of a dependency in relation to available updates
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeStatus {
    /// Dependency is up to date
    UpToDate,
    /// Patch update available (0.0.x)
    PatchAvailable(String),
    /// Minor update available (0.x.0)
    MinorAvailable(String),
    /// Major update available (x.0.0)
    MajorAvailable(String),
    /// Version requirements don't allow update
    Constrained(String),
    /// Failed to check for updates
    CheckFailed(String),
}

impl Default for UpgradeStatus {
    fn default() -> Self {
        Self::UpToDate
    }
}

impl fmt::Display for UpgradeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpToDate => write!(f, "up to date"),
            Self::PatchAvailable(v) => write!(f, "patch available: {v}"),
            Self::MinorAvailable(v) => write!(f, "minor update available: {v}"),
            Self::MajorAvailable(v) => write!(f, "major update available: {v}"),
            Self::Constrained(v) => write!(f, "constrained (latest: {v})"),
            Self::CheckFailed(msg) => write!(f, "check failed: {msg}"),
        }
    }
}
