use serde::{Deserialize, Serialize};

/// Version management configuration.
///
/// Controls version resolution, snapshot generation, and version bump strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    /// Format for snapshot versions (default: "{version}-{commit}.snapshot")
    pub snapshot_format: String,

    /// Length of commit hash in snapshot versions (default: 7)
    pub commit_hash_length: u8,

    /// Whether to allow snapshot versions on main branch (default: false)
    pub allow_snapshot_on_main: bool,

    /// Pre-release identifier format
    pub prerelease_format: Option<String>,

    /// Build metadata format
    pub build_metadata_format: Option<String>,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            snapshot_format: "{version}-{commit}.snapshot".to_string(),
            commit_hash_length: 7,
            allow_snapshot_on_main: false,
            prerelease_format: None,
            build_metadata_format: None,
        }
    }
}
