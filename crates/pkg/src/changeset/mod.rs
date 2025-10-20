//! Changeset management module for tracking package changes and version bumps.
//!
//! **What**: Provides core functionality for creating, managing, storing, and querying changesets,
//! which are the source of truth for package versioning and release management.
//!
//! **How**: This module implements changeset storage (file-based and in-memory), changeset lifecycle
//! management (create, update, archive), history tracking, and Git integration for commit-based changes.
//!
//! **Why**: To maintain a clear, auditable record of what packages changed, what version bumps they need,
//! and what commits are associated with those changes, enabling reproducible releases and comprehensive
//! version history.
//!
//! # Core Concepts
//!
//! ## Changeset as Source of Truth
//!
//! A changeset is the central data structure that describes:
//! - Which packages have changes
//! - What type of version bump they need (major, minor, patch, none)
//! - Which environments the changes target
//! - What commits are associated with the changes
//! - When the changeset was created and last updated
//!
//! ## Changeset Lifecycle
//!
//! 1. **Create**: Initialize a new changeset for a branch with specific version bump and environments
//! 2. **Update**: Modify changeset properties (packages, bump type, environments)
//! 3. **Add Commits**: Associate Git commits with the changeset
//! 4. **Archive**: Move the changeset to history when released
//! 5. **Query**: Search through changeset history
//!
//! # Features
//!
//! - **Changeset Creation**: Create new changesets with branch, bump type, and environment targeting
//! - **Changeset Storage**: Pluggable storage system (file-based by default)
//! - **History Management**: Archive changesets and maintain searchable history
//! - **Git Integration**: Automatically add commits from Git and detect affected packages
//! - **Update Tracking**: Track when changesets are created and modified
//! - **Query API**: Search history by date, package, environment, or bump type
//! - **Validation**: Ensure changesets are valid before saving
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changeset::ChangesetManager;
//! use sublime_pkg_tools::types::{Changeset, VersionBump};
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//! let config = PackageToolsConfig::default();
//!
//! let manager = ChangesetManager::new(workspace_root, fs, config).await?;
//!
//! // Create a new changeset
//! let changeset = manager.create(
//!     "feature-branch",
//!     VersionBump::Minor,
//!     vec!["production".to_string()]
//! ).await?;
//!
//! // Load and update
//! let mut changeset = manager.load("feature-branch").await?;
//! changeset.add_package("my-package");
//! manager.update(&changeset).await?;
//!
//! // Add commits from Git (TODO: will be implemented on story 6.4)
//! // let summary = manager.add_commits_from_git("feature-branch", "main", "HEAD").await?;
//! // println!("Added {} commits", summary.commits_added);
//!
//! // Archive when released (TODO: will be implemented on story 6.5)
//! // manager.archive("feature-branch", release_info).await?;
//! # Ok(())
//! # }
//! ```</parameter>
//! ```
//!
//! # Storage System
//!
//! The module uses a trait-based storage system for flexibility:
//!
//! ```rust,ignore
//! # use sublime_pkg_tools::changeset::ChangesetStorage;
//! # use sublime_pkg_tools::types::{Changeset, ArchivedChangeset};
//! # use sublime_pkg_tools::error::ChangesetResult;
//! # use async_trait::async_trait;
//! #
//! // Custom storage implementation
//! struct DatabaseStorage {
//!     // database connection
//! }
//!
//! #[async_trait]
//! impl ChangesetStorage for DatabaseStorage {
//!     async fn save(&self, changeset: &Changeset) -> ChangesetResult<()> {
//!         // TODO: will be implemented on story 6.1
//!         todo!("Store changeset in database")
//!     }
//!
//!     async fn load(&self, branch: &str) -> ChangesetResult<Changeset> {
//!         // TODO: will be implemented on story 6.1
//!         todo!("Load changeset from database")
//!     }
//!
//!     // ... other trait methods
//! #   async fn exists(&self, branch: &str) -> ChangesetResult<bool> { todo!() }
//! #   async fn delete(&self, branch: &str) -> ChangesetResult<()> { todo!() }
//! #   async fn list_pending(&self) -> ChangesetResult<Vec<String>> { todo!() }
//! #   async fn archive(&self, changeset: &Changeset, release_info: crate::types::ReleaseInfo) -> ChangesetResult<()> { todo!() }
//! #   async fn load_archived(&self, id: &str) -> ChangesetResult<ArchivedChangeset> { todo!() }
//! #   async fn list_archived(&self) -> ChangesetResult<Vec<String>> { todo!() }
//! }
//! ```
//!
//! # History Queries
//!
//! Query archived changesets using flexible filters:
//!
//! ```rust,ignore
//! # use sublime_pkg_tools::changeset::ChangesetHistory;
//! # use sublime_standard_tools::filesystem::FileSystemManager;
//! # use std::path::PathBuf;
//! # use chrono::{Utc, Duration};
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let workspace_root = PathBuf::from(".");
//! # let fs = FileSystemManager::new();
//! // TODO: will be implemented on story 6.5
//! // let history = ChangesetHistory::new(workspace_root, fs).await?;
//! //
//! // // Query by date range
//! // let start = Utc::now() - Duration::days(30);
//! // let end = Utc::now();
//! // let recent = history.query_by_date(start, end).await?;
//! //
//! // // Query by package
//! // let pkg_history = history.query_by_package("my-package").await?;
//! //
//! // // Query by environment
//! // let prod_releases = history.query_by_environment("production").await?;
//! # Ok(())
//! # }
//! ```</parameter>
//! ```
//!
//! # Git Integration
//!
//! Automatically detect affected packages from Git commits:
//!
//! ```rust,ignore
//! # use sublime_pkg_tools::changeset::ChangesetManager;
//! # use sublime_pkg_tools::config::PackageToolsConfig;
//! # use sublime_standard_tools::filesystem::FileSystemManager;
//! # use std::path::PathBuf;
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let workspace_root = PathBuf::from(".");
//! # let fs = FileSystemManager::new();
//! # let config = PackageToolsConfig::default();
//! let manager = ChangesetManager::new(workspace_root, fs, config).await?;
//!
//! // TODO: will be implemented on story 6.4
//! // // Automatically detect affected packages from commits
//! // let summary = manager.add_commits_from_git("feature-branch", "main", "HEAD").await?;
//! //
//! // println!("Commits added: {}", summary.commits_added);
//! // println!("New packages: {:?}", summary.new_packages);
//! // println!("Existing packages: {:?}", summary.existing_packages);
//! # Ok(())
//! # }
//! ```</parameter>
//! ```
//!
//! # Module Structure
//!
//! This module will contain:
//! - `manager`: The main `ChangesetManager` for orchestrating changeset operations
//! - `storage`: Storage trait and implementations (file-based, in-memory)
//! - `history`: History query API and archived changeset management
//! - `update_summary`: Summary information for changeset updates
//! - `package_detector`: Git integration for detecting affected packages

#![allow(clippy::todo)]

// Internal modules
mod git_integration;
mod manager;
mod storage;

#[cfg(test)]
mod tests;

// Public API - re-exports
pub use git_integration::PackageDetector;
pub use manager::ChangesetManager;
pub use storage::{ChangesetStorage, FileBasedChangesetStorage};

// Additional modules will be implemented in subsequent stories (Epic 6)
// - history: Story 6.5
// - update_summary: Story 6.4</parameter>
