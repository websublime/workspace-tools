//! # Filesystem Path Utilities
//!
//! ## What
//! This file provides path handling utilities specific to Node.js projects,
//! including functions for finding project roots, normalizing paths, and
//! handling Node.js-specific path conventions.
//!
//! ## How
//! The file implements the `PathExt` trait for the standard `Path` type,
//! extending it with Node.js-specific functionality. It also implements
//! methods for the `NodePathKind` enum and `PathUtils` struct to provide
//! centralized path handling utilities.
//!
//! ## Why
//! Node.js projects follow specific conventions for directory structures
//! and file locations. These utilities simplify working with these conventions
//! and provide a consistent approach to path handling across the crate.

use super::{NodePathKind, PathUtils, types::PathExt};
use crate::error::{Error, FileSystemError, FileSystemResult, Result};
use std::path::{Component, Path, PathBuf};

impl NodePathKind {
    /// Returns the default path string for the given Node.js path kind.
    ///
    /// # Returns
    ///
    /// A string slice containing the conventional path name for this path kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::NodePathKind;
    ///
    /// assert_eq!(NodePathKind::NodeModules.default_path(), "node_modules");
    /// assert_eq!(NodePathKind::PackageJson.default_path(), "package.json");
    /// ```
    #[must_use]
    pub fn default_path(self) -> &'static str {
        match self {
            Self::NodeModules => "node_modules",
            Self::PackageJson => "package.json",
            Self::Src => "src",
            Self::Dist => "dist",
            Self::Test => "test",
        }
    }
}

impl PathUtils {
    /// Finds the root directory of a Node.js project by traversing upward
    /// from the given starting directory until it finds a package.json file.
    ///
    /// # Arguments
    ///
    /// * `start` - The path to start searching from
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - The path to the project root if found
    /// * `None` - If no project root was found
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathUtils;
    ///
    /// // This will find the project root if running from within a Node.js project
    /// let project_root = PathUtils::find_project_root(Path::new("."));
    /// ```
    #[must_use]
    pub fn find_project_root(start: &Path) -> Option<PathBuf> {
        let mut current = Some(start);

        // Lock files that indicate project root
        let lock_files = [
            "package-lock.json",   // npm
            "npm-shrinkwrap.json", // npm
            "yarn.lock",           // yarn
            "pnpm-lock.yaml",      // pnpm
            "bun.lockb",           // bun
            "jsr.json",            // JSR
        ];

        while let Some(path) = current {
            for lock_file in &lock_files {
                if path.join(lock_file).exists() {
                    return Some(path.to_path_buf());
                }
            }

            current = path.parent();
        }
        None
    }

    /// Gets the current working directory as a `PathBuf`.
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The current working directory
    /// * `Err(FileSystemError)` - If the current directory cannot be determined
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::PathUtils;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let current_dir = PathUtils::current_dir()?;
    /// println!("Current directory: {}", current_dir.display());
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the current directory cannot be determined.
    pub fn current_dir() -> FileSystemResult<PathBuf> {
        std::env::current_dir().map_err(std::convert::Into::into)
    }

    /// Makes a path relative to a base path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to make relative
    /// * `base` - The base path to make it relative to
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The relative path
    /// * `Err(FileSystemError)` - If the path is not a child of the base path
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathUtils;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let base = Path::new("/home/user/project");
    /// let path = Path::new("/home/user/project/src/main.js");
    /// let relative = PathUtils::make_relative(path, base)?;
    /// assert_eq!(relative, Path::new("src/main.js"));
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the path is not a child of the base path.
    pub fn make_relative(path: &Path, base: &Path) -> FileSystemResult<PathBuf> {
        path.strip_prefix(base).map(std::path::Path::to_path_buf).map_err(|e| {
            FileSystemError::Validation {
                path: path.to_path_buf(),
                reason: format!("Path is not a child of base path: {e}"),
            }
        })
    }
}

impl PathExt for Path {
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
    fn normalize(&self) -> PathBuf {
        let mut components = Vec::new();
        for component in self.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    components.push(component);
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    components.pop();
                }
                Component::Normal(name) => {
                    components.push(Component::Normal(name));
                }
            }
        }
        components.into_iter().collect()
    }

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
    fn is_in_project(&self) -> bool {
        let mut current = Some(self);
        while let Some(path) = current {
            if path.join("package.json").exists() {
                return true;
            }
            current = path.parent();
        }
        false
    }

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
    fn relative_to_project(&self) -> Option<PathBuf> {
        let mut current = Some(self);
        while let Some(path) = current {
            if path.join("package.json").exists() {
                return self.strip_prefix(path).ok().map(std::path::Path::to_path_buf);
            }
            current = path.parent();
        }
        None
    }

    /// Joins a Node.js path kind to this path.
    ///
    /// This is a convenience method for joining paths like "`node_modules`",
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
    fn node_path(&self, kind: NodePathKind) -> PathBuf {
        self.join(kind.default_path())
    }

    /// Canonicalizes a path, resolving symlinks if needed.
    ///
    /// If the path is a symlink, it resolves to the target path.
    /// Otherwise, it returns the path as-is.
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
    fn canonicalize(&self) -> Result<PathBuf> {
        if self.is_symlink() {
            return Path::canonicalize(self)
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, self)));
        }

        Ok(self.to_path_buf())
    }
}
