//! Changelog types and structures
//!
//! Defines types for conventional commit parsing, changelog generation,
//! and template processing for automatic changelog creation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A parsed conventional commit
///
/// Represents a commit message parsed according to conventional commit standards.
/// Used for generating structured changelogs based on commit history.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::changelog::ConventionalCommit;
///
/// let commit = ConventionalCommit {
///     commit_type: "feat".to_string(),
///     scope: Some("auth".to_string()),
///     description: "add OAuth2 support".to_string(),
///     body: None,
///     breaking_change: false,
///     hash: "abc123".to_string(),
///     author: "Jane Doe".to_string(),
///     date: "2024-01-15".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionalCommit {
    /// Commit type (feat, fix, docs, etc.)
    pub commit_type: String,
    /// Optional scope (component/module affected)
    pub scope: Option<String>,
    /// Commit description
    pub description: String,
    /// Optional commit body
    pub body: Option<String>,
    /// Whether this is a breaking change
    pub breaking_change: bool,
    /// Git commit hash
    pub hash: String,
    /// Commit author
    pub author: String,
    /// Commit date
    pub date: String,
}

/// Changelog generation result
///
/// Contains the generated changelog content and metadata about the generation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogResult {
    /// Generated changelog content
    pub content: String,
    /// Package name this changelog is for
    pub package_name: String,
    /// Version this changelog covers
    pub version: String,
    /// Number of commits included
    pub commit_count: usize,
    /// Whether breaking changes were found
    pub has_breaking_changes: bool,
    /// Path where changelog was written (if written to file)
    pub output_path: Option<String>,
}

/// Changelog generation request
///
/// Specifies parameters for generating a changelog for a specific package or the entire monorepo.
#[derive(Debug, Clone)]
pub struct ChangelogRequest {
    /// Package name (None for monorepo-wide changelog)
    pub package_name: Option<String>,
    /// Version to generate changelog for
    pub version: String,
    /// Git reference to start from (e.g., last tag)
    pub since: Option<String>,
    /// Git reference to end at (e.g., HEAD)
    pub until: Option<String>,
    /// Whether to include all commit types or only notable ones
    pub include_all_commits: bool,
    /// Whether to write to file or just return content
    pub write_to_file: bool,
    /// Custom output path (defaults to package directory)
    pub output_path: Option<String>,
}

impl Default for ChangelogRequest {
    fn default() -> Self {
        Self {
            package_name: None,
            version: "Unreleased".to_string(),
            since: None,
            until: Some("HEAD".to_string()),
            include_all_commits: false,
            write_to_file: true,
            output_path: None,
        }
    }
}

/// Grouped commits for changelog generation
///
/// Organizes commits by type, scope, or other criteria for structured changelog output.
#[derive(Debug, Clone)]
pub struct GroupedCommits {
    /// Commits grouped by type (feat, fix, etc.)
    pub by_type: HashMap<String, Vec<ConventionalCommit>>,
    /// Commits grouped by scope
    pub by_scope: HashMap<String, Vec<ConventionalCommit>>,
    /// Breaking changes (special group)
    pub breaking_changes: Vec<ConventionalCommit>,
    /// All commits in chronological order
    pub all_commits: Vec<ConventionalCommit>,
}

impl GroupedCommits {
    /// Create new empty grouped commits
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_type: HashMap::new(),
            by_scope: HashMap::new(),
            breaking_changes: Vec::new(),
            all_commits: Vec::new(),
        }
    }

    /// Add a commit to the appropriate groups
    pub fn add_commit(&mut self, commit: ConventionalCommit) {
        // Add to breaking changes if applicable
        if commit.breaking_change {
            self.breaking_changes.push(commit.clone());
        }

        // Add to type group
        self.by_type.entry(commit.commit_type.clone()).or_default().push(commit.clone());

        // Add to scope group if scope exists
        if let Some(scope) = &commit.scope {
            self.by_scope.entry(scope.clone()).or_default().push(commit.clone());
        }

        // Add to all commits
        self.all_commits.push(commit);
    }

    /// Get commits by type with fallback to empty vec
    #[must_use]
    pub fn get_by_type(&self, commit_type: &str) -> &Vec<ConventionalCommit> {
        static EMPTY_VEC: Vec<ConventionalCommit> = Vec::new();
        self.by_type.get(commit_type).unwrap_or(&EMPTY_VEC)
    }

    /// Get commits by scope with fallback to empty vec
    #[must_use]
    pub fn get_by_scope(&self, scope: &str) -> &Vec<ConventionalCommit> {
        static EMPTY_VEC: Vec<ConventionalCommit> = Vec::new();
        self.by_scope.get(scope).unwrap_or(&EMPTY_VEC)
    }

    /// Get total number of commits
    #[must_use]
    pub fn total_commits(&self) -> usize {
        self.all_commits.len()
    }

    /// Check if there are breaking changes
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        !self.breaking_changes.is_empty()
    }
}

impl Default for GroupedCommits {
    fn default() -> Self {
        Self::new()
    }
}

/// Template variables for changelog generation
///
/// Variables that can be used in changelog templates for customization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariables {
    /// Package name
    pub package_name: String,
    /// Version being released
    pub version: String,
    /// Release date
    pub date: String,
    /// Git repository URL
    pub repository_url: Option<String>,
    /// Compare URL for this version
    pub compare_url: Option<String>,
    /// Previous version for comparison
    pub previous_version: Option<String>,
}

impl TemplateVariables {
    /// Create new template variables
    #[must_use]
    pub fn new(package_name: String, version: String) -> Self {
        Self {
            package_name,
            version,
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            repository_url: None,
            compare_url: None,
            previous_version: None,
        }
    }

    /// Set repository URL for generating links
    #[must_use]
    pub fn with_repository_url(mut self, url: String) -> Self {
        self.repository_url = Some(url);
        self
    }

    /// Set previous version for comparison links
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn with_previous_version(mut self, version: String) -> Self {
        self.previous_version = Some(version.clone());

        // Generate compare URL if repository URL is available
        if let Some(repo_url) = &self.repository_url {
            self.compare_url = Some(format!(
                "{repo_url}/compare/{version}...{current_version}",
                current_version = self.version
            ));
        }

        self
    }
}
