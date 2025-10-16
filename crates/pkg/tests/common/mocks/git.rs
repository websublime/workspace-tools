//! # Mock Git Repository Implementation
//!
//! This module provides an in-memory mock Git repository implementation for testing.
//!
//! ## What
//!
//! `MockGitRepository` is an in-memory implementation that simulates Git operations
//! for testing purposes without requiring an actual Git repository.
//!
//! ## How
//!
//! Commits and file changes are stored in memory. The mock provides methods to
//! add commits, track file changes, and query repository state.
//!
//! ## Why
//!
//! Mock Git repository provides:
//! - Fast test execution without Git overhead
//! - Predictable test behavior
//! - Easy setup of complex Git histories
//! - Ability to test error conditions

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// In-memory mock Git repository for testing
///
/// This struct maintains an in-memory representation of a Git repository,
/// allowing tests to run without a real Git repository.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::common::mocks::MockGitRepository;
///
/// let repo = MockGitRepository::new("/repo");
/// repo.add_commit("abc123", "Initial commit", vec!["/file.txt"]);
/// let commits = repo.get_commits();
/// assert_eq!(commits.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct MockGitRepository {
    /// The root path of the repository
    root_path: PathBuf,
    /// All commits in the repository
    commits: Arc<Mutex<Vec<MockCommit>>>,
    /// Current branch name
    current_branch: Arc<Mutex<String>>,
    /// All branches in the repository
    branches: Arc<Mutex<HashMap<String, String>>>,
    /// Tags in the repository
    tags: Arc<Mutex<HashMap<String, String>>>,
    /// Working directory changes
    working_changes: Arc<Mutex<Vec<PathBuf>>>,
}

/// Represents a commit in the mock repository
///
/// This struct contains all the information about a single commit.
#[derive(Debug, Clone)]
pub struct MockCommit {
    /// The commit hash
    pub hash: String,
    /// Short version of the hash (first 7 characters)
    pub short_hash: String,
    /// The commit message
    pub message: String,
    /// Full commit message (with body)
    pub full_message: String,
    /// The author name
    pub author: String,
    /// The author email
    pub author_email: String,
    /// The commit date
    pub date: DateTime<Utc>,
    /// Files changed in this commit
    pub files_changed: Vec<PathBuf>,
    /// Lines added in this commit
    pub lines_added: usize,
    /// Lines deleted in this commit
    pub lines_deleted: usize,
    /// Parent commit hashes
    pub parents: Vec<String>,
}

impl MockCommit {
    /// Creates a new mock commit with default values
    ///
    /// # Arguments
    ///
    /// * `hash` - The commit hash
    /// * `message` - The commit message
    /// * `files` - Files changed in this commit
    ///
    /// # Returns
    ///
    /// A new `MockCommit` instance
    pub fn new(hash: impl Into<String>, message: impl Into<String>, files: Vec<PathBuf>) -> Self {
        let hash = hash.into();
        let short_hash = hash.chars().take(7).collect();
        let message = message.into();

        Self {
            hash,
            short_hash,
            message: message.clone(),
            full_message: message,
            author: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            date: Utc::now(),
            files_changed: files,
            lines_added: 10,
            lines_deleted: 5,
            parents: vec![],
        }
    }

    /// Creates a builder for constructing a mock commit
    ///
    /// # Arguments
    ///
    /// * `hash` - The commit hash
    ///
    /// # Returns
    ///
    /// A new `MockCommitBuilder` instance
    pub fn builder(hash: impl Into<String>) -> MockCommitBuilder {
        MockCommitBuilder::new(hash)
    }

    /// Checks if this is a merge commit
    ///
    /// # Returns
    ///
    /// `true` if the commit has more than one parent
    pub fn is_merge_commit(&self) -> bool {
        self.parents.len() > 1
    }
}

/// Builder for constructing mock commits
#[derive(Debug)]
pub struct MockCommitBuilder {
    hash: String,
    message: Option<String>,
    full_message: Option<String>,
    author: Option<String>,
    author_email: Option<String>,
    date: Option<DateTime<Utc>>,
    files_changed: Vec<PathBuf>,
    lines_added: usize,
    lines_deleted: usize,
    parents: Vec<String>,
}

