use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Release information added when changeset is applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// When the changeset was applied
    pub applied_at: DateTime<Utc>,
    /// Who applied the changeset
    pub applied_by: String,
    /// Git commit where it was applied
    pub git_commit: String,
    /// Environment-specific release information
    pub environments_released: HashMap<String, EnvironmentRelease>,
}

/// Environment-specific release information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentRelease {
    /// When released to this environment
    pub released_at: DateTime<Utc>,
    /// Git tag created for this release
    pub tag: String,
}
