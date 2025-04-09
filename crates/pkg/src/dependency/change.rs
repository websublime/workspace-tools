//! # Dependency Change Module
//!
//! This module provides structures for tracking changes to dependencies.
//!
//! When comparing different versions of a package, it's important to track
//! how dependencies have changed between versions. The `DependencyChange` struct
//! represents such changes, including which dependencies were added, removed,
//! or updated, and whether these changes are potentially breaking.

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::ChangeType;

/// Represents a change to a dependency between package versions.
///
/// When comparing two versions of a package, this structure captures
/// changes to individual dependencies, including:
/// - Dependencies that were added
/// - Dependencies that were removed
/// - Dependencies that were updated to new versions
/// - Whether the change is potentially breaking (according to semver)
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{DependencyChange, ChangeType};
///
/// // Represent a dependency being added
/// let added = DependencyChange::new(
///     "react",
///     None,
///     Some("^17.0.0"),
///     ChangeType::Added
/// );
///
/// // Represent a dependency being updated
/// let updated = DependencyChange::new(
///     "lodash",
///     Some("^4.0.0"),
///     Some("^4.17.21"),
///     ChangeType::Updated
/// );
///
/// // Check if the update is breaking
/// if !updated.breaking {
///     println!("Safe to update lodash");
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyChange {
    /// Name of the dependency
    pub name: String,
    /// Previous version (None if newly added)
    pub previous_version: Option<String>,
    /// Current version (None if removed)
    pub current_version: Option<String>,
    /// Type of change
    pub change_type: ChangeType,
    /// Whether this is a breaking change based on semver
    pub breaking: bool,
}

impl DependencyChange {
    /// Creates a new dependency change record.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency
    /// * `previous_version` - The previous version (None if newly added)
    /// * `current_version` - The current version (None if removed)
    /// * `change_type` - The type of change (Added, Removed, Updated, Unchanged)
    ///
    /// # Returns
    ///
    /// A new `DependencyChange` with automatically calculated `breaking` flag based on:
    /// - Removals are always breaking
    /// - Major version upgrades are breaking
    /// - Adding or minor/patch updates are not breaking
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyChange, ChangeType};
    ///
    /// // New dependency - not breaking
    /// let added = DependencyChange::new(
    ///     "express",
    ///     None,
    ///     Some("^4.18.1"),
    ///     ChangeType::Added
    /// );
    /// assert!(!added.breaking);
    ///
    /// // Removed dependency - breaking
    /// let removed = DependencyChange::new(
    ///     "old-pkg",
    ///     Some("^1.0.0"),
    ///     None,
    ///     ChangeType::Removed
    /// );
    /// assert!(removed.breaking);
    ///
    /// // Minor update - not breaking
    /// let minor_update = DependencyChange::new(
    ///     "lodash",
    ///     Some("^4.17.20"),
    ///     Some("^4.17.21"),
    ///     ChangeType::Updated
    /// );
    /// assert!(!minor_update.breaking);
    ///
    /// // Major update - breaking
    /// let major_update = DependencyChange::new(
    ///     "react",
    ///     Some("^17.0.2"),
    ///     Some("^18.0.0"),
    ///     ChangeType::Updated
    /// );
    /// assert!(major_update.breaking);
    /// ```
    pub fn new(
        name: &str,
        previous_version: Option<&str>,
        current_version: Option<&str>,
        change_type: ChangeType,
    ) -> Self {
        // Determine if this is a breaking change based on semver
        let breaking = match (previous_version, current_version) {
            (Some(prev), Some(curr)) => {
                // Clean up the version strings
                let clean_prev = prev.trim_start_matches('^').trim_start_matches('~');
                let clean_curr = curr.trim_start_matches('^').trim_start_matches('~');

                if let (Ok(prev_ver), Ok(curr_ver)) =
                    (Version::parse(clean_prev), Version::parse(clean_curr))
                {
                    // Breaking if major version increases
                    curr_ver.major > prev_ver.major
                } else {
                    // If we can't parse the version, conservatively assume it might be breaking
                    true
                }
            }
            // Only removals are considered breaking changes, not additions
            (Some(_), None) => true, // Removed dependency is breaking
            (None, Some(_) | None) => false, // Added dependency is not breaking
        };

        Self {
            name: name.to_string(),
            previous_version: previous_version.map(String::from),
            current_version: current_version.map(String::from),
            change_type,
            breaking,
        }
    }
}
