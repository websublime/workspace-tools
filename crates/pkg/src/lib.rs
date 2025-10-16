//! # sublime_pkg_tools
//!
//! **What**: A comprehensive package and version management toolkit for Node.js projects with changeset support.
//!
//! **How**: This crate provides a library-first approach to managing packages, versions, changesets, and
//! dependencies in Node.js projects. It supports both single-package and monorepo configurations, offering
//! robust APIs for version resolution, dependency propagation, changelog generation, and upgrade management.
//!
//! **Why**: To provide a flexible, enterprise-grade solution for package management that treats changesets
//! as the source of truth and enables complex workflows without enforcing opinionated processes.
//!
//! ## Core Concepts
//!
//! ### Changeset as Source of Truth
//!
//! The changeset is the central data structure that describes what packages changed, what version bump
//! they need, which environments they target, and what commits are associated with those changes.
//! All operations (versioning, changelog generation, upgrades) flow from the changeset.
//!
//! ### Library Not CLI
//!
//! This crate provides libraries and APIs, not command-line tools. Integration with CLI tools happens
//! in separate crates that consume these APIs.
//!
//! ### Simple Data Model
//!
//! The core data structures are intentionally simple and serializable, making them easy to persist,
//! version control, and integrate with other tools.
//!
//! ## Modules
//!
//! - [`config`]: Configuration loading, validation, and management
//! - [`error`]: Error types and error handling utilities
//! - [`types`]: Core data structures (Version, VersionBump, Changeset, etc.)
//! - [`changeset`]: Changeset creation, management, storage, and history
//! - [`version`]: Version resolution, dependency propagation, and application
//! - [`changes`]: Analysis of file changes and commit ranges
//! - [`changelog`]: Changelog generation with conventional commits support
//! - [`upgrade`]: Dependency upgrade detection and application
//! - [`audit`]: Health checks, dependency audits, and issue detection
//!
//! ## Features
//!
//! - **Version Management**: Independent and unified versioning strategies
//! - **Dependency Propagation**: Automatic version updates across dependent packages
//! - **Changeset Management**: Create, update, archive, and query changesets
//! - **Changes Analysis**: Analyze file changes and commits to determine affected packages
//! - **Changelog Generation**: Generate changelogs using Keep a Changelog or Conventional Commits formats
//! - **Dependency Upgrades**: Detect and apply external dependency upgrades
//! - **Audit & Health Checks**: Comprehensive dependency audits and health reports
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::{
//!     config::PackageToolsConfig,
//!     changeset::ChangesetManager,
//!     version::{VersionResolver, VersioningStrategy},
//!     types::{VersionBump, Changeset},
//! };
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration
//! let config = PackageToolsConfig::default();
//!
//! // Initialize filesystem
//! let fs = FileSystemManager::new();
//!
//! // Create changeset manager
//! let workspace_root = PathBuf::from(".");
//! let changeset_manager = ChangesetManager::new(workspace_root.clone(), fs.clone(), config.clone()).await?;
//!
//! // Create a new changeset
//! let changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
//! changeset_manager.create(changeset).await?;
//!
//! // Initialize version resolver
//! let version_resolver = VersionResolver::new(
//!     workspace_root,
//!     VersioningStrategy::Independent,
//!     fs,
//!     config,
//! ).await?;
//!
//! // Apply versions (dry run)
//! let result = version_resolver.apply_versions(&changeset, true).await?;
//! println!("Would update {} packages", result.resolution.updates.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuration
//!
//! Configuration can be loaded from TOML files, environment variables, or programmatically.
//! See the [`config`] module for detailed configuration options.
//!
//! ```toml
//! [package_tools.changeset]
//! path = ".changesets"
//! history_path = ".changesets/history"
//! available_environments = ["development", "staging", "production"]
//! default_environments = ["production"]
//!
//! [package_tools.version]
//! strategy = "independent"
//! default_bump = "patch"
//!
//! [package_tools.dependency]
//! propagation_bump = "patch"
//! propagate_dependencies = true
//! propagate_dev_dependencies = false
//! ```
//!
//! ## Design Principles
//!
//! 1. **Library First**: All functionality exposed as library APIs
//! 2. **Simple Data Model**: Minimal, serializable data structures
//! 3. **Single Responsibility**: Each module has a clear, focused purpose
//! 4. **Testability**: Dependency injection and trait abstractions enable comprehensive testing
//! 5. **No Opinionated Workflow**: Tools not frameworks - users control the workflow
//!
//! ## Dependencies
//!
//! This crate relies on:
//! - [`sublime_standard_tools`]: Filesystem, configuration, command execution
//! - [`sublime_git_tools`]: Git repository operations and commit analysis
//!
//! ## Error Handling
//!
//! All operations return [`Result`] types with detailed error information. See the [`error`]
//! module for error types and recovery strategies.
//!
//! ## Thread Safety
//!
//! Most types in this crate are `Send + Sync` and can be used safely across async tasks.
//! Shared state is protected with appropriate synchronization primitives.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

// Module declarations - will be implemented in subsequent stories
pub mod audit;
pub mod changelog;
pub mod changes;
pub mod changeset;
pub mod config;
pub mod error;
pub mod types;
pub mod upgrade;
pub mod version;

/// The version of the sublime_pkg_tools crate.
///
/// This constant contains the version string as defined in `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of the sublime_pkg_tools crate.
///
/// This function provides a convenient way to retrieve the crate version at runtime.
///
/// # Returns
///
/// A string slice containing the version number in semver format (e.g., "0.1.0").
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::version;
///
/// let ver = version();
/// assert!(!ver.is_empty());
/// println!("sublime_pkg_tools version: {}", ver);
/// ```
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        let ver = version();
        assert!(!ver.is_empty(), "Version string should not be empty");
    }

    #[test]
    fn test_version_constant_matches_function() {
        assert_eq!(VERSION, version(), "VERSION constant should match version() function");
    }

    #[test]
    fn test_version_format() {
        let ver = version();
        // Basic check that version follows semver-like format (contains at least one dot)
        assert!(ver.contains('.'), "Version should follow semver format with at least one dot");
    }
}
