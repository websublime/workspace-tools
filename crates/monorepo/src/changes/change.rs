//! Change tracking models for monorepo packages.
//!
//! This module provides data structures for representing and tracking changes to packages
//! within a monorepo. It enables structured recording of changes for versioning, changelog
//! generation, and release management.

use crate::ChangeError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Unique identifier for a change or changeset.
///
/// This wrapper around UUID provides a type-safe way to identify distinct changes
/// and changesets throughout the system.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::ChangeId;
/// use std::str::FromStr;
///
/// // Generate a new random ID
/// let id = ChangeId::new();
///
/// // Convert to string for serialization
/// let id_string = id.to_string();
///
/// // Parse from a string
/// let parsed_id = ChangeId::from_str(&id_string).expect("Invalid ChangeId string");
///
/// // Create with default
/// let default_id = ChangeId::default(); // Generates a new random UUID
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeId(Uuid);

impl ChangeId {
    /// Creates a new random change ID.
    ///
    /// Generates a new v4 UUID to uniquely identify a change or changeset.
    ///
    /// # Returns
    ///
    /// A new `ChangeId` with a random UUID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Gets the underlying UUID.
    ///
    /// # Returns
    ///
    /// A reference to the underlying UUID.
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
///
/// Represents the nature of a change to a package, following conventional commit types.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::ChangeType;
///
/// let feature = ChangeType::Feature;
/// let fix = ChangeType::Fix;
/// let custom = ChangeType::Custom("migration".to_string());
///
/// // Convert to string representation
/// assert_eq!(feature.to_string(), "feature");
/// assert_eq!(fix.to_string(), "fix");
/// assert_eq!(custom.to_string(), "migration");
/// ```
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
///
/// Represents a specific change to a package in the monorepo, including its type,
/// description, and metadata such as author and associated issues.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{Change, ChangeType};
///
/// // Create a basic change
/// let change = Change::new(
///     "my-package",
///     ChangeType::Feature,
///     "Add new button component",
///     false
/// );
///
/// // Create a breaking change with author and issues
/// let breaking_change = Change::new(
///     "core",
///     ChangeType::Breaking,
///     "Refactor API to use new parameter format",
///     true
/// )
/// .with_author("Jane Smith")
/// .with_issues(vec!["#123", "JIRA-456"]);
///
/// // Check if change is released
/// assert!(!change.is_released());
///
/// // Get a summary of the change
/// assert_eq!(breaking_change.summary(), "breaking!: Refactor API to use new parameter format");
/// ```
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

    /// Target environments
    #[serde(default)]
    pub environments: Vec<String>,
}

impl Change {
    /// Creates a new change record.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package that changed
    /// * `change_type` - Type of change (feature, fix, etc.)
    /// * `description` - Description of the change
    /// * `breaking` - Whether this is a breaking change
    ///
    /// # Returns
    ///
    /// A new `Change` instance with default values for other fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// let change = Change::new(
    ///     "ui-components",
    ///     ChangeType::Feature,
    ///     "Add new dropdown component",
    ///     false
    /// );
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `author` - The name of the change author
    ///
    /// # Returns
    ///
    /// The modified `Change` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// let change = Change::new(
    ///     "api",
    ///     ChangeType::Fix,
    ///     "Fix authentication issue",
    ///     false
    /// )
    /// .with_author("Jane Smith");
    /// ```
    #[must_use]
    pub fn with_author<S: Into<String>>(mut self, author: S) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the release version of the change.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string (e.g., "1.2.0")
    ///
    /// # Returns
    ///
    /// The modified `Change` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// let change = Change::new(
    ///     "core",
    ///     ChangeType::Feature,
    ///     "Add new API endpoint",
    ///     false
    /// )
    /// .with_release_version("2.1.0");
    /// ```
    #[must_use]
    pub fn with_release_version<S: Into<String>>(mut self, version: S) -> Self {
        self.release_version = Some(version.into());
        self
    }

    /// Adds related issues to the change.
    ///
    /// # Arguments
    ///
    /// * `issues` - Collection of issue references (e.g., "#123", "JIRA-456")
    ///
    /// # Returns
    ///
    /// The modified `Change` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// let change = Change::new(
    ///     "api",
    ///     ChangeType::Fix,
    ///     "Fix authentication issue",
    ///     false
    /// )
    /// .with_issues(vec!["#123", "JIRA-456"]);
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `environments` - Collection of environment names (e.g., "production", "staging")
    ///
    /// # Returns
    ///
    /// The modified `Change` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// let change = Change::new(
    ///     "ui-components",
    ///     ChangeType::Feature,
    ///     "Add dark mode toggle",
    ///     false
    /// )
    /// .with_environments(vec!["production", "staging"]);
    /// ```
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
    ///
    /// # Returns
    ///
    /// `true` if the change has been released (has a release version), `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// let unreleased = Change::new("api", ChangeType::Fix, "Bug fix", false);
    /// assert!(!unreleased.is_released());
    ///
    /// let released = Change::new("api", ChangeType::Fix, "Bug fix", false)
    ///     .with_release_version("1.0.0");
    /// assert!(released.is_released());
    /// ```
    #[must_use]
    pub fn is_released(&self) -> bool {
        self.release_version.is_some()
    }

    /// Checks whether this change applies to a specific environment.
    ///
    /// If no environments are specified for the change, it applies to all environments.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment to check against
    ///
    /// # Returns
    ///
    /// `true` if the change applies to the specified environment, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// // Change with no specific environments (applies to all)
    /// let all_envs = Change::new("api", ChangeType::Fix, "Bug fix", false);
    /// assert!(all_envs.applies_to_environment("production"));
    /// assert!(all_envs.applies_to_environment("staging"));
    ///
    /// // Change with specific environments
    /// let prod_only = Change::new("api", ChangeType::Fix, "Bug fix", false)
    ///     .with_environments(vec!["production"]);
    /// assert!(prod_only.applies_to_environment("production"));
    /// assert!(!prod_only.applies_to_environment("staging"));
    /// ```
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
    ///
    /// The summary includes the change type, breaking indicator if applicable, and description.
    ///
    /// # Returns
    ///
    /// A formatted string summary of the change.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType};
    ///
    /// let feature = Change::new("ui", ChangeType::Feature, "Add button", false);
    /// assert_eq!(feature.summary(), "feature: Add button");
    ///
    /// let breaking = Change::new("api", ChangeType::Fix, "Update auth logic", true);
    /// assert_eq!(breaking.summary(), "fix!: Update auth logic");
    /// ```
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
