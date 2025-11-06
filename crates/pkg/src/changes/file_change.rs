//! File change data structures for tracking individual file modifications.
//!
//! **What**: Provides types for representing individual file changes, including the type
//! of change (added, modified, deleted, renamed, copied), file paths, and associated statistics.
//!
//! **How**: Defines serializable structures that map Git file status changes to domain types,
//! tracking both absolute and package-relative paths, change types, line statistics, and
//! associated commit information.
//!
//! **Why**: To provide detailed, type-safe information about file-level changes that can be
//! aggregated into package-level change reports, enabling accurate tracking of what changed
//! and supporting changelog generation and version bumping decisions.
//!
//! # Examples
//!
//! ## Creating a file change
//!
//! ```rust
//! use sublime_pkg_tools::changes::{FileChange, FileChangeType};
//! use std::path::PathBuf;
//!
//! let change = FileChange {
//!     path: PathBuf::from("packages/core/src/index.ts"),
//!     package_relative_path: PathBuf::from("src/index.ts"),
//!     change_type: FileChangeType::Modified,
//!     lines_added: Some(15),
//!     lines_deleted: Some(3),
//!     commits: vec!["abc123".to_string()],
//! };
//!
//! assert_eq!(change.change_type, FileChangeType::Modified);
//! assert_eq!(change.lines_added, Some(15));
//! ```
//!
//! ## Filtering by change type
//!
//! ```rust
//! use sublime_pkg_tools::changes::{FileChange, FileChangeType};
//! use std::path::PathBuf;
//!
//! let changes = vec![
//!     FileChange {
//!         path: PathBuf::from("file1.ts"),
//!         package_relative_path: PathBuf::from("file1.ts"),
//!         change_type: FileChangeType::Added,
//!         lines_added: Some(100),
//!         lines_deleted: None,
//!         commits: vec![],
//!     },
//!     FileChange {
//!         path: PathBuf::from("file2.ts"),
//!         package_relative_path: PathBuf::from("file2.ts"),
//!         change_type: FileChangeType::Modified,
//!         lines_added: Some(10),
//!         lines_deleted: Some(5),
//!         commits: vec![],
//!     },
//! ];
//!
//! let added: Vec<_> = changes.iter()
//!     .filter(|c| c.change_type == FileChangeType::Added)
//!     .collect();
//! assert_eq!(added.len(), 1);
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sublime_git_tools::GitFileStatus;

/// Type of change applied to a file.
///
/// Maps directly to Git status indicators and represents the operation
/// performed on a file in the repository.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::changes::FileChangeType;
///
/// let change_type = FileChangeType::Modified;
/// assert!(change_type.is_modification());
/// assert!(!change_type.is_addition());
///
/// let added = FileChangeType::Added;
/// assert!(added.is_addition());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeType {
    /// File was newly added to the repository.
    Added,
    /// File was modified (content changed).
    Modified,
    /// File was deleted from the repository.
    Deleted,
    /// File was renamed or moved.
    Renamed,
    /// File was copied to a new location.
    Copied,
    /// File is untracked (not yet added to Git).
    Untracked,
}

impl FileChangeType {
    /// Returns whether this change type represents an addition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::FileChangeType;
    ///
    /// assert!(FileChangeType::Added.is_addition());
    /// assert!(FileChangeType::Untracked.is_addition());
    /// assert!(!FileChangeType::Modified.is_addition());
    /// ```
    #[must_use]
    pub fn is_addition(&self) -> bool {
        matches!(self, Self::Added | Self::Untracked | Self::Copied)
    }

    /// Returns whether this change type represents a modification.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::FileChangeType;
    ///
    /// assert!(FileChangeType::Modified.is_modification());
    /// assert!(FileChangeType::Renamed.is_modification());
    /// assert!(!FileChangeType::Added.is_modification());
    /// ```
    #[must_use]
    pub fn is_modification(&self) -> bool {
        matches!(self, Self::Modified | Self::Renamed)
    }

