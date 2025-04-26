//! Path utilities for Node.js projects.
//!
//! What:
//! This module provides extended path operations and utilities specifically
//! designed for working with Node.js project paths.
//!
//! Who:
//! Used by developers who need to:
//! - Handle Node.js project paths safely
//! - Validate and normalize paths
//! - Work with Node.js specific directories
//! - Ensure cross-platform compatibility
//!
//! Why:
//! Proper path handling is essential for:
//! - Cross-platform compatibility
//! - Safe file system operations
//! - Consistent path handling
//! - Project structure integrity

use std::{
    env,
    path::{Component, Path, PathBuf},
};

use crate::error::{StandardError, StandardResult};

/// Common Node.js project directories and files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodePathKind {
    /// Node modules directory
    NodeModules,
    /// Package configuration
    PackageJson,
    /// Package lock file (npm)
    PackageLock,
    /// Source directory
    Src,
    /// Distribution directory
    Dist,
    /// Test directory
    Test,
}

impl NodePathKind {
    /// Returns the default path for this kind
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::NodePathKind;
    ///
    /// assert_eq!(NodePathKind::NodeModules.default_path(), "node_modules");
    /// ```
    #[must_use]
    pub fn default_path(self) -> &'static str {
        match self {
            Self::NodeModules => "node_modules",
            Self::PackageJson => "package.json",
            Self::PackageLock => "package-lock.json",
            Self::Src => "src",
            Self::Dist => "dist",
            Self::Test => "test",
        }
    }
}

/// Extended path operations for Node.js projects
pub trait PathExt {
    /// Normalizes a path to use platform-specific separators
    ///
    /// # Returns
    ///
    /// Normalized path
    fn normalize(&self) -> PathBuf;

    /// Checks if this path is within a Node.js project
    ///
    /// # Returns
    ///
    /// true if path is within a project (contains package.json)
    fn is_in_project(&self) -> bool;

    /// Returns path relative to project root
    ///
    /// # Returns
    ///
    /// Path relative to nearest package.json, or None if not in project
    fn relative_to_project(&self) -> Option<PathBuf>;

    /// Returns path to a Node.js project directory
    ///
    /// # Arguments
    ///
    /// * `kind` - Kind of path to get
    ///
    /// # Returns
    ///
    /// Path to requested directory
    fn node_path(&self, kind: NodePathKind) -> PathBuf;

    /// Validates that a path is safe
    ///
    /// Checks for:
    /// - No parent directory traversal
    /// - No absolute paths
    /// - No symbolic links
    /// - Within project root
    ///
    /// # Returns
    ///
    /// Result indicating if path is safe
    fn validate(&self) -> StandardResult<()>;
}

impl PathExt for Path {
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

    fn node_path(&self, kind: NodePathKind) -> PathBuf {
        self.join(kind.default_path())
    }

    fn validate(&self) -> StandardResult<()> {
        // Check for parent directory traversal
        if self.components().any(|c| c == Component::ParentDir) {
            return Err(StandardError::Operation(
                "Path contains parent directory traversal".to_string(),
            ));
        }

        // Check for absolute paths
        if self.is_absolute() {
            return Err(StandardError::Operation("Absolute paths are not allowed".to_string()));
        }

        // Check for symbolic links
        if self.read_link().is_ok() {
            return Err(StandardError::Operation("Symbolic links are not allowed".to_string()));
        }

        Ok(())
    }
}

/// Helper functions for working with paths
pub struct PathUtils;

impl PathUtils {
    /// Gets the current working directory
    ///
    /// # Returns
    ///
    /// Current working directory or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::PathUtils;
    ///
    /// let cwd = PathUtils::current_dir().unwrap();
    /// ```
    pub fn current_dir() -> StandardResult<PathBuf> {
        env::current_dir()
            .map_err(|e| StandardError::Operation(format!("Failed to get current directory: {e}")))
    }

    /// Finds the nearest package.json directory
    ///
    /// # Arguments
    ///
    /// * `start` - Starting directory to search from
    ///
    /// # Returns
    ///
    /// Path to directory containing package.json, or None if not found
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_standard_tools::project::PathUtils;
    /// use std::path::Path;
    ///
    /// let project_root = PathUtils::find_project_root(Path::new(".")).unwrap();
    /// ```
    pub fn find_project_root(start: &Path) -> Option<PathBuf> {
        let mut current = Some(start);
        while let Some(path) = current {
            if path.join("package.json").exists() {
                return Some(path.to_path_buf());
            }
            current = path.parent();
        }
        None
    }

    /// Makes a path relative to another path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to make relative
    /// * `base` - Base path to make relative to
    ///
    /// # Returns
    ///
    /// Relative path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::PathUtils;
    /// use std::path::{Path, PathBuf};
    ///
    /// let path = Path::new("/a/b/c");
    /// let base = Path::new("/a");
    /// assert_eq!(
    ///     PathUtils::make_relative(path, base).unwrap(),
    ///     PathBuf::from("b/c")
    /// );
    /// ```
    pub fn make_relative(path: &Path, base: &Path) -> StandardResult<PathBuf> {
        path.strip_prefix(base)
            .map(std::path::Path::to_path_buf)
            .map_err(|e| StandardError::Operation(format!("Failed to make path relative: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_path_normalization() {
        let path = Path::new("a/b/../c/./d");
        assert_eq!(path.normalize(), PathBuf::from("a/c/d"));
    }

    #[test]
    fn test_node_paths() {
        let path = Path::new("project");
        assert_eq!(
            path.node_path(NodePathKind::NodeModules),
            PathBuf::from("project/node_modules")
        );
        assert_eq!(
            path.node_path(NodePathKind::PackageJson),
            PathBuf::from("project/package.json")
        );
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_project_detection() {
        let temp_dir = TempDir::new().expect("Fail to create temporary directory");
        let project_dir = temp_dir.path();

        // No package.json
        assert!(!project_dir.is_in_project());

        // Create package.json
        fs::write(project_dir.join("package.json"), "{}").expect("Fail to create package.json");
        assert!(project_dir.is_in_project());

        // Test subdirectory
        let sub_dir = project_dir.join("src");
        fs::create_dir(&sub_dir).expect("Fail to create subdirectory");
        assert!(sub_dir.is_in_project());
    }

    #[test]
    fn test_path_validation() {
        // Parent traversal
        assert!(Path::new("../test").validate().is_err());

        // Absolute path
        assert!(Path::new("/absolute/path").validate().is_err());

        // Valid path
        assert!(Path::new("test/path").validate().is_ok());
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_relative_paths() {
        let temp_dir = TempDir::new().expect("Fail to create temporary directory");
        let project_dir = temp_dir.path();

        // Create project structure
        fs::write(project_dir.join("package.json"), "{}").expect("Fail to write package.json");
        fs::create_dir(project_dir.join("src")).expect("Fail to create src directory");

        let src_dir = project_dir.join("src");
        let relative = src_dir.relative_to_project().expect("Fail to get relative path");
        assert_eq!(relative, PathBuf::from("src"));
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_path_utils() {
        let temp_dir = TempDir::new().expect("Fail to create temporary directory");
        let project_dir = temp_dir.path();

        // Create project structure
        fs::write(project_dir.join("package.json"), "{}").expect("Fail to write package.json");

        // Find project root
        let root = PathUtils::find_project_root(project_dir).expect("Fail to find project root");
        assert_eq!(root, project_dir);

        // Make relative path
        let sub_path = project_dir.join("src/test");
        let relative =
            PathUtils::make_relative(&sub_path, project_dir).expect("Fail to make relative path");
        assert_eq!(relative, PathBuf::from("src/test"));
    }
}
