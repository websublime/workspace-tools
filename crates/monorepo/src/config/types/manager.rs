//! Configuration manager type definitions

use std::path::PathBuf;
use super::MonorepoConfig;

/// Type alias for pattern matcher function
pub type PatternMatcher = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Configuration manager that handles loading, saving, and managing monorepo configurations
/// 
/// Uses direct ownership instead of Arc<RwLock<>> for better Rust ownership semantics.
/// Methods that need to modify config return new ConfigManager instances.
#[derive(Debug)]
pub struct ConfigManager {
    /// The current configuration
    pub(crate) config: MonorepoConfig,

    /// Path to the configuration file
    pub(crate) config_path: Option<PathBuf>,

    /// Whether to auto-save on changes
    pub(crate) auto_save: bool,
}