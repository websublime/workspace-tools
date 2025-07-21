//! # Path Utilities
//!
//! ## What
//! This module provides utilities for working with filesystem paths,
//! including a utility struct and an extension trait for Node.js projects.
//!
//! ## How
//! The `PathUtils` struct provides static methods for common path operations,
//! while the `PathExt` trait extends the standard `Path` type with project-specific methods.
//!
//! ## Why
//! Centralized path utilities make it easier to work with Node.js project
//! structures and provide consistent path handling across the codebase.

use super::path_types::NodePathKind;
use crate::error::Result;
use std::path::PathBuf;

/// Utility struct for file system path operations.
///
/// This struct provides static methods for common path operations
/// related to Node.js projects, such as finding project roots and
/// manipulating paths.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::filesystem::PathUtils;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Find the current directory
/// let current = PathUtils::current_dir()?;
///
/// // Find a project root (if in a Node.js project)
/// if let Some(root) = PathUtils::find_project_root(&current) {
///     println!("Project root found at: {}", root.display());
/// }
/// # Ok(())
/// # }
/// ```
pub struct PathUtils;

/// Extension trait for Path to provide Node.js-specific path utilities.
///
/// This trait adds methods to the standard Path type for working with Node.js
/// project directories, normalizing paths, and handling Node.js path conventions.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::filesystem::{NodePathKind, PathExt};
///
/// let path = Path::new("/some/path");
/// let node_modules = path.node_path(NodePathKind::NodeModules);
/// assert_eq!(node_modules, Path::new("/some/path/node_modules"));
///
/// let normalized = Path::new("/a/b/../c").normalize();
/// assert_eq!(normalized, Path::new("/a/c"));
/// ```
pub trait PathExt {
    /// Normalizes a path by resolving `.` and `..` components.
    ///
    /// This method processes the path components to:
    /// - Remove `.` (current directory) components
    /// - Resolve `..` (parent directory) components by removing the previous component
    /// - Preserve root and prefix components
    /// - Keep normal path components unchanged
    ///
    /// # Returns
    ///
    /// A normalized `PathBuf` with all `.` and `..` components resolved.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// let path = Path::new("/home/user/../projects/./app");
    /// let normalized = path.normalize();
    /// assert_eq!(normalized, Path::new("/home/projects/app"));
    /// ```
    fn normalize(&self) -> PathBuf;

    /// Checks if this path is inside a Node.js project by looking for
    /// a package.json file in the current directory or any parent directory.
    ///
    /// # Returns
    ///
    /// `true` if the path is inside a Node.js project, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// // Assuming the current directory is in a Node.js project
    /// let path = Path::new(".");
    /// if path.is_in_project() {
    ///     println!("This path is in a Node.js project");
    /// }
    /// ```
    fn is_in_project(&self) -> bool;

    /// Gets the path relative to the nearest Node.js project root.
    ///
    /// This method searches upward from the current path to find a directory
    /// containing a package.json file, then returns the path relative to that
    /// project root.
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - The path relative to the project root if a project root was found
    /// * `None` - If no project root was found
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// // Assuming the current directory is in a Node.js project
    /// let path = Path::new("src/components/Button.js");
    /// if let Some(relative) = path.relative_to_project() {
    ///     println!("Path relative to project: {}", relative.display());
    /// }
    /// ```
    fn relative_to_project(&self) -> Option<PathBuf>;

    /// Joins a Node.js path kind to this path.
    ///
    /// This is a convenience method for joining paths like `"node_modules"`,
    /// "package.json", etc. to the current path.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of Node.js path to join
    ///
    /// # Returns
    ///
    /// A new `PathBuf` with the Node.js path kind joined to this path.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{NodePathKind, PathExt};
    ///
    /// let project_path = Path::new("/projects/my-app");
    /// let node_modules = project_path.node_path(NodePathKind::NodeModules);
    /// assert_eq!(node_modules, Path::new("/projects/my-app/node_modules"));
    /// ```
    fn node_path(&self, kind: NodePathKind) -> PathBuf;

    /// Canonicalizes a path, resolving symlinks if needed.
    ///
    /// If the path is a symlink, it resolves to the target path.
    /// Otherwise, it returns the path as-is.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::error::FileSystemError`] if:
    /// - The path does not exist
    /// - Insufficient permissions to access the path
    /// - The path contains invalid characters for the current platform
    /// - A symbolic link loop is encountered
    /// - An I/O error occurs during path resolution
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The canonicalized path
    /// * `Err(FileSystemError)` - If the path cannot be canonicalized
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = Path::new("Cargo.toml");
    /// let canonical = path.canonicalize()?;
    /// println!("Canonical path: {}", canonical.display());
    /// # Ok(())
    /// # }
    /// ```
    fn canonicalize(&self) -> Result<PathBuf>;
}
