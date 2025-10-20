//! Package-level changes data structures and operations.
//!
//! **What**: Provides types for representing changes affecting a single package, including
//! file changes, commit information, version information, and statistics.
//!
//! **How**: Aggregates file changes and commits for a specific package, tracks current and
//! next versions, and provides helper methods for filtering and analyzing package-level changes.
//!
//! **Why**: To provide a comprehensive view of all changes affecting a package, supporting
//! version bumping decisions, changelog generation, and release planning.
//!
//! # Examples
//!
//! ## Creating package changes
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changes::{PackageChanges, FileChange, FileChangeType, PackageChangeStats};
//! use sublime_pkg_tools::types::Version;
//! use sublime_standard_tools::monorepo::WorkspacePackage;
//! use std::path::PathBuf;
//!
//! let workspace_pkg = WorkspacePackage::new(
//!     "@myorg/core".to_string(),
//!     "1.0.0".to_string(),
//!     PathBuf::from("packages/core"),
//!     PathBuf::from("/workspace/packages/core"),
//! );
//!
//! let mut changes = PackageChanges::new(workspace_pkg);
//! changes.current_version = Some(Version::parse("1.0.0").unwrap());
//!
//! // Add file changes
//! changes.add_file(FileChange::new(
//!     PathBuf::from("packages/core/src/index.ts"),
//!     PathBuf::from("src/index.ts"),
//!     FileChangeType::Modified,
//! ));
//!
//! assert!(changes.has_changes);
//! assert_eq!(changes.files.len(), 1);
//! ```

use crate::changes::{CommitInfo, FileChange, FileChangeType, PackageChangeStats};
use crate::types::{Version, VersionBump};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sublime_standard_tools::monorepo::WorkspacePackage;

/// Changes for a single package.
///
/// Contains comprehensive information about all changes affecting a package,
/// including file changes, commits, version information, and statistics.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changes::{PackageChanges, FileChange, FileChangeType};
/// use sublime_standard_tools::monorepo::WorkspacePackage;
/// use std::path::PathBuf;
///
/// let workspace_pkg = WorkspacePackage::new(
///     "@myorg/core".to_string(),
///     "1.0.0".to_string(),
///     PathBuf::from("packages/core"),
///     PathBuf::from("/workspace/packages/core"),
/// );
///
/// let changes = PackageChanges::new(workspace_pkg);
/// assert_eq!(changes.package_name(), "@myorg/core");
/// assert!(!changes.has_changes);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageChanges {
    /// Package information from the workspace.
    ///
    /// Contains name, version, location, and dependencies.
    #[serde(skip, default = "default_workspace_package")]
    pub package_info: WorkspacePackage,

    /// Package name (for serialization since WorkspacePackage doesn't serialize).
    pub package_name: String,

    /// Package version (for serialization).
    pub package_version: String,

    /// Package location relative to workspace root (for serialization).
    pub package_location: PathBuf,

    /// Current version from package.json.
    ///
    /// This is the version before any changes are applied.
    pub current_version: Option<Version>,

    /// Next version calculated from changeset bump.
    ///
    /// This will be `None` for working directory analysis without a changeset.
    /// It will be populated when analyzing with version preview.
    pub next_version: Option<Version>,

    /// Bump type applied to calculate next version.
    ///
    /// This indicates whether this package should receive a major, minor,
    /// or patch version bump based on the changeset.
    pub bump_type: Option<VersionBump>,

    /// All files changed in this package.
    pub files: Vec<FileChange>,

    /// All commits affecting this package.
    ///
    /// For working directory analysis, this will be empty.
    /// For commit range analysis, this contains all commits in the range
    /// that modified files in this package.
    pub commits: Vec<CommitInfo>,

    /// Whether this package has any changes.
    pub has_changes: bool,

    /// Change statistics for this package.
    pub stats: PackageChangeStats,
}

