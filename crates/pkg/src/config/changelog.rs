use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Changelog generation configuration.
///
/// Controls changelog formatting, content, and generation behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogConfig {
    /// Whether to include commit hash in changelog entries
    pub include_commit_hash: bool,

    /// Whether to include author information
    pub include_authors: bool,

    /// Whether to group changes by commit type
    pub group_by_type: bool,

    /// Whether to include release date
    pub include_date: bool,

    /// Maximum number of commits to include per release
    pub max_commits_per_release: Option<u32>,

    /// Template file for changelog generation
    pub template_file: Option<PathBuf>,

    /// Custom sections for changelog
    pub custom_sections: HashMap<String, String>,

    /// Whether to link to commits in remote repository
    pub link_commits: bool,

    /// Base URL for commit links
    pub commit_url_format: Option<String>,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            include_commit_hash: true,
            include_authors: true,
            group_by_type: true,
            include_date: true,
            max_commits_per_release: Some(1000),
            template_file: None,
            custom_sections: HashMap::new(),
            link_commits: false,
            commit_url_format: None,
        }
    }
}
