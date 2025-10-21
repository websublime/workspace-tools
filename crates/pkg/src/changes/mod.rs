//! Changes analysis module for detecting and analyzing file and commit changes.
//!
//! **What**: Provides comprehensive analysis of file changes and commits in both single-package
//! and monorepo configurations, mapping changes to affected packages.
//!
//! **How**: This module integrates with Git to analyze working directory changes and commit ranges,
//! maps changed files to their respective packages, and calculates version impacts based on
//! changeset information.
//!
//! **Why**: To enable accurate detection of which packages are affected by changes and to provide
//! detailed information about what changed, supporting informed version bumping and changelog generation.
//!
//! # Features
//!
//! - **Working Directory Analysis**: Analyze uncommitted changes in the working directory
//! - **Commit Range Analysis**: Analyze changes between two Git commits or refs
//! - **Package Mapping**: Map changed files to their containing packages
//! - **Commit Association**: Associate commits with the packages they affect
//! - **Version Preview**: Calculate next versions based on changeset and changes
//! - **Change Statistics**: Provide detailed statistics about changes (files, lines, commits)
//! - **Multi-Package Support**: Handle both single-package and monorepo structures
//! - **Change Filtering**: Filter changes by type, package, or directory
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changes::ChangesAnalyzer;
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_git_tools::Repo;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//! let config = PackageToolsConfig::default();
//! let git_repo = Repo::open(".")?;
//!
//! let analyzer = ChangesAnalyzer::new(workspace_root, git_repo, fs, config).await?;
//!
//! // Analyze working directory changes
//! let changes = analyzer.analyze_working_directory().await?;
//! for package_change in changes.packages {
//!     println!("Package: {}", package_change.package_name());
//!     println!("  Files changed: {}", package_change.files.len());
//!     println!("  Has changes: {}", package_change.has_changes);
//! }
//!
//! // Analyze commit range
//! let changes = analyzer.analyze_commit_range("main", "HEAD").await?;
//! println!("Total packages affected: {}", changes.summary.packages_with_changes);
//! # Ok(())
//! # }
//! ```
//!
//! # Change Types
//!
//! The module tracks different types of file changes:
//! - **Added**: New files created
//! - **Modified**: Existing files changed
//! - **Deleted**: Files removed
//! - **Renamed**: Files moved or renamed
//! - **Copied**: Files copied to new locations
//!
//! # Integration with Changesets
//!
//! This module can be used in conjunction with changesets to provide comprehensive
//! version impact analysis:
//!
//! ```rust,ignore
//! # use sublime_pkg_tools::changes::ChangesAnalyzer;
//! # use sublime_pkg_tools::changeset::ChangesetManager;
//! # use sublime_pkg_tools::config::PackageToolsConfig;
//! # use sublime_git_tools::Repo;
//! # use sublime_standard_tools::filesystem::FileSystemManager;
//! # use std::path::PathBuf;
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let workspace_root = PathBuf::from(".");
//! # let fs = FileSystemManager::new();
//! # let config = PackageToolsConfig::default();
//! # let git_repo = Repo::open(".")?;
//! let analyzer = ChangesAnalyzer::new(workspace_root.clone(), git_repo.clone(), fs.clone(), config.clone()).await?;
//! let changeset_manager = ChangesetManager::new(workspace_root, fs, config).await?;
//!
//! let changeset = changeset_manager.load("my-changeset").await?;
//! let changes = analyzer.analyze_with_versions("main", "HEAD", &changeset).await?;
//!
//! for package_change in &changes.packages {
//!     if let (Some(current), Some(next)) = (&package_change.current_version, &package_change.next_version) {
//!         println!("Package: {} -> {}", current, next);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! This module will contain:
//! - `analyzer`: The main `ChangesAnalyzer` for orchestrating change analysis
//! - `package_changes`: Per-package change information and statistics
//! - `file_change`: Individual file change details
//! - `commit_info`: Commit information and metadata
//! - `stats`: Change statistics and summaries

// Analyzer module - Story 7.1
mod analyzer;
pub use analyzer::ChangesAnalyzer;

// Mapping module - Story 7.2
pub mod mapping;
pub use mapping::PackageMapper;

// Report types - Story 7.3
mod report;
pub use report::{AnalysisMode, ChangesReport};

// Package changes - Story 7.3
mod package_changes;
pub use package_changes::PackageChanges;

// File changes - Story 7.3
mod file_change;
pub use file_change::{FileChange, FileChangeType};

// Commit info - Story 7.3 (minimal implementation)
mod commit_info;
pub use commit_info::CommitInfo;

// Statistics - Story 7.3
mod stats;
pub use stats::{ChangesSummary, PackageChangeStats};

// Tests module
#[cfg(test)]
mod tests;