impl PackageChanges {
    /// Creates a new `PackageChanges` for a package.
    ///
    /// # Arguments
    ///
    /// * `package_info` - Workspace package information
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::PackageChanges;
    /// use sublime_standard_tools::monorepo::WorkspacePackage;
    /// use std::path::PathBuf;
    ///
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    ///
    /// let changes = PackageChanges::new(workspace_pkg);
    /// assert!(!changes.has_changes);
    /// ```
    #[must_use]
    pub fn new(package_info: WorkspacePackage) -> Self {
        let package_name = package_info.name.clone();
        let package_version = package_info.version.clone();
        let package_location = package_info.location.clone();

        Self {
            package_info,
            package_name,
            package_version,
            package_location,
            current_version: None,
            next_version: None,
            bump_type: None,
            files: Vec::new(),
            commits: Vec::new(),
            has_changes: false,
            stats: PackageChangeStats::new(),
        }
    }

    /// Returns the package name.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::PackageChanges;
    /// # use sublime_standard_tools::monorepo::WorkspacePackage;
    /// # use std::path::PathBuf;
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    /// let changes = PackageChanges::new(workspace_pkg);
    /// assert_eq!(changes.package_name(), "@myorg/core");
    /// ```
    #[must_use]
    pub fn package_name(&self) -> &str {
        &self.package_info.name
    }

    /// Returns the package location relative to workspace root.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::PackageChanges;
    /// # use sublime_standard_tools::monorepo::WorkspacePackage;
    /// # use std::path::{Path, PathBuf};
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    /// let changes = PackageChanges::new(workspace_pkg);
    /// assert_eq!(changes.package_location(), Path::new("packages/core"));
    /// ```
    #[must_use]
    pub fn package_location(&self) -> &Path {
        &self.package_info.location
    }

    /// Adds a file change to this package.
    ///
    /// Automatically updates statistics and the `has_changes` flag.
    ///
    /// # Arguments
    ///
    /// * `file_change` - The file change to add
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::{PackageChanges, FileChange, FileChangeType};
    /// # use sublime_standard_tools::monorepo::WorkspacePackage;
    /// # use std::path::PathBuf;
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    /// let mut changes = PackageChanges::new(workspace_pkg);
    ///
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("packages/core/src/index.ts"),
    ///     PathBuf::from("src/index.ts"),
    ///     FileChangeType::Modified,
    /// ));
    ///
    /// assert!(changes.has_changes);
    /// assert_eq!(changes.files.len(), 1);
    /// ```
    pub fn add_file(&mut self, file_change: FileChange) {
        // Update statistics
        self.stats.files_changed += 1;
        match file_change.change_type {
            FileChangeType::Added | FileChangeType::Untracked | FileChangeType::Copied => {
                self.stats.files_added += 1;
            }
            FileChangeType::Modified | FileChangeType::Renamed => {
                self.stats.files_modified += 1;
            }
            FileChangeType::Deleted => {
                self.stats.files_deleted += 1;
            }
        }

        if let Some(added) = file_change.lines_added {
            self.stats.lines_added += added;
        }
        if let Some(deleted) = file_change.lines_deleted {
            self.stats.lines_deleted += deleted;
        }

        self.files.push(file_change);
        self.has_changes = true;
    }

    /// Adds a commit to this package.
    ///
    /// Updates the commit count in statistics.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit to add
    pub fn add_commit(&mut self, commit: CommitInfo) {
        self.commits.push(commit);
        self.stats.commits = self.commits.len();
    }

    /// Gets files by change type.
    ///
    /// # Arguments
    ///
    /// * `change_type` - The type of change to filter by
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::{PackageChanges, FileChange, FileChangeType};
    /// # use sublime_standard_tools::monorepo::WorkspacePackage;
    /// # use std::path::PathBuf;
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    /// let mut changes = PackageChanges::new(workspace_pkg);
    ///
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("file1.ts"),
    ///     PathBuf::from("file1.ts"),
    ///     FileChangeType::Added,
    /// ));
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("file2.ts"),
    ///     PathBuf::from("file2.ts"),
    ///     FileChangeType::Modified,
    /// ));
    ///
    /// let added = changes.files_by_type(FileChangeType::Added);
    /// assert_eq!(added.len(), 1);
    /// ```
    #[must_use]
    pub fn files_by_type(&self, change_type: FileChangeType) -> Vec<&FileChange> {
        self.files.iter().filter(|f| f.change_type == change_type).collect()
    }

