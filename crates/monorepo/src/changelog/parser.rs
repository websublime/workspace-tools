//! Conventional commit parser
//!
//! Parses Git commit messages according to conventional commit standards
//! for structured changelog generation.

use super::types::ConventionalCommit;
use crate::error::{Error, Result};
use regex::Regex;
use std::sync::OnceLock;

/// Conventional commit parser
///
/// Parses Git commit messages according to the conventional commit specification.
/// Supports parsing commit type, scope, description, body, and breaking change indicators.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::changelog::ConventionalCommitParser;
///
/// let parser = ConventionalCommitParser::new();
///
/// // Parse a feature commit with scope
/// let commit = parser.parse_commit_message(
///     "feat(auth): add OAuth2 support\n\nImplements OAuth2 authentication flow",
///     "abc123",
///     "Jane Doe",
///     "2024-01-15"
/// )?;
///
/// assert_eq!(commit.commit_type, "feat");
/// assert_eq!(commit.scope, Some("auth".to_string()));
/// assert_eq!(commit.description, "add OAuth2 support");
/// ```
pub struct ConventionalCommitParser;

impl ConventionalCommitParser {
    /// Create a new conventional commit parser
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Parse a commit message into a ConventionalCommit
    ///
    /// # Arguments
    ///
    /// * `message` - The full commit message
    /// * `hash` - The Git commit hash
    /// * `author` - The commit author
    /// * `date` - The commit date
    ///
    /// # Returns
    ///
    /// A parsed ConventionalCommit or an error if parsing fails
    pub fn parse_commit_message(
        &self,
        message: &str,
        hash: &str,
        author: &str,
        date: &str,
    ) -> Result<ConventionalCommit> {
        let lines: Vec<&str> = message.lines().collect();
        if lines.is_empty() {
            return Err(Error::changelog("Empty commit message".to_string()));
        }

        let header = lines[0];
        let body =
            if lines.len() > 2 { Some(lines[2..].join("\n").trim().to_string()) } else { None };

        // Parse the header using regex
        let (commit_type, scope, description, breaking_from_header) = self.parse_header(header)?;

        // Check for breaking changes in body
        let breaking_from_body = body.as_ref().map_or(false, |b| b.contains("BREAKING CHANGE:"));

        let breaking_change = breaking_from_header || breaking_from_body;

        Ok(ConventionalCommit {
            commit_type,
            scope,
            description,
            body,
            breaking_change,
            hash: hash.to_string(),
            author: author.to_string(),
            date: date.to_string(),
        })
    }

    /// Parse the commit header (first line)
    ///
    /// # Arguments
    ///
    /// * `header` - The commit header line
    ///
    /// # Returns
    ///
    /// A tuple of (type, scope, description, breaking_change)
    #[allow(clippy::unused_self)]
    #[allow(clippy::expect_used)]
    fn parse_header(&self, header: &str) -> Result<(String, Option<String>, String, bool)> {
        static CONVENTIONAL_REGEX: OnceLock<Regex> = OnceLock::new();

        let regex = CONVENTIONAL_REGEX.get_or_init(|| {
            // Regex pattern for conventional commits:
            // type(scope)!: description
            // type!: description
            // type(scope): description
            // type: description
            Regex::new(
                r"^(?P<type>\w+)(?:\((?P<scope>[^)]+)\))?(?P<breaking>!)?:\s*(?P<description>.+)$",
            )
            .expect("Invalid regex pattern")
        });

        if let Some(captures) = regex.captures(header) {
            let commit_type = captures
                .name("type")
                .map(|m| m.as_str().to_string())
                .ok_or_else(|| Error::changelog("Missing commit type".to_string()))?;

            let scope = captures.name("scope").map(|m| m.as_str().to_string());

            let description =
                captures
                    .name("description")
                    .map(|m| m.as_str().trim().to_string())
                    .ok_or_else(|| Error::changelog("Missing commit description".to_string()))?;

            let breaking_change = captures.name("breaking").is_some();

            Ok((commit_type, scope, description, breaking_change))
        } else {
            // Fallback for non-conventional commits
            // Treat the entire header as description with "chore" type
            Ok(("chore".to_string(), None, header.trim().to_string(), false))
        }
    }

    /// Check if a commit type should be included in changelog
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The commit type to check
    /// * `include_all` - Whether to include all commit types
    ///
    /// # Returns
    ///
    /// True if the commit type should be included
    #[must_use]
    pub fn should_include_commit(&self, commit_type: &str, include_all: bool) -> bool {
        if include_all {
            return true;
        }

        // Include only notable commit types by default
        matches!(commit_type, "feat" | "fix" | "perf" | "refactor" | "revert" | "breaking")
    }

    /// Get display name for commit type
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The commit type
    ///
    /// # Returns
    ///
    /// A human-readable display name for the commit type
    #[must_use]
    pub fn get_type_display_name(&self, commit_type: &str) -> &'static str {
        match commit_type {
            "feat" => "Features",
            "fix" => "Bug Fixes",
            "perf" => "Performance Improvements",
            "refactor" => "Code Refactoring",
            "revert" => "Reverts",
            "docs" => "Documentation",
            "style" => "Styles",
            "test" => "Tests",
            "build" => "Build System",
            "ci" => "Continuous Integration",
            "chore" => "Chores",
            "breaking" => "BREAKING CHANGES",
            _ => "Other Changes",
        }
    }

    /// Parse multiple commits from git log output
    ///
    /// # Arguments
    ///
    /// * `commits` - Vector of repository commits from git crate
    ///
    /// # Returns
    ///
    /// Vector of parsed conventional commits
    pub fn parse_commits(
        &self,
        commits: &[sublime_git_tools::RepoCommit],
    ) -> Result<Vec<ConventionalCommit>> {
        let mut parsed_commits = Vec::new();

        for commit in commits {
            match self.parse_commit_message(
                &commit.message,
                &commit.hash,
                &commit.author_name,
                &commit.author_date,
            ) {
                Ok(conventional_commit) => {
                    parsed_commits.push(conventional_commit);
                }
                Err(e) => {
                    log::warn!("Failed to parse commit {}: {}. Skipping.", commit.hash, e);
                    // Continue with other commits instead of failing entirely
                }
            }
        }

        Ok(parsed_commits)
    }

    /// Filter commits for a specific package
    ///
    /// # Arguments
    ///
    /// * `commits` - All commits to filter
    /// * `package_path` - Relative path to the package
    /// * `changed_files` - Files changed in each commit (optional optimization)
    ///
    /// # Returns
    ///
    /// Vector of commits that affect the specified package
    #[must_use]
    pub fn filter_commits_for_package(
        &self,
        commits: &[ConventionalCommit],
        package_path: &str,
        changed_files: Option<&std::collections::HashMap<String, Vec<String>>>,
    ) -> Vec<ConventionalCommit> {
        commits
            .iter()
            .filter(|commit| {
                // If we have changed files data, use it for accurate filtering
                if let Some(files_map) = changed_files {
                    if let Some(files) = files_map.get(&commit.hash) {
                        return files.iter().any(|file| file.starts_with(package_path));
                    }
                }

                // Fallback: check if commit message mentions the package
                // This is less accurate but better than including all commits
                let package_name = package_path.split('/').last().unwrap_or(package_path);

                commit.description.contains(package_name)
                    || commit.scope.as_ref().map_or(false, |scope| scope == package_name)
                    || commit.body.as_ref().map_or(false, |body| body.contains(package_name))
            })
            .cloned()
            .collect()
    }
}

impl Default for ConventionalCommitParser {
    fn default() -> Self {
        Self::new()
    }
}
