//! CLI Configuration management
//!
//! Manages CLI-specific configuration including verbosity levels,
//! output formatting preferences, and user settings.

use crate::error::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// CLI-specific configuration
///
/// Contains settings that affect how the CLI behaves, including
/// output formatting, verbosity levels, and user preferences.
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Verbosity level (0 = normal, 1+ = increasingly verbose)
    pub verbosity: u8,
    
    /// Quiet mode (suppress all output except errors)
    pub quiet: bool,
    
    /// Use colored output
    pub use_color: bool,
    
    /// Debug mode enabled
    pub debug: bool,
    
    /// Path to configuration file (if any)
    pub config_file: Option<PathBuf>,
    
    /// User preferences loaded from config file
    pub preferences: UserPreferences,
}

/// User preferences that can be saved to a configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Default output format
    pub default_output_format: String,
    
    /// Default working directory
    pub default_directory: Option<PathBuf>,
    
    /// Whether to use colors by default
    pub use_colors: bool,
    
    /// Default verbosity level
    pub default_verbosity: u8,
    
    /// Preferred editor for interactive commands
    pub editor: Option<String>,
    
    /// Custom aliases for commands
    pub aliases: std::collections::HashMap<String, String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            default_output_format: "human".to_string(),
            default_directory: None,
            use_colors: true,
            default_verbosity: 0,
            editor: std::env::var("EDITOR").ok(),
            aliases: std::collections::HashMap::new(),
        }
    }
}

impl CliConfig {
    /// Create a new CLI configuration
    ///
    /// # Arguments
    ///
    /// * `config_file` - Optional path to configuration file
    /// * `verbosity` - Verbosity level from command line
    /// * `quiet` - Quiet mode flag
    /// * `use_color` - Whether to use colored output
    /// * `debug` - Debug mode flag
    ///
    /// # Returns
    ///
    /// A new CliConfig instance
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read or parsed
    pub fn new(
        config_file: Option<&Path>,
        verbosity: u8,
        quiet: bool,
        use_color: bool,
        debug: bool,
    ) -> CliResult<Self> {
        let preferences = if let Some(path) = config_file {
            Self::load_preferences(path)?
        } else {
            // Try to load from default locations
            Self::load_default_preferences()?
        };

        // Command line arguments override config file settings
        let final_use_color = if !use_color {
            false  // Command line --no-color overrides everything
        } else {
            preferences.use_colors
        };

        let final_verbosity = if verbosity > 0 {
            verbosity  // Command line verbosity overrides config
        } else {
            preferences.default_verbosity
        };

        Ok(Self {
            verbosity: final_verbosity,
            quiet,
            use_color: final_use_color,
            debug,
            config_file: config_file.map(|p| p.to_path_buf()),
            preferences,
        })
    }

    /// Load preferences from a specific file
    fn load_preferences(path: &Path) -> CliResult<UserPreferences> {
        if !path.exists() {
            return Ok(UserPreferences::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| CliError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let preferences: UserPreferences = toml::from_str(&content)
            .map_err(|e| CliError::ConfigError(format!("Failed to parse config file: {}", e)))?;

        Ok(preferences)
    }

    /// Load preferences from default locations
    fn load_default_preferences() -> CliResult<UserPreferences> {
        // Try common config locations
        let config_dirs = [
            dirs::config_dir().map(|d| d.join("monorepo").join("config.toml")),
            Some(PathBuf::from(".monorepo.toml")),
            Some(PathBuf::from("monorepo.toml")),
        ];

        for config_path in config_dirs.iter().flatten() {
            if config_path.exists() {
                return Self::load_preferences(config_path);
            }
        }

        Ok(UserPreferences::default())
    }

    /// Save current preferences to a configuration file
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to save the configuration
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn save_preferences(&self, path: &Path) -> CliResult<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CliError::ConfigError(format!("Failed to create config directory: {}", e)))?;
        }

        let content = toml::to_string_pretty(&self.preferences)
            .map_err(|e| CliError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| CliError::ConfigError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Check if the given verbosity level should be logged
    pub fn should_log(&self, level: u8) -> bool {
        !self.quiet && self.verbosity >= level
    }

    /// Check if debug information should be shown
    pub fn is_debug(&self) -> bool {
        self.debug || self.verbosity >= 3
    }

    /// Get the effective editor command
    pub fn get_editor(&self) -> String {
        self.preferences.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "vi".to_string())
    }

    /// Resolve a command alias if it exists
    pub fn resolve_alias<'a>(&'a self, command: &'a str) -> &'a str {
        self.preferences.aliases.get(command).map(|s| s.as_str()).unwrap_or(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = CliConfig::new(None, 0, false, true, false).unwrap();
        assert!(!config.quiet);
        assert!(config.use_color);
        assert!(!config.debug);
        assert_eq!(config.verbosity, 0);
    }

    #[test]
    fn test_verbosity_override() {
        let config = CliConfig::new(None, 2, false, true, false).unwrap();
        assert_eq!(config.verbosity, 2);
        assert!(config.should_log(1));
        assert!(config.should_log(2));
        assert!(!config.should_log(3));
    }

    #[test]
    fn test_quiet_mode() {
        let config = CliConfig::new(None, 1, true, true, false).unwrap();
        assert!(config.quiet);
        assert!(!config.should_log(1));
    }

    #[test]
    fn test_config_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"
default_output_format = "json"
use_colors = false
default_verbosity = 1

[aliases]
analyze = "analyze --detailed"
        "#).unwrap();

        let config = CliConfig::new(Some(temp_file.path()), 0, false, true, false).unwrap();
        
        assert_eq!(config.preferences.default_output_format, "json");
        assert!(!config.use_color); // Config file setting should be applied
        assert_eq!(config.verbosity, 1); // Config file default should be used
        assert_eq!(config.resolve_alias("analyze"), "analyze --detailed");
    }

    #[test]
    fn test_command_line_overrides() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"
use_colors = true
default_verbosity = 1
        "#).unwrap();

        let config = CliConfig::new(Some(temp_file.path()), 2, false, false, false).unwrap();
        
        assert!(!config.use_color); // Command line --no-color should override
        assert_eq!(config.verbosity, 2); // Command line verbosity should override
    }
}