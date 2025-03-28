//! Workspace discovery functionality.

use std::path::PathBuf;

/// Options for discovering a workspace.
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to auto-detect the project root.
    #[must_use]
    pub fn auto_detect_root(mut self, value: bool) -> Self {
        self.auto_detect_root = value;
        self
    }

    /// Sets whether to detect the package manager.
    #[must_use]
    pub fn detect_package_manager(mut self, value: bool) -> Self {
        self.detect_package_manager = value;
        self
    }

    /// Sets package patterns to include.
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
    #[must_use]
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets whether to include private packages.
    #[must_use]
    pub fn include_private(mut self, value: bool) -> Self {
        self.include_private = value;
        self
    }

    /// Adds additional package paths to include.
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

impl Default for DiscoveryOptions {
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
