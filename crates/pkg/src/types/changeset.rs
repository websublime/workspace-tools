//! Changeset data structures for tracking package changes and releases.
//!
//! **What**: Provides core data structures for representing changesets, which are collections
//! of package changes associated with a branch, including version bumps, target environments,
//! affected packages, and commit history. Also includes archived changesets with release metadata.
//!
//! **How**: Defines serializable structures using `serde` for JSON persistence. Changesets track
//! the branch, version bump type, target environments, affected packages, and associated commits.
//! Archived changesets add release information including when and by whom the release was applied.
//!
//! **Why**: Changesets are the source of truth for coordinating package releases across a
//! monorepo. They enable developers to batch changes together, track what will be released,
//! and maintain a history of releases with full metadata for auditing and rollback purposes.
//!
//! # Core Types
//!
//! ## Changeset
//!
//! The primary data structure representing a set of changes to be released. It includes:
//! - Branch name identifying the changeset
//! - Version bump type (Major, Minor, Patch, None)
//! - Target deployment environments
//! - List of affected package names
//! - Commit IDs included in the changeset
//! - Creation and update timestamps
//!
//! ## ArchivedChangeset
//!
//! A changeset that has been released and archived, containing:
//! - The original changeset data
//! - Release metadata (when, by whom, git commit, actual versions)
//!
//! ## ReleaseInfo
//!
//! Metadata added when a changeset is applied and archived, including:
//! - Application timestamp
//! - User or system that applied the release
//! - Git commit hash of the release
//! - Actual versions released per package
//!
//! # Examples
//!
//! ## Creating a new changeset
//!
//! ```rust
//! use sublime_pkg_tools::types::{Changeset, VersionBump};
//!
//! let changeset = Changeset::new(
//!     "feature/oauth-integration",
//!     VersionBump::Minor,
//!     vec!["production".to_string()],
//! );
//!
//! assert_eq!(changeset.branch, "feature/oauth-integration");
//! assert_eq!(changeset.bump, VersionBump::Minor);
//! assert_eq!(changeset.environments, vec!["production"]);
//! assert!(changeset.packages.is_empty());
//! assert!(changeset.changes.is_empty());
//! ```
//!
//! ## Adding packages and commits
//!
//! ```rust
//! use sublime_pkg_tools::types::{Changeset, VersionBump};
//!
//! let mut changeset = Changeset::new(
//!     "feat/new-api",
//!     VersionBump::Major,
//!     vec!["staging".to_string(), "production".to_string()],
//! );
//!
//! changeset.add_package("@myorg/auth");
//! changeset.add_package("@myorg/core");
//! changeset.add_commit("abc123def456");
//!
//! assert_eq!(changeset.packages.len(), 2);
//! assert_eq!(changeset.changes.len(), 1);
//! ```
//!
//! ## Validating a changeset
//!
//! ```rust
//! use sublime_pkg_tools::types::{Changeset, VersionBump};
//!
//! let mut changeset = Changeset::new(
//!     "feature/new-feature",
//!     VersionBump::Minor,
//!     vec!["production".to_string()],
//! );
//!
//! // Empty changeset should fail validation
//! let result = changeset.validate(&["production", "staging"]);
//! assert!(result.is_err());
//!
//! // Add packages to make it valid
//! changeset.add_package("@myorg/api");
//! let result = changeset.validate(&["production", "staging"]);
//! assert!(result.is_ok());
//! ```
//!
//! ## Serializing to JSON
//!
//! ```rust
//! use sublime_pkg_tools::types::{Changeset, VersionBump};
//!
//! let changeset = Changeset::new(
//!     "feature/test",
//!     VersionBump::Patch,
//!     vec!["dev".to_string()],
//! );
//!
//! let json = serde_json::to_string_pretty(&changeset).unwrap();
//! assert!(json.contains("\"branch\""));
//! assert!(json.contains("\"bump\""));
//! ```
//!
//! ## Creating an archived changeset
//!
//! ```rust
//! use sublime_pkg_tools::types::{Changeset, ArchivedChangeset, ReleaseInfo, VersionBump};
//! use std::collections::HashMap;
//!
//! let mut changeset = Changeset::new(
//!     "feature/release",
//!     VersionBump::Minor,
//!     vec!["production".to_string()],
//! );
//! changeset.add_package("@myorg/core");
//!
//! let mut versions = HashMap::new();
//! versions.insert("@myorg/core".to_string(), "1.5.0".to_string());
//!
//! let release_info = ReleaseInfo::new(
//!     "ci-bot",
//!     "abc123def456",
//!     versions,
//! );
//!
//! let archived = ArchivedChangeset::new(changeset, release_info);
//! assert_eq!(archived.release_info.applied_by, "ci-bot");
//! ```

