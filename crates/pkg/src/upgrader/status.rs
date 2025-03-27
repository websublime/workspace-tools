//! Status of dependency upgrades.

use std::fmt;

use serde::{Deserialize, Serialize};

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

/// Represents an available upgrade for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableUpgrade {
    /// Package name containing the dependency
    pub package_name: String,
    /// Dependency name
    pub dependency_name: String,
    /// Current version of the dependency
    pub current_version: String,
    /// Latest available version that's compatible with requirements
    pub compatible_version: Option<String>,
    /// Latest overall version (may not be compatible with current requirements)
    pub latest_version: Option<String>,
    /// Status of this dependency's upgradability
    #[serde(skip)]
    pub status: UpgradeStatus,
}

impl fmt::Display for AvailableUpgrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} in {}: {} -> {}",
            self.dependency_name,
            self.package_name,
            self.current_version,
            match &self.compatible_version {
                Some(v) => v,
                None => "no compatible version",
            }
        )
    }
}
