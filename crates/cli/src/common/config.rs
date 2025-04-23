use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: Option<GeneralConfig>,
    pub daemon: Option<DaemonConfig>,
    pub monitor: Option<MonitorConfig>,
    pub watcher: Option<WatcherConfig>,
    pub github: Option<GithubConfig>,
    pub repositories: Option<Vec<RepositoryConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub log_level: Option<String>,
    pub auto_start_daemon: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub socket_path: Option<String>,
    pub pid_file: Option<String>,
    pub polling_interval_ms: Option<u64>,
    pub inactive_polling_ms: Option<u64>,
    pub log_max_size_bytes: Option<u64>,
    pub log_max_files: Option<usize>,
    pub log_check_interval_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub refresh_rate_ms: Option<u64>,
    pub default_view: Option<String>,
    pub color_theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub use_git_hooks: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    pub enable_integration: Option<bool>,
    pub token_path: Option<String>,
    pub fetch_interval_s: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub path: String,
    pub name: Option<String>,
    pub active: Option<bool>,
    pub branch: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[allow(clippy::should_implement_trait)]
impl Config {
    pub fn default() -> Self {
        Config {
            general: Some(GeneralConfig {
                log_level: Some("info".to_string()),
                auto_start_daemon: Some(true),
            }),
            daemon: Some(DaemonConfig {
                socket_path: Some(expand_path("~/.local/share/workspace-cli/daemon.sock")),
                pid_file: Some(expand_path("~/.local/share/workspace-cli/daemon.pid")),
                polling_interval_ms: Some(500),
                inactive_polling_ms: Some(5000),
                // Add log rotation configuration with sensible defaults
                log_max_size_bytes: Some(10 * 1024 * 1024), // 10 MB
                log_max_files: Some(5),                     // Keep 5 rotated log files
                log_check_interval_ms: Some(3600000),       // Check every hour (in milliseconds)
            }),
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
                token_path: Some(expand_path("~/.config/workspace-cli/github_token")),
                fetch_interval_s: Some(300),
            }),
            repositories: Some(Vec::new()),
        }
    }

    pub fn load(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

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

pub fn expand_path(path: &str) -> String {
    let path = path.trim();

    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.strip_prefix("~/").unwrap()).to_string_lossy().into_owned();
        }
    }

    path.to_string()
}