use crate::error::{ChangesetError, ChangesetResult};
use crate::types::VersionBump;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a changeset for a branch.
///
/// A changeset is the central data structure for tracking changes to packages in a workspace.
/// It associates a branch with a version bump type, target environments, affected packages,
/// and the commits that are included in the changeset.
///
/// Changesets are typically stored as JSON files and serve as the source of truth for
/// what will be released when the branch is merged.
///
/// # Fields
///
/// - `branch`: The git branch name this changeset is associated with
/// - `bump`: The type of version bump to apply (Major, Minor, Patch, or None)
/// - `environments`: Target deployment environments (e.g., ["staging", "production"])
/// - `packages`: List of package names that are affected by this changeset
/// - `changes`: List of git commit hashes included in this changeset
/// - `created_at`: Timestamp when the changeset was created
/// - `updated_at`: Timestamp when the changeset was last modified
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::{Changeset, VersionBump};
///
/// let mut changeset = Changeset::new(
///     "feature/oauth",
///     VersionBump::Minor,
///     vec!["production".to_string()],
/// );
///
/// changeset.add_package("@myorg/auth");
/// changeset.add_commit("abc123");
///
/// assert_eq!(changeset.branch, "feature/oauth");
/// assert!(changeset.has_package("@myorg/auth"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Changeset {
    /// Branch name (e.g., "feat/oauth-integration").
    ///
    /// This is the git branch that the changeset is associated with. Typically,
    /// one changeset exists per active branch.
    pub branch: String,

    /// Version bump to apply to all affected packages.
    ///
    /// Determines how package versions will be incremented when the changeset is applied.
    /// Can be Major (breaking changes), Minor (new features), Patch (bug fixes), or None.
    pub bump: VersionBump,

    /// Target deployment environments.
    ///
    /// List of environments where the packages should be deployed, such as
    /// ["staging", "production"]. These must match configured available environments.
    pub environments: Vec<String>,

    /// Package names affected (e.g., ["@myorg/auth", "@myorg/core"]).
    ///
    /// List of package names that have changes and will receive version bumps
    /// when this changeset is applied.
    pub packages: Vec<String>,

    /// Commit IDs included in this changeset.
    ///
    /// List of git commit hashes that are part of this changeset. These commits
    /// represent the actual changes being released.
    pub changes: Vec<String>,

    /// When changeset was created.
    ///
    /// UTC timestamp recording when this changeset was first created.
    pub created_at: DateTime<Utc>,

    /// When changeset was last updated.
    ///
    /// UTC timestamp recording the last time this changeset was modified
    /// (e.g., packages or commits added).
    pub updated_at: DateTime<Utc>,
}

