//! Commit information data structures for tracking commit metadata.
//!
//! **What**: Provides types for representing commit information including metadata,
//! affected packages, and change statistics.
//!
//! **How**: Defines serializable structures that wrap Git commit data with domain-specific
//! information about which packages were affected and what changed.
//!
//! **Why**: To provide comprehensive commit metadata that can be associated with package
//! changes, supporting changelog generation, version history, and traceability.
//!
//! # Examples
//!
//! ## Creating commit info
//!
//! ```rust
//! use sublime_pkg_tools::changes::CommitInfo;
//! use chrono::Utc;
//!
//! let commit = CommitInfo {
//!     hash: "abc123def456".to_string(),
//!     short_hash: "abc123d".to_string(),
//!     author: "John Doe".to_string(),
//!     author_email: "john@example.com".to_string(),
//!     date: Utc::now(),
//!     message: "feat: add new feature".to_string(),
//!     full_message: "feat: add new feature\n\nThis adds a new feature.".to_string(),
//!     affected_packages: vec!["@myorg/core".to_string()],
//!     files_changed: 5,
//!     lines_added: 100,
//!     lines_deleted: 20,
//! };
//!
//! assert_eq!(commit.short_hash, "abc123d");
//! assert!(commit.affects_package("@myorg/core"));
//! ```
//!
//! ## Checking commit properties
//!
//! ```rust
//! use sublime_pkg_tools::changes::CommitInfo;
//! use chrono::Utc;
//!
//! let commit = CommitInfo {
//!     hash: "abc123def456".to_string(),
//!     short_hash: "abc123d".to_string(),
//!     author: "John Doe".to_string(),
//!     author_email: "john@example.com".to_string(),
//!     date: Utc::now(),
//!     message: "Merge pull request #123".to_string(),
//!     full_message: "Merge pull request #123 from feature/test".to_string(),
//!     affected_packages: vec![],
//!     files_changed: 0,
//!     lines_added: 0,
//!     lines_deleted: 0,
//! };
//!
//! assert!(commit.is_merge_commit());
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a commit affecting packages.
///
/// Contains comprehensive metadata about a commit including author information,
/// timestamps, commit message, affected packages, and change statistics.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::changes::CommitInfo;
/// use chrono::Utc;
///
/// let commit = CommitInfo {
///     hash: "abc123def456".to_string(),
///     short_hash: "abc123d".to_string(),
///     author: "Jane Smith".to_string(),
///     author_email: "jane@example.com".to_string(),
///     date: Utc::now(),
///     message: "fix: resolve bug in parser".to_string(),
///     full_message: "fix: resolve bug in parser\n\nFixes #456".to_string(),
///     affected_packages: vec!["@myorg/parser".to_string()],
///     files_changed: 3,
///     lines_added: 20,
///     lines_deleted: 10,
/// };
///
/// assert_eq!(commit.net_lines_changed(), 10);
/// assert!(commit.affects_package("@myorg/parser"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Full commit hash (SHA).
    pub hash: String,

    /// Short commit hash (first 7 characters).
    pub short_hash: String,

    /// Commit author name.
    pub author: String,

    /// Commit author email.
    pub author_email: String,

    /// Commit timestamp.
    pub date: DateTime<Utc>,

    /// Commit message (first line/subject).
    pub message: String,

    /// Full commit message including body.
    pub full_message: String,

    /// List of package names affected by this commit.
    ///
    /// A single commit can affect multiple packages in a monorepo.
    pub affected_packages: Vec<String>,

    /// Number of files changed in this commit.
    pub files_changed: usize,

    /// Total lines added in this commit.
    pub lines_added: usize,

    /// Total lines deleted in this commit.
    pub lines_deleted: usize,
}

impl CommitInfo {
    /// Creates a new `CommitInfo` with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `hash` - Full commit hash
    /// * `author` - Author name
    /// * `author_email` - Author email
    /// * `date` - Commit timestamp
    /// * `message` - Commit message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::CommitInfo;
    /// use chrono::Utc;
    ///
    /// let commit = CommitInfo::new(
    ///     "abc123def456".to_string(),
    ///     "John Doe".to_string(),
    ///     "john@example.com".to_string(),
    ///     Utc::now(),
    ///     "feat: add feature".to_string(),
    /// );
    ///
    /// assert_eq!(commit.hash, "abc123def456");
    /// assert_eq!(commit.short_hash, "abc123d");
    /// ```
    #[must_use]
    pub fn new(
        hash: String,
        author: String,
        author_email: String,
        date: DateTime<Utc>,
        message: String,
    ) -> Self {
        let short_hash = if hash.len() >= 7 { hash[..7].to_string() } else { hash.clone() };

        Self {
            hash,
            short_hash,
            author,
            author_email,
            date,
            full_message: message.clone(),
            message: message.lines().next().unwrap_or("").to_string(),
            affected_packages: Vec::new(),
            files_changed: 0,
            lines_added: 0,
            lines_deleted: 0,
        }
    }

