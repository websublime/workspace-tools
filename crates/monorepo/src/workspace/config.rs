//! Workspace manifest handling.
//!
//! This module provides data structures for representing workspace configuration
//! and manifest information. It defines how a workspace is structured and where
//! to find its packages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Workspace configuration.
///
/// Defines the structure and settings of a monorepo workspace,
/// including its location, package patterns, and package manager.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use sublime_monorepo_tools::WorkspaceConfig;
///
/// // Create a basic configuration
/// let config = WorkspaceConfig::new(PathBuf::from("/path/to/workspace"))
///     .with_packages(vec!["packages/*", "apps/*"])
///     .with_package_manager(Some("npm"));
///
/// assert_eq!(config.package_manager, Some("npm".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Path to the workspace root
    pub root_path: PathBuf,

    /// Package patterns to include
    #[serde(default)]
    pub packages: Vec<String>,

    /// Package manager to use
    #[serde(default)]
    pub package_manager: Option<String>,

    /// Additional configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

impl WorkspaceConfig {
    /// Creates a new workspace configuration.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Path to the workspace root directory
    ///
    /// # Returns
    ///
    /// A new workspace configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::new(PathBuf::from("/path/to/workspace"));
    /// ``
    #[must_use]
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path, packages: Vec::new(), package_manager: None, config: HashMap::new() }
    }

    /// Sets the package patterns.
    ///
    /// # Arguments
    ///
    /// * `packages` - Collection of glob patterns for finding packages
    ///
    /// # Returns
    ///
    /// The modified configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::new(PathBuf::from("/path/to/workspace"))
    ///     .with_packages(vec!["packages/*", "apps/*", "libs/*"]);
    /// ```
    #[must_use]
    pub fn with_packages<I, S>(mut self, packages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.packages = packages.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the package manager.
    ///
    /// # Arguments
    ///
    /// * `package_manager` - Package manager name (e.g., "npm", "yarn", "pnpm")
    ///
    /// # Returns
    ///
    /// The modified configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::new(PathBuf::from("/path/to/workspace"))
    ///     .with_package_manager(Some("pnpm"));
    /// ```
    #[must_use]
    pub fn with_package_manager<S: Into<String>>(mut self, package_manager: Option<S>) -> Self {
        self.package_manager = package_manager.map(Into::into);
        self
    }

    /// Adds additional configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - Configuration key
    /// * `value` - Configuration value
    ///
    /// # Returns
    ///
    /// The modified configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::new(PathBuf::from("/path/to/workspace"))
    ///     .with_config("npmClient", "yarn")
    ///     .with_config("useWorkspaces", true);
    /// ```
    #[must_use]
    pub fn with_config<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.config.insert(key.into(), value.into());
        self
    }
}