    /// Returns whether this change type represents a deletion.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::FileChangeType;
    ///
    /// assert!(FileChangeType::Deleted.is_deletion());
    /// assert!(!FileChangeType::Modified.is_deletion());
    /// ```
    #[must_use]
    pub fn is_deletion(&self) -> bool {
        matches!(self, Self::Deleted)
    }

    /// Converts from `sublime_git_tools::GitFileStatus` to `FileChangeType`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::FileChangeType;
    /// use sublime_git_tools::GitFileStatus;
    ///
    /// let change_type = FileChangeType::from_git_status(&GitFileStatus::Modified);
    /// assert_eq!(change_type, FileChangeType::Modified);
    /// ```
    #[must_use]
    pub fn from_git_status(status: &GitFileStatus) -> Self {
        match status {
            GitFileStatus::Added => Self::Added,
            GitFileStatus::Modified => Self::Modified,
            GitFileStatus::Deleted => Self::Deleted,
            GitFileStatus::Untracked => Self::Untracked,
        }
    }
}

/// Details of a single file change.
///
/// Contains comprehensive information about what happened to a file, including
/// paths (both absolute and package-relative), the type of change, line statistics,
/// and associated commit information.
///
/// # Examples
///
/// ## Creating a file change
///
/// ```rust
/// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
/// use std::path::PathBuf;
///
/// let change = FileChange {
///     path: PathBuf::from("packages/core/src/api.ts"),
///     package_relative_path: PathBuf::from("src/api.ts"),
///     change_type: FileChangeType::Added,
///     lines_added: Some(50),
///     lines_deleted: None,
///     commits: vec![],
/// };
///
/// assert!(change.is_addition());
/// assert_eq!(change.net_line_change(), Some(50));
/// ```
///
/// ## Checking for package.json changes
///
/// ```rust
/// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
/// use std::path::PathBuf;
///
/// let change = FileChange {
///     path: PathBuf::from("packages/core/package.json"),
///     package_relative_path: PathBuf::from("package.json"),
///     change_type: FileChangeType::Modified,
///     lines_added: Some(2),
///     lines_deleted: Some(1),
///     commits: vec![],
/// };
///
/// assert!(change.is_package_json());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    /// File path relative to workspace root.
    ///
    /// This is the full path from the workspace root to the file.
    /// Example: `packages/core/src/index.ts`
    pub path: PathBuf,

    /// File path relative to package root.
    ///
    /// This is the path from the package's root directory to the file.
    /// Example: `src/index.ts` (when the package is at `packages/core`)
    pub package_relative_path: PathBuf,

    /// Type of change applied to the file.
    pub change_type: FileChangeType,

    /// Number of lines added (if available).
    ///
    /// This information may not be available for certain operations like
    /// working directory analysis without computing diffs. It will be `None`
    /// for deleted files.
    pub lines_added: Option<usize>,

    /// Number of lines deleted (if available).
    ///
    /// This information may not be available for certain operations like
    /// working directory analysis without computing diffs. It will be `None`
    /// for newly added files.
    pub lines_deleted: Option<usize>,

    /// Commit hashes that modified this file.
    ///
    /// For working directory analysis, this will be empty. For commit range
    /// analysis, this contains all commits in the range that modified this file.
    pub commits: Vec<String>,
}