impl MockCommitBuilder {
    /// Creates a new builder
    fn new(hash: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            message: None,
            full_message: None,
            author: None,
            author_email: None,
            date: None,
            files_changed: vec![],
            lines_added: 10,
            lines_deleted: 5,
            parents: vec![],
        }
    }

    /// Sets the commit message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Sets the full commit message
    pub fn full_message(mut self, message: impl Into<String>) -> Self {
        self.full_message = Some(message.into());
        self
    }

    /// Sets the author name
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the author email
    pub fn author_email(mut self, email: impl Into<String>) -> Self {
        self.author_email = Some(email.into());
        self
    }

    /// Sets the commit date
    pub fn date(mut self, date: DateTime<Utc>) -> Self {
        self.date = Some(date);
        self
    }

    /// Sets the files changed
    pub fn files(mut self, files: Vec<PathBuf>) -> Self {
        self.files_changed = files;
        self
    }

    /// Adds a single file
    pub fn add_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.files_changed.push(file.into());
        self
    }

    /// Sets lines added
    pub fn lines_added(mut self, lines: usize) -> Self {
        self.lines_added = lines;
        self
    }

    /// Sets lines deleted
    pub fn lines_deleted(mut self, lines: usize) -> Self {
        self.lines_deleted = lines;
        self
    }

    /// Sets parent commits
    pub fn parents(mut self, parents: Vec<String>) -> Self {
        self.parents = parents;
        self
    }

    /// Adds a parent commit
    pub fn add_parent(mut self, parent: impl Into<String>) -> Self {
        self.parents.push(parent.into());
        self
    }

    /// Builds the mock commit
    pub fn build(self) -> MockCommit {
        let message = self.message.unwrap_or_else(|| "Test commit".to_string());
        let hash = self.hash;
        let short_hash = hash.chars().take(7).collect();

        MockCommit {
            hash,
            short_hash,
            message: message.clone(),
            full_message: self.full_message.unwrap_or(message),
            author: self.author.unwrap_or_else(|| "Test Author".to_string()),
            author_email: self.author_email.unwrap_or_else(|| "test@example.com".to_string()),
            date: self.date.unwrap_or_else(Utc::now),
            files_changed: self.files_changed,
            lines_added: self.lines_added,
            lines_deleted: self.lines_deleted,
            parents: self.parents,
        }
    }
}

impl MockGitRepository {
    /// Creates a new mock Git repository
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root path of the repository
    ///
    /// # Returns
    ///
    /// A new `MockGitRepository` instance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let repo = MockGitRepository::new("/repo");
    /// ```
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        let mut branches = HashMap::new();
        branches.insert("main".to_string(), "".to_string());

