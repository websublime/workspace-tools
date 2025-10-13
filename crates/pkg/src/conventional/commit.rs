use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{error::CommitTypeParseError, VersionBump};

/// Parsed conventional commit representation.
///
/// Contains all semantic information extracted from a conventional
/// commit message according to the specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionalCommit {
    /// Type of commit (feat, fix, etc.)
    pub commit_type: CommitType,
    /// Optional scope of the change
    pub scope: Option<String>,
    /// Whether this is a breaking change
    pub breaking: bool,
    /// Short description of the change
    pub description: String,
    /// Optional longer description
    pub body: Option<String>,
    /// Optional footer information
    pub footer: Option<String>,
    /// Git commit hash
    pub hash: String,
    /// Commit author
    pub author: String,
    /// Commit timestamp
    pub date: DateTime<Utc>,
}

/// Standard conventional commit types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommitType {
    /// New feature
    Feat,
    /// Bug fix
    Fix,
    /// Breaking change
    Breaking,
    /// Documentation changes
    Docs,
    /// Code style changes (formatting, etc.)
    Style,
    /// Code refactoring
    Refactor,
    /// Performance improvements
    Perf,
    /// Test changes
    Test,
    /// Build system changes
    Build,
    /// CI/CD changes
    Ci,
    /// Maintenance tasks
    Chore,
    /// Revert previous commit
    Revert,
    /// Unknown/custom commit type
    Other(String),
}

/// Configuration for a specific commit type.
#[derive(Debug, Clone)]
pub struct CommitTypeConfig {
    /// Version bump this type should trigger
    pub version_bump: VersionBump,
    /// Whether to include in changelog
    pub include_in_changelog: bool,
    /// Changelog section title
    pub changelog_section: Option<String>,
}

/// Breaking change information extracted from commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    /// Description of the breaking change
    pub description: String,
    /// Location where breaking change was found (footer, body, etc.)
    pub location: String,
}

impl CommitType {
    /// Gets string representation of commit type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Feat => "feat",
            Self::Fix => "fix",
            Self::Breaking => "breaking",
            Self::Docs => "docs",
            Self::Style => "style",
            Self::Refactor => "refactor",
            Self::Perf => "perf",
            Self::Test => "test",
            Self::Build => "build",
            Self::Ci => "ci",
            Self::Chore => "chore",
            Self::Revert => "revert",
            Self::Other(s) => s,
        }
    }
}

impl std::fmt::Display for CommitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CommitType {
    type Err = CommitTypeParseError;

    /// Parses a string into a CommitType.
    ///
    /// Converts string representations into their corresponding CommitType variants.
    /// Standard commit type strings are parsed into their specific variants, while
    /// any other non-empty string becomes an `Other` variant.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(CommitType)` for valid strings, or `Err(CommitTypeParseError)`
    /// for empty strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use sublime_pkg_tools::conventional::CommitType;
    ///
    /// assert_eq!(CommitType::from_str("feat").unwrap(), CommitType::Feat);
    /// assert_eq!(CommitType::from_str("fix").unwrap(), CommitType::Fix);
    /// assert_eq!(CommitType::from_str("custom").unwrap(), CommitType::Other("custom".to_string()));
    /// assert!(CommitType::from_str("").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CommitTypeParseError::Empty` if the input string is empty.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(CommitTypeParseError::Empty);
        }

        let commit_type = match s.to_lowercase().as_str() {
            "feat" => Self::Feat,
            "fix" => Self::Fix,
            "breaking" => Self::Breaking,
            "docs" => Self::Docs,
            "style" => Self::Style,
            "refactor" => Self::Refactor,
            "perf" => Self::Perf,
            "test" => Self::Test,
            "build" => Self::Build,
            "ci" => Self::Ci,
            "chore" => Self::Chore,
            "revert" => Self::Revert,
            _ => Self::Other(s.to_string()),
        };

        Ok(commit_type)
    }
}
