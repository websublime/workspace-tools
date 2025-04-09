use crate::ChangeError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Unique identifier for a change or changeset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeId(Uuid);

impl ChangeId {
    /// Creates a new random change ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Gets the underlying UUID.
    #[must_use]
    pub fn uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for ChangeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ChangeId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for ChangeId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Types of changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    /// New feature or enhancement.
    Feature,
    /// Bug fix.
    Fix,
    /// Documentation changes.
    Documentation,
    /// Performance improvements.
    Performance,
    /// Code refactoring without behavior change.
    Refactor,
    /// Test changes.
    Test,
    /// Chore (e.g., build system, dependencies).
    Chore,
    /// Build system changes.
    Build,
    /// CI/CD changes.
    CI,
    /// Reverts a previous change.
    Revert,
    /// Code style changes.
    Style,
    /// Custom change type.
    Custom(String),
    /// Unknown change type.
    Unknown,
    /// Breaking change type.
    Breaking,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeType::Feature => write!(f, "feature"),
            ChangeType::Fix => write!(f, "fix"),
            ChangeType::Documentation => write!(f, "docs"),
            ChangeType::Performance => write!(f, "perf"),
            ChangeType::Refactor => write!(f, "refactor"),
            ChangeType::Test => write!(f, "test"),
            ChangeType::Chore => write!(f, "chore"),
            ChangeType::Build => write!(f, "build"),
            ChangeType::CI => write!(f, "ci"),
            ChangeType::Revert => write!(f, "revert"),
            ChangeType::Style => write!(f, "style"),
            ChangeType::Custom(s) => write!(f, "{s}"),
            ChangeType::Unknown => write!(f, "unknown"),
            ChangeType::Breaking => write!(f, "breaking"),
        }
    }
}

/// A single change record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Unique identifier for the change
    #[serde(default = "ChangeId::new")]
    pub id: ChangeId,

    /// Package name
    pub package: String,

    /// Change type (feature, fix, etc.)
    pub change_type: ChangeType,

    /// Description
    pub description: String,

    /// Whether this is breaking
    #[serde(default)]
    pub breaking: bool,

    /// Creation timestamp
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,

    /// Author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Related issues
    #[serde(default)]
    pub issues: Vec<String>,

    /// Release version (None if unreleased)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_version: Option<String>,

    #[serde(default)]
    pub environments: Vec<String>,
}

impl Change {
    /// Creates a new change record.
    #[must_use]
    pub fn new<S: Into<String>>(
        package: S,
        change_type: ChangeType,
        description: S,
        breaking: bool,
    ) -> Self {
        Self {
            id: ChangeId::new(),
            package: package.into(),
            change_type,
            description: description.into(),
            breaking,
            timestamp: Utc::now(),
            author: None,
            issues: Vec::new(),
            release_version: None,
            environments: Vec::new(),
        }
    }

    /// Sets the author of the change.
    #[must_use]
    pub fn with_author<S: Into<String>>(mut self, author: S) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the release version of the change.
    #[must_use]
    pub fn with_release_version<S: Into<String>>(mut self, version: S) -> Self {
        self.release_version = Some(version.into());
        self
    }

    /// Adds related issues to the change.
    #[must_use]
    pub fn with_issues<I, S>(mut self, issues: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.issues = issues.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the target environments for this change.
    #[must_use]
    pub fn with_environments<I, S>(mut self, environments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.environments = environments.into_iter().map(Into::into).collect();
        self
    }

    /// Checks whether this change is released.
    #[must_use]
    pub fn is_released(&self) -> bool {
        self.release_version.is_some()
    }

    /// Checks whether this change applies to a specific environment.
    #[must_use]
    pub fn applies_to_environment(&self, environment: &str) -> bool {
        // If environments is empty, the change applies to all environments
        if self.environments.is_empty() {
            return true;
        }

        // Otherwise, check if the environment is in the list
        self.environments.iter().any(|env| env == environment)
    }

    /// Gets a summary of the change.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{}{}: {}",
            self.change_type,
            if self.breaking { "!" } else { "" },
            self.description
        )
    }
}

/// Type alias for Change operation results.
pub type ChangeResult<T> = Result<T, ChangeError>;