        Self {
            root_path: root_path.into(),
            commits: Arc::new(Mutex::new(Vec::new())),
            current_branch: Arc::new(Mutex::new("main".to_string())),
            branches: Arc::new(Mutex::new(branches)),
            tags: Arc::new(Mutex::new(HashMap::new())),
            working_changes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Gets the root path of the repository
    ///
    /// # Returns
    ///
    /// A reference to the root path
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Adds a commit to the repository
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit to add
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let repo = MockGitRepository::new("/repo");
    /// let commit = MockCommit::new("abc123", "Initial commit", vec![]);
    /// repo.add_commit_obj(commit);
    /// ```
    pub fn add_commit_obj(&self, commit: MockCommit) {
        let mut commits = self.commits.lock().unwrap();
        commits.push(commit);
    }

    /// Adds a simple commit to the repository
    ///
    /// # Arguments
    ///
    /// * `hash` - The commit hash
    /// * `message` - The commit message
    /// * `files` - Files changed in this commit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let repo = MockGitRepository::new("/repo");
    /// repo.add_commit("abc123", "Initial commit", vec![PathBuf::from("/file.txt")]);
    /// ```
    pub fn add_commit(
        &self,
        hash: impl Into<String>,
        message: impl Into<String>,
        files: Vec<PathBuf>,
    ) {
        let commit = MockCommit::new(hash, message, files);
        self.add_commit_obj(commit);
    }

    /// Gets all commits in the repository
    ///
    /// # Returns
    ///
    /// A vector of all commits
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let repo = MockGitRepository::new("/repo");
    /// let commits = repo.get_commits();
    /// ```
    pub fn get_commits(&self) -> Vec<MockCommit> {
        let commits = self.commits.lock().unwrap();
        commits.clone()
    }

    /// Gets commits in a specific range
    ///
    /// # Arguments
    ///
    /// * `from` - The starting commit hash (exclusive)
    /// * `to` - The ending commit hash (inclusive)
    ///
    /// # Returns
    ///
    /// A vector of commits in the range, or all commits if range is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let repo = MockGitRepository::new("/repo");
    /// let commits = repo.get_commits_range("abc123", "def456");
    /// ```
    pub fn get_commits_range(&self, from: &str, to: &str) -> Vec<MockCommit> {
        let commits = self.commits.lock().unwrap();

        let from_idx = commits.iter().position(|c| c.hash == from);
        let to_idx = commits.iter().position(|c| c.hash == to);

        match (from_idx, to_idx) {
            (Some(start), Some(end)) if start < end => commits[(start + 1)..=end].to_vec(),
            (None, Some(end)) => commits[..=end].to_vec(),
            _ => commits.clone(),
        }
    }

    /// Gets the current branch name
    ///
    /// # Returns
    ///
    /// The name of the current branch
    pub fn current_branch(&self) -> String {
        let branch = self.current_branch.lock().unwrap();
        branch.clone()
    }

    /// Sets the current branch
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name
    pub fn set_current_branch(&self, branch: impl Into<String>) {
        let mut current = self.current_branch.lock().unwrap();
        *current = branch.into();
    }

    /// Adds a branch to the repository
    ///
    /// # Arguments
    ///
    /// * `name` - The branch name
    /// * `commit_hash` - The commit hash the branch points to
    pub fn add_branch(&self, name: impl Into<String>, commit_hash: impl Into<String>) {
        let mut branches = self.branches.lock().unwrap();
        branches.insert(name.into(), commit_hash.into());
    }

    /// Gets all branches in the repository
    ///
    /// # Returns
    ///
    /// A map of branch names to commit hashes
    pub fn get_branches(&self) -> HashMap<String, String> {
        let branches = self.branches.lock().unwrap();
        branches.clone()
    }

    /// Adds a tag to the repository
    ///
    /// # Arguments
    ///
    /// * `name` - The tag name
    /// * `commit_hash` - The commit hash the tag points to
    pub fn add_tag(&self, name: impl Into<String>, commit_hash: impl Into<String>) {
        let mut tags = self.tags.lock().unwrap();
        tags.insert(name.into(), commit_hash.into());
    }

    /// Gets all tags in the repository
    ///
    /// # Returns
    ///
    /// A map of tag names to commit hashes
    pub fn get_tags(&self) -> HashMap<String, String> {
        let tags = self.tags.lock().unwrap();
        tags.clone()
    }

    /// Adds a file to working directory changes
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the changed file
    pub fn add_working_change(&self, path: impl Into<PathBuf>) {
        let mut changes = self.working_changes.lock().unwrap();
        changes.push(path.into());
    }

    /// Gets all working directory changes
    ///
    /// # Returns
    ///
    /// A vector of changed file paths
    pub fn get_working_changes(&self) -> Vec<PathBuf> {
        let changes = self.working_changes.lock().unwrap();
        changes.clone()
    }

    /// Clears all working directory changes
    pub fn clear_working_changes(&self) {
        let mut changes = self.working_changes.lock().unwrap();
        changes.clear();
    }

    /// Clears all commits from the repository
    pub fn clear_commits(&self) {
        let mut commits = self.commits.lock().unwrap();
        commits.clear();
    }

    /// Gets commits that affected a specific file
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path of the file
    ///
    /// # Returns
    ///
    /// A vector of commits that changed the file
    pub fn get_commits_for_file(&self, file_path: &Path) -> Vec<MockCommit> {
        let commits = self.commits.lock().unwrap();
        commits.iter().filter(|c| c.files_changed.iter().any(|f| f == file_path)).cloned().collect()
    }

    /// Gets the total number of commits
    ///
    /// # Returns
    ///
    /// The number of commits in the repository
    pub fn commit_count(&self) -> usize {
        let commits = self.commits.lock().unwrap();
        commits.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_repository() {
        let repo = MockGitRepository::new("/repo");
        assert_eq!(repo.root_path(), Path::new("/repo"));
        assert_eq!(repo.commit_count(), 0);
        assert_eq!(repo.current_branch(), "main");
    }

    #[test]
    fn test_add_commit() {
        let repo = MockGitRepository::new("/repo");
        repo.add_commit("abc123", "Initial commit", vec![PathBuf::from("/file.txt")]);

        let commits = repo.get_commits();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].message, "Initial commit");
    }

