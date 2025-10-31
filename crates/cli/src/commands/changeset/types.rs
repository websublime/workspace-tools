//! Shared types for changeset commands.
//!
//! This module provides common type definitions used across multiple changeset
//! command implementations for JSON output and display formatting.
//!
//! # What
//!
//! Provides:
//! - `ChangesetInfo` - Common structure for changeset information in JSON output
//! - `format_bump_type()` - Utility function for formatting bump types
//! - Conversion implementations from internal types to output types
//!
//! # How
//!
//! These types serve as a standardized format for changeset data across all commands,
//! ensuring consistent JSON output structure and field naming. The conversion traits
//! make it easy to transform internal `Changeset` types into output-friendly formats.
//!
//! # Why
//!
//! Centralizing output types:
//! - Ensures consistent JSON structure across all commands
//! - Eliminates duplicate type definitions
//! - Makes it easy to modify output format in one place
//! - Provides clear documentation of the output schema
//! - Simplifies testing and validation
//!
//! # Examples
//!
//! ```rust,ignore
//! use super::types::{ChangesetInfo, format_bump_type};
//! use sublime_pkg_tools::types::Changeset;
//!
//! fn output_changeset(changeset: Changeset) {
//!     let info: ChangesetInfo = changeset.into();
//!     println!("Branch: {}", info.branch);
//!     println!("Bump: {}", info.bump);
//! }
//! ```

use serde::Serialize;
use sublime_pkg_tools::types::{ArchivedChangeset, Changeset, VersionBump};

/// Common changeset information structure for JSON output.
///
/// This structure provides a standardized format for changeset data across
/// all commands. All timestamp fields use RFC3339 format for consistency
/// and interoperability.
///
/// # Fields
///
/// * `branch` - Branch name (also serves as unique identifier)
/// * `bump` - Version bump type as lowercase string (major, minor, patch, none)
/// * `packages` - List of affected package names
/// * `environments` - List of target environment names
/// * `commits` - List of commit SHA identifiers
/// * `created_at` - Creation timestamp in RFC3339 format
/// * `updated_at` - Last update timestamp in RFC3339 format
///
/// # Examples
///
/// ```rust,ignore
/// use super::types::ChangesetInfo;
/// use serde_json;
///
/// let info = ChangesetInfo {
///     branch: "feature/new-api".to_string(),
///     bump: "minor".to_string(),
///     packages: vec!["my-package".to_string()],
///     environments: vec!["production".to_string()],
///     commits: vec!["abc123".to_string()],
///     created_at: "2025-10-31T10:00:00Z".to_string(),
///     updated_at: "2025-10-31T12:00:00Z".to_string(),
/// };
///
/// let json = serde_json::to_string_pretty(&info)?;
/// println!("{}", json);
/// ```
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ChangesetInfo {
    /// Branch name (also serves as unique identifier).
    pub branch: String,
    /// Version bump type (major, minor, patch, none).
    pub bump: String,
    /// List of affected packages.
    pub packages: Vec<String>,
    /// Target environments.
    pub environments: Vec<String>,
    /// List of commit IDs.
    pub commits: Vec<String>,
    /// Creation timestamp (RFC3339 format).
    pub created_at: String,
    /// Last update timestamp (RFC3339 format).
    pub updated_at: String,
}

impl From<Changeset> for ChangesetInfo {
    /// Converts a `Changeset` into `ChangesetInfo` for output.
    ///
    /// This conversion:
    /// - Lowercases the bump type for consistency
    /// - Formats timestamps as RFC3339 strings
    /// - Clones all vector fields for ownership
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use super::types::ChangesetInfo;
    /// use sublime_pkg_tools::types::Changeset;
    ///
    /// let changeset: Changeset = load_changeset().await?;
    /// let info: ChangesetInfo = changeset.into();
    /// ```
    fn from(changeset: Changeset) -> Self {
        Self {
            branch: changeset.branch,
            bump: format_bump_type(changeset.bump),
            packages: changeset.packages,
            environments: changeset.environments,
            commits: changeset.changes,
            created_at: changeset.created_at.to_rfc3339(),
            updated_at: changeset.updated_at.to_rfc3339(),
        }
    }
}

impl From<&Changeset> for ChangesetInfo {
    /// Converts a reference to a `Changeset` into `ChangesetInfo` for output.
    ///
    /// This is useful when you don't want to consume the original `Changeset`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use super::types::ChangesetInfo;
    /// use sublime_pkg_tools::types::Changeset;
    ///
    /// let changeset: Changeset = load_changeset().await?;
    /// let info: ChangesetInfo = (&changeset).into();
    /// // changeset is still available here
    /// ```
    fn from(changeset: &Changeset) -> Self {
        Self {
            branch: changeset.branch.clone(),
            bump: format_bump_type(changeset.bump),
            packages: changeset.packages.clone(),
            environments: changeset.environments.clone(),
            commits: changeset.changes.clone(),
            created_at: changeset.created_at.to_rfc3339(),
            updated_at: changeset.updated_at.to_rfc3339(),
        }
    }
}

