//! Project structure and management for Node.js projects.
//!
//! What:
//! This module provides functionality for managing Node.js project structures,
//! including package manager detection, file system operations, and path handling.
//!
//! Who:
//! Used by developers who need to:
//! - Detect and work with Node.js project structures
//! - Manage package manager operations
//! - Handle project-specific file operations
//! - Work with project paths reliably
//!
//! Why:
//! Consistent project structure management is essential for:
//! - Reliable tool operation across different projects
//! - Proper package manager integration
//! - Safe file system operations
//! - Cross-platform compatibility

mod fs;
mod package;
mod structure;
mod utils;

pub use fs::{FileSystem, FileSystemManager};
pub use package::{PackageManager, PackageManagerKind};
pub use structure::{PackageJson, Project, ProjectManager, ValidationStatus};
pub use utils::{NodePathKind, PathExt, PathUtils};

use std::path::PathBuf;

/// Configuration for project detection and management
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Root directory to start searching from
    root: Option<PathBuf>,
    /// Whether to automatically detect package manager
    detect_package_manager: bool,
    /// Whether to validate project structure
    validate_structure: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self { root: None, detect_package_manager: true, validate_structure: true }
    }
}

impl ProjectConfig {
    /// Creates a new ProjectConfig with default settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the root directory for project detection
    ///
    /// # Arguments
    ///
    /// * `root` - Root directory path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_root("/path/to/project");
    /// ```
    #[must_use]
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Sets whether to automatically detect package manager
    ///
    /// # Arguments
    ///
    /// * `detect` - Whether to detect package manager
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .detect_package_manager(true);
    /// ```
    #[must_use]
    pub fn detect_package_manager(mut self, detect: bool) -> Self {
        self.detect_package_manager = detect;
        self
    }

    /// Sets whether to validate project structure
    ///
    /// # Arguments
    ///
    /// * `validate` - Whether to validate structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .validate_structure(true);
    /// ```
    #[must_use]
    pub fn validate_structure(mut self, validate: bool) -> Self {
        self.validate_structure = validate;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config() {
        let config = ProjectConfig::new()
            .with_root("/test/path")
            .detect_package_manager(true)
            .validate_structure(true);

        assert_eq!(config.root, Some(PathBuf::from("/test/path")));
        assert!(config.detect_package_manager);
        assert!(config.validate_structure);
    }

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default();

        assert_eq!(config.root, None);
        assert!(config.detect_package_manager);
        assert!(config.validate_structure);
    }
}
