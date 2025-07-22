//! # Node.js Repository Type Definitions
//!
//! ## What
//! This file defines the core types for representing different kinds of Node.js
//! repositories in a hierarchical type system. It provides the fundamental
//! `RepoKind` enum that distinguishes between simple repositories and various
//! types of monorepos, creating a clean abstraction layer.
//!
//! ## How
//! The types are defined as enums that model the hierarchical relationship
//! between repository types. `RepoKind` serves as the root type that can
//! represent either a simple repository or a monorepo with a specific type.
//! This design enables type-safe handling of different repository structures
//! while maintaining a unified interface.
//!
//! ## Why
//! Previously, there was no unified concept for repository types, leading to
//! scattered logic and unclear relationships between simple and monorepo projects.
//! This module establishes a clear type hierarchy that reflects real-world
//! relationships and enables consistent handling of all Node.js repository types.

use crate::monorepo::MonorepoKind;

/// Represents the type of Node.js repository.
///
/// This enum provides a fundamental abstraction that distinguishes between
/// simple repositories (single package.json) and monorepos (multiple packages
/// with workspace configuration). It serves as the foundation for repository
/// type detection and enables type-specific processing.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::node::RepoKind;
/// use sublime_standard_tools::monorepo::MonorepoKind;
///
/// // Simple repository
/// let simple_repo = RepoKind::Simple;
/// assert_eq!(simple_repo.name(), "simple");
/// assert!(!simple_repo.is_monorepo());
///
/// // Monorepo repository
/// let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
/// assert_eq!(yarn_mono.name(), "yarn monorepo");
/// assert!(yarn_mono.is_monorepo());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoKind {
    /// Simple Node.js repository with a single package.json
    ///
    /// This represents the most common type of Node.js project where
    /// there is a single package.json file at the root and no workspace
    /// configuration. All dependencies and scripts are managed at the
    /// project level.
    Simple,

    /// Monorepo repository with specific monorepo type
    ///
    /// This represents a more complex repository structure where multiple
    /// packages are managed within a single repository using workspace
    /// functionality provided by various package managers. The specific
    /// `MonorepoKind` determines the package manager and configuration
    /// format used for workspace management.
    Monorepo(MonorepoKind),
}

impl RepoKind {
    /// Returns a human-readable name for the repository kind.
    ///
    /// This method provides a consistent way to get a display name for
    /// any repository type, useful for logging, user interfaces, and
    /// error messages.
    ///
    /// # Returns
    ///
    /// A string describing the repository type in a human-readable format.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert_eq!(RepoKind::Simple.name(), "simple");
    /// assert_eq!(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces).name(), "yarn monorepo");
    /// assert_eq!(RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces).name(), "pnpm monorepo");
    /// ```
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Simple => "simple".to_string(),
            Self::Monorepo(kind) => format!("{} monorepo", kind.name()),
        }
    }

    /// Checks if this repository is a monorepo.
    ///
    /// This is a convenience method for determining whether the repository
    /// uses monorepo structure and workspace functionality, regardless of
    /// the specific monorepo type.
    ///
    /// # Returns
    ///
    /// `true` if the repository is any type of monorepo, `false` if it's
    /// a simple repository.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert!(!RepoKind::Simple.is_monorepo());
    /// assert!(RepoKind::Monorepo(MonorepoKind::NpmWorkSpace).is_monorepo());
    /// assert!(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces).is_monorepo());
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        matches!(self, Self::Monorepo(_))
    }

    /// Gets the monorepo kind if this is a monorepo repository.
    ///
    /// This method provides access to the specific monorepo type when
    /// the repository is confirmed to be a monorepo. It's useful for
    /// accessing monorepo-specific functionality and configuration.
    ///
    /// # Returns
    ///
    /// * `Some(&MonorepoKind)` - If this is a monorepo repository
    /// * `None` - If this is a simple repository
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let simple = RepoKind::Simple;
    /// assert_eq!(simple.monorepo_kind(), None);
    ///
    /// let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
    /// assert_eq!(yarn_mono.monorepo_kind(), Some(&MonorepoKind::YarnWorkspaces));
    /// ```
    #[must_use]
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind> {
        match self {
            Self::Simple => None,
            Self::Monorepo(kind) => Some(kind),
        }
    }

    /// Checks if this repository matches a specific monorepo kind.
    ///
    /// This is a convenience method for checking if the repository is not
    /// only a monorepo but also matches a specific monorepo type. It's
    /// useful for conditional logic based on monorepo implementation.
    ///
    /// # Arguments
    ///
    /// * `kind` - The monorepo kind to check against
    ///
    /// # Returns
    ///
    /// `true` if this is a monorepo of the specified kind, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
    /// assert!(yarn_mono.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
    /// assert!(!yarn_mono.is_monorepo_kind(&MonorepoKind::PnpmWorkspaces));
    ///
    /// let simple = RepoKind::Simple;
    /// assert!(!simple.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
    /// ```
    #[must_use]
    pub fn is_monorepo_kind(&self, kind: &MonorepoKind) -> bool {
        match self {
            Self::Simple => false,
            Self::Monorepo(repo_kind) => repo_kind == kind,
        }
    }
}
