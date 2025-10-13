use serde::{Deserialize, Serialize};

use crate::{ResolvedVersion, VersionBump};

/// Information about a propagated dependency update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagatedUpdate {
    /// Package name that needs updating
    pub package_name: String,
    /// Reason for the update
    pub reason: PropagationReason,
    /// Suggested version bump
    pub suggested_bump: VersionBump,
    /// Current version of the package
    pub current_version: ResolvedVersion,
    /// Calculated next version
    pub next_version: ResolvedVersion,
}

/// Reason for a propagated dependency update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropagationReason {
    /// Direct changes to the package
    DirectChanges { commits: Vec<String> },
    /// Runtime dependency was updated
    DependencyUpdate { dependency: String, old_version: String, new_version: String },
    /// Development dependency was updated
    DevDependencyUpdate { dependency: String, old_version: String, new_version: String },
    /// Optional dependency was updated
    OptionalDependencyUpdate { dependency: String, old_version: String, new_version: String },
    /// Peer dependency was updated
    PeerDependencyUpdate { dependency: String, old_version: String, new_version: String },
}
