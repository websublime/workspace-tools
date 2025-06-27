//! Configuration management components
//!
//! This module provides focused components for different aspects of configuration management:
//! - **Persistence**: Loading and saving configuration files
//! - **Reader**: Read-only access to configuration sections
//! - **Writer**: Configuration updates and modifications  
//! - **Workspace**: Workspace pattern management
//! - **Matcher**: Pattern matching and validation
//!
//! These components replace the monolithic ConfigManager with focused, single-responsibility components.

pub mod persistence;
pub mod reader;
pub mod writer;
pub mod workspace;
pub mod matcher;

pub use persistence::ConfigPersistence;
pub use reader::ConfigReader;
pub use writer::ConfigWriter;
pub use workspace::WorkspacePatternManager;
pub use matcher::{PatternMatcher, MultiPatternMatcher};

/// Configuration manager factory for creating focused components
pub struct ConfigComponents;

impl ConfigComponents {
    /// Create a new configuration persistence component
    #[must_use]
    pub fn persistence() -> ConfigPersistence {
        ConfigPersistence::new()
    }

    /// Create a configuration reader for the given config
    #[must_use]
    pub fn reader<'a>(
        config: &'a crate::config::MonorepoConfig,
        config_path: Option<&'a std::path::Path>,
    ) -> ConfigReader<'a> {
        ConfigReader::new(config, config_path)
    }

    /// Create a configuration writer with the given config
    #[must_use]
    pub fn writer(config: crate::config::MonorepoConfig) -> ConfigWriter {
        ConfigWriter::new(config)
    }

    /// Create a configuration writer from file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// Configuration writer loaded from file
    ///
    /// # Errors
    /// Returns an error if the file cannot be loaded
    pub fn writer_from_file(path: impl AsRef<std::path::Path>) -> crate::error::Result<ConfigWriter> {
        ConfigWriter::from_file(path)
    }

    /// Create a workspace pattern manager
    #[must_use]
    pub fn workspace_manager(workspace_config: crate::config::WorkspaceConfig) -> WorkspacePatternManager {
        WorkspacePatternManager::new(workspace_config)
    }

    /// Create a pattern matcher for a specific pattern
    ///
    /// # Arguments
    /// * `pattern` - Pattern string to create matcher for
    ///
    /// # Returns
    /// Pattern matcher instance
    ///
    /// # Errors
    /// Returns an error if the pattern is invalid
    pub fn pattern_matcher(pattern: &str) -> crate::error::Result<PatternMatcher> {
        PatternMatcher::from_str(pattern)
    }

    /// Create a multi-pattern matcher from pattern strings
    ///
    /// # Arguments
    /// * `patterns` - Pattern strings to compile
    ///
    /// # Returns
    /// Multi-pattern matcher instance
    ///
    /// # Errors
    /// Returns an error if any pattern is invalid
    pub fn multi_pattern_matcher(patterns: &[String]) -> crate::error::Result<MultiPatternMatcher> {
        MultiPatternMatcher::from_patterns(patterns)
    }
}