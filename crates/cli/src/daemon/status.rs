//! Status information for the daemon service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Status of the daemon service
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Whether the daemon is running
    pub running: bool,
    /// PID of the daemon process
    pub pid: Option<u32>,
    /// Uptime in seconds
    pub uptime: Option<u64>,
    /// Number of repositories being monitored
    pub repository_count: usize,
    /// Information about monitored repositories
    pub repositories: Vec<RepositoryStatus>,
    /// Time the daemon was started
    pub started_at: Option<SystemTime>,
    /// Memory usage in bytes (if available)
    pub memory_usage: Option<u64>,
    /// CPU usage percentage (if available)
    pub cpu_usage: Option<f64>,
    /// Socket path for IPC communication
    pub socket_path: Option<PathBuf>,
}

/// Status of a repository being monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Name of the repository
    pub name: String,
    /// Path to the repository
    pub path: PathBuf,
    /// Current branch
    pub branch: Option<String>,
    /// Last commit hash
    pub last_commit: Option<String>,
    /// Last time the repository was checked
    pub last_checked: Option<SystemTime>,
    /// Number of pending changes
    pub pending_changes: usize,
    /// Whether the repository is active (being monitored)
    pub active: bool,
    /// Repository type (git, svn, etc.)
    pub repo_type: String,
}

/// General status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInfo {
    /// Daemon status
    pub daemon: DaemonStatus,
    /// System information
    pub system: SystemInfo,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Hostname
    pub hostname: Option<String>,
    /// Username
    pub username: Option<String>,
    /// Current working directory
    pub cwd: Option<PathBuf>,
    /// Environment variables
    pub env: HashMap<String, String>,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: hostname(),
            username: username(),
            cwd: std::env::current_dir().ok(),
            env: std::env::vars().collect(),
        }
    }
}

impl StatusInfo {
    /// Create a new status info
    pub fn new(daemon: DaemonStatus) -> Self {
        Self { daemon, system: SystemInfo::default() }
    }
}

/// Get current hostname
fn hostname() -> Option<String> {
    #[cfg(unix)]
    {
        let mut buf = [0u8; 256];
        let res = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if res == 0 {
            // Find the null terminator
            let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            String::from_utf8(buf[..len].to_vec()).ok()
        } else {
            None
        }
    }

    #[cfg(windows)]
    {
        std::env::var("COMPUTERNAME").ok()
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Get current username
fn username() -> Option<String> {
    #[cfg(unix)]
    {
        std::env::var("USER").ok()
    }

    #[cfg(windows)]
    {
        std::env::var("USERNAME").ok()
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}
