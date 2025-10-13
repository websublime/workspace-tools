use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::conventional::ConventionalCommit;

/// Individual changelog entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    /// Entry type (feat, fix, etc.)
    pub entry_type: String,
    /// Scope of the change
    pub scope: Option<String>,
    /// Description of the change
    pub description: String,
    /// Whether this is a breaking change
    pub breaking: bool,
    /// Commit hash
    pub commit_hash: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
}

impl ChangelogEntry {
    /// Creates a new changelog entry from a conventional commit.
    ///
    /// # Arguments
    ///
    /// * `commit` - The conventional commit to convert
    /// * `include_hash` - Whether to include the commit hash
    /// * `include_author` - Whether to include author information
    #[must_use]
    pub fn from_commit(
        commit: &ConventionalCommit,
        include_hash: bool,
        include_author: bool,
    ) -> Self {
        Self {
            entry_type: commit.commit_type.as_str().to_string(),
            scope: commit.scope.clone(),
            description: commit.description.clone(),
            breaking: commit.breaking,
            commit_hash: if include_hash { Some(commit.hash.clone()) } else { None },
            author: if include_author { Some(commit.author.clone()) } else { None },
            timestamp: commit.date,
        }
    }

    /// Formats the entry as a Markdown list item.
    #[must_use]
    pub fn format_markdown(&self) -> String {
        let mut result = String::new();

        // Add breaking change indicator
        if self.breaking {
            result.push_str("**BREAKING:** ");
        }

        // Add scope if present
        if let Some(scope) = &self.scope {
            result.push_str(&format!("**{}:** ", scope));
        }

        // Add description
        result.push_str(&self.description);

        // Add commit hash if present
        if let Some(hash) = &self.commit_hash {
            result.push_str(&format!(" ({})", &hash[..7.min(hash.len())]));
        }

        // Add author if present
        if let Some(author) = &self.author {
            result.push_str(&format!(" by {}", author));
        }

        result
    }
}
