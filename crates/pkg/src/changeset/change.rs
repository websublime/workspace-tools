use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::changeset::{entry::ChangesetPackage, release::ReleaseInfo};

/// Reason for a package change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeReason {
    /// Direct changes to the package
    DirectChanges { commits: Vec<String> },
    /// Dependency update propagation
    DependencyUpdate { dependency: String, old_version: String, new_version: String },
    /// Dev dependency update propagation
    DevDependencyUpdate { dependency: String, old_version: String, new_version: String },
}

/// Core changeset data structure.
///
/// Represents a set of changes to be applied across multiple packages,
/// including version bumps and release targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    /// Branch where changes originated
    pub branch: String,
    /// When changeset was created
    pub created_at: DateTime<Utc>,
    /// Author of the changeset
    pub author: String,
    /// Target environments for release
    pub releases: Vec<String>,
    /// Package changes included in this changeset
    pub packages: Vec<ChangesetPackage>,
    /// Release information (populated when applied)
    pub release_info: Option<ReleaseInfo>,
}

impl Default for Changeset {
    fn default() -> Self {
        Self {
            branch: String::new(),
            created_at: Utc::now(),
            author: String::new(),
            releases: vec!["dev".to_string()],
            packages: Vec::new(),
            release_info: None,
        }
    }
}
