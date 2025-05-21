//! # sublime_git_tools
//!
//! A high-level Rust interface to Git operations with robust error handling, built on libgit2.
//!
//! ## Overview
//!
//! `sublime_git_tools` provides a user-friendly API for working with Git repositories. It wraps the
//! powerful but complex libgit2 library to offer a more ergonomic interface for common Git operations.
//!
//! This crate is designed for Rust applications that need to:
//!
//! - Create, clone, or manipulate Git repositories
//! - Manage branches, commits, and tags
//! - Track file changes between commits or branches
//! - Push/pull with remote repositories
//! - Get detailed commit histories
//! - Detect changes in specific parts of a repository
//!
//! ## Main Features
//!
//! ### Repository Management
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new repository
//! let repo = Repo::create("/path/to/new/repo")?;
//!
//! // Open an existing repository
//! let repo = Repo::open("./my-project")?;
//!
//! // Clone a remote repository
//! let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Branch and Commit Operations
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let repo = Repo::create("/tmp/example")?;
//! // Create a new branch
//! repo.create_branch("feature/new-feature")?;
//!
//! // Checkout a branch
//! repo.checkout("feature/new-feature")?;
//!
//! // Add files and commit
//! repo.add("src/main.rs")?;
//! let commit_id = repo.commit("feat: update main.rs")?;
//!
//! // Or add all changes and commit in one step
//! let commit_id = repo.commit_changes("feat: implement new feature")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### File Change Detection
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let repo = Repo::create("/tmp/example")?;
//! // Get all changed files since a tag or commit
//! let changed_files = repo.get_all_files_changed_since_sha("HEAD~1")?;
//!
//! // Get all changed files with their status (Added, Modified, Deleted)
//! let changed_files_with_status = repo
//!     .get_all_files_changed_since_sha_with_status("HEAD~1")?;
//!
//! // Get changes in specific packages since a branch
//! let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
//! let package_changes = repo
//!     .get_all_files_changed_since_branch(&packages, "main")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Commit History
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let repo = Repo::create("/tmp/example")?;
//! // Get all commits since a specific tag
//! let commits = repo.get_commits_since(
//!     Some("HEAD~1".to_string()),
//!     &None
//! )?;
//!
//! // Get commits affecting a specific file
//! let file_commits = repo.get_commits_since(
//!     None,
//!     &Some("src/main.rs".to_string())
//! )?;
//! # Ok(())
//! # }
//! ```

#![doc = include_str!("../SPEC.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

mod repo;
mod types;

#[cfg(test)]
mod tests;

pub use types::{GitChangedFile, GitFileStatus, Repo, RepoCommit, RepoError, RepoTags};
