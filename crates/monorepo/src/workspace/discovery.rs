//! Workspace discovery functionality.
//!
//! This module provides options for discovering packages within a monorepo workspace,
//! including pattern-based inclusion and exclusion as well as depth limits and
//! other discovery configuration options.

use std::path::PathBuf;

/// Options for discovering a workspace.
///
/// Controls how packages are discovered within a workspace, including
/// which directories to search, patterns to include or exclude, and
/// other discovery parameters.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::DiscoveryOptions;
///
/// // Create with default options
/// let defaults = DiscoveryOptions::default();
///
/// // Create with custom options
/// let options = DiscoveryOptions::new()
///     .auto_detect_root(true)
///     .include_patterns(vec!["**/package.json", "packages/*/package.json"])
///     .exclude_patterns(vec!["**/node_modules/**", "**/dist/**"])
///     .max_depth(5)
///     .include_private(false);
/// ```
#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    /// Whether to automatically detect the project root
    pub auto_detect_root: bool,
    /// Whether to detect the package manager
    pub detect_package_manager: bool,
    /// Package patterns to include
    pub include_patterns: Vec<String>,
    /// Package patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum depth to search for packages
    pub max_depth: usize,
    /// Whether to discover private packages
    pub include_private: bool,
    /// Custom package.json paths to include
    pub additional_package_paths: Vec<PathBuf>,
}

impl DiscoveryOptions {
    /// Creates new discovery options with default settings.
    ///
    /// # Returns
    ///
    /// New discovery options with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// let options = DiscoveryOptions::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to auto-detect the project root.
    ///
    /// When true, attempts to find the project root automatically by
    /// looking for common root indicators like .git, package.json, etc.
    ///
    /// # Arguments
    ///
    /// * `value` - Whether to auto-detect the root
    ///
    /// # Returns
    ///
    /// The modified options.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// let options = DiscoveryOptions::new().auto_detect_root(true);
    /// ```
    #[must_use]
    pub fn auto_detect_root(mut self, value: bool) -> Self {
        self.auto_detect_root = value;
        self
    }

    /// Sets whether to detect the package manager.
    ///
    /// When true, attempts to determine which package manager is used
    /// by the workspace (npm, yarn, pnpm, etc.)
    ///
    /// # Arguments
    ///
    /// * `value` - Whether to detect the package manager
    ///
    /// # Returns
    ///
    /// The modified options.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// let options = DiscoveryOptions::new().detect_package_manager(true);
    /// ```
    #[must_use]
    pub fn detect_package_manager(mut self, value: bool) -> Self {
        self.detect_package_manager = value;
        self
    }

    /// Sets package patterns to include.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Collection of glob patterns for finding packages
    ///
    /// # Returns
    ///
    /// The modified options.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// let options = DiscoveryOptions::new()
    ///     .include_patterns(vec!["packages/*/package.json", "apps/*/package.json"]);
    /// ```
    #[must_use]
    pub fn include_patterns<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.include_patterns = patterns.into_iter().map(Into::into).collect();
        self
    }

    /// Sets package patterns to exclude.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Collection of glob patterns to exclude
    ///
    /// # Returns
    ///
    /// The modified options.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// let options = DiscoveryOptions::new()
    ///     .exclude_patterns(vec!["**/node_modules/**", "**/dist/**", "**/build/**"]);
    /// ```
    #[must_use]
    pub fn exclude_patterns<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exclude_patterns = patterns.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the maximum depth to search for packages.
    ///
    /// Limits how deep in the directory tree to look for packages.
    ///
    /// # Arguments
    ///
    /// * `depth` - Maximum directory depth
    ///
    /// # Returns
    ///
    /// The modified options.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// let options = DiscoveryOptions::new().max_depth(3);
    /// ```
    #[must_use]
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets whether to include private packages.
    ///
    /// When true, includes packages marked as private in their package.json.
    ///
    /// # Arguments
    ///
    /// * `value` - Whether to include private packages
    ///
    /// # Returns
    ///
    /// The modified options.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// // Include private packages
    /// let options = DiscoveryOptions::new().include_private(true);
    ///
    /// // Exclude private packages
    /// let options = DiscoveryOptions::new().include_private(false);
    /// ```
    #[must_use]
    pub fn include_private(mut self, value: bool) -> Self {
        self.include_private = value;
        self
    }

    /// Adds additional package paths to include.
    ///
    /// Explicitly includes these paths in addition to those found by patterns.
    ///
    /// # Arguments
    ///
    /// * `paths` - Collection of paths to package.json files
    ///
    /// # Returns
    ///
    /// The modified options.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::DiscoveryOptions;
    ///
    /// let options = DiscoveryOptions::new()
    ///     .additional_package_paths(vec![
    ///         PathBuf::from("special/package/package.json"),
    ///     ]);
    /// ```
    #[must_use]
    pub fn additional_package_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.additional_package_paths = paths.into_iter().map(Into::into).collect();
        self
    }
}

#[allow(clippy::doc_link_with_quotes)]
impl Default for DiscoveryOptions {
    /// Creates default discovery options.
    ///
    /// Default settings:
    /// - auto_detect_root: true
    /// - detect_package_manager: true
    /// - include_patterns: ["**/package.json"]
    /// - exclude_patterns: ["**/node_modules/**", "**/bower_components/**"]
    /// - max_depth: 10
    /// - include_private: true
    /// - additional_package_paths: []
    ///
    /// # Returns
    ///
    /// Default discovery options.
    fn default() -> Self {
        Self {
            auto_detect_root: true,
            detect_package_manager: true,
            include_patterns: vec!["**/package.json".to_string()],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/bower_components/**".to_string(),
            ],
            max_depth: 10,
            include_private: true,
            additional_package_paths: Vec::new(),
        }
    }
}
