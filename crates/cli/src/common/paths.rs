use anyhow::{anyhow, Context, Result};
use dirs;
use std::env;
use std::path::{Path, PathBuf};

/// Get the user's config directory
pub fn get_config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|p| p.join("workspace-cli"))
        .ok_or_else(|| anyhow!("Could not determine config directory"))
}

/// Get the path to the config file
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.toml"))
}

/// Get the user's data directory
pub fn get_data_dir() -> Result<PathBuf> {
    dirs::data_local_dir()
        .map(|p| p.join("workspace-cli"))
        .ok_or_else(|| anyhow!("Could not determine data directory"))
}

/// Get the path to the registry file
pub fn get_registry_path() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("registry.toml"))
}

/// Get the default socket path
pub fn get_default_socket_path() -> Result<String> {
    Ok(get_data_dir()?.join("daemon.sock").to_string_lossy().into_owned())
}

/// Get the default PID file path
pub fn get_default_pid_path() -> Result<String> {
    Ok(get_data_dir()?.join("daemon.pid").to_string_lossy().into_owned())
}

/// Expand a path that might contain tilde
pub fn expand_path(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let path_without_tilde = &path[2..];
        Ok(home.join(path_without_tilde))
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Find the project root directory
pub fn find_project_root(start_dir: Option<PathBuf>) -> Result<PathBuf> {
    let start =
        start_dir.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Use sublime_standard_tools to find project root
    match sublime_standard_tools::get_project_root_path(Some(start.clone())) {
        Some(root) => Ok(root),
        None => {
            // Fallback to current directory if no root found
            Ok(start)
        }
    }
}
