//! Core data types and structures for package management operations.
//!
//! **What**: Provides fundamental data structures used throughout the package tools system,
//! including Version, VersionBump, Changeset, PackageInfo, and dependency-related types.
//!
//! **How**: This module defines simple, serializable data structures that represent the core
//! concepts of package management: versions, version bumps, changesets, packages, and their
//! relationships. All types are designed to be easily serialized to/from JSON and TOML.
//!
//! **Why**: To provide a clean, type-safe foundation for all package management operations,
//! ensuring consistency across the system and enabling easy persistence and integration with
//! external tools.
//!
//! # Core Types
//!
//! ## Version
//!
//! Represents a semantic version (major.minor.patch) with support for version comparison
//! and bumping operations.
//!
//! ## VersionBump
//!
//! Enum representing the type of version change: Major, Minor, Patch, or None.
//!
//! ## Changeset
//!
//! The central data structure representing a set of changes to one or more packages,
//! including the version bump type, target environments, associated commits, and metadata.
//!
//! ## ArchivedChangeset
//!
//! A changeset that has been released and archived, including release information
//! such as when it was applied, by whom, and what versions were released.
//!
//! ## PackageInfo
//!
//! Information about a package, including its package.json content, workspace context,
//! and filesystem location.
//!
//! ## DependencyGraph
//!
//! Represents the dependency relationships between packages in a workspace, supporting
//! operations like finding dependents and detecting circular dependencies.
//!
//! # Features
//!
//! - **Serialization**: All types implement `Serialize` and `Deserialize`
//! - **Cloning**: Types are `Clone` for easy duplication
//! - **Debug**: Comprehensive debug output for troubleshooting
//! - **Display**: Human-readable string representations
//! - **Validation**: Built-in validation methods for data integrity
//! - **Conversion**: Utilities for converting between related types
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::{Changeset, VersionBump};
//!
//! // Create a new changeset
//! let changeset = Changeset::new(
//!     "feature-branch",
//!     VersionBump::Minor,
//!     vec!["production".to_string()]
//! );
//!
//! // Add packages and commits
//! // TODO: will be implemented on story 4.3
//! // changeset.add_package("my-package");
//! // changeset.add_commit("abc123", "feat: add new feature");
//!
//! println!("Changeset for branch: {}", changeset.branch);
//! println!("Version bump: {:?}", changeset.bump);
//! ```
//!
//! # Version Handling
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::{Version, VersionBump};
//!
//! // TODO: will be implemented on story 4.1
//! // let version = Version::parse("1.2.3")?;
//! // let bumped = version.bump(VersionBump::Minor);
//! // assert_eq!(bumped.to_string(), "1.3.0");
//! ```
//!
//! # Dependency Graph
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::DependencyGraph;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // TODO: will be implemented on story 4.4
//! // let packages = load_packages().await?;
//! // let graph = DependencyGraph::from_packages(&packages)?;
//! //
//! // // Find all packages that depend on a specific package
//! // let dependents = graph.dependents("my-package");
//! // println!("Packages depending on my-package: {:?}", dependents);
//! //
//! // // Detect circular dependencies
//! // let cycles = graph.detect_cycles();
//! // if !cycles.is_empty() {
//! //     println!("Warning: Circular dependencies detected!");
//! // }
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! This module will contain:
//! - `version`: Version types and manipulation
//! - `changeset`: Changeset and related structures
//! - `package`: Package information and metadata
//! - `dependency`: Dependency graph and relationship types
//! - `release`: Release information for archived changesets

#![allow(clippy::todo)]

// Module will be implemented in subsequent stories (Epic 4)
