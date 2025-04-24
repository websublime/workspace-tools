use serde::{Deserialize, Serialize};

use crate::common::config::expand_path;

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

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: Some(expand_path("~/.local/share/workspace-cli/daemon.sock")),
            pid_file: Some(expand_path("~/.local/share/workspace-cli/daemon.pid")),
            polling_interval_ms: Some(500),
            inactive_polling_ms: Some(5000),
            log_max_size_bytes: Some(10 * 1024 * 1024), // 10 MB
            log_max_files: Some(5),                     // 5 rotated log files
            log_check_interval_ms: Some(3600000),       // Check every hour (in milliseconds)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DaemonConfigBuilder {
    config: DaemonConfig,
}

impl DaemonConfigBuilder {
    pub fn new() -> Self {
        Self { config: DaemonConfig::default() }
    }

    pub fn socket_path(mut self, path: impl Into<String>) -> Self {
        self.config.socket_path = Some(path.into());
        self
    }

    pub fn pid_file(mut self, path: impl Into<String>) -> Self {
        self.config.pid_file = Some(path.into());
        self
    }

    pub fn polling_interval_ms(mut self, interval: u64) -> Self {
        self.config.polling_interval_ms = Some(interval);
        self
    }

    pub fn inactive_polling_ms(mut self, interval: u64) -> Self {
        self.config.inactive_polling_ms = Some(interval);
        self
    }

    pub fn log_max_size_bytes(mut self, size: u64) -> Self {
        self.config.log_max_size_bytes = Some(size);
        self
    }

    pub fn log_max_files(mut self, count: usize) -> Self {
        self.config.log_max_files = Some(count);
        self
    }

    pub fn log_check_interval_ms(mut self, interval: u64) -> Self {
        self.config.log_check_interval_ms = Some(interval);
        self
    }
}