    /// Creates a `CommitInfo` from `sublime_git_tools::RepoCommit`.
    ///
    /// # Arguments
    ///
    /// * `commit` - The Git commit from sublime_git_tools
    /// * `affected_packages` - List of package names affected by this commit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::CommitInfo;
    /// use sublime_git_tools::RepoCommit;
    ///
    /// let git_commit = RepoCommit {
    ///     hash: "abc123".to_string(),
    ///     author_name: "John".to_string(),
    ///     author_email: "john@example.com".to_string(),
    ///     author_date: "2024-01-01".to_string(),
    ///     message: "feat: add feature".to_string(),
    /// };
    ///
    /// let commit_info = CommitInfo::from_git_commit(&git_commit, vec!["@myorg/core".to_string()]);
    /// ```
    ///
    /// # Note
    ///
    /// This will be implemented in Story 7.4 when commit range analysis is added.
    /// For now, this is a placeholder for working directory analysis.
    #[must_use]
    pub fn from_git_commit(
        _commit: &sublime_git_tools::RepoCommit,
        affected_packages: Vec<String>,
    ) -> Self {
        // TODO: will be implemented on story 7.4
        Self {
            hash: String::new(),
            short_hash: String::new(),
            author: String::new(),
            author_email: String::new(),
            date: Utc::now(),
            message: String::new(),
            full_message: String::new(),
            affected_packages,
            files_changed: 0,
            lines_added: 0,
            lines_deleted: 0,
        }
    }

    /// Checks if this commit is a merge commit.
    ///
    /// Detects merge commits by looking for common merge commit message patterns.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::CommitInfo;
    /// use chrono::Utc;
    ///
    /// let merge_commit = CommitInfo {
    ///     hash: "abc123".to_string(),
    ///     short_hash: "abc123".to_string(),
    ///     author: "John".to_string(),
    ///     author_email: "john@example.com".to_string(),
    ///     date: Utc::now(),
    ///     message: "Merge pull request #123".to_string(),
    ///     full_message: "Merge pull request #123".to_string(),
    ///     affected_packages: vec![],
    ///     files_changed: 0,
    ///     lines_added: 0,
    ///     lines_deleted: 0,
    /// };
    ///
    /// assert!(merge_commit.is_merge_commit());
    /// ```
    #[must_use]
    pub fn is_merge_commit(&self) -> bool {
        self.message.starts_with("Merge ")
            || self.message.starts_with("Merge pull request")
            || self.message.starts_with("Merge branch")
    }

    /// Checks if this commit affects a specific package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::CommitInfo;
    /// use chrono::Utc;
    ///
    /// let commit = CommitInfo {
    ///     hash: "abc123".to_string(),
    ///     short_hash: "abc123".to_string(),
    ///     author: "John".to_string(),
    ///     author_email: "john@example.com".to_string(),
    ///     date: Utc::now(),
    ///     message: "feat: add feature".to_string(),
    ///     full_message: "feat: add feature".to_string(),
    ///     affected_packages: vec!["@myorg/core".to_string(), "@myorg/utils".to_string()],
    ///     files_changed: 5,
    ///     lines_added: 100,
    ///     lines_deleted: 20,
    /// };
    ///
    /// assert!(commit.affects_package("@myorg/core"));
    /// assert!(commit.affects_package("@myorg/utils"));
    /// assert!(!commit.affects_package("@myorg/other"));
    /// ```
    #[must_use]
    pub fn affects_package(&self, package_name: &str) -> bool {
        self.affected_packages.iter().any(|p| p == package_name)
    }

    /// Calculates the net line change (added - deleted).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::CommitInfo;
    /// use chrono::Utc;
    ///
    /// let commit = CommitInfo {
    ///     hash: "abc123".to_string(),
    ///     short_hash: "abc123".to_string(),
    ///     author: "John".to_string(),
    ///     author_email: "john@example.com".to_string(),
    ///     date: Utc::now(),
    ///     message: "feat: add feature".to_string(),
    ///     full_message: "feat: add feature".to_string(),
    ///     affected_packages: vec![],
    ///     files_changed: 5,
    ///     lines_added: 100,
    ///     lines_deleted: 20,
    /// };
    ///
    /// assert_eq!(commit.net_lines_changed(), 80);
    /// ```
    #[must_use]
    pub fn net_lines_changed(&self) -> i64 {
        self.lines_added as i64 - self.lines_deleted as i64
    }

    /// Returns the total number of line changes (added + deleted).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::CommitInfo;
    /// use chrono::Utc;
    ///
    /// let commit = CommitInfo {
    ///     hash: "abc123".to_string(),
    ///     short_hash: "abc123".to_string(),
    ///     author: "John".to_string(),
    ///     author_email: "john@example.com".to_string(),
    ///     date: Utc::now(),
    ///     message: "feat: add feature".to_string(),
    ///     full_message: "feat: add feature".to_string(),
    ///     affected_packages: vec![],
    ///     files_changed: 5,
    ///     lines_added: 100,
    ///     lines_deleted: 20,
    /// };
    ///
    /// assert_eq!(commit.total_lines_changed(), 120);
    /// ```
    #[must_use]
    pub fn total_lines_changed(&self) -> usize {
        self.lines_added + self.lines_deleted
    }
}
