use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::{
    config::ConventionalConfig,
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
    /// Conventional commit configuration
    pub(crate) conventional_config: ConventionalConfig,
}

impl ConventionalCommitParser {
    /// Creates a new conventional commit parser with default configuration.
    ///
    /// # Errors
    ///
    /// Returns error if regex compilation fails.
    pub fn new() -> PackageResult<Self> {
        Self::with_config(ConventionalConfig::default())
    }

    /// Creates a new conventional commit parser with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Conventional commit configuration
    ///
    /// # Errors
    ///
    /// Returns error if regex compilation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::ConventionalCommitParser;
    /// use sublime_pkg_tools::config::ConventionalConfig;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConventionalConfig::default();
    /// let parser = ConventionalCommitParser::with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_config(config: ConventionalConfig) -> PackageResult<Self> {
        let commit_regex = Regex::new(
            r"^(?P<type>\w+)(?:\((?P<scope>.+)\))?\s*(?P<breaking>!)?\s*:\s*(?P<description>.+)$",
        )
        .map_err(|e| ConventionalCommitError::ParseFailed {
            commit: "regex compilation".to_string(),
            reason: e.to_string(),
        })?;

        // Build breaking change regex from configuration patterns
        let patterns = if config.breaking_change_patterns.is_empty() {
            vec!["BREAKING CHANGE:".to_string(), "BREAKING-CHANGE:".to_string()]
        } else {
            config.breaking_change_patterns.clone()
        };

        let pattern = patterns.iter().map(|p| regex::escape(p)).collect::<Vec<_>>().join("|");
        let breaking_regex_pattern = format!(r"(?i)({})(.+)", pattern);

        let breaking_regex = Regex::new(&breaking_regex_pattern).map_err(|e| {
            ConventionalCommitError::ParseFailed {
                commit: "breaking change regex compilation".to_string(),
                reason: e.to_string(),
            }
        })?;

        // Build type configuration from conventional config
        let mut type_config = HashMap::new();
        for (type_name, commit_type_config) in &config.types {
            let version_bump = match commit_type_config.bump.as_str() {
                "major" => VersionBump::Major,
                "minor" => VersionBump::Minor,
                "patch" => VersionBump::Patch,
                "none" => VersionBump::None,
                _ => {
                    // Use default bump type from config
                    match config.default_bump_type.as_str() {
                        "major" => VersionBump::Major,
                        "minor" => VersionBump::Minor,
                        "patch" => VersionBump::Patch,
                        _ => VersionBump::None,
                    }
                }
            };

            type_config.insert(
                type_name.clone(),
                CommitTypeConfig {
                    version_bump,
                    include_in_changelog: commit_type_config.changelog,
                    changelog_section: commit_type_config.changelog_title.clone(),
                },
            );
        }

        Ok(Self { commit_regex, breaking_regex, type_config, conventional_config: config })
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

        // Check for breaking changes in body/footer if enabled
        let breaking_in_content = if self.conventional_config.parse_breaking_changes {
            body.as_ref().map(|b| self.breaking_regex.is_match(b)).unwrap_or(false)
        } else {
            false
        };

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

        let type_str = match commit_type {
            CommitType::Other(s) => s.as_str(),
            _ => commit_type.as_str(),
        };

        self.type_config.get(type_str).map(|config| config.version_bump).unwrap_or_else(|| {
            // Use default bump type from config
            match self.conventional_config.default_bump_type.as_str() {
                "major" => VersionBump::Major,
                "minor" => VersionBump::Minor,
                "patch" => VersionBump::Patch,
                _ => VersionBump::None,
            }
        })
    }

    /// Checks if a commit type should be included in changelog.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The commit type to check
    #[must_use]
    pub fn should_include_in_changelog(&self, commit_type: &CommitType) -> bool {
        let type_str = match commit_type {
            CommitType::Other(s) => s.as_str(),
            _ => commit_type.as_str(),
        };

        self.type_config.get(type_str).map(|config| config.include_in_changelog).unwrap_or(false)
    }

    /// Gets the changelog section title for a commit type.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The commit type to get the section title for
    ///
    /// # Returns
    ///
    /// Optional section title for changelog organization.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::{ConventionalCommitParser, CommitType};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let parser = ConventionalCommitParser::new()?;
    /// let title = parser.get_changelog_section(&CommitType::Feat);
    /// assert_eq!(title, Some("Features"));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_changelog_section(&self, commit_type: &CommitType) -> Option<&str> {
        let type_str = match commit_type {
            CommitType::Other(s) => s.as_str(),
            _ => commit_type.as_str(),
        };

        self.type_config.get(type_str).and_then(|config| config.changelog_section.as_deref())
    }

    /// Checks if conventional commits are required by configuration.
    ///
    /// # Returns
    ///
    /// True if all commits must follow conventional format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::ConventionalCommitParser;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let parser = ConventionalCommitParser::new()?;
    /// let required = parser.are_conventional_commits_required();
    /// println!("Conventional commits required: {}", required);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn are_conventional_commits_required(&self) -> bool {
        self.conventional_config.require_conventional_commits
    }
}
