//! Plugin system configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    /// Enabled plugins
    pub enabled: Vec<String>,

    /// Plugin directories
    pub plugin_dirs: Vec<PathBuf>,

    /// Plugin-specific configurations
    pub configs: HashMap<String, serde_json::Value>,
}
