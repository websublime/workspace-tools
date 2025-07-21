//! # Node.js Repository Abstractions
//!
//! ## What
//! This file defines the traits and abstractions for working with Node.js
//! repositories in a unified way. It provides the `RepositoryInfo` trait that
//! enables consistent access to repository information regardless of whether
//! the repository is a simple project or a complex monorepo structure.
//!
//! ## How
//! The module defines traits that establish contracts for repository behavior.
//! The `RepositoryInfo` trait provides a common interface that can be implemented
//! by different repository types, enabling polymorphic handling of various
//! Node.js project structures while maintaining type safety and consistency.
//!
//! ## Why
//! A unified repository interface is essential for building tools that work
//! across different Node.js project types. This abstraction layer allows
//! code to work with repositories generically, without needing to know the
//! specific implementation details of simple projects vs. monorepos.

use std::path::Path;

use super::{PackageManager, RepoKind};

/// Common interface for all Node.js repository types.
///
/// This trait provides a unified API for accessing repository information
/// regardless of whether it's a simple repository or a monorepo. All
/// repository implementations must provide these basic capabilities to
/// enable consistent tooling and processing.
///
/// The trait focuses on the fundamental characteristics that all Node.js
/// repositories share: location, type identification, and package management
/// configuration. More specific functionality is handled by dedicated
/// implementations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::node::{RepositoryInfo, RepoKind};
/// use std::path::Path;
///
/// fn analyze_repository(repo: &impl RepositoryInfo) {
///     println!("Repository root: {}", repo.root().display());
///     println!("Repository type: {}", repo.kind().name());
///     
///     if repo.kind().is_monorepo() {
///         println!("This is a monorepo with multiple packages");
///     } else {
///         println!("This is a simple repository");
///     }
///     
///     if let Some(manager) = repo.package_manager() {
///         println!("Package manager: {}", manager.command());
///     }
/// }
/// ```
///
/// # Implementation Guidelines
///
/// When implementing this trait:
/// - `root()` should return the absolute path to the repository root
/// - `kind()` should reflect the actual repository structure detected
/// - `package_manager()` should return None if detection failed or was disabled
/// - All methods should be consistent with the repository's actual state
pub trait RepositoryInfo: Send + Sync {
    /// Returns the root directory of the repository.
    ///
    /// This is the top-level directory that contains the repository's
    /// configuration files (package.json, workspace configuration, etc.)
    /// and serves as the base path for all repository operations.
    ///
    /// # Returns
    ///
    /// A reference to the Path representing the repository's root directory.
    /// This path should be absolute to ensure consistency across different
    /// working directory contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::node::RepositoryInfo;
    /// # fn example(repo: &impl RepositoryInfo) {
    /// let root = repo.root();
    /// println!("Repository located at: {}", root.display());
    ///
    /// // Use root to construct paths to repository files
    /// let package_json = root.join("package.json");
    /// # }
    /// ```
    fn root(&self) -> &Path;

    /// Returns the type of repository.
    ///
    /// This identifies whether the repository is a simple Node.js project
    /// or a monorepo, and if it's a monorepo, what type of monorepo it is.
    /// This information is crucial for determining what operations are
    /// available and how they should be performed.
    ///
    /// # Returns
    ///
    /// A `RepoKind` enum value representing the repository type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::node::{RepositoryInfo, RepoKind};
    /// # use sublime_standard_tools::monorepo::MonorepoKind;
    /// # fn example(repo: &impl RepositoryInfo) {
    /// match repo.kind() {
    ///     RepoKind::Simple => {
    ///         println!("Simple repository - single package");
    ///     }
    ///     RepoKind::Monorepo(MonorepoKind::YarnWorkspaces) => {
    ///         println!("Yarn workspaces monorepo");
    ///     }
    ///     RepoKind::Monorepo(kind) => {
    ///         println!("Monorepo type: {}", kind.name());
    ///     }
    /// }
    /// # }
    /// ```
    fn kind(&self) -> RepoKind;

    /// Returns the package manager detected for this repository.
    ///
    /// Package managers are detected based on lock files, configuration
    /// files, and other heuristics. This information is used to determine
    /// the correct commands to run for dependency management and other
    /// package manager operations.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageManager)` - If a package manager was detected
    /// * `None` - If no package manager was detected, detection failed,
    ///   or detection was disabled in the configuration
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::node::RepositoryInfo;
    /// # fn example(repo: &impl RepositoryInfo) {
    /// if let Some(manager) = repo.package_manager() {
    ///     println!("Using {} package manager", manager.command());
    ///     
    ///     if manager.supports_workspaces() {
    ///         println!("Workspace support available");
    ///     }
    /// } else {
    ///     println!("No package manager detected");
    /// }
    /// # }
    /// ```
    fn package_manager(&self) -> Option<&PackageManager>;

    /// Checks if this repository has a package manager configured.
    ///
    /// This is a convenience method that checks whether a package manager
    /// was detected and is available for operations. It's equivalent to
    /// checking if `package_manager()` returns `Some(_)`.
    ///
    /// # Returns
    ///
    /// `true` if a package manager is available, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::node::RepositoryInfo;
    /// # fn example(repo: &impl RepositoryInfo) {
    /// if repo.has_package_manager() {
    ///     println!("Can perform package management operations");
    /// } else {
    ///     println!("Package management not available");
    /// }
    /// # }
    /// ```
    fn has_package_manager(&self) -> bool {
        self.package_manager().is_some()
    }

    /// Checks if this repository is a monorepo.
    ///
    /// This is a convenience method that checks the repository kind to
    /// determine if it represents a monorepo structure. It's equivalent
    /// to calling `self.kind().is_monorepo()`.
    ///
    /// # Returns
    ///
    /// `true` if the repository is any type of monorepo, `false` if it's
    /// a simple repository.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::node::RepositoryInfo;
    /// # fn example(repo: &impl RepositoryInfo) {
    /// if repo.is_monorepo() {
    ///     println!("Multi-package repository");
    /// } else {
    ///     println!("Single-package repository");
    /// }
    /// # }
    /// ```
    fn is_monorepo(&self) -> bool {
        self.kind().is_monorepo()
    }

    /// Gets a display name for this repository.
    ///
    /// This provides a human-readable identifier for the repository that
    /// can be used in logging, user interfaces, and error messages. The
    /// default implementation uses the repository kind's name, but
    /// implementations can override this to provide more specific names.
    ///
    /// # Returns
    ///
    /// A string that can be used to identify this repository in user-facing
    /// contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::node::RepositoryInfo;
    /// # fn example(repo: &impl RepositoryInfo) {
    /// println!("Processing {} repository", repo.display_name());
    /// # }
    /// ```
    fn display_name(&self) -> String {
        self.kind().name()
    }
}
