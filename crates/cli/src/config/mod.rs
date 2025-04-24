mod daemon_config;
pub use daemon_config::{DaemonConfig, DaemonConfigBuilder};

use std::path::{Path, PathBuf};

use crate::common::errors::{CliError, CliResult};

pub trait ConfigLoader {
    fn load<P: AsRef<Path>>(path: P) -> CliResult<Self>
    where
        Self: Sized;

    fn save<P: AsRef<Path>>(&self, path: P) -> CliResult<()>;
}

/// General configuration container with all settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub general: Option<GeneralConfig>,
    pub daemon: Option<DaemonConfig>,
    pub monitor: Option<MonitorConfig>,
    pub watcher: Option<WatcherConfig>,
    pub github: Option<GithubConfig>,
    pub repositories: Option<Vec<RepositoryConfig>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneralConfig {
    pub log_level: Option<String>,
    pub auto_start_daemon: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitorConfig {
    pub refresh_rate_ms: Option<u64>,
    pub default_view: Option<String>,
    pub color_theme: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WatcherConfig {
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub use_git_hooks: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GithubConfig {
    pub enable_integration: Option<bool>,
    pub token_path: Option<String>,
    pub fetch_interval_s: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepositoryConfig {
    pub path: String,
    pub name: Option<String>,
    pub active: Option<bool>,
    pub branch: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: Some(GeneralConfig {
                log_level: Some("info".to_string()),
                auto_start_daemon: Some(true),
            }),
            daemon: Some(DaemonConfig::default()),
            monitor: Some(MonitorConfig {
                refresh_rate_ms: Some(1000),
                default_view: Some("overview".to_string()),
                color_theme: Some("default".to_string()),
            }),
            watcher: Some(WatcherConfig {
                include_patterns: Some(vec![
                    "**/*.rs".to_string(),
                    "**/*.toml".to_string(),
                    "**/*.html".to_string(),
                    "**/*.vue".to_string(),
                    "**/*.css".to_string(),
                    "**/*.json".to_string(),
                    "**/*.js".to_string(),
                    "**/*.mjs".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.d.ts".to_string(),
                    "**/*.mts".to_string(),
                    "**/*.d.mts".to_string(),
                    "**/*.tsx".to_string(),
                ]),
                exclude_patterns: Some(vec![
                    "**/node_modules/**".to_string(),
                    "**/target/**".to_string(),
                    "**/.git/**".to_string(),
                    "**/build/**".to_string(),
                    "**/dist/**".to_string(),
                ]),
                use_git_hooks: Some(true),
            }),
            github: Some(GithubConfig {
                enable_integration: Some(false),
                token_path: Some(crate::common::utils::expand_path(
                    "~/.config/workspace-cli/github_token",
                )),
                fetch_interval_s: Some(300),
            }),
            repositories: Some(Vec::new()),
        }
    }
}

impl ConfigLoader for Config {
    fn load<P: AsRef<Path>>(path: P) -> CliResult<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(CliError::Io)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| CliError::Config(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> CliResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e)))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(CliError::Io)?;
        }

        std::fs::write(path, content).map_err(CliError::Io)?;

        Ok(())
    }
}

/// Get the default config path
pub fn get_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("WORKSPACE_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    let mut path =
        if let Some(config_dir) = dirs::config_dir() { config_dir } else { PathBuf::from(".") };

    path.push("workspace-cli");
    path.push("config.toml");
    path
}