impl FileChange {
    /// Creates a new `FileChange` with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `path` - Full path from workspace root
    /// * `package_relative_path` - Path relative to package root
    /// * `change_type` - Type of change
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let change = FileChange::new(
    ///     PathBuf::from("packages/core/src/index.ts"),
    ///     PathBuf::from("src/index.ts"),
    ///     FileChangeType::Modified,
    /// );
    ///
    /// assert_eq!(change.change_type, FileChangeType::Modified);
    /// assert!(change.commits.is_empty());
    /// ```
    #[must_use]
    pub fn new(path: PathBuf, package_relative_path: PathBuf, change_type: FileChangeType) -> Self {
        Self {
            path,
            package_relative_path,
            change_type,
            lines_added: None,
            lines_deleted: None,
            commits: Vec::new(),
        }
    }

    /// Returns whether this change represents an addition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let change = FileChange::new(
    ///     PathBuf::from("file.ts"),
    ///     PathBuf::from("file.ts"),
    ///     FileChangeType::Added,
    /// );
    ///
    /// assert!(change.is_addition());
    /// ```
    #[must_use]
    pub fn is_addition(&self) -> bool {
        self.change_type.is_addition()
    }

    /// Returns whether this change represents a modification.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let change = FileChange::new(
    ///     PathBuf::from("file.ts"),
    ///     PathBuf::from("file.ts"),
    ///     FileChangeType::Modified,
    /// );
    ///
    /// assert!(change.is_modification());
    /// ```
    #[must_use]
    pub fn is_modification(&self) -> bool {
        self.change_type.is_modification()
    }

    /// Returns whether this change represents a deletion.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let change = FileChange::new(
    ///     PathBuf::from("file.ts"),
    ///     PathBuf::from("file.ts"),
    ///     FileChangeType::Deleted,
    /// );
    ///
    /// assert!(change.is_deletion());
    /// ```
    #[must_use]
    pub fn is_deletion(&self) -> bool {
        self.change_type.is_deletion()
    }

    /// Returns whether this file is a package.json file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let change = FileChange::new(
    ///     PathBuf::from("packages/core/package.json"),
    ///     PathBuf::from("package.json"),
    ///     FileChangeType::Modified,
    /// );
    ///
    /// assert!(change.is_package_json());
    /// ```
    #[must_use]
    pub fn is_package_json(&self) -> bool {
        self.package_relative_path.file_name().and_then(|name| name.to_str())
            == Some("package.json")
    }

    /// Calculates the net line change (added - deleted).
    ///
    /// Returns `None` if line statistics are not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let mut change = FileChange::new(
    ///     PathBuf::from("file.ts"),
    ///     PathBuf::from("file.ts"),
    ///     FileChangeType::Modified,
    /// );
    /// change.lines_added = Some(20);
    /// change.lines_deleted = Some(5);
    ///
    /// assert_eq!(change.net_line_change(), Some(15));
    /// ```
    #[must_use]
    pub fn net_line_change(&self) -> Option<i64> {
        match (self.lines_added, self.lines_deleted) {
            (Some(added), Some(deleted)) => Some(added as i64 - deleted as i64),
            (Some(added), None) => Some(added as i64),
            (None, Some(deleted)) => Some(-(deleted as i64)),
            (None, None) => None,
        }
    }

    /// Returns the total number of line changes (added + deleted).
    ///
    /// Returns `None` if line statistics are not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let mut change = FileChange::new(
    ///     PathBuf::from("file.ts"),
    ///     PathBuf::from("file.ts"),
    ///     FileChangeType::Modified,
    /// );
    /// change.lines_added = Some(20);
    /// change.lines_deleted = Some(5);
    ///
    /// assert_eq!(change.total_line_changes(), Some(25));
    /// ```
    #[must_use]
    pub fn total_line_changes(&self) -> Option<usize> {
        match (self.lines_added, self.lines_deleted) {
            (Some(added), Some(deleted)) => Some(added + deleted),
            (Some(added), None) => Some(added),
            (None, Some(deleted)) => Some(deleted),
            (None, None) => None,
        }
    }

    /// Returns the file extension if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::PathBuf;
    ///
    /// let change = FileChange::new(
    ///     PathBuf::from("packages/core/src/index.ts"),
    ///     PathBuf::from("src/index.ts"),
    ///     FileChangeType::Modified,
    /// );
    ///
    /// assert_eq!(change.extension(), Some("ts"));
    /// ```
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|ext| ext.to_str())
    }

    /// Returns the parent directory of the file relative to the package root.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{FileChange, FileChangeType};
    /// use std::path::{Path, PathBuf};
    ///
    /// let change = FileChange::new(
    ///     PathBuf::from("packages/core/src/api/index.ts"),
    ///     PathBuf::from("src/api/index.ts"),
    ///     FileChangeType::Modified,
    /// );
    ///
    /// assert_eq!(change.package_relative_dir(), Some(Path::new("src/api")));
    /// ```
    #[must_use]
    pub fn package_relative_dir(&self) -> Option<&std::path::Path> {
        self.package_relative_path.parent()
    }
}
