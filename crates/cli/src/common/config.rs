use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

use crate::common::paths;

/// Configuration for the workspace CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General configuration settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Daemon-specific configuration
    #[serde(default)]
    pub daemon: DaemonConfig,

    /// Monitor-specific configuration
    #[serde(default)]
    pub monitor: MonitorConfig,

    /// File watcher configuration
    #[serde(default)]
    pub watcher: WatcherConfig,

    /// GitHub integration configuration
    #[serde(default)]
    pub github: GithubConfig,

    /// Configured repositories
    #[serde(default)]
    pub repositories: Vec<RepositoryConfig>,
}

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Whether to auto-start the daemon
    #[serde(default = "default_true")]
    pub auto_start_daemon: bool,
}

/// Daemon-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Path to the daemon socket
    #[serde(default = "default_socket_path")]
    pub socket_path: String,

    /// Path to the daemon PID file
    #[serde(default = "default_pid_file")]
    pub pid_file: String,

    /// Active polling interval in milliseconds
    #[serde(default = "default_polling_interval")]
    pub polling_interval_ms: u64,

    /// Inactive polling interval in milliseconds
    #[serde(default = "default_inactive_polling")]
    pub inactive_polling_ms: u64,
}

/// Monitor-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Refresh rate in milliseconds
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate_ms: u64,

    /// Default view to show on startup
    #[serde(default = "default_view")]
    pub default_view: String,

    /// Color theme
    #[serde(default = "default_theme")]
    pub color_theme: String,
}

/// File watcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Patterns to include when watching for changes
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Patterns to exclude when watching for changes
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Whether to use git hooks
    #[serde(default = "default_true")]
    pub use_git_hooks: bool,
}

/// GitHub integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    /// Whether to enable GitHub integration
    #[serde(default)]
    pub enable_integration: bool,

    /// Path to file containing GitHub token
    #[serde(default = "default_token_path")]
    pub token_path: String,

    /// Fetch interval in seconds
    #[serde(default = "default_fetch_interval")]
    pub fetch_interval_s: u64,
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Path to the repository
    pub path: String,

    /// Name of the repository
    pub name: String,

    /// Whether the repository is active
    #[serde(default = "default_true")]
    pub active: bool,

    /// Branch to monitor
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Repository-specific include patterns
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Repository-specific exclude patterns
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

impl Config {
    /// Load configuration from default location
    pub fn load() -> Result<Self> {
        let config_path = paths::get_config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        Self::load_from(&config_path)
    }

    /// Load configuration from specified path
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save configuration to default location
    pub fn save(&self) -> Result<()> {
        let config_path = paths::get_config_path()?;
        self.save_to(&config_path)
    }

    /// Save configuration to specified path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        let parent = path.parent().context("Failed to get parent directory of config path")?;

        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Add a repository to the configuration
    pub fn add_repository(&mut self, repo: RepositoryConfig) {
        // Check if repository with same name exists
        if let Some(pos) = self.repositories.iter().position(|r| r.name == repo.name) {
            // Replace existing repository
            self.repositories[pos] = repo;
        } else {
            // Add new repository
            self.repositories.push(repo);
        }
    }

    /// Remove a repository from the configuration
    pub fn remove_repository(&mut self, name: &str) -> bool {
        let len = self.repositories.len();
        self.repositories.retain(|r| r.name != name);
        self.repositories.len() < len
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: GeneralConfig::default(),
            daemon: DaemonConfig::default(),
            monitor: MonitorConfig::default(),
            watcher: WatcherConfig::default(),
            github: GithubConfig::default(),
            repositories: Vec::new(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig { log_level: default_log_level(), auto_start_daemon: default_true() }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        DaemonConfig {
            socket_path: default_socket_path(),
            pid_file: default_pid_file(),
            polling_interval_ms: default_polling_interval(),
            inactive_polling_ms: default_inactive_polling(),
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        MonitorConfig {
            refresh_rate_ms: default_refresh_rate(),
            default_view: default_view(),
            color_theme: default_theme(),
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        WatcherConfig {
            include_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.toml".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
            ],
            use_git_hooks: default_true(),
        }
    }
}

impl Default for GithubConfig {
    fn default() -> Self {
        GithubConfig {
            enable_integration: false,
            token_path: default_token_path(),
            fetch_interval_s: default_fetch_interval(),
        }
    }
}

// Default value functions
fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

fn default_socket_path() -> String {
    paths::get_default_socket_path()
        .unwrap_or_else(|_| "~/.local/share/workspace-cli/daemon.sock".to_string())
}

fn default_pid_file() -> String {
    paths::get_default_pid_path()
        .unwrap_or_else(|_| "~/.local/share/workspace-cli/daemon.pid".to_string())
}

fn default_polling_interval() -> u64 {
    500
}

fn default_inactive_polling() -> u64 {
    5000
}

fn default_refresh_rate() -> u64 {
    1000
}

fn default_view() -> String {
    "overview".to_string()
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_token_path() -> String {
    "~/.config/workspace-cli/github_token".to_string()
}

fn default_fetch_interval() -> u64 {
    300
}

fn default_branch() -> String {
    "main".to_string()
}
