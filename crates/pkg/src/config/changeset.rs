use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Changeset management configuration.
///
/// Controls how changesets are created, stored, and managed throughout
/// the development lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetConfig {
    /// Path where changesets are stored (default: ".changesets")
    pub path: PathBuf,

    /// Path where changeset history is stored (default: ".changesets/history")
    pub history_path: PathBuf,

    /// Available environments for releases
    pub available_environments: Vec<String>,

    /// Default environments when creating changesets
    pub default_environments: Vec<String>,

    /// Format for changeset filenames (default: "{branch}-{datetime}.json")
    pub filename_format: String,

    /// Maximum number of pending changesets to keep
    pub max_pending_changesets: Option<u32>,

    /// Whether to auto-archive applied changesets
    pub auto_archive_applied: bool,
}

impl Default for ChangesetConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from(".changesets"),
            history_path: PathBuf::from(".changesets/history"),
            available_environments: vec![
                "dev".to_string(),
                "test".to_string(),
                "qa".to_string(),
                "staging".to_string(),
                "prod".to_string(),
            ],
            default_environments: vec!["dev".to_string()],
            filename_format: "{branch}-{datetime}.json".to_string(),
            max_pending_changesets: Some(100),
            auto_archive_applied: true,
        }
    }
}
