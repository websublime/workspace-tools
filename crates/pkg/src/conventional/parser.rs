use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::{
    conventional::commit::{CommitType, CommitTypeConfig, ConventionalCommit},
    error::ConventionalCommitError,
    PackageResult, VersionBump,
};

/// Service for parsing conventional commit messages.
#[derive(Debug, Clone)]
pub struct ConventionalCommitParser {
    /// Regex for parsing conventional commit format
    pub(crate) commit_regex: Regex,
    /// Regex for detecting breaking changes
    pub(crate) breaking_regex: Regex,
    /// Configuration for commit type mappings
    pub(crate) type_config: HashMap<String, CommitTypeConfig>,
}

impl ConventionalCommitParser {
    /// Creates a new conventional commit parser with default configuration.
    ///
    /// # Errors
    ///
    /// Returns error if regex compilation fails.
    pub fn new() -> PackageResult<Self> {
        let commit_regex = Regex::new(
            r"^(?P<type>\w+)(?:\((?P<scope>.+)\))?\s*(?P<breaking>!)?\s*:\s*(?P<description>.+)$",
        )
        .map_err(|e| ConventionalCommitError::ParseFailed {
            commit: "regex compilation".to_string(),
            reason: e.to_string(),
        })?;

        let breaking_regex = Regex::new(r"(?i)breaking\s+change\s*:\s*(.+)").map_err(|e| {
            ConventionalCommitError::ParseFailed {
                commit: "breaking change regex compilation".to_string(),
                reason: e.to_string(),
            }
        })?;

        let mut type_config = HashMap::new();
        type_config.insert(
            "feat".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::Minor,
                include_in_changelog: true,
                changelog_section: Some("Features".to_string()),
            },
        );
        type_config.insert(
            "fix".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::Patch,
                include_in_changelog: true,
                changelog_section: Some("Bug Fixes".to_string()),
            },
        );
        type_config.insert(
            "perf".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::Patch,
                include_in_changelog: true,
                changelog_section: Some("Performance Improvements".to_string()),
            },
        );
        type_config.insert(
            "docs".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::None,
                include_in_changelog: false,
                changelog_section: None,
            },
        );
        type_config.insert(
            "style".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::None,
                include_in_changelog: false,
                changelog_section: None,
            },
        );
        type_config.insert(
            "refactor".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::None,
                include_in_changelog: false,
                changelog_section: None,
            },
        );
        type_config.insert(
            "test".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::None,
                include_in_changelog: false,
                changelog_section: None,
            },
        );
        type_config.insert(
            "build".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::None,
                include_in_changelog: false,
                changelog_section: None,
            },
        );
        type_config.insert(
            "ci".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::None,
                include_in_changelog: false,
                changelog_section: None,
            },
        );
        type_config.insert(
            "chore".to_string(),
            CommitTypeConfig {
                version_bump: VersionBump::None,
                include_in_changelog: false,
                changelog_section: None,
            },
        );

        Ok(Self { commit_regex, breaking_regex, type_config })
    }

    /// Parses a commit message into a conventional commit.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message to parse
    /// * `hash` - Git commit hash
    /// * `author` - Commit author
    /// * `date` - Commit timestamp
    ///
    /// # Errors
    ///
    /// Returns error if commit message doesn't follow conventional format.
    pub fn parse(
        &self,
        message: &str,
        hash: String,
        author: String,
        date: DateTime<Utc>,
    ) -> PackageResult<ConventionalCommit> {
        let lines: Vec<&str> = message.lines().collect();
        if lines.is_empty() {
            return Err(ConventionalCommitError::InvalidFormat {
                commit: message.to_string(),
                reason: "Empty commit message".to_string(),
            }
            .into());
        }

        let first_line = lines[0];
        let captures = self.commit_regex.captures(first_line).ok_or_else(|| {
            ConventionalCommitError::InvalidFormat {
                commit: message.to_string(),
                reason: "Does not match conventional commit format".to_string(),
            }
        })?;

        let commit_type_str = captures
            .name("type")
            .ok_or_else(|| ConventionalCommitError::ParseFailed {
                commit: message.to_string(),
                reason: "Missing commit type".to_string(),
            })?
            .as_str();

        let commit_type = CommitType::from_str(commit_type_str).map_err(|e| {
            ConventionalCommitError::ParseFailed {
                commit: message.to_string(),
                reason: format!("Failed to parse commit type '{}': {}", commit_type_str, e),
            }
        })?;

        let scope = captures.name("scope").map(|m| m.as_str().to_string());
        let breaking_marker = captures.name("breaking").is_some();
        let description = captures
            .name("description")
            .ok_or_else(|| ConventionalCommitError::ParseFailed {
                commit: message.to_string(),
                reason: "Missing commit description".to_string(),
            })?
            .as_str()
            .to_string();

        // Parse body and footer
        let body = if lines.len() > 2 { Some(lines[2..].join("\n")) } else { None };

        // Check for breaking changes in body/footer
        let breaking_in_content =
            body.as_ref().map(|b| self.breaking_regex.is_match(b)).unwrap_or(false);

        let breaking = breaking_marker || breaking_in_content;

        Ok(ConventionalCommit {
            commit_type,
            scope,
            breaking,
            description,
            body,
            footer: None, // TODO: Parse footer properly
            hash,
            author,
            date,
        })
    }

    /// Gets the version bump for a commit type.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The commit type to check
    /// * `is_breaking` - Whether this is a breaking change
    #[must_use]
    pub fn get_version_bump(&self, commit_type: &CommitType, is_breaking: bool) -> VersionBump {
        if is_breaking {
            return VersionBump::Major;
        }

        match commit_type {
            CommitType::Other(type_str) => self
                .type_config
                .get(type_str)
                .map(|config| config.version_bump)
                .unwrap_or(VersionBump::None),
            _ => {
                let type_str = commit_type.as_str();
                self.type_config
                    .get(type_str)
                    .map(|config| config.version_bump)
                    .unwrap_or(VersionBump::None)
            }
        }
    }

    /// Checks if a commit type should be included in changelog.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The commit type to check
    #[must_use]
    pub fn should_include_in_changelog(&self, commit_type: &CommitType) -> bool {
        let type_str = match commit_type {
            CommitType::Other(s) => s,
            _ => commit_type.as_str(),
        };

        self.type_config.get(type_str).map(|config| config.include_in_changelog).unwrap_or(false)
    }
}