/// Archived changeset information structure for JSON output.
///
/// This structure provides a standardized format for archived changeset data,
/// including both the changeset information and release details. All timestamp
/// fields use RFC3339 format for consistency and interoperability.
///
/// # Fields
///
/// * `branch` - Branch name (also serves as unique identifier)
/// * `bump` - Version bump type as lowercase string (major, minor, patch, none)
/// * `packages` - List of affected package names
/// * `environments` - List of target environment names
/// * `commits` - List of commit SHA identifiers
/// * `created_at` - Changeset creation timestamp in RFC3339 format
/// * `updated_at` - Changeset last update timestamp in RFC3339 format
/// * `versions` - Map of package names to their released versions
/// * `git_commit` - Git commit hash of the release
/// * `applied_at` - Release application timestamp in RFC3339 format
/// * `applied_by` - User/system that performed the release
///
/// # Examples
///
/// ```rust,ignore
/// use super::types::ArchivedChangesetInfo;
/// use serde_json;
/// use std::collections::HashMap;
///
/// let mut versions = HashMap::new();
/// versions.insert("my-package".to_string(), "1.2.0".to_string());
///
/// let info = ArchivedChangesetInfo {
///     branch: "feature/new-api".to_string(),
///     bump: "minor".to_string(),
///     packages: vec!["my-package".to_string()],
///     environments: vec!["production".to_string()],
///     commits: vec!["abc123".to_string()],
///     created_at: "2025-10-31T10:00:00Z".to_string(),
///     updated_at: "2025-10-31T12:00:00Z".to_string(),
///     versions,
///     git_commit: "abc123def456".to_string(),
///     applied_at: "2025-10-31T14:00:00Z".to_string(),
///     applied_by: "ci-bot".to_string(),
/// };
///
/// let json = serde_json::to_string_pretty(&info)?;
/// println!("{}", json);
/// ```
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ArchivedChangesetInfo {
    /// Branch name (also serves as unique identifier).
    pub branch: String,
    /// Version bump type (major, minor, patch, none).
    pub bump: String,
    /// List of affected packages.
    pub packages: Vec<String>,
    /// Target environments.
    pub environments: Vec<String>,
    /// List of commit IDs.
    pub commits: Vec<String>,
    /// Changeset creation timestamp (RFC3339 format).
    pub created_at: String,
    /// Changeset last update timestamp (RFC3339 format).
    pub updated_at: String,
    /// Package versions map (package name -> version).
    pub versions: std::collections::HashMap<String, String>,
    /// Git commit hash of the release.
    pub git_commit: String,
    /// Release timestamp (RFC3339 format).
    pub applied_at: String,
    /// User/system that performed the release.
    pub applied_by: String,
}

impl From<ArchivedChangeset> for ArchivedChangesetInfo {
    /// Converts an `ArchivedChangeset` into `ArchivedChangesetInfo` for output.
    ///
    /// This conversion:
    /// - Lowercases the bump type for consistency
    /// - Formats all timestamps as RFC3339 strings
    /// - Clones all vector and string fields for ownership
    /// - Preserves optional changelog entry
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use super::types::ArchivedChangesetInfo;
    /// use sublime_pkg_tools::types::ArchivedChangeset;
    ///
    /// let archived: ArchivedChangeset = load_archived_changeset().await?;
    /// let info: ArchivedChangesetInfo = archived.into();
    /// ```
    fn from(archived: ArchivedChangeset) -> Self {
        Self {
            branch: archived.changeset.branch,
            bump: format_bump_type(archived.changeset.bump),
            packages: archived.changeset.packages,
            environments: archived.changeset.environments,
            commits: archived.changeset.changes,
            created_at: archived.changeset.created_at.to_rfc3339(),
            updated_at: archived.changeset.updated_at.to_rfc3339(),
            versions: archived.release_info.versions,
            git_commit: archived.release_info.git_commit,
            applied_at: archived.release_info.applied_at.to_rfc3339(),
            applied_by: archived.release_info.applied_by,
        }
    }
}

impl From<&ArchivedChangeset> for ArchivedChangesetInfo {
    /// Converts a reference to an `ArchivedChangeset` into `ArchivedChangesetInfo` for output.
    ///
    /// This is useful when you don't want to consume the original `ArchivedChangeset`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use super::types::ArchivedChangesetInfo;
    /// use sublime_pkg_tools::types::ArchivedChangeset;
    ///
    /// let archived: ArchivedChangeset = load_archived_changeset().await?;
    /// let info: ArchivedChangesetInfo = (&archived).into();
    /// // archived is still available here
    /// ```
    fn from(archived: &ArchivedChangeset) -> Self {
        Self {
            branch: archived.changeset.branch.clone(),
            bump: format_bump_type(archived.changeset.bump),
            packages: archived.changeset.packages.clone(),
            environments: archived.changeset.environments.clone(),
            commits: archived.changeset.changes.clone(),
            created_at: archived.changeset.created_at.to_rfc3339(),
            updated_at: archived.changeset.updated_at.to_rfc3339(),
            versions: archived.release_info.versions.clone(),
            git_commit: archived.release_info.git_commit.clone(),
            applied_at: archived.release_info.applied_at.to_rfc3339(),
            applied_by: archived.release_info.applied_by.clone(),
        }
    }
}

/// Formats a bump type for display.
///
/// Converts a `VersionBump` enum variant into a lowercase string representation
/// suitable for output. This ensures consistent formatting across all commands.
///
/// # Arguments
///
/// * `bump` - The version bump enum variant to format
///
/// # Returns
///
/// Returns a lowercase string representation of the bump type.
///
/// # Examples
///
/// ```rust,ignore
/// use super::types::format_bump_type;
/// use sublime_pkg_tools::types::VersionBump;
///
/// assert_eq!(format_bump_type(VersionBump::Major), "major");
/// assert_eq!(format_bump_type(VersionBump::Minor), "minor");
/// assert_eq!(format_bump_type(VersionBump::Patch), "patch");
/// assert_eq!(format_bump_type(VersionBump::None), "none");
/// ```
pub(crate) fn format_bump_type(bump: VersionBump) -> String {
    match bump {
        VersionBump::Major => "major".to_string(),
        VersionBump::Minor => "minor".to_string(),
        VersionBump::Patch => "patch".to_string(),
        VersionBump::None => "none".to_string(),
    }
}
