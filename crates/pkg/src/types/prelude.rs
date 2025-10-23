//! Prelude module for commonly used types.
//!
//! **What**: Provides a convenient single import for the most commonly used types from
//! the `sublime_pkg_tools::types` module, reducing boilerplate in user code.
//!
//! **How**: Re-exports the core types and traits that are frequently used together,
//! allowing users to import everything with a single `use` statement.
//!
//! **Why**: To improve ergonomics and reduce boilerplate by providing a curated set
//! of imports that cover the most common use cases. This follows Rust community best
//! practices for library design.
//!
//! # Usage
//!
//! Instead of importing each type individually:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::{Version, VersionBump, Changeset, PackageInfo};
//! use sublime_pkg_tools::types::{Named, Versionable, Identifiable};
//! use sublime_pkg_tools::types::{DependencyType, DependencyUpdate};
//! ```
//!
//! You can import them all at once:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::prelude::*;
//! ```
//!
//! # What's Included
//!
//! The prelude includes:
//!
//! ## Core Types
//! - [`Version`] - Semantic version representation
//! - [`VersionBump`] - Version bump types (Major, Minor, Patch, None)
//! - [`VersioningStrategy`] - Independent or Unified versioning
//!
//! ## Changeset Types
//! - [`Changeset`] - The central changeset data structure
//! - [`ArchivedChangeset`] - Archived changesets with release info
//! - [`ReleaseInfo`] - Release metadata
//! - [`UpdateSummary`] - Summary of changeset updates
//!
//! ## Package Types
//! - [`PackageInfo`] - Package metadata and information
//! - [`DependencyType`] - Type of dependency (Regular, Dev, Peer, Optional)
//!
//! ## Dependency Types
//! - [`DependencyUpdate`] - Dependency version update information
//! - [`CircularDependency`] - Circular dependency detection result
//! - [`UpdateReason`] - Why a package is being updated
//! - [`VersionProtocol`] - Version specification protocols
//! - [`LocalLinkType`] - Types of local links (File, Link, Portal)
//!
//! ## Traits
//! - [`Named`] - Trait for types with names
//! - [`Versionable`] - Trait for types with versions
//! - [`Identifiable`] - Trait for types with name and version
//! - [`HasDependencies`] - Trait for types with dependencies
//!
//! ## Type Aliases
//! - [`PackageName`] - Package name string
//! - [`VersionSpec`] - Version specification string
//! - [`CommitHash`] - Git commit hash string
//! - [`BranchName`] - Git branch name string
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::prelude::*;
//!
//! // Now all core types are available
//! let version = Version::new(1, 2, 3);
//! let changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
//! ```
//!
//! ## With Traits
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::prelude::*;
//!
//! fn print_info<T: Identifiable>(item: &T) {
//!     println!("{}", item.identifier());
//! }
//! ```
//!
//! # Design Note
//!
//! The prelude intentionally includes only the most commonly used types. Less frequently
//! used types and implementation details should be imported explicitly from their respective
//! modules to keep the prelude focused and avoid namespace pollution.

// Re-export core version types
pub use crate::types::{Version, VersionBump, VersioningStrategy};

// Re-export changeset types
pub use crate::types::{ArchivedChangeset, Changeset, ReleaseInfo, UpdateSummary};

// Re-export package types
pub use crate::types::{DependencyType, PackageInfo};

// Re-export dependency types
pub use crate::types::{
    CircularDependency, DependencyUpdate, LocalLinkType, UpdateReason, VersionProtocol,
};

// Re-export PackageUpdate from version module
pub use crate::types::PackageUpdate;

// Re-export common traits
pub use crate::types::{HasDependencies, Identifiable, Named, Versionable};

// Re-export type aliases
pub use crate::types::{BranchName, CommitHash, PackageName, VersionSpec};

// Re-export helper functions for protocol handling
pub use crate::types::{
    extract_protocol_path, is_local_protocol, is_workspace_protocol, parse_protocol,
    should_skip_protocol,
};
