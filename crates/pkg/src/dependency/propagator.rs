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
    DirectChanges {
        /// List of commit hashes that caused the change
        commits: Vec<String>,
    },
    /// Runtime dependency was updated
    DependencyUpdate {
        /// Name of the updated dependency
        dependency: String,
        /// Previous version of the dependency
        old_version: String,
        /// New version of the dependency
        new_version: String,
    },
    /// Development dependency was updated
    DevDependencyUpdate {
        /// Name of the updated dev dependency
        dependency: String,
        /// Previous version of the dev dependency
        old_version: String,
        /// New version of the dev dependency
        new_version: String,
    },
    /// Optional dependency was updated
    OptionalDependencyUpdate {
        /// Name of the updated optional dependency
        dependency: String,
        /// Previous version of the optional dependency
        old_version: String,
        /// New version of the optional dependency
        new_version: String,
    },
    /// Peer dependency was updated
    PeerDependencyUpdate {
        /// Name of the updated peer dependency
        dependency: String,
        /// Previous version of the peer dependency
        old_version: String,
        /// New version of the peer dependency
        new_version: String,
    },
}
