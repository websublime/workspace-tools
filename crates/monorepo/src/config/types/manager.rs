//! Configuration manager type definitions

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use super::MonorepoConfig;

/// Type alias for pattern matcher function
pub type PatternMatcher = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Configuration manager that handles loading, saving, and managing monorepo configurations
pub struct ConfigManager {
    /// The current configuration
    pub(crate) config: Arc<RwLock<MonorepoConfig>>,

    /// Path to the configuration file
    pub(crate) config_path: Option<PathBuf>,

    /// Whether to auto-save on changes
    pub(crate) auto_save: bool,
}