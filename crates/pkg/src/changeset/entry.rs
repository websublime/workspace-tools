use serde::{Deserialize, Serialize};

use crate::{changeset::ChangeReason, ResolvedVersion, VersionBump};

/// Package-specific changes within a changeset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetPackage {
    /// Package name
    pub name: String,
    /// Version bump type
    pub bump: VersionBump,
    /// Current version
    pub current_version: ResolvedVersion,
    /// Next version after bump
    pub next_version: ResolvedVersion,
    /// Reason for the change
    pub reason: ChangeReason,
    /// Optional dependency that triggered this change
    pub dependency: Option<String>,
    /// Individual change entries
    pub changes: Vec<ChangeEntry>,
}

/// Individual change entry within a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    /// Type of change (feat, fix, etc.)
    pub change_type: String,
    /// Description of the change
    pub description: String,
    /// Whether this is a breaking change
    pub breaking: bool,
    /// Associated commit hash
    pub commit: Option<String>,
}
