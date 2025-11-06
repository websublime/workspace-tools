//! Changelog generation module for creating and managing package changelogs.
//!
//! **What**: Provides automated changelog generation with support for multiple formats,
//! including Keep a Changelog and Conventional Commits.
//!
//! **How**: This module parses commit messages (especially Conventional Commits), detects
//! version changes from Git tags, and generates formatted changelog entries. It supports
//! both single-package and monorepo configurations.
//!
//! **Why**: To automate the creation of human-readable changelogs that document changes
//! between versions, making it easier for users to understand what has changed.
//!
//! # Features
//!
//! - **Conventional Commits**: Parse and categorize commits using the Conventional Commits format
//! - **Multiple Formats**: Support for Keep a Changelog, Conventional Commits, and custom formats
//! - **Monorepo Support**: Generate changelogs per package or at the root level
//! - **Git Integration**: Detect versions from Git tags and analyze commit ranges
//! - **Template System**: Customizable templates for changelog sections and entries
//! - **Breaking Changes**: Automatic detection and highlighting of breaking changes
//! - **Issue Linking**: Automatic linking to issue trackers (GitHub, GitLab, etc.)
//! - **Author Attribution**: Optional author information in changelog entries
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changelog::ChangelogGenerator;
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
//! // TODO: will be implemented on story 8.2
//! // let generator = ChangelogGenerator::new(workspace_root, git_repo, fs, config).await?;
//! //
//! // // Generate changelog for a specific version
//! // let changelog = generator.generate_for_version("my-package", "2.0.0").await?;
//! // println!("{}", changelog.to_markdown());
//! //
//! // // Update CHANGELOG.md file
//! // generator.update_changelog("my-package", "2.0.0", false).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Conventional Commits
//!
//! This module supports parsing Conventional Commits with the following format:
//!
//! ```text
//! <type>[optional scope]: <description>
//!
//! [optional body]
//!
//! [optional footer(s)]
//! ```
//!
//! Common types include: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`.
//!
//! Breaking changes are indicated by:
//! - `BREAKING CHANGE:` footer
//! - `!` after the type/scope (e.g., `feat!:` or `feat(api)!:`)
//!
//! # Module Structure
//!
//! This module will contain:
//! - `generator`: The main `ChangelogGenerator` for creating changelogs
//! - `parser`: Conventional commit parser and commit analysis
//! - `formatter`: Different changelog format implementations
//! - `section`: Changelog section types and management
//! - `entry`: Individual changelog entry structures
//! - `template`: Template engine for custom formats

#![allow(clippy::todo)]

// Internal modules
mod collector;
mod conventional;
mod formatter;
mod generator;
mod merge_message;
mod parser;
mod types;
mod version_detection;

// Public re-exports
pub use collector::ChangelogCollector;
pub use conventional::{CommitFooter, ConventionalCommit, SectionType};
pub use formatter::{
    ConventionalCommitsFormatter, CustomTemplateFormatter, KeepAChangelogFormatter,
};
pub use generator::ChangelogGenerator;
pub use merge_message::{MergeMessageContext, generate_merge_commit_message};
pub use parser::{ChangelogParser, ParsedChangelog, ParsedVersion};
pub use types::{
    Changelog, ChangelogEntry, ChangelogMetadata, ChangelogSection, GeneratedChangelog,
};
pub use version_detection::VersionTag;

// Internal modules
#[cfg(test)]
mod tests;
