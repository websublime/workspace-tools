//! Changelog generation module
//!
//! This module provides comprehensive changelog generation capabilities based on
//! conventional commits with customizable templates and multiple output formats.
//!
//! # What
//! Automatically generates changelogs from Git commit history using conventional commit
//! standards. Supports package-specific changelogs in monorepos with configurable
//! templates, grouping strategies, and output formats.
//!
//! # How
//! Leverages the git crate for commit history, parses conventional commits using regex,
//! groups commits by type/scope, and applies templates to generate formatted changelogs
//! in Markdown, Plain Text, or JSON formats.
//!
//! # Why
//! Essential for maintaining clear project history, automating release documentation,
//! and providing users with structured information about changes in each version.
//!
//! # Examples
//!
//! ```rust
//! use sublime_monorepo_tools::changelog::{ChangelogManager, ChangelogRequest};
//! use sublime_monorepo_tools::core::MonorepoProject;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create changelog manager from project
//! let project = Arc::new(MonorepoProject::new(".")?);
//! let manager = ChangelogManager::from_project(project)?;
//!
//! // Generate changelog for specific package
//! let request = ChangelogRequest {
//!     package_name: Some("my-package".to_string()),
//!     version: "1.0.0".to_string(),
//!     since: Some("v0.9.0".to_string()),
//!     write_to_file: true,
//!     ..Default::default()
//! };
//!
//! let result = manager.generate_changelog(request).await?;
//! println!("Generated changelog with {} commits", result.commit_count);
//!
//! // Parse conventional commits for analysis
//! let commits = manager.parse_conventional_commits(
//!     Some("packages/my-package"), 
//!     "v1.0.0"
//! ).await?;
//!
//! for commit in commits {
//!     if commit.breaking_change {
//!         println!("Breaking change: {}", commit.description);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod types;
mod parser;
mod generator;
mod manager;

// Re-export main types for public API
pub use types::{
    ConventionalCommit, ChangelogResult, ChangelogRequest, GroupedCommits, TemplateVariables,
};

pub use parser::ConventionalCommitParser;
pub use generator::ChangelogGenerator;
pub use manager::ChangelogManager;