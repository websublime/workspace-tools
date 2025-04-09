//! Changeset implementation.
//!
//! This module provides functionality for grouping related changes together into
//! changesets. Changesets are used to track multiple changes that should be considered
//! as a unit, often for release management purposes.

use crate::{Change, ChangeId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Collection of related changes.
///
/// A changeset groups multiple changes that are logically related, such as
/// those made in a single pull request or those that should be released together.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{Change, ChangeType, Changeset};
///
/// // Create individual changes
/// let change1 = Change::new("ui", ChangeType::Feature, "Add button component", false);
/// let change2 = Change::new("api", ChangeType::Fix, "Fix validation error", false);
///
/// // Group changes into a changeset
/// let changeset = Changeset::new(
///     Some("PR #123: UI and API improvements"),
///     vec![change1, change2]
/// );
///
/// // Access information
/// assert_eq!(changeset.changes.len(), 2);
/// assert_eq!(
///     changeset.summary,
///     Some("PR #123: UI and API improvements".to_string())
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    /// Unique identifier
    #[serde(default = "ChangeId::new")]
    pub id: ChangeId,

    /// Summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Changes in this set
    pub changes: Vec<Change>,

    /// Creation timestamp
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

impl Changeset {
    /// Creates a new changeset.
    ///
    /// # Arguments
    ///
    /// * `summary` - Optional summary description of the changeset
    /// * `changes` - Collection of changes to include in the changeset
    ///
    /// # Returns
    ///
    /// A new `Changeset` instance containing the specified changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType, Changeset};
    ///
    /// // Create changes
    /// let change1 = Change::new("ui", ChangeType::Feature, "Add button", false);
    /// let change2 = Change::new("api", ChangeType::Fix, "Fix validation", false);
    ///
    /// // Create changeset with summary
    /// let changeset_with_summary = Changeset::new(
    ///     Some("Feature release"),
    ///     vec![change1.clone(), change2.clone()]
    /// );
    ///
    /// // Create changeset without summary
    /// let changeset_no_summary = Changeset::new::<String>(None, vec![change1, change2]);
    /// ```
    #[must_use]
    pub fn new<S: Into<String>>(summary: Option<S>, changes: Vec<Change>) -> Self {
        Self {
            id: ChangeId::new(),
            summary: summary.map(Into::into),
            changes,
            created_at: Utc::now(),
        }
    }

    /// Checks if all changes in the changeset are released.
    ///
    /// # Returns
    ///
    /// `true` if the changeset is not empty and all changes have been released,
    /// `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType, Changeset};
    ///
    /// // Create unreleased change
    /// let unreleased = Change::new("ui", ChangeType::Feature, "Add button", false);
    ///
    /// // Create released change
    /// let released = Change::new("api", ChangeType::Fix, "Fix validation", false)
    ///     .with_release_version("1.0.0");
    ///
    /// // Changeset with all released changes
    /// let all_released = Changeset::new::<String>(None, vec![released.clone()]);
    /// assert!(all_released.is_released());
    ///
    /// // Changeset with mixed release status
    /// let mixed = Changeset::new::<String>(None, vec![unreleased, released]);
    /// assert!(!mixed.is_released());
    ///
    /// // Empty changeset
    /// let empty = Changeset::new::<String>(None, vec![]);
    /// assert!(!empty.is_released());
    /// ```
    #[must_use]
    pub fn is_released(&self) -> bool {
        !self.changes.is_empty() && self.changes.iter().all(Change::is_released)
    }

    /// Gets the package names included in this changeset.
    ///
    /// # Returns
    ///
    /// A list of unique package names affected by this changeset.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{Change, ChangeType, Changeset};
    ///
    /// let change1 = Change::new("ui", ChangeType::Feature, "Add button", false);
    /// let change2 = Change::new("api", ChangeType::Fix, "Fix validation", false);
    /// let change3 = Change::new("ui", ChangeType::Fix, "Fix styling", false);
    ///
    /// let changeset = Changeset::new::<String>(None, vec![change1, change2, change3]);
    ///
    /// // Get unique package names
    /// let packages = changeset.package_names();
    /// assert_eq!(packages.len(), 2);
    /// assert!(packages.contains(&"ui".to_string()));
    /// assert!(packages.contains(&"api".to_string()));
    /// ```
    #[must_use]
    pub fn package_names(&self) -> Vec<String> {
        self.changes.iter().map(|c| c.package.clone()).collect::<HashSet<_>>().into_iter().collect()
    }
}