    /// Checks if package.json was modified.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::{PackageChanges, FileChange, FileChangeType};
    /// # use sublime_standard_tools::monorepo::WorkspacePackage;
    /// # use std::path::PathBuf;
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    /// let mut changes = PackageChanges::new(workspace_pkg);
    ///
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("packages/core/package.json"),
    ///     PathBuf::from("package.json"),
    ///     FileChangeType::Modified,
    /// ));
    ///
    /// assert!(changes.package_json_modified());
    /// ```
    #[must_use]
    pub fn package_json_modified(&self) -> bool {
        self.files.iter().any(|f| f.is_package_json())
    }

    /// Groups files by directory (relative to package root).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::{PackageChanges, FileChange, FileChangeType};
    /// # use sublime_standard_tools::monorepo::WorkspacePackage;
    /// # use std::path::PathBuf;
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    /// let mut changes = PackageChanges::new(workspace_pkg);
    ///
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("packages/core/src/index.ts"),
    ///     PathBuf::from("src/index.ts"),
    ///     FileChangeType::Modified,
    /// ));
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("packages/core/src/utils.ts"),
    ///     PathBuf::from("src/utils.ts"),
    ///     FileChangeType::Added,
    /// ));
    ///
    /// let by_dir = changes.files_by_directory();
    /// assert!(by_dir.contains_key(std::path::Path::new("src")));
    /// ```
    #[must_use]
    pub fn files_by_directory(&self) -> HashMap<PathBuf, Vec<&FileChange>> {
        let mut result: HashMap<PathBuf, Vec<&FileChange>> = HashMap::new();

        for file in &self.files {
            let dir = file.package_relative_dir().unwrap_or_else(|| Path::new("")).to_path_buf();
            result.entry(dir).or_default().push(file);
        }

        result
    }

    /// Returns files grouped by extension.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::{PackageChanges, FileChange, FileChangeType};
    /// # use sublime_standard_tools::monorepo::WorkspacePackage;
    /// # use std::path::PathBuf;
    /// let workspace_pkg = WorkspacePackage::new(
    ///     "@myorg/core".to_string(),
    ///     "1.0.0".to_string(),
    ///     PathBuf::from("packages/core"),
    ///     PathBuf::from("/workspace/packages/core"),
    /// );
    /// let mut changes = PackageChanges::new(workspace_pkg);
    ///
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("file1.ts"),
    ///     PathBuf::from("file1.ts"),
    ///     FileChangeType::Modified,
    /// ));
    /// changes.add_file(FileChange::new(
    ///     PathBuf::from("file2.json"),
    ///     PathBuf::from("file2.json"),
    ///     FileChangeType::Modified,
    /// ));
    ///
    /// let by_ext = changes.files_by_extension();
    /// assert_eq!(by_ext.get("ts").map(|v| v.len()), Some(1));
    /// ```
    #[must_use]
    pub fn files_by_extension(&self) -> HashMap<String, Vec<&FileChange>> {
        let mut result: HashMap<String, Vec<&FileChange>> = HashMap::new();

        for file in &self.files {
            if let Some(ext) = file.extension() {
                result.entry(ext.to_string()).or_default().push(file);
            } else {
                result.entry("(no extension)".to_string()).or_default().push(file);
            }
        }

        result
    }

    /// Returns all added files.
    #[must_use]
    pub fn added_files(&self) -> Vec<&FileChange> {
        self.files.iter().filter(|f| f.is_addition()).collect()
    }

    /// Returns all modified files.
    #[must_use]
    pub fn modified_files(&self) -> Vec<&FileChange> {
        self.files.iter().filter(|f| f.is_modification()).collect()
    }

    /// Returns all deleted files.
    #[must_use]
    pub fn deleted_files(&self) -> Vec<&FileChange> {
        self.files.iter().filter(|f| f.is_deletion()).collect()
    }
}

/// Default WorkspacePackage for deserialization.
fn default_workspace_package() -> WorkspacePackage {
    WorkspacePackage {
        name: String::new(),
        version: String::new(),
        location: PathBuf::new(),
        absolute_path: PathBuf::new(),
        workspace_dependencies: Vec::new(),
        workspace_dev_dependencies: Vec::new(),
    }
}
