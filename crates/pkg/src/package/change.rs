//! # Package Change Types Module
//!
//! This module defines the types of changes that can occur to packages or dependencies.
//!
//! When tracking changes between package versions or during dependency upgrades,
//! it's useful to categorize the types of changes that have occurred. The `ChangeType`
//! enum provides a standard way to represent these changes.

use serde::{Deserialize, Serialize};

/// Types of changes that can occur between package versions or dependencies.
///
/// This enum represents the different ways a package or dependency can change
/// between versions, including additions, removals, updates, or no change.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::ChangeType;
///
/// // Use change types to categorize dependency modifications
/// let changes = vec![
///     ("react", ChangeType::Updated),
///     ("express", ChangeType::Added),
///     ("moment", ChangeType::Removed),
///     ("lodash", ChangeType::Unchanged),
/// ];
///
/// // Display changes
/// for (pkg, change_type) in changes {
///     println!("Package {}: {}", pkg, change_type);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// Package or dependency was added
    Added,
    /// Package or dependency was removed
    Removed,
    /// Package or dependency version was updated
    Updated,
    /// No change detected
    Unchanged,
}

impl std::fmt::Display for ChangeType {
    /// Formats the change type as a lowercase string.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::ChangeType;
    ///
    /// assert_eq!(ChangeType::Added.to_string(), "added");
    /// assert_eq!(ChangeType::Removed.to_string(), "removed");
    /// assert_eq!(ChangeType::Updated.to_string(), "updated");
    /// assert_eq!(ChangeType::Unchanged.to_string(), "unchanged");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Updated => write!(f, "updated"),
            Self::Unchanged => write!(f, "unchanged"),
        }
    }
}
