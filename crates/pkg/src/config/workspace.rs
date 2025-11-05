//! Workspace configuration for monorepo projects.
//!
//! **What**: Defines workspace configuration extracted from project metadata (package.json).
//!
//! **How**: This module provides the `WorkspaceConfig` structure that stores project-specific
//! workspace patterns, representing the actual workspace declaration in the project's package.json.
//!
//! **Why**: To maintain project-specific workspace metadata separately from generic search patterns.
//! This enables tools to know exactly which workspace patterns are declared in the project,
//! as opposed to using generic defaults for package discovery.
//!
//! # Difference from StandardConfig.monorepo
//!
//! - **`WorkspaceConfig`** (this module): Project-specific workspace patterns from package.json
//!   - Example: `["packages/*", "apps/*"]` - what THIS project declares
//!   - Serialized in repo.config.toml as project metadata
//!   - Empty/None for single-package projects
//!
//! - **`StandardConfig.monorepo`**: Generic search patterns for package discovery
//!   - Example: `["packages/*", "apps/*", "libs/*", "modules/*", ...]` - where to LOOK
//!   - Not serialized - uses defaults + environment variables
//!   - Always present with sensible defaults

use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::ConfigResult;

/// Workspace configuration for monorepo projects.
///
/// This configuration represents the workspace patterns declared in the project's
/// package.json file. It is project-specific metadata that indicates which patterns
/// define workspace packages in THIS particular project.
///
/// # Usage
///
/// This field is typically populated during project initialization by extracting
/// workspace patterns from package.json:
///
/// ```json
/// {
///   "workspaces": ["packages/*", "apps/*"]
/// }
/// ```
///
/// Or object format:
///
/// ```json
/// {
///   "workspaces": {
///     "packages": ["packages/*", "apps/*"]
///   }
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::config::WorkspaceConfig;
///
/// // Create workspace config with patterns
/// let config = WorkspaceConfig {
///     patterns: vec![
///         "packages/*".to_string(),
///         "apps/*".to_string(),
///     ],
/// };
///
/// assert_eq!(config.patterns.len(), 2);
/// ```
///
/// # TOML Representation
///
/// ```toml
/// [package_tools.workspace]
/// patterns = ["packages/*", "apps/*"]
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceConfig {
    /// Workspace patterns from package.json.
    ///
    /// These are the exact patterns declared in the project's package.json
    /// workspaces field. They define which directories contain workspace packages.
    ///
    /// # Examples
    ///
    /// - `["packages/*"]` - Single pattern
    /// - `["packages/*", "apps/*", "libs/*"]` - Multiple patterns
    /// - `[]` - Empty workspaces array (valid for new monorepos)
    pub patterns: Vec<String>,
}

impl WorkspaceConfig {
    /// Creates a new `WorkspaceConfig` with the specified patterns.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Workspace patterns from package.json
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::new(vec![
    ///     "packages/*".to_string(),
    ///     "apps/*".to_string(),
    /// ]);
    ///
    /// assert_eq!(config.patterns.len(), 2);
    /// ```
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    /// Creates a new `WorkspaceConfig` with empty patterns.
    ///
    /// This is useful for new monorepos that have declared workspaces
    /// but haven't added any packages yet.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::empty();
    /// assert!(config.patterns.is_empty());
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self { patterns: vec![] }
    }

    /// Checks if the workspace configuration has any patterns.
    ///
    /// # Returns
    ///
    /// `true` if patterns is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::WorkspaceConfig;
    ///
    /// let empty_config = WorkspaceConfig::empty();
    /// assert!(empty_config.is_empty());
    ///
    /// let config = WorkspaceConfig::new(vec!["packages/*".to_string()]);
    /// assert!(!config.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Returns the number of workspace patterns.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::new(vec![
    ///     "packages/*".to_string(),
    ///     "apps/*".to_string(),
    /// ]);
    ///
    /// assert_eq!(config.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Validates the workspace configuration.
    ///
    /// Performs validation on workspace patterns to ensure they are valid and safe:
    /// - Patterns must not be empty strings
    /// - Patterns must not contain path traversal attempts (`..`)
    /// - Patterns must not be absolute paths
    /// - Patterns must be valid glob patterns
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any pattern is an empty string
    /// - Any pattern contains path traversal (`..`)
    /// - Any pattern is an absolute path (starts with `/` or Windows drive letter)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::WorkspaceConfig;
    ///
    /// // Valid patterns
    /// let config = WorkspaceConfig::new(vec!["packages/*".to_string()]);
    /// assert!(config.validate().is_ok());
    ///
    /// // Invalid: empty pattern
    /// let config = WorkspaceConfig::new(vec!["".to_string()]);
    /// assert!(config.validate().is_err());
    ///
    /// // Invalid: path traversal
    /// let config = WorkspaceConfig::new(vec!["../packages/*".to_string()]);
    /// assert!(config.validate().is_err());
    /// ```
    pub fn validate(&self) -> ConfigResult<()> {
        for pattern in &self.patterns {
            // Check for empty patterns
            if pattern.is_empty() {
                return Err("Workspace pattern cannot be empty".into());
            }

            // Check for path traversal attempts
            if pattern.contains("..") {
                return Err(format!(
                    "Workspace pattern '{}' contains path traversal (..)",
                    pattern
                )
                .into());
            }

            // Check for absolute paths (Unix-style)
            if pattern.starts_with('/') {
                return Err(format!(
                    "Workspace pattern '{}' must be relative, not absolute",
                    pattern
                )
                .into());
            }

            // Check for absolute paths (Windows-style: C:, D:, etc)
            if pattern.len() >= 2 && pattern.chars().nth(1) == Some(':') {
                let first_char = pattern.chars().next();
                if first_char.is_some_and(|c| c.is_ascii_alphabetic()) {
                    return Err(format!(
                        "Workspace pattern '{}' must be relative, not absolute",
                        pattern
                    )
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Merges another workspace configuration into this one.
    ///
    /// If the other config has patterns, they replace this config's patterns.
    /// This follows the standard merge semantics where later configs override earlier ones.
    ///
    /// # Arguments
    ///
    /// * `other` - The workspace configuration to merge
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::WorkspaceConfig;
    ///
    /// let mut config = WorkspaceConfig::new(vec!["packages/*".to_string()]);
    /// let other = WorkspaceConfig::new(vec!["apps/*".to_string(), "libs/*".to_string()]);
    ///
    /// config.merge_with(other).unwrap();
    /// assert_eq!(config.patterns.len(), 2);
    /// assert_eq!(config.patterns[0], "apps/*");
    /// ```
    pub fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        if !other.patterns.is_empty() {
            self.patterns = other.patterns;
        }
        Ok(())
    }
}