    #[test]
    fn test_commit_builder() {
        let commit = MockCommit::builder("abc123")
            .message("Test commit")
            .author("John Doe")
            .author_email("john@example.com")
            .add_file("/test.txt")
            .lines_added(20)
            .lines_deleted(10)
            .build();

        assert_eq!(commit.hash, "abc123");
        assert_eq!(commit.message, "Test commit");
        assert_eq!(commit.author, "John Doe");
        assert_eq!(commit.author_email, "john@example.com");
        assert_eq!(commit.files_changed.len(), 1);
        assert_eq!(commit.lines_added, 20);
        assert_eq!(commit.lines_deleted, 10);
    }

    #[test]
    fn test_commit_range() {
        let repo = MockGitRepository::new("/repo");
        repo.add_commit("commit1", "First", vec![]);
        repo.add_commit("commit2", "Second", vec![]);
        repo.add_commit("commit3", "Third", vec![]);

        let range = repo.get_commits_range("commit1", "commit3");
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].hash, "commit2");
        assert_eq!(range[1].hash, "commit3");
    }

    #[test]
    fn test_branches() {
        let repo = MockGitRepository::new("/repo");
        repo.add_branch("develop", "abc123");
        repo.add_branch("feature", "def456");

        let branches = repo.get_branches();
        assert_eq!(branches.len(), 3); // main + 2 added
        assert_eq!(branches.get("develop"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_tags() {
        let repo = MockGitRepository::new("/repo");
        repo.add_tag("v1.0.0", "abc123");
        repo.add_tag("v1.1.0", "def456");

        let tags = repo.get_tags();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags.get("v1.0.0"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_working_changes() {
        let repo = MockGitRepository::new("/repo");
        repo.add_working_change("/file1.txt");
        repo.add_working_change("/file2.txt");

        let changes = repo.get_working_changes();
        assert_eq!(changes.len(), 2);

        repo.clear_working_changes();
        let changes = repo.get_working_changes();
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_commits_for_file() {
        let repo = MockGitRepository::new("/repo");
        repo.add_commit("commit1", "First", vec![PathBuf::from("/file1.txt")]);
        repo.add_commit("commit2", "Second", vec![PathBuf::from("/file2.txt")]);
        repo.add_commit(
            "commit3",
            "Third",
            vec![PathBuf::from("/file1.txt"), PathBuf::from("/file3.txt")],
        );

        let commits = repo.get_commits_for_file(Path::new("/file1.txt"));
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].hash, "commit1");
        assert_eq!(commits[1].hash, "commit3");
    }

    #[test]
    fn test_is_merge_commit() {
        let commit = MockCommit::builder("abc123")
            .parents(vec!["parent1".to_string(), "parent2".to_string()])
            .build();

        assert!(commit.is_merge_commit());

        let regular_commit = MockCommit::builder("def456").build();
        assert!(!regular_commit.is_merge_commit());
    }
}
