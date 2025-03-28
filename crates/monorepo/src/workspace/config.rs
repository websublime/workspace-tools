//! Workspace manifest handling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Workspace configuration.
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
    #[must_use]
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path, packages: Vec::new(), package_manager: None, config: HashMap::new() }
    }

    /// Sets the package patterns.
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
    #[must_use]
    pub fn with_package_manager<S: Into<String>>(mut self, package_manager: Option<S>) -> Self {
        self.package_manager = package_manager.map(Into::into);
        self
    }

    /// Adds additional configuration.
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
