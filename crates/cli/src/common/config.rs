use anyhow::{self, Context, Result};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};
use toml::{self, Value};

use crate::common::paths;

/// Sources of configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigSource {
    /// Default built-in configuration
    Defaults,
    /// System-wide configuration
    SystemConfig,
    /// User configuration in user's config directory
    UserConfig,
    /// Project-specific configuration in the current workspace
    ProjectConfig,
    /// Environment variables
    Environment,
    /// Command line arguments
    CommandLine,
}

/// Configuration for the workspace CLI
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

    /// Additional fields for forward compatibility
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
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
    /// Load configuration from default location with all standard sources
    pub fn load() -> Result<Self> {
        Self::load_with_sources(vec![
            ConfigSource::Defaults,
            ConfigSource::SystemConfig,
            ConfigSource::UserConfig,
            ConfigSource::ProjectConfig,
            ConfigSource::Environment,
        ])
    }

    /// Load configuration from specified sources in order of precedence
    pub fn load_with_sources(sources: Vec<ConfigSource>) -> Result<Self> {
        let mut config = Config::default();

        for source in sources {
            match source {
                ConfigSource::Defaults => {
                    // Default config is already applied by the Default trait
                    debug!("Loaded default configuration");
                }
                ConfigSource::SystemConfig => {
                    if let Some(system_config_path) = Self::find_system_config() {
                        if system_config_path.exists() {
                            match Self::load_from(&system_config_path) {
                                Ok(system_config) => {
                                    config = config.merge_with(&system_config);
                                    debug!(
                                        "Loaded system configuration from {}",
                                        system_config_path.display()
                                    );
                                }
                                Err(err) => {
                                    warn!("Failed to load system configuration: {}", err);
                                }
                            }
                        }
                    }
                }
                ConfigSource::UserConfig => {
                    let user_config_path = paths::get_config_path()?;
                    if user_config_path.exists() {
                        match Self::load_from(&user_config_path) {
                            Ok(user_config) => {
                                config = config.merge_with(&user_config);
                                debug!(
                                    "Loaded user configuration from {}",
                                    user_config_path.display()
                                );
                            }
                            Err(err) => {
                                warn!("Failed to load user configuration: {}", err);
                            }
                        }
                    }
                }
                ConfigSource::ProjectConfig => {
                    if let Ok(project_root) = paths::find_project_root(None) {
                        let project_config_path = project_root.join(".workspace-cli.toml");
                        if project_config_path.exists() {
                            match Self::load_from(&project_config_path) {
                                Ok(project_config) => {
                                    config = config.merge_with(&project_config);
                                    debug!(
                                        "Loaded project configuration from {}",
                                        project_config_path.display()
                                    );
                                }
                                Err(err) => {
                                    warn!("Failed to load project configuration: {}", err);
                                }
                            }
                        }
                    }
                }
                ConfigSource::Environment => {
                    config = config.merge_from_env();
                    debug!("Merged configuration from environment variables");
                }
                ConfigSource::CommandLine => {
                    // Command line is handled separately when parsing CLI arguments
                    debug!("Command line options will be applied separately");
                }
            }
        }

        Ok(config)
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

    /// Merge this configuration with another one (other takes precedence)
    pub fn merge_with(&self, other: &Config) -> Config {
        let mut result = self.clone();

        // Merge general config
        result.general.log_level = if other.general.log_level != default_log_level() {
            other.general.log_level.clone()
        } else {
            self.general.log_level.clone()
        };
        result.general.auto_start_daemon = other.general.auto_start_daemon;

        // Merge daemon config
        if other.daemon.socket_path != default_socket_path() {
            result.daemon.socket_path = other.daemon.socket_path.clone();
        }
        if other.daemon.pid_file != default_pid_file() {
            result.daemon.pid_file = other.daemon.pid_file.clone();
        }
        if other.daemon.polling_interval_ms != default_polling_interval() {
            result.daemon.polling_interval_ms = other.daemon.polling_interval_ms;
        }
        if other.daemon.inactive_polling_ms != default_inactive_polling() {
            result.daemon.inactive_polling_ms = other.daemon.inactive_polling_ms;
        }

        // Merge monitor config
        if other.monitor.refresh_rate_ms != default_refresh_rate() {
            result.monitor.refresh_rate_ms = other.monitor.refresh_rate_ms;
        }
        if other.monitor.default_view != default_view() {
            result.monitor.default_view = other.monitor.default_view.clone();
        }
        if other.monitor.color_theme != default_theme() {
            result.monitor.color_theme = other.monitor.color_theme.clone();
        }

        // Merge watcher config
        if !other.watcher.include_patterns.is_empty() {
            result.watcher.include_patterns = other.watcher.include_patterns.clone();
        }
        if !other.watcher.exclude_patterns.is_empty() {
            result.watcher.exclude_patterns = other.watcher.exclude_patterns.clone();
        }
        result.watcher.use_git_hooks = other.watcher.use_git_hooks;

        // Merge GitHub config
        result.github.enable_integration = other.github.enable_integration;
        if other.github.token_path != default_token_path() {
            result.github.token_path = other.github.token_path.clone();
        }
        if other.github.fetch_interval_s != default_fetch_interval() {
            result.github.fetch_interval_s = other.github.fetch_interval_s;
        }

        // Merge repositories (keeping all repositories from both configs)
        let mut all_repositories = self.repositories.clone();

        // Add repositories from other that don't exist in self
        for repo in &other.repositories {
            if !all_repositories.iter().any(|r| r.name == repo.name) {
                all_repositories.push(repo.clone());
            } else {
                // Replace existing repository with the same name
                if let Some(pos) = all_repositories.iter().position(|r| r.name == repo.name) {
                    all_repositories[pos] = repo.clone();
                }
            }
        }

        result.repositories = all_repositories;

        // Merge extra fields
        for (key, value) in &other.extra {
            result.extra.insert(key.clone(), value.clone());
        }

        result
    }

    /// Merge configuration from environment variables
    pub fn merge_from_env(&self) -> Config {
        let mut result = self.clone();

        // LOG_LEVEL environment variable
        if let Ok(log_level) = env::var("WORKSPACE_LOG_LEVEL") {
            result.general.log_level = log_level;
        }

        // AUTO_START_DAEMON environment variable
        if let Ok(auto_start) = env::var("WORKSPACE_AUTO_START_DAEMON") {
            result.general.auto_start_daemon = auto_start == "true" || auto_start == "1";
        }

        // SOCKET_PATH environment variable
        if let Ok(socket_path) = env::var("WORKSPACE_SOCKET_PATH") {
            result.daemon.socket_path = socket_path;
        }

        // PID_FILE environment variable
        if let Ok(pid_file) = env::var("WORKSPACE_PID_FILE") {
            result.daemon.pid_file = pid_file;
        }

        // GITHUB_TOKEN_PATH environment variable
        if let Ok(token_path) = env::var("WORKSPACE_GITHUB_TOKEN_PATH") {
            result.github.token_path = token_path;
        }

        // GITHUB_TOKEN environment variable
        if let Ok(_token) = env::var("WORKSPACE_GITHUB_TOKEN") {
            // If token is provided via env var, enable GitHub integration
            result.github.enable_integration = true;
        }

        result
    }

    /// Find system-wide configuration file
    fn find_system_config() -> Option<PathBuf> {
        // Look in standard system config directories
        // Linux: /etc/workspace-cli/config.toml
        // macOS: /Library/Application Support/workspace-cli/config.toml
        // Windows: C:\ProgramData\workspace-cli\config.toml

        #[cfg(target_os = "linux")]
        {
            let path = PathBuf::from("/etc/workspace-cli/config.toml");
            if path.exists() {
                return Some(path);
            }
        }

        #[cfg(target_os = "macos")]
        {
            let path = PathBuf::from("/Library/Application Support/workspace-cli/config.toml");
            if path.exists() {
                return Some(path);
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(program_data) = dirs::data_dir() {
                let path = program_data.join("workspace-cli").join("config.toml");
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
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

    /// Get a configuration value by key path
    pub fn get_value(&self, key_path: &str) -> Result<String> {
        let parts: Vec<&str> = key_path.split('.').collect();

        if parts.is_empty() || parts.len() > 3 {
            return Err(anyhow::anyhow!(
                "Invalid key format. Use section.key or section.subsection.key"
            ));
        }

        match parts[0] {
            "general" => {
                if parts.len() == 2 {
                    match parts[1] {
                        "log_level" => Ok(self.general.log_level.clone()),
                        "auto_start_daemon" => Ok(self.general.auto_start_daemon.to_string()),
                        _ => Err(anyhow::anyhow!("Unknown key: general.{}", parts[1])),
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid key depth for general section"))
                }
            }
            "daemon" => {
                if parts.len() == 2 {
                    match parts[1] {
                        "socket_path" => Ok(self.daemon.socket_path.clone()),
                        "pid_file" => Ok(self.daemon.pid_file.clone()),
                        "polling_interval_ms" => Ok(self.daemon.polling_interval_ms.to_string()),
                        "inactive_polling_ms" => Ok(self.daemon.inactive_polling_ms.to_string()),
                        _ => Err(anyhow::anyhow!("Unknown key: daemon.{}", parts[1])),
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid key depth for daemon section"))
                }
            }
            "monitor" => {
                if parts.len() == 2 {
                    match parts[1] {
                        "refresh_rate_ms" => Ok(self.monitor.refresh_rate_ms.to_string()),
                        "default_view" => Ok(self.monitor.default_view.clone()),
                        "color_theme" => Ok(self.monitor.color_theme.clone()),
                        _ => Err(anyhow::anyhow!("Unknown key: monitor.{}", parts[1])),
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid key depth for monitor section"))
                }
            }
            "watcher" => {
                if parts.len() == 2 {
                    match parts[1] {
                        "include_patterns" => Ok(format!("{:?}", self.watcher.include_patterns)),
                        "exclude_patterns" => Ok(format!("{:?}", self.watcher.exclude_patterns)),
                        "use_git_hooks" => Ok(self.watcher.use_git_hooks.to_string()),
                        _ => Err(anyhow::anyhow!("Unknown key: watcher.{}", parts[1])),
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid key depth for watcher section"))
                }
            }
            "github" => {
                if parts.len() == 2 {
                    match parts[1] {
                        "enable_integration" => Ok(self.github.enable_integration.to_string()),
                        "token_path" => Ok(self.github.token_path.clone()),
                        "fetch_interval_s" => Ok(self.github.fetch_interval_s.to_string()),
                        _ => Err(anyhow::anyhow!("Unknown key: github.{}", parts[1])),
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid key depth for github section"))
                }
            }
            "repositories" => {
                if parts.len() == 3 {
                    let repo_name = parts[1];
                    let repo_key = parts[2];

                    if let Some(repo) = self.repositories.iter().find(|r| r.name == repo_name) {
                        match repo_key {
                            "path" => Ok(repo.path.clone()),
                            "name" => Ok(repo.name.clone()),
                            "active" => Ok(repo.active.to_string()),
                            "branch" => Ok(repo.branch.clone()),
                            "include_patterns" => Ok(format!("{:?}", repo.include_patterns)),
                            "exclude_patterns" => Ok(format!("{:?}", repo.exclude_patterns)),
                            _ => Err(anyhow::anyhow!(
                                "Unknown key: repositories.{}.{}",
                                repo_name,
                                repo_key
                            )),
                        }
                    } else {
                        Err(anyhow::anyhow!("Repository not found: {}", repo_name))
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid key format for repositories. Use repositories.name.key"
                    ))
                }
            }
            _ => Err(anyhow::anyhow!("Unknown configuration section: {}", parts[0])),
        }
    }

    /// Set a configuration value by key path
    pub fn set_value(&mut self, key_path: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key_path.split('.').collect();

        if parts.is_empty() || parts.len() > 3 {
            return Err(anyhow::anyhow!(
                "Invalid key format. Use section.key or section.subsection.key"
            ));
        }

        match parts[0] {
            "general" => {
                if parts.len() == 2 {
                    match parts[1] {
                        "log_level" => {
                            self.general.log_level = value.to_string();
                            Ok(())
                        }
                        "auto_start_daemon" => {
                            self.general.auto_start_daemon = value
                                .parse()
                                .map_err(|_| anyhow::anyhow!("Invalid boolean value: {}", value))?;
                            Ok(())
                        }
                        _ => Err(anyhow::anyhow!("Unknown key: general.{}", parts[1])),
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid key depth for general section"))
                }
            }
            "daemon" => {
                if parts.len() == 2 {
                    match parts[1] {
                        "socket_path" => {
                            self.daemon.socket_path = value.to_string();
                            Ok(())
                        }
                        "pid_file" => {
                            self.daemon.pid_file = value.to_string();
                            Ok(())
                        }
                        "polling_interval_ms" => {
                            self.daemon.polling_interval_ms = value
                                .parse()
                                .map_err(|_| anyhow::anyhow!("Invalid number: {}", value))?;
                            Ok(())
                        }
                        "inactive_polling_ms" => {
                            self.daemon.inactive_polling_ms = value
                                .parse()
                                .map_err(|_| anyhow::anyhow!("Invalid number: {}", value))?;
                            Ok(())
                        }
                        _ => Err(anyhow::anyhow!("Unknown key: daemon.{}", parts[1])),
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid key depth for daemon section"))
                }
            }
            // ... similar implementation for other sections ...
            _ => Err(anyhow::anyhow!("Unknown configuration section: {}", parts[0])),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> std::result::Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate log level
        match self.general.log_level.as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {}
            _ => errors.push(format!(
                "Invalid log level: {}. Must be one of: error, warn, info, debug, trace",
                self.general.log_level
            )),
        }

        // Validate monitor view
        match self.monitor.default_view.as_str() {
            "overview" | "changes" | "packages" | "graph" => {}
            _ => errors.push(format!(
                "Invalid default view: {}. Must be one of: overview, changes, packages, graph",
                self.monitor.default_view
            )),
        }

        // Validate monitor theme
        match self.monitor.color_theme.as_str() {
            "default" | "dark" | "light" => {}
            _ => errors.push(format!(
                "Invalid color theme: {}. Must be one of: default, dark, light",
                self.monitor.color_theme
            )),
        }

        // Validate repository names
        let mut repo_names = std::collections::HashSet::new();
        for repo in &self.repositories {
            if repo.name.is_empty() {
                errors.push("Repository name cannot be empty".to_string());
            } else if !repo_names.insert(repo.name.clone()) {
                errors.push(format!("Duplicate repository name: {}", repo.name));
            }

            // Validate repository path
            if repo.path.is_empty() {
                errors
                    .push(format!("Repository path cannot be empty for repository: {}", repo.name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Apply command-line overrides to configuration
    pub fn apply_cli_overrides(&mut self, cli_args: HashMap<String, String>) -> Result<()> {
        for (key, value) in cli_args {
            self.set_value(&key, &value)?;
        }
        Ok(())
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
                "**/*.tsx".to_string(),
                "**/*.mjs".to_string(),
                "**/*.mts".to_string(),
                "**/*.json".to_string(),
                "**/*.html".to_string(),
                "**/*.css".to_string(),
                "**/*.scss".to_string(),
                "**/*.sass".to_string(),
                "**/*.less".to_string(),
                "**/*.styl".to_string(),
                "**/*.vue".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
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