impl Changeset {
    /// Creates a new changeset with the specified branch, bump type, and environments.
    ///
    /// The changeset is initialized with empty packages and changes lists, and both
    /// timestamps are set to the current UTC time.
    ///
    /// # Arguments
    ///
    /// * `branch` - The git branch name for this changeset
    /// * `bump` - The version bump type to apply
    /// * `environments` - Target deployment environments
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let changeset = Changeset::new(
    ///     "feature/new-api",
    ///     VersionBump::Major,
    ///     vec!["production".to_string()],
    /// );
    ///
    /// assert_eq!(changeset.branch, "feature/new-api");
    /// assert!(changeset.packages.is_empty());
    /// ```
    #[must_use]
    pub fn new(branch: impl Into<String>, bump: VersionBump, environments: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            branch: branch.into(),
            bump,
            environments,
            packages: Vec::new(),
            changes: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Adds a package to the changeset if not already present.
    ///
    /// Updates the `updated_at` timestamp when a new package is added.
    /// Duplicate package names are ignored.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/api",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    ///
    /// changeset.add_package("@myorg/core");
    /// changeset.add_package("@myorg/core"); // Duplicate, ignored
    ///
    /// assert_eq!(changeset.packages.len(), 1);
    /// ```
    pub fn add_package(&mut self, package: impl Into<String>) {
        let package = package.into();
        if !self.packages.contains(&package) {
            self.packages.push(package);
            self.updated_at = Utc::now();
        }
    }

    /// Adds a commit hash to the changeset if not already present.
    ///
    /// Updates the `updated_at` timestamp when a new commit is added.
    /// Duplicate commit hashes are ignored.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit hash to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Patch,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// changeset.add_commit("abc123");
    /// changeset.add_commit("def456");
    ///
    /// assert_eq!(changeset.changes.len(), 2);
    /// ```
    pub fn add_commit(&mut self, commit: impl Into<String>) {
        let commit = commit.into();
        if !self.changes.contains(&commit) {
            self.changes.push(commit);
            self.updated_at = Utc::now();
        }
    }

    /// Removes a package from the changeset.
    ///
    /// Updates the `updated_at` timestamp if the package was present and removed.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name to remove
    ///
    /// # Returns
    ///
    /// `true` if the package was present and removed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Minor,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// changeset.add_package("@myorg/core");
    /// assert!(changeset.remove_package("@myorg/core"));
    /// assert!(!changeset.remove_package("@myorg/core")); // Already removed
    /// ```
    pub fn remove_package(&mut self, package: &str) -> bool {
        if let Some(pos) = self.packages.iter().position(|p| p == package) {
            self.packages.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Checks if the changeset contains a specific package.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name to check
    ///
    /// # Returns
    ///
    /// `true` if the package is in the changeset, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Patch,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// changeset.add_package("@myorg/auth");
    /// assert!(changeset.has_package("@myorg/auth"));
    /// assert!(!changeset.has_package("@myorg/other"));
    /// ```
    #[must_use]
    pub fn has_package(&self, package: &str) -> bool {
        self.packages.contains(&package.to_string())
    }

    /// Checks if the changeset contains a specific commit.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit hash to check
    ///
    /// # Returns
    ///
    /// `true` if the commit is in the changeset, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Minor,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// changeset.add_commit("abc123");
    /// assert!(changeset.has_commit("abc123"));
    /// assert!(!changeset.has_commit("xyz789"));
    /// ```
    #[must_use]
    pub fn has_commit(&self, commit: &str) -> bool {
        self.changes.contains(&commit.to_string())
    }

    /// Returns whether the changeset is empty (has no packages).
    ///
    /// # Returns
    ///
    /// `true` if the changeset has no packages, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Minor,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// assert!(changeset.is_empty());
    ///
    /// changeset.add_package("@myorg/core");
    /// assert!(!changeset.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Updates the bump type and updates the timestamp.
    ///
    /// # Arguments
    ///
    /// * `bump` - The new version bump type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Patch,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// changeset.set_bump(VersionBump::Major);
    /// assert_eq!(changeset.bump, VersionBump::Major);
    /// ```
    pub fn set_bump(&mut self, bump: VersionBump) {
        self.bump = bump;
        self.updated_at = Utc::now();
    }

    /// Updates the environments and updates the timestamp.
    ///
    /// # Arguments
    ///
    /// * `environments` - The new list of target environments
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Minor,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// changeset.set_environments(vec!["staging".to_string(), "production".to_string()]);
    /// assert_eq!(changeset.environments.len(), 2);
    /// ```
    pub fn set_environments(&mut self, environments: Vec<String>) {
        self.environments = environments;
        self.updated_at = Utc::now();
    }

    /// Validates the changeset structure and data.
    ///
    /// Checks for:
    /// - Non-empty branch name
    /// - At least one package
    /// - Valid environments (all must be in the available list)
    /// - Non-empty environment list
    ///
    /// # Arguments
    ///
    /// * `available_environments` - Slice of valid environment names
    ///
    /// # Returns
    ///
    /// `Ok(())` if the changeset is valid, or a `ValidationFailed` error with details.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feature/test",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    ///
    /// // Validation fails - no packages
    /// assert!(changeset.validate(&["production", "staging"]).is_err());
    ///
    /// changeset.add_package("@myorg/core");
    /// assert!(changeset.validate(&["production", "staging"]).is_ok());
    /// ```
    pub fn validate(&self, available_environments: &[&str]) -> ChangesetResult<()> {
        let mut errors = Vec::new();

        // Validate branch name
        if self.branch.trim().is_empty() {
            errors.push("Branch name cannot be empty".to_string());
        }

        // Validate packages
        if self.packages.is_empty() {
            errors.push("Changeset must contain at least one package".to_string());
        }

        // Validate environments
        if self.environments.is_empty() {
            errors.push("Changeset must target at least one environment".to_string());
        }

        for env in &self.environments {
            if !available_environments.contains(&env.as_str()) {
                errors.push(format!(
                    "Environment '{}' is not in available environments: {:?}",
                    env, available_environments
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ChangesetError::ValidationFailed { errors })
        }
    }

    /// Marks the changeset as updated with the current timestamp.
    ///
    /// This is useful when making multiple modifications and wanting to update
    /// the timestamp explicitly at the end.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/test",
    ///     VersionBump::Minor,
    ///     vec!["dev".to_string()],
    /// );
    ///
    /// let old_timestamp = changeset.updated_at;
    /// std::thread::sleep(std::time::Duration::from_millis(10));
    /// changeset.touch();
    /// assert!(changeset.updated_at > old_timestamp);
    /// ```
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Changeset after being released and archived.
///
/// When a changeset is applied (packages are released), it is moved from the active
/// changesets directory to the history/archive directory with additional metadata
/// about the release.
///
/// This structure preserves the original changeset data and adds information about
/// when and how the release was performed.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::{Changeset, ArchivedChangeset, ReleaseInfo, VersionBump};
/// use std::collections::HashMap;
///
/// let mut changeset = Changeset::new(
///     "feature/new-feature",
///     VersionBump::Minor,
///     vec!["production".to_string()],
/// );
/// changeset.add_package("@myorg/api");
///
/// let mut versions = HashMap::new();
/// versions.insert("@myorg/api".to_string(), "2.1.0".to_string());
///
/// let release_info = ReleaseInfo::new("developer@example.com", "abc123def", versions);
/// let archived = ArchivedChangeset::new(changeset, release_info);
///
/// assert!(archived.changeset.has_package("@myorg/api"));
/// assert_eq!(archived.release_info.applied_by, "developer@example.com");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchivedChangeset {
    /// Original changeset data.
    ///
    /// The complete changeset as it existed before being applied, including
    /// all packages, commits, and metadata.
    pub changeset: Changeset,

    /// Release metadata.
    ///
    /// Information about when and how the changeset was applied, including
    /// who applied it, the git commit, and the actual versions released.
    pub release_info: ReleaseInfo,
}

impl ArchivedChangeset {
    /// Creates a new archived changeset with the given changeset and release info.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The original changeset that was released
    /// * `release_info` - Metadata about the release
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Changeset, ArchivedChangeset, ReleaseInfo, VersionBump};
    /// use std::collections::HashMap;
    ///
    /// let changeset = Changeset::new(
    ///     "feature/test",
    ///     VersionBump::Patch,
    ///     vec!["production".to_string()],
    /// );
    ///
    /// let release_info = ReleaseInfo::new(
    ///     "ci-bot",
    ///     "abc123",
    ///     HashMap::new(),
    /// );
    ///
    /// let archived = ArchivedChangeset::new(changeset, release_info);
    /// assert_eq!(archived.changeset.branch, "feature/test");
    /// ```
    #[must_use]
    pub fn new(changeset: Changeset, release_info: ReleaseInfo) -> Self {
        Self { changeset, release_info }
    }
}

/// Release metadata added when changeset is archived.
///
/// This structure captures all relevant information about the release process,
/// including timing, attribution, and the actual package versions that were published.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::ReleaseInfo;
/// use std::collections::HashMap;
///
/// let mut versions = HashMap::new();
/// versions.insert("@myorg/core".to_string(), "3.0.0".to_string());
/// versions.insert("@myorg/utils".to_string(), "1.5.2".to_string());
///
/// let release_info = ReleaseInfo::new(
///     "ci-system",
///     "abc123def456789",
///     versions,
/// );
///
/// assert_eq!(release_info.applied_by, "ci-system");
/// assert_eq!(release_info.git_commit, "abc123def456789");
/// assert_eq!(release_info.versions.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseInfo {
    /// When release was applied.
    ///
    /// UTC timestamp of when the changeset was applied and packages were released.
    pub applied_at: DateTime<Utc>,

    /// Who applied the release (e.g., "ci-bot", "user@example.com").
    ///
    /// Identifier of the user or system that performed the release. This can be
    /// an email address, username, or system identifier like "ci-bot".
    pub applied_by: String,

    /// Git commit hash of the release commit.
    ///
    /// The commit hash where the version updates were committed to the repository.
    /// This allows correlation between the changeset and the actual git history.
    pub git_commit: String,

    /// Actual versions released per package.
    ///
    /// Map of package names to the version strings that were published.
    /// This captures the actual versions at release time, which may differ
    /// from calculated versions if manual adjustments were made.
    pub versions: HashMap<String, String>,
}

impl ReleaseInfo {
    /// Creates a new release info with the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `applied_by` - Identifier of who/what applied the release
    /// * `git_commit` - Git commit hash of the release
    /// * `versions` - Map of package names to released versions
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::ReleaseInfo;
    /// use std::collections::HashMap;
    ///
    /// let mut versions = HashMap::new();
    /// versions.insert("@myorg/api".to_string(), "2.0.0".to_string());
    ///
    /// let release_info = ReleaseInfo::new(
    ///     "developer@example.com",
    ///     "abc123",
    ///     versions,
    /// );
    ///
    /// assert_eq!(release_info.applied_by, "developer@example.com");
    /// ```
    #[must_use]
    pub fn new(
        applied_by: impl Into<String>,
        git_commit: impl Into<String>,
        versions: HashMap<String, String>,
    ) -> Self {
        Self {
            applied_at: Utc::now(),
            applied_by: applied_by.into(),
            git_commit: git_commit.into(),
            versions,
        }
    }

    /// Gets the version for a specific package.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name to look up
    ///
    /// # Returns
    ///
    /// The version string if the package exists in the release, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::ReleaseInfo;
    /// use std::collections::HashMap;
    ///
    /// let mut versions = HashMap::new();
    /// versions.insert("@myorg/core".to_string(), "1.2.3".to_string());
    ///
    /// let release_info = ReleaseInfo::new("ci", "abc", versions);
    ///
    /// assert_eq!(release_info.get_version("@myorg/core"), Some("1.2.3"));
    /// assert_eq!(release_info.get_version("@myorg/other"), None);
    /// ```
    #[must_use]
    pub fn get_version(&self, package: &str) -> Option<&str> {
        self.versions.get(package).map(String::as_str)
    }

    /// Returns the number of packages in the release.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::ReleaseInfo;
    /// use std::collections::HashMap;
    ///
    /// let mut versions = HashMap::new();
    /// versions.insert("pkg1".to_string(), "1.0.0".to_string());
    /// versions.insert("pkg2".to_string(), "2.0.0".to_string());
    ///
    /// let release_info = ReleaseInfo::new("ci", "abc", versions);
    /// assert_eq!(release_info.package_count(), 2);
    /// ```
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.versions.len()
    }
}

/// Summary of updates made when adding commits from Git.
///
/// This structure provides detailed information about what changed when commits
/// were added to a changeset from Git, including newly discovered packages,
/// commit counts, and the full list of affected packages.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::UpdateSummary;
///
/// let summary = UpdateSummary {
///     commits_added: 3,
///     commit_ids: vec!["abc123".to_string(), "def456".to_string(), "ghi789".to_string()],
///     new_packages: vec!["@myorg/new-pkg".to_string()],
///     existing_packages: vec!["@myorg/existing-pkg".to_string()],
/// };
///
/// assert_eq!(summary.commits_added, 3);
/// assert_eq!(summary.total_packages(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateSummary {
    /// Number of commits that were added to the changeset.
    pub commits_added: usize,

    /// List of commit IDs that were added.
    pub commit_ids: Vec<String>,

    /// Packages that were newly added to the changeset.
    pub new_packages: Vec<String>,

    /// Packages that were already in the changeset before the update.
    pub existing_packages: Vec<String>,
}

impl UpdateSummary {
    /// Creates a new `UpdateSummary`.
    ///
    /// # Parameters
    ///
    /// * `commits_added` - Number of commits added
    /// * `commit_ids` - List of commit IDs added
    /// * `new_packages` - Newly discovered packages
    /// * `existing_packages` - Packages that were already tracked
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::UpdateSummary;
    ///
    /// let summary = UpdateSummary::new(
    ///     2,
    ///     vec!["abc123".to_string(), "def456".to_string()],
    ///     vec!["new-pkg".to_string()],
    ///     vec!["old-pkg".to_string()],
    /// );
    ///
    /// assert_eq!(summary.commits_added, 2);
    /// ```
    #[must_use]
    pub fn new(
        commits_added: usize,
        commit_ids: Vec<String>,
        new_packages: Vec<String>,
        existing_packages: Vec<String>,
    ) -> Self {
        Self { commits_added, commit_ids, new_packages, existing_packages }
    }

    /// Creates an empty `UpdateSummary` indicating no changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::UpdateSummary;
    ///
    /// let summary = UpdateSummary::empty();
    ///
    /// assert_eq!(summary.commits_added, 0);
    /// assert!(summary.commit_ids.is_empty());
    /// assert!(summary.new_packages.is_empty());
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self {
            commits_added: 0,
            commit_ids: Vec::new(),
            new_packages: Vec::new(),
            existing_packages: Vec::new(),
        }
    }

    /// Returns the total number of packages in the changeset after the update.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::UpdateSummary;
    ///
    /// let summary = UpdateSummary::new(
    ///     1,
    ///     vec!["abc123".to_string()],
    ///     vec!["new-pkg".to_string()],
    ///     vec!["old-pkg1".to_string(), "old-pkg2".to_string()],
    /// );
    ///
    /// assert_eq!(summary.total_packages(), 3);
    /// ```
    #[must_use]
    pub fn total_packages(&self) -> usize {
        self.new_packages.len() + self.existing_packages.len()
    }

    /// Checks if any new packages were discovered.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::UpdateSummary;
    ///
    /// let summary = UpdateSummary::new(
    ///     1,
    ///     vec!["abc123".to_string()],
    ///     vec!["new-pkg".to_string()],
    ///     vec![],
    /// );
    ///
    /// assert!(summary.has_new_packages());
    /// ```
    #[must_use]
    pub fn has_new_packages(&self) -> bool {
        !self.new_packages.is_empty()
    }

    /// Checks if any commits were added.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::UpdateSummary;
    ///
    /// let empty = UpdateSummary::empty();
    /// assert!(!empty.has_commits());
    ///
    /// let with_commits = UpdateSummary::new(
    ///     1,
    ///     vec!["abc123".to_string()],
    ///     vec![],
    ///     vec![],
    /// );
    /// assert!(with_commits.has_commits());
    /// ```
    #[must_use]
    pub fn has_commits(&self) -> bool {
        self.commits_added > 0
    }
}
