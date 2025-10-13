use serde::{Deserialize, Serialize};

/// Release management configuration.
///
/// Controls release planning, execution, and post-release operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    /// Release strategy (independent/unified)
    pub strategy: String,

    /// Git tag format (default: "{package}@{version}")
    pub tag_format: String,

    /// Environment-specific tag format
    pub env_tag_format: String,

    /// Whether to create Git tags during release
    pub create_tags: bool,

    /// Whether to push tags to remote
    pub push_tags: bool,

    /// Whether to create changelog during release
    pub create_changelog: bool,

    /// Changelog file name pattern
    pub changelog_file: String,

    /// Commit message format for releases
    pub commit_message: String,

    /// Whether dry-run is enabled by default
    pub dry_run_by_default: bool,

    /// Maximum concurrent package releases
    pub max_concurrent_releases: u32,

    /// Release timeout in seconds
    pub release_timeout: u64,
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            strategy: "independent".to_string(),
            tag_format: "{package}@{version}".to_string(),
            env_tag_format: "{package}@{version}-{environment}".to_string(),
            create_tags: true,
            push_tags: true,
            create_changelog: true,
            changelog_file: "CHANGELOG.md".to_string(),
            commit_message: "chore(release): {package}@{version}".to_string(),
            dry_run_by_default: false,
            max_concurrent_releases: 5,
            release_timeout: 300,
        }
    }
}
