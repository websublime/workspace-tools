#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

//! # sublime_pkg_tools
//!
//! A comprehensive package and version management toolkit for Node.js projects
//! with advanced changeset support, conventional commit parsing, and multi-environment
//! release management.
//!
//! ## What
//!
//! This crate provides a complete solution for managing package versions, releases,
//! and dependencies in Node.js monorepos and single-package projects. It offers:
//!
//! - **Version Management**: Semantic versioning with snapshot versions for development
//! - **Changeset System**: Controlled, reviewable version bumps with complete audit trails
//! - **Release Management**: Multi-environment releases with dry-run capabilities
//! - **Dependency Analysis**: Graph-based dependency tracking and update propagation
//! - **Conventional Commits**: Automatic version bump calculation from commit messages
//! - **Registry Integration**: NPM registry publishing and package information retrieval
//! - **Changelog Generation**: Automated changelog creation from commits and changesets
//!
//! ## How
//!
//! The crate is built on a modular architecture that integrates with existing
//! tools from the sublime ecosystem:
//!
//! - Uses `sublime_standard_tools` for configuration, filesystem, and command execution
//! - Uses `sublime_git_tools` for all Git operations
//! - Provides a unified API for package management operations
//! - Supports both independent and unified versioning strategies
//! - Enables continuous deployment through snapshot versions
//!
//! ## Why
//!
//! Managing package versions and releases in modern JavaScript projects is complex,
//! especially in monorepo environments. This crate addresses common pain points:
//!
//! - **Version Conflicts**: Snapshot versions prevent conflicts during development
//! - **Manual Releases**: Automated release processes reduce human error
//! - **Dependency Chaos**: Graph analysis ensures consistent dependency updates
//! - **Poor Visibility**: Complete audit trails and changelogs improve transparency
//! - **Environment Management**: Multi-environment support enables proper deployment pipelines
//!
//! ## Core Concepts
//!
//! ### Snapshot Versions
//!
//! Development branches use snapshot versions that are calculated dynamically
//! and never written to `package.json`. This enables continuous deployment
//! without version conflicts:
//!
//! ```text
//! Branch: feat/auth
//! ├─ Commit 1: 1.2.3-abc123d.snapshot → Deploy to dev
//! ├─ Commit 2: 1.2.3-def456g.snapshot → Deploy to dev
//! └─ Merge to main: 1.2.3 → 1.3.0 → Deploy to prod
//! ```
//!
//! ### Changesets
//!
//! Changesets are JSON files that describe intended version changes across
//! multiple packages. They are created on feature branches and applied when
//! merging to main:
//!
//! ```json
//! {
//!   "branch": "feat/auth",
//!   "created_at": "2024-01-15T10:30:00Z",
//!   "author": "developer@example.com",
//!   "releases": ["dev", "qa"],
//!   "packages": [
//!     {
//!       "name": "@myorg/auth-service",
//!       "bump": "minor",
//!       "current_version": "1.2.3",
//!       "next_version": "1.3.0",
//!       "reason": "DirectChanges",
//!       "changes": [...]
//!     }
//!   ]
//! }
//! ```
//!
//! ### Version Strategies
//!
//! Two versioning strategies are supported:
//!
//! - **Independent**: Each package maintains its own version independently
//! - **Unified**: All packages share the same version number
//!
//! ## Usage Examples
//!
//! ### Basic Version Operations
//!
//! ```rust
//! use sublime_pkg_tools::version::{Version, VersionBump};
//! use std::str::FromStr;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse and manipulate versions
//! let version = Version::from_str("1.2.3")?;
//! let bumped = version.bump(VersionBump::Minor);
//! assert_eq!(bumped.to_string(), "1.3.0");
//! # Ok(())
//! # }
//! ```
//!
//! ### Configuration Loading
//!
//! ```ignore
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_standard_tools::config::ConfigManager;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config_manager = ConfigManager::<PackageToolsConfig>::builder()
//!     .with_defaults(PackageToolsConfig::default())
//!     .with_env_prefix("SUBLIME_PACKAGE_TOOLS")
//!     .build();
//!
//! let config = config_manager.load().await?;
//! println!("Changeset path: {:?}", config.changeset.path);
//! # Ok(())
//! # }
//! ```
//!
//! ### Conventional Commit Parsing
//!
//! ```rust
//! use sublime_pkg_tools::conventional::ConventionalCommitParser;
//! use chrono::Utc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let parser = ConventionalCommitParser::new()?;
//! let commit = parser.parse(
//!     "feat(auth): add OAuth2 support",
//!     "abc123".to_string(),
//!     "Developer".to_string(),
//!     Utc::now(),
//! )?;
//!
//! println!("Type: {:?}, Breaking: {}", commit.commit_type, commit.breaking);
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The crate follows a service-oriented architecture with clear separation of concerns:
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │  VersionResolver │    │ ChangesetManager│    │ ReleaseManager  │
//! │                 │    │                 │    │                 │
//! │ - resolve()     │    │ - create()      │    │ - plan()        │
//! │ - snapshot()    │    │ - apply()       │    │ - execute()     │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!            │                       │                       │
//!            └───────────────────────┼───────────────────────┘
//!                                    │
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │DependencyAnalyzer│    │RegistryClient   │    │ChangelogGenerator│
//! │                 │    │                 │    │                 │
//! │ - analyze()     │    │ - publish()     │    │ - generate()    │
//! │ - propagate()   │    │ - info()        │    │ - format()      │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## Integration
//!
//! This crate integrates seamlessly with other sublime tools:
//!
//! - **Configuration**: Uses `sublime_standard_tools::config` for unified configuration
//! - **Filesystem**: Uses `sublime_standard_tools::filesystem` for all file operations
//! - **Commands**: Uses `sublime_standard_tools::command` for shell command execution
//! - **Git Operations**: Uses `sublime_git_tools::Repo` for all Git interactions
//! - **Project Detection**: Uses `sublime_standard_tools::project` for project structure analysis
//!
//! ## Error Handling
//!
//! All operations return `PackageResult<T>` which is an alias for `Result<T, PackageError>`.
//! The error system provides detailed context for debugging and user feedback:
//!
//! ```rust
//! use sublime_pkg_tools::error::{PackageError, VersionError};
//!
//! # fn example() {
//! let error = PackageError::Version(VersionError::InvalidFormat {
//!     version: "not-a-version".to_string(),
//!     reason: "Invalid semver format".to_string(),
//! });
//!
//! println!("Error: {}", error);
//! // Output: Version error: Invalid version format: 'not-a-version' - Invalid semver format
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! Currently, all features are enabled by default. Future versions may introduce
//! optional features for specific functionality like registry integration or
//! advanced template systems.
//!
//! ## Platform Support
//!
//! This crate supports all platforms supported by the underlying dependencies:
//! - Windows
//! - macOS
//! - Linux
//! - Other Unix-like systems
//!
//! ## Version Compatibility
//!
//! This crate follows semantic versioning. Breaking changes will only be introduced
//! in major version updates, with clear migration guides provided.

pub mod changelog;
pub mod changeset;
pub mod config;
pub mod conventional;
pub mod dependency;
pub mod error;
pub mod registry;
pub mod release;
pub mod version;

// Re-export commonly used types for convenience
pub use error::{PackageError, PackageResult};
pub use version::{ResolvedVersion, SnapshotVersion, Version, VersionBump, VersionResolver};

/// Library version information.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Gets the library version.
///
/// # Returns
///
/// The current version of the sublime_pkg_tools crate.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools;
///
/// let version = sublime_pkg_tools::version();
/// println!("sublime_pkg_tools version: {}", version);
/// ```
#[must_use]
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version = version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn test_version_constant() {
        assert_eq!(VERSION, version());
    }
}
