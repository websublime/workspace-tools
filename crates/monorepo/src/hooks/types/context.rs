//! Hook execution context types
//!
//! This module defines the execution context types that provide hooks with
//! information about the current Git operation and repository state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Context information provided to hooks during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecutionContext {
    /// The Git repository root path
    pub repository_root: PathBuf,

    /// Current branch name
    pub current_branch: String,

    /// Previous branch (for post-checkout hooks)
    pub previous_branch: Option<String>,

    /// Current commit hash
    pub current_commit: Option<String>,

    /// Previous commit hash
    pub previous_commit: Option<String>,

    /// List of files changed in this operation
    pub changed_files: Vec<String>,

    /// List of packages affected by the changes
    pub affected_packages: Vec<String>,

    /// Environment variables available during execution
    pub environment: HashMap<String, String>,

    /// Git operation type (commit, push, merge, etc.)
    pub operation_type: GitOperationType,

    /// Remote repository information (for push operations)
    pub remote_info: Option<RemoteInfo>,

    /// Commit information for operations involving commits
    pub commits: Vec<CommitInfo>,

    /// Whether this is a merge commit
    pub is_merge: bool,

    /// Working directory where the hook should execute
    pub working_directory: PathBuf,

    /// Additional metadata specific to the hook type
    pub metadata: HashMap<String, String>,
}

/// Git operation types that can trigger hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitOperationType {
    /// Commit operation (pre-commit, post-commit)
    Commit,
    /// Push operation (pre-push, post-push)
    Push,
    /// Merge operation (pre-merge-commit, post-merge)
    Merge,
    /// Rebase operation
    Rebase,
    /// Checkout operation (post-checkout)
    Checkout,
    /// Receive operation (pre-receive, post-receive)
    Receive,
    /// Update operation (pre-update, post-update)
    Update,
    /// Unknown operation type
    Unknown,
}

/// Remote repository information for push operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    /// Remote name (e.g., "origin")
    pub name: String,
    /// Remote URL
    pub url: String,
    /// Target branch being pushed to
    pub target_branch: String,
    /// Source branch being pushed from
    pub source_branch: String,
}

/// Individual commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Author email
    pub author_email: String,
    /// Author name
    pub author_name: String,
    /// Timestamp of the commit
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Files changed in this commit
    pub changed_files: Vec<String>,
}

impl Default for GitOperationType {
    fn default() -> Self {
        Self::Unknown
    }
}
