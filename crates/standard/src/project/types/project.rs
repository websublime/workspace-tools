//! # Core Project Types
//!
//! ## What
//! This module defines the fundamental types and traits for representing
//! Node.js projects in a unified way.
//!
//! ## How
//! The `ProjectKind` enum differentiates between project types, while
//! the `ProjectInfo` trait provides a common interface for all projects.
//!
//! ## Why
//! Clean type hierarchies enable uniform handling of different project
//! structures while maintaining type safety and performance.

use crate::monorepo::MonorepoKind;
use crate::node::{PackageManager, RepoKind};
use crate::project::types::ProjectValidationStatus;
use package_json::PackageJson;
use std::path::Path;

/// Represents the type of Node.js project.
///
/// This enum uses the repository-first approach where all projects
/// are fundamentally repositories with different characteristics.
/// This creates a cleaner hierarchy and better separation of concerns.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectKind;
/// use sublime_standard_tools::node::RepoKind;
/// use sublime_standard_tools::monorepo::MonorepoKind;
///
/// let simple = ProjectKind::Repository(RepoKind::Simple);
/// let yarn_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    /// A repository-based project (simple or monorepo)
    Repository(RepoKind),
}

impl ProjectKind {
    /// Returns a human-readable name for the project kind.
    ///
    /// This method delegates to the underlying repository kind for
    /// consistent naming across the type hierarchy.
    ///
    /// # Returns
    ///
    /// A string describing the project type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert_eq!(ProjectKind::Repository(RepoKind::Simple).name(), "simple");
    /// assert_eq!(ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces)).name(), "yarn monorepo");
    /// ```
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Repository(repo_kind) => repo_kind.name(),
        }
    }

    /// Checks if this is a monorepo project.
    ///
    /// This method delegates to the underlying repository kind for
    /// consistent behavior across the type hierarchy.
    ///
    /// # Returns
    ///
    /// `true` if the project is any type of monorepo, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert!(!ProjectKind::Repository(RepoKind::Simple).is_monorepo());
    /// assert!(ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::NpmWorkSpace)).is_monorepo());
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        match self {
            Self::Repository(repo_kind) => repo_kind.is_monorepo(),
        }
    }

    /// Returns the repository kind for this project.
    ///
    /// This provides direct access to the underlying repository type
    /// for repository-specific operations.
    ///
    /// # Returns
    ///
    /// A reference to the `RepoKind` representing the repository type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let simple = ProjectKind::Repository(RepoKind::Simple);
    /// assert_eq!(simple.repo_kind(), &RepoKind::Simple);
    ///
    /// let yarn_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
    /// assert_eq!(yarn_mono.repo_kind(), &RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
    /// ```
    #[must_use]
    pub fn repo_kind(&self) -> &RepoKind {
        match self {
            Self::Repository(repo_kind) => repo_kind,
        }
    }

    /// Gets the monorepo kind if this is a monorepo project.
    ///
    /// This method delegates to the underlying repository kind for
    /// consistent behavior across the type hierarchy.
    ///
    /// # Returns
    ///
    /// * `Some(&MonorepoKind)` - If this is a monorepo project
    /// * `None` - If this is a simple project
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let simple = ProjectKind::Repository(RepoKind::Simple);
    /// assert_eq!(simple.monorepo_kind(), None);
    ///
    /// let yarn_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
    /// assert_eq!(yarn_mono.monorepo_kind(), Some(&MonorepoKind::YarnWorkspaces));
    /// ```
    #[must_use]
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind> {
        match self {
            Self::Repository(repo_kind) => repo_kind.monorepo_kind(),
        }
    }
}

/// Common interface for all Node.js project types.
///
/// This trait provides a unified API for accessing project information
/// regardless of whether it's a simple repository or a monorepo.
/// All project implementations must provide these basic capabilities.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectInfo;
/// use std::path::Path;
///
/// fn print_project_info(project: &impl ProjectInfo) {
///     println!("Project root: {}", project.root().display());
///     println!("Project type: {}", project.kind().name());
///     
///     if let Some(pm) = project.package_manager() {
///         println!("Package manager: {:?}", pm.kind());
///     }
/// }
/// ```
pub trait ProjectInfo: Send + Sync {
    /// Returns the root directory of the project.
    ///
    /// # Returns
    ///
    /// A reference to the Path representing the project's root directory.
    fn root(&self) -> &Path;

    /// Returns the package manager for the project, if detected.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageManager)` - If a package manager was detected
    /// * `None` - If no package manager was detected or detection was disabled
    fn package_manager(&self) -> Option<&PackageManager>;

    /// Returns the parsed package.json for the project, if available.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageJson)` - If package.json was successfully parsed
    /// * `None` - If package.json was not found or could not be parsed
    fn package_json(&self) -> Option<&PackageJson>;

    /// Returns the validation status of the project.
    ///
    /// # Returns
    ///
    /// A reference to the `ProjectValidationStatus` indicating the validation state.
    fn validation_status(&self) -> &ProjectValidationStatus;

    /// Returns the kind of project.
    ///
    /// # Returns
    ///
    /// The `ProjectKind` enum value representing the type of project.
    fn kind(&self) -> ProjectKind;
}