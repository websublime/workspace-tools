use anyhow::Result;
use chrono;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs::{self, metadata, File, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

#[cfg(windows)]
use std::net::{SocketAddr, TcpStream};

use crate::common::config::{get_config_path, Config, DaemonConfig, RepositoryConfig};

/// Maximum message size for IPC communications
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Default timeout for daemon operations in milliseconds
const DEFAULT_OPERATION_TIMEOUT_MS: u64 = 5000;

/// Daemon error types
#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Failed to start daemon process: {0}")]
    StartError(#[source] io::Error),

    #[error("Failed to stop daemon process: {0}")]
    StopError(#[source] io::Error),

    #[error("Daemon socket error: {0}")]
    SocketError(#[source] io::Error),

    #[error("Failed to read daemon status: {0}")]
    StatusReadError(#[source] io::Error),

    #[error("Failed to write daemon PID: {0}")]
    PidWriteError(#[source] io::Error),

    #[error("Invalid daemon configuration: {0}")]
    ConfigError(String),

    #[error("IPC communication error: {0}")]
    IpcError(#[source] io::Error),

    #[error("IPC message serialization error: {0}")]
    SerializationError(#[source] bincode::Error),

    #[error("Daemon is not running")]
    NotRunning,

    #[error("Daemon is already running (PID: {0})")]
    AlreadyRunning(u32),

    #[error("Operation timed out")]
    Timeout,

    #[error("Invalid message received: {0}")]
    InvalidMessage(String),

    #[error("Command execution error: {0}")]
    CommandError(String),
}

/// IPC message types for daemon communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonMessage {
    /// Status request
    StatusRequest,
    /// Status response
    StatusResponse {
        running: bool,
        pid: Option<u32>,
        uptime_seconds: Option<u64>,
        monitored_repos: Option<usize>,
    },
    /// Request to add a repository for monitoring
    AddRepositoryRequest { path: String, name: Option<String> },
    /// Response after adding a repository
    AddRepositoryResponse { success: bool, error: Option<String> },
    /// Request to remove a repository from monitoring
    RemoveRepositoryRequest {
        identifier: String, // Name or path
    },
    /// Response after removing a repository
    RemoveRepositoryResponse { success: bool, error: Option<String> },
    /// Generic command request
    CommandRequest { command: String, args: Vec<String> },
    /// Generic command response
    CommandResponse { success: bool, message: String, data: Option<String> },
    /// Shutdown request
    ShutdownRequest,
    /// Shutdown response
    ShutdownResponse { success: bool },
}

/// Daemon status information
#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub socket_path: Option<PathBuf>,
    pub uptime_seconds: Option<u64>,
    pub monitored_repos: Option<usize>,
}

impl DaemonStatus {
    /// Creates a new daemon status
    pub fn new(running: bool) -> Self {
        Self { running, pid: None, socket_path: None, uptime_seconds: None, monitored_repos: None }
    }

    /// Creates a daemon status with PID
    pub fn with_pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Sets the socket path
    pub fn with_socket_path(mut self, path: PathBuf) -> Self {
        self.socket_path = Some(path);
        self
    }

    /// Sets the uptime in seconds
    pub fn with_uptime(mut self, seconds: u64) -> Self {
        self.uptime_seconds = Some(seconds);
        self
    }

    /// Sets the number of monitored repositories
    pub fn with_monitored_repos(mut self, count: usize) -> Self {
        self.monitored_repos = Some(count);
        self
    }
}

/// Daemon manager for controlling and communicating with the daemon process
pub struct DaemonManager {
    config: DaemonConfig,
}

impl Default for DaemonManager {
    fn default() -> Self {
        Self { config: Config::default().daemon.unwrap() }
    }
}

impl DaemonManager {
    /// Creates a new daemon manager using configuration
    pub fn new(config: DaemonConfig) -> Result<Self, DaemonError> {
        // Validate configuration
        if config.socket_path.is_none() {
            return Err(DaemonError::ConfigError(
                "Missing socket_path in daemon configuration".to_string(),
            ));
        }

        if config.pid_file.is_none() {
            return Err(DaemonError::ConfigError(
                "Missing pid_file in daemon configuration".to_string(),
            ));
        }

        Ok(Self { config })
    }

    /// Gets the socket path
    pub fn socket_path(&self) -> PathBuf {
        PathBuf::from(self.config.socket_path.as_ref().unwrap())
    }

    /// Gets the PID file path
    pub fn pid_file_path(&self) -> PathBuf {
        PathBuf::from(self.config.pid_file.as_ref().unwrap())
    }

    /// Checks if the daemon is running
    pub fn is_running(&self) -> Result<bool, DaemonError> {
        // Get the socket path
        let socket_path = self.socket_path();

        // Check if PID file exists
        let pid_file = self.pid_file_path();

        // If PID file doesn't exist, daemon is not running
        if !pid_file.exists() {
            debug!("PID file doesn't exist at {}", pid_file.display());
            return Ok(false);
        }

        // Read the PID
        let pid = match self.read_pid() {
            Ok(pid) => pid,
            Err(e) => {
                debug!("Failed to read PID file: {}", e);
                return Ok(false);
            }
        };

        debug!("Found PID {} in PID file", pid);

        // Check if the process is running
        let process_running = match check_process_running(pid) {
            true => {
                debug!("Process with PID {} is running", pid);
                true
            }
            false => {
                debug!("Process with PID {} is not running", pid);
                return Ok(false);
            }
        };

        // Check if socket file exists
        if !socket_path.exists() {
            debug!("Socket file doesn't exist at {}", socket_path.display());
            return Ok(false);
        }

        debug!("Socket file exists at {}", socket_path.display());

        // Try to connect to the socket as final verification
        match UnixStream::connect(&socket_path) {
            Ok(_) => {
                debug!("Successfully connected to daemon socket");
                Ok(true)
            }
            Err(e) => {
                debug!("Failed to connect to daemon socket: {}", e);

                // If the process is running but we can't connect, the socket might be stale
                // Let's remove it
                if process_running {
                    debug!(
                        "Process is running but socket connection failed - socket might be stale"
                    );

                    if let Err(remove_err) = fs::remove_file(&socket_path) {
                        debug!("Failed to remove stale socket file: {}", remove_err);
                    }
                }

                Ok(false)
            }
        }
    }

    /// Gets detailed daemon status
    pub fn status(&self) -> Result<DaemonStatus, DaemonError> {
        if !self.is_running()? {
            return Ok(DaemonStatus::new(false));
        }

        let pid = self.read_pid()?;
        let mut status = DaemonStatus::new(true).with_pid(pid).with_socket_path(self.socket_path());

        // Try to get more detailed status via IPC
        match self.send_message(&DaemonMessage::StatusRequest) {
            Ok(DaemonMessage::StatusResponse {
                running,
                pid: _,
                uptime_seconds,
                monitored_repos,
            }) => {
                // We know it's running because we got a response
                debug_assert!(running);

                if let Some(uptime) = uptime_seconds {
                    status = status.with_uptime(uptime);
                }

                if let Some(repos) = monitored_repos {
                    status = status.with_monitored_repos(repos);
                }

                Ok(status)
            }
            Ok(_) => Err(DaemonError::InvalidMessage("Unexpected response type".to_string())),
            Err(e) => {
                warn!("Failed to get detailed status via IPC: {}", e);
                // Return basic status even if detailed IPC fails
                Ok(status)
            }
        }
    }

    /// Starts the daemon process
    pub fn start(&self, log_level: Option<&str>) -> Result<(), DaemonError> {
        // Check if already running
        if self.is_running()? {
            let pid = self.read_pid()?;
            return Err(DaemonError::AlreadyRunning(pid));
        }

        // Clean up any stale files
        self.cleanup_files()?;

        // Create directories if they don't exist
        if let Some(parent) = self.socket_path().parent() {
            fs::create_dir_all(parent).map_err(DaemonError::SocketError)?;
        }

        if let Some(parent) = self.pid_file_path().parent() {
            fs::create_dir_all(parent).map_err(DaemonError::PidWriteError)?;
        }

        // Find the workspace-daemon binary
        let daemon_bin = std::env::current_exe()
            .map_err(DaemonError::StartError)?
            .with_file_name("workspace-daemon");

        // Create a log file path for daemon output
        let log_path = self.pid_file_path().with_file_name("daemon.log");

        info!("Daemon output will be captured to: {}", log_path.display());
        let log_file = match File::create(&log_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create log file: {}", e);
                return Err(DaemonError::StartError(e));
            }
        };

        // Set up command to run the daemon with output captured
        let mut cmd = Command::new(&daemon_bin);
        cmd.arg("run");
        cmd.env("RUST_LOG", log_level.unwrap_or("info"));

        // Add socket path and pid file arguments
        //cmd.arg("--socket-path").arg(self.socket_path().to_string_lossy().to_string());
        //cmd.arg("--pid-file").arg(self.pid_file_path().to_string_lossy().to_string());
        cmd.env("WORKSPACE_SOCKET_PATH", self.socket_path().to_string_lossy().to_string())
            .env("WORKSPACE_PID_FILE", self.pid_file_path().to_string_lossy().to_string());

        // Add config path if we have it
        if let Ok(config_path) = get_config_path().canonicalize() {
            cmd.env("WORKSPACE_CONFIG_PATH", config_path.to_string_lossy().to_string());
        }

        // Capture stdout and stderr to our log file
        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(log_file.try_clone().unwrap()))
            .stderr(Stdio::from(log_file));

        // This is where you should add the working dir and env vars
        cmd.current_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("./")))
            .env("RUST_BACKTRACE", "1");

        // Start daemon process
        info!("Starting daemon process with command: {:?}", cmd);
        let child = match cmd.spawn() {
            Ok(child) => {
                info!("Daemon process spawned with PID {}", child.id());
                child
            }
            Err(e) => {
                error!("Failed to spawn daemon process: {}", e);
                return Err(DaemonError::StartError(e));
            }
        };

        // Write PID to file
        self.write_pid(child.id() as u32)?;
        info!("Wrote PID {} to file {}", child.id(), self.pid_file_path().display());

        // Wait for the daemon to initialize
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(30);

        info!("Waiting for daemon to initialize (timeout: {}s)...", timeout.as_secs());

        let mut socket_exists = false;

        while start_time.elapsed() < timeout {
            // Check if the PID file still exists and the process is running
            if !self.pid_file_path().exists() || !check_process_running(child.id() as u32) {
                // Read the log file to see what went wrong
                if let Ok(log_content) = fs::read_to_string(&log_path) {
                    error!("Daemon process terminated. Log output:\n{}", log_content);
                }

                return Err(DaemonError::StartError(io::Error::new(
                    io::ErrorKind::Other,
                    "Daemon process terminated unexpectedly during startup",
                )));
            }

            // Check if the socket file exists
            if !socket_exists && self.socket_path().exists() {
                info!("Socket file created at {}", self.socket_path().display());
                socket_exists = true;
            }

            // Try to connect to the socket
            if socket_exists {
                match UnixStream::connect(self.socket_path()) {
                    Ok(_) => {
                        info!("Successfully connected to daemon socket");
                        return Ok(());
                    }
                    Err(e) => {
                        debug!("Socket exists but connection failed: {}", e);
                    }
                }
            }

            // Sleep before next check
            thread::sleep(Duration::from_millis(500));
        }

        // If we get here, the daemon didn't respond within the timeout
        // Try to read the log file to see what's happening
        if let Ok(log_content) = fs::read_to_string(&log_path) {
            warn!("Daemon startup timed out. Log output:\n{}", log_content);
        }

        if socket_exists {
            warn!("Socket file was created but daemon didn't accept connections within timeout");
        } else {
            warn!("Socket file wasn't created within timeout");
        }

        Err(DaemonError::Timeout)
    }

    /// Stops the daemon process
    pub fn stop(&self) -> Result<(), DaemonError> {
        // Check if the daemon is running
        if !self.is_running()? {
            return Err(DaemonError::NotRunning);
        }

        info!("Sending shutdown request to daemon...");

        // Try to gracefully shut down the daemon through IPC
        match self.send_message(&DaemonMessage::ShutdownRequest) {
            Ok(DaemonMessage::ShutdownResponse { success }) => {
                if success {
                    info!("Daemon is shutting down gracefully");

                    // Wait for the daemon to stop
                    let start_time = std::time::Instant::now();
                    let timeout = Duration::from_millis(DEFAULT_OPERATION_TIMEOUT_MS);

                    while start_time.elapsed() < timeout {
                        if !self.is_running()? {
                            // Clean up socket and PID file
                            self.cleanup_files()?;
                            info!("Daemon stopped successfully");
                            return Ok(());
                        }
                        thread::sleep(Duration::from_millis(100));
                    }

                    warn!("Daemon did not stop in time, forcing termination");
                }
            }
            Ok(_) => {
                return Err(DaemonError::InvalidMessage(
                    "Unexpected response to shutdown request".to_string(),
                ));
            }
            Err(e) => {
                warn!("Failed to send shutdown request: {}", e);
                // Fall back to force kill if IPC fails
            }
        }

        // If we get here, either the IPC failed or the graceful shutdown timed out
        // Try to kill the process directly
        let pid = self.read_pid()?;
        info!("Forcefully terminating daemon process (PID: {})", pid);

        #[cfg(unix)]
        {
            // On Unix, send SIGTERM to the process
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => {
                    // Wait for the process to actually terminate
                    let start_time = std::time::Instant::now();
                    let timeout = Duration::from_millis(DEFAULT_OPERATION_TIMEOUT_MS);

                    while start_time.elapsed() < timeout {
                        if !self.is_running()? {
                            break;
                        }
                        thread::sleep(Duration::from_millis(100));
                    }

                    // If still running, try SIGKILL
                    if self.is_running()? {
                        warn!("Process did not respond to SIGTERM, sending SIGKILL");
                        match kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                            Ok(_) => (),
                            Err(e) => {
                                warn!("Failed to send SIGKILL: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to send SIGTERM: {}", e);
                }
            }
        }

        // Clean up files regardless of how we stopped it
        self.cleanup_files()?;

        info!("Daemon stopped");
        Ok(())
    }

    /// Restart the daemon process
    pub fn restart(&self, log_level: Option<&str>) -> Result<(), DaemonError> {
        if self.is_running()? {
            self.stop()?;
        }
        self.start(log_level)
    }

    /// Sends a command to the daemon
    pub fn send_command(&self, command: &str, args: &[String]) -> Result<String, DaemonError> {
        // Check if the daemon is running
        if !self.is_running()? {
            return Err(DaemonError::NotRunning);
        }

        // Create a command request
        let request =
            DaemonMessage::CommandRequest { command: command.to_string(), args: args.to_vec() };

        // Send the request and get the response
        match self.send_message(&request) {
            Ok(DaemonMessage::CommandResponse { success, message, data }) => {
                if success {
                    Ok(data.unwrap_or(message))
                } else {
                    Err(DaemonError::CommandError(message))
                }
            }
            Ok(_) => Err(DaemonError::InvalidMessage(
                "Unexpected response to command request".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Adds a repository for monitoring
    pub fn add_repository(&self, path: &str, name: Option<&str>) -> Result<(), DaemonError> {
        // Check if the daemon is running
        if !self.is_running()? {
            return Err(DaemonError::NotRunning);
        }

        if self.is_running()? {
            // Create an add repository request
            let request = DaemonMessage::AddRepositoryRequest {
                path: path.to_string(),
                name: name.map(String::from),
            };

            // Send the request and get the response
            match self.send_message(&request) {
                Ok(DaemonMessage::AddRepositoryResponse { success, error }) => {
                    if !success {
                        return Err(DaemonError::CommandError(
                            error.unwrap_or_else(|| {
                                "Failed to add repository to daemon".to_string()
                            }),
                        ));
                    }
                    // Continue to also add to config
                }
                Ok(_) => {
                    return Err(DaemonError::InvalidMessage(
                        "Unexpected response to add repository request".to_string(),
                    ));
                }
                Err(e) => {
                    warn!("Failed to add repository to running daemon: {}", e);
                    // Continue to add to config anyway
                }
            }
        }

        // Next, add to config file so it persists for future daemon runs
        let config_path = get_config_path();
        let mut config = if config_path.exists() {
            match Config::load(&config_path) {
                Ok(config) => config,
                Err(e) => {
                    return Err(DaemonError::ConfigError(format!("Failed to load config: {}", e)));
                }
            }
        } else {
            Config::default()
        };

        // Create new repository config
        let repo_config = RepositoryConfig {
            path: path.to_string(),
            name: name.map(String::from),
            active: Some(true),
            branch: None,
            include_patterns: None,
            exclude_patterns: None,
        };

        // Add to config
        if config.repositories.is_none() {
            config.repositories = Some(Vec::new());
        }

        if let Some(repos) = &mut config.repositories {
            // Check if repo already exists by path
            if repos.iter().any(|r| r.path == path) {
                return Err(DaemonError::ConfigError(format!(
                    "Repository already exists at path: {}",
                    path
                )));
            }

            // Add the new repo
            repos.push(repo_config);
        }

        // Save config
        let content = match toml::to_string_pretty(&config) {
            Ok(content) => content,
            Err(e) => {
                return Err(DaemonError::ConfigError(format!("Failed to serialize config: {}", e)));
            }
        };

        // Create parent directories if needed
        if let Some(parent) = config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(DaemonError::ConfigError(format!(
                    "Failed to create config directory: {}",
                    e
                )));
            }
        }

        // Write config file
        if let Err(e) = fs::write(&config_path, content) {
            return Err(DaemonError::ConfigError(format!("Failed to write config file: {}", e)));
        }

        Ok(())
    }

    /// Removes a repository from monitoring
    pub fn remove_repository(&self, identifier: &str) -> Result<(), DaemonError> {
        // Check if the daemon is running
        if !self.is_running()? {
            return Err(DaemonError::NotRunning);
        }

        // First, remove from daemon if it's running
        if self.is_running()? {
            // Create a remove repository request
            let request =
                DaemonMessage::RemoveRepositoryRequest { identifier: identifier.to_string() };

            // Send the request and check response
            match self.send_message(&request) {
                Ok(DaemonMessage::RemoveRepositoryResponse { success, error }) => {
                    if !success {
                        warn!(
                            "Failed to remove repository from running daemon: {}",
                            error.unwrap_or_else(|| "Unknown error".to_string())
                        );
                        // Continue to remove from config anyway
                    }
                }
                Ok(_) => {
                    warn!("Unexpected response to remove repository request");
                    // Continue to remove from config anyway
                }
                Err(e) => {
                    warn!("Failed to remove repository from running daemon: {}", e);
                    // Continue to remove from config anyway
                }
            }
        }

        // Next, remove from config file so it persists for future daemon runs
        let config_path = get_config_path();
        if !config_path.exists() {
            return Err(DaemonError::ConfigError("Config file does not exist".to_string()));
        }

        let mut config = match Config::load(&config_path) {
            Ok(config) => config,
            Err(e) => {
                return Err(DaemonError::ConfigError(format!("Failed to load config: {}", e)));
            }
        };

        // Remove from config
        let mut removed = false;
        if let Some(repos) = &mut config.repositories {
            let initial_len = repos.len();
            repos.retain(|r| {
                r.path != identifier && r.name.as_ref() != Some(&identifier.to_string())
            });
            removed = repos.len() < initial_len;
        }

        if !removed {
            return Err(DaemonError::ConfigError(format!(
                "Repository not found with identifier: {}",
                identifier
            )));
        }

        // Save config
        let content = match toml::to_string_pretty(&config) {
            Ok(content) => content,
            Err(e) => {
                return Err(DaemonError::ConfigError(format!("Failed to serialize config: {}", e)));
            }
        };

        // Write config file
        if let Err(e) = fs::write(&config_path, content) {
            return Err(DaemonError::ConfigError(format!("Failed to write config file: {}", e)));
        }

        Ok(())
    }

    // Private helper methods

    /// Sends a message to the daemon and receives a response
    fn send_message(&self, message: &DaemonMessage) -> Result<DaemonMessage, DaemonError> {
        // First check if the daemon is reachable
        if let Ok(connected) = self.check_daemon_connection() {
            if !connected {
                return Err(DaemonError::NotRunning);
            }
        }

        // Connect to the daemon socket with retry
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 1..=max_retries {
            #[cfg(unix)]
            let stream_result = UnixStream::connect(self.socket_path());

            #[cfg(windows)]
            let stream_result = {
                // On Windows, use a TCP socket (this is a placeholder for real implementation)
                let addr = SocketAddr::from(([127, 0, 0, 1], 8899));
                TcpStream::connect_timeout(&addr, Duration::from_millis(5000))
            };

            match stream_result {
                Ok(mut stream) => {
                    // Try to set timeouts but don't fail if they're not supported
                    if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(5000))) {
                        debug!("Socket read timeout not supported on client: {}", e);
                        // Continue anyway
                    }

                    if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(5000))) {
                        debug!("Socket write timeout not supported on client: {}", e);
                        // Continue anyway
                    }

                    // Use non-blocking mode as a fallback
                    let use_nonblocking = match stream.set_nonblocking(true) {
                        Ok(_) => {
                            debug!("Using non-blocking mode for client socket");
                            true
                        }
                        Err(e) => {
                            debug!("Failed to set non-blocking mode, using blocking IO: {}", e);
                            false
                        }
                    };

                    // Serialize the message
                    let serialized =
                        bincode::serialize(message).map_err(DaemonError::SerializationError)?;

                    // Send length prefix
                    let len = serialized.len() as u32;
                    let len_bytes = len.to_be_bytes();

                    if use_nonblocking {
                        if let Err(e) =
                            write_with_timeout(&mut stream, &len_bytes, Duration::from_millis(5000))
                        {
                            warn!("Failed to write message length on attempt {}: {}", attempt, e);
                            last_error = Some(e);
                            continue; // Try again
                        }
                    } else if let Err(e) = stream.write_all(&len_bytes) {
                        warn!("Failed to write message length on attempt {}: {}", attempt, e);
                        last_error = Some(e);
                        continue; // Try again
                    }

                    // Send message body
                    if use_nonblocking {
                        if let Err(e) = write_with_timeout(
                            &mut stream,
                            &serialized,
                            Duration::from_millis(5000),
                        ) {
                            warn!("Failed to write message body on attempt {}: {}", attempt, e);
                            last_error = Some(e);
                            continue; // Try again
                        }
                    } else if let Err(e) = stream.write_all(&serialized) {
                        warn!("Failed to write message body on attempt {}: {}", attempt, e);
                        last_error = Some(e);
                        continue; // Try again
                    }

                    // Read response length
                    let mut len_bytes = [0u8; 4];

                    if use_nonblocking {
                        if let Err(e) = read_with_timeout(
                            &mut stream,
                            &mut len_bytes,
                            Duration::from_millis(5000),
                        ) {
                            warn!("Failed to read response length on attempt {}: {}", attempt, e);
                            last_error = Some(e);
                            continue; // Try again
                        }
                    } else if let Err(e) = stream.read_exact(&mut len_bytes) {
                        warn!("Failed to read response length on attempt {}: {}", attempt, e);
                        last_error = Some(e);
                        continue; // Try again
                    }

                    let len = u32::from_be_bytes(len_bytes) as usize;

                    // Check for size limits
                    if len > MAX_MESSAGE_SIZE {
                        return Err(DaemonError::InvalidMessage(format!(
                            "Response message too large: {} bytes",
                            len
                        )));
                    }

                    // Read response
                    let mut response_bytes = vec![0u8; len];

                    if use_nonblocking {
                        if let Err(e) = read_with_timeout(
                            &mut stream,
                            &mut response_bytes,
                            Duration::from_millis(5000),
                        ) {
                            warn!("Failed to read response body on attempt {}: {}", attempt, e);
                            last_error = Some(e);
                            continue; // Try again
                        }
                    } else if let Err(e) = stream.read_exact(&mut response_bytes) {
                        warn!("Failed to read response body on attempt {}: {}", attempt, e);
                        last_error = Some(e);
                        continue; // Try again
                    }

                    // Deserialize response
                    let response: DaemonMessage = match bincode::deserialize(&response_bytes) {
                        Ok(resp) => resp,
                        Err(e) => {
                            warn!("Failed to deserialize response on attempt {}: {}", attempt, e);
                            last_error =
                                Some(io::Error::new(io::ErrorKind::InvalidData, format!("{}", e)));
                            continue; // Try again
                        }
                    };

                    // Success!
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Failed to connect to daemon on attempt {}: {}", attempt, e);
                    last_error = Some(e);

                    // Wait before retrying
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }

        // If we get here, all attempts failed
        Err(DaemonError::SocketError(last_error.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Failed to connect to daemon after multiple attempts",
            )
        })))
    }

    /// Reads the daemon PID from the PID file
    pub fn read_pid(&self) -> Result<u32, DaemonError> {
        let pid_file = self.pid_file_path();

        if !pid_file.exists() {
            return Err(DaemonError::NotRunning);
        }

        let mut file = File::open(&pid_file).map_err(DaemonError::StatusReadError)?;

        let mut pid_str = String::new();
        file.read_to_string(&mut pid_str).map_err(DaemonError::StatusReadError)?;

        pid_str.trim().parse::<u32>().map_err(|_| {
            DaemonError::StatusReadError(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid PID in PID file",
            ))
        })
    }

    /// Writes the daemon PID to the PID file
    fn write_pid(&self, pid: u32) -> Result<(), DaemonError> {
        let pid_file = self.pid_file_path();

        if let Some(parent) = pid_file.parent() {
            fs::create_dir_all(parent).map_err(DaemonError::PidWriteError)?;
        }

        let mut file = File::create(&pid_file).map_err(DaemonError::PidWriteError)?;

        file.write_all(pid.to_string().as_bytes()).map_err(DaemonError::PidWriteError)?;

        Ok(())
    }

    /// Cleans up socket and PID files
    fn cleanup_files(&self) -> Result<(), DaemonError> {
        // Remove socket file if it exists
        let socket_path = self.socket_path();
        if socket_path.exists() {
            fs::remove_file(&socket_path).map_err(DaemonError::SocketError)?;
        }

        // Remove PID file if it exists
        let pid_file = self.pid_file_path();
        if pid_file.exists() {
            fs::remove_file(&pid_file).map_err(DaemonError::PidWriteError)?;
        }

        Ok(())
    }

    /// Cross-platform check for IPC connection
    fn check_daemon_connection(&self) -> Result<bool, DaemonError> {
        #[cfg(unix)]
        {
            match UnixStream::connect(self.socket_path()) {
                Ok(_) => Ok(true),
                Err(e) => {
                    debug!("Failed to connect to daemon socket: {}", e);
                    Ok(false)
                }
            }
        }

        #[cfg(windows)]
        {
            // On Windows, use a TCP socket on localhost with a specific port
            // This is just a placeholder - in a real implementation, you'd use named pipes
            // or another Windows IPC mechanism
            let addr = SocketAddr::from(([127, 0, 0, 1], 8899));
            match TcpStream::connect_timeout(&addr, Duration::from_millis(500)) {
                Ok(_) => Ok(true),
                Err(e) => {
                    debug!("Failed to connect to daemon TCP socket: {}", e);
                    Ok(false)
                }
            }
        }
    }
}

/// Daemon server for handling IPC communication in the daemon process
pub struct DaemonServer {
    socket_path: PathBuf,
    config_path: Option<PathBuf>,
    running: Arc<AtomicBool>,
    start_time: std::time::Instant,
    repositories: Vec<String>, // Placeholder for repository tracking
    last_heartbeat: std::time::Instant,
    heartbeat_interval_ms: u64,
    last_log_check: std::time::Instant,
    log_check_interval_ms: u64,
}

impl DaemonServer {
    /// Creates a new daemon server
    pub fn new(socket_path: PathBuf, config_path: Option<PathBuf>) -> Self {
        // Initialize with default values
        let mut log_check_interval_ms = 3600000; // 1 hour default

        // Try to load config
        if let Some(ref config_path) = config_path {
            if let Ok(config) = Config::load(config_path) {
                if let Some(daemon_config) = config.daemon {
                    if let Some(interval) = daemon_config.log_check_interval_ms {
                        log_check_interval_ms = interval;
                    }
                }
            }
        }

        Self {
            socket_path,
            config_path,
            running: Arc::new(AtomicBool::new(true)),
            start_time: std::time::Instant::now(),
            repositories: Vec::new(),
            last_heartbeat: std::time::Instant::now(),
            heartbeat_interval_ms: 30000, // 30 seconds
            last_log_check: std::time::Instant::now(),
            log_check_interval_ms,
        }
    }

    /// Loads repositories from configuration
    pub fn load_repositories_from_config(&mut self) -> Result<(), DaemonError> {
        use crate::common::config::{get_config_path, Config};

        // Use provided config path or default
        let config_path = self.config_path.clone().unwrap_or_else(get_config_path);

        if !config_path.exists() {
            warn!("Config file does not exist at {:?}, no repositories loaded", config_path);
            return Ok(());
        }

        info!("Loading repositories from config file: {:?}", config_path);

        // Try to load the config file
        match Config::load(&config_path) {
            Ok(config) => {
                // Extract repositories from config
                if let Some(repos) = config.repositories {
                    let active_repos: Vec<String> = repos
                        .iter()
                        .filter(|r| r.active.unwrap_or(true))
                        .map(|r| r.path.clone())
                        .collect();

                    info!("Found {} active repositories in config", active_repos.len());
                    self.repositories = active_repos;
                } else {
                    info!("No repositories defined in config");
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to load config file: {}", e);
                Err(DaemonError::ConfigError(format!("Failed to load config: {}", e)))
            }
        }
    }

    /// Runs the daemon server
    pub fn run(&mut self) -> Result<(), DaemonError> {
        // Remove socket file if it already exists
        if self.socket_path.exists() {
            info!("Removing existing socket file at {}", self.socket_path.display());
            if let Err(e) = fs::remove_file(&self.socket_path) {
                error!("Failed to remove existing socket file: {}", e);
                return Err(DaemonError::SocketError(e));
            }
        }

        // Create directory for socket if it doesn't exist
        if let Some(parent) = self.socket_path.parent() {
            info!("Ensuring socket directory exists at {}", parent.display());
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create socket directory: {}", e);
                return Err(DaemonError::SocketError(e));
            }
        }

        // Create the Unix socket
        info!("Creating Unix socket at {}", self.socket_path.display());
        let listener = match UnixListener::bind(&self.socket_path) {
            Ok(listener) => {
                info!("Successfully bound to socket at {}", self.socket_path.display());
                listener
            }
            Err(e) => {
                error!("Failed to bind to socket at {}: {}", self.socket_path.display(), e);
                return Err(DaemonError::SocketError(e));
            }
        };

        // Set socket permissions to be world accessible
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            info!("Setting socket permissions to be accessible");
            if let Err(e) =
                fs::set_permissions(&self.socket_path, fs::Permissions::from_mode(0o777))
            {
                warn!("Failed to set socket permissions: {}", e);
                // Continue anyway, might still work
            }
        }

        // Load repositories from config
        if let Err(e) = self.load_repositories_from_config() {
            warn!("Error loading repositories from config: {}", e);
            // Continue running even if we can't load repos
        }

        info!(
            "Daemon server started at {:?} with {} repositories",
            self.socket_path,
            self.repositories.len()
        );

        // Set up signal handling for graceful shutdown
        let running = self.running.clone();

        #[cfg(unix)]
        {
            use ctrlc::set_handler;
            if let Err(e) = set_handler(move || {
                info!("Received termination signal, shutting down daemon...");
                running.store(false, Ordering::SeqCst);
            }) {
                error!("Failed to set signal handler: {}", e);
                return Err(DaemonError::StartError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to set signal handler: {}", e),
                )));
            }
        }

        // Main server loop
        info!("Entering main server loop");

        // Reset heartbeat timer at startup
        self.last_heartbeat = std::time::Instant::now();

        while self.running.load(Ordering::SeqCst) {
            // Check heartbeat
            self.check_heartbeat();

            // Check log rotation
            self.check_and_rotate_logs();

            // Set the listener to non-blocking mode
            if let Err(e) = listener.set_nonblocking(true) {
                error!("Failed to set non-blocking mode: {}", e);
                return Err(DaemonError::SocketError(e));
            }

            // Accept connections with timeout
            match listener.accept() {
                Ok((stream, _)) => {
                    debug!("Accepted new client connection");
                    // Handle client connection
                    if let Err(e) = self.handle_client(stream) {
                        error!("Error handling client: {}", e);
                        // Continue serving other clients even if one fails
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No connection available, sleep and continue
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                    return Err(DaemonError::SocketError(e));
                }
            }
        }

        info!("Daemon server shutting down");

        // Clean up socket file
        if self.socket_path.exists() {
            if let Err(e) = fs::remove_file(&self.socket_path) {
                error!("Failed to remove socket file: {}", e);
                return Err(DaemonError::SocketError(e));
            }
        }

        Ok(())
    }

    pub fn get_running(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    // Add a heartbeat check method
    fn check_heartbeat(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_heartbeat).as_millis() > self.heartbeat_interval_ms as u128
        {
            info!(
                "Daemon heartbeat: running for {} seconds, monitoring {} repositories",
                self.start_time.elapsed().as_secs(),
                self.repositories.len()
            );
            self.last_heartbeat = now;
        }
    }

    /// Handles a client connection
    fn handle_client(&mut self, mut stream: UnixStream) -> Result<(), DaemonError> {
        // Try to set timeouts, but don't fail if they're not supported
        // This is particularly important for macOS which has limited socket timeout support
        if let Err(e) = stream.set_read_timeout(Some(Duration::from_millis(2000))) {
            debug!("Socket read timeout not supported: {}", e);
            // Continue anyway - we'll use non-blocking mode as a fallback
        }

        if let Err(e) = stream.set_write_timeout(Some(Duration::from_millis(2000))) {
            debug!("Socket write timeout not supported: {}", e);
            // Continue anyway
        }

        // As a fallback, set the socket to non-blocking mode
        // This is more universally supported across Unix platforms
        match stream.set_nonblocking(true) {
            Ok(_) => {
                debug!("Set socket to non-blocking mode");
            }
            Err(e) => {
                error!("Failed to set non-blocking mode: {}", e);
                return Err(DaemonError::SocketError(e));
            }
        }

        // Read the message length with non-blocking handling
        let mut len_bytes = [0u8; 4];
        match read_with_timeout(&mut stream, &mut len_bytes, Duration::from_millis(2000)) {
            Ok(_) => {
                // Successfully read the length
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                debug!("Read timeout while reading message length");
                return Ok(());
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("Client disconnected while reading message length");
                return Ok(());
            }
            Err(e) => {
                return Err(DaemonError::IpcError(e));
            }
        }

        let len = u32::from_be_bytes(len_bytes) as usize;

        // Check for message size limits
        if len > MAX_MESSAGE_SIZE {
            return Err(DaemonError::InvalidMessage(format!("Message too large: {} bytes", len)));
        }

        if len == 0 {
            debug!("Received zero-length message, ignoring");
            return Ok(());
        }

        // Read the message body with non-blocking handling
        let mut message_bytes = vec![0u8; len];
        match read_with_timeout(&mut stream, &mut message_bytes, Duration::from_millis(2000)) {
            Ok(_) => {
                // Successfully read the message
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                debug!("Read timeout while reading message body");
                return Ok(());
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("Client disconnected while reading message body");
                return Ok(());
            }
            Err(e) => {
                return Err(DaemonError::IpcError(e));
            }
        }

        // Set socket back to blocking mode for the response write
        // This is not strictly necessary but helps ensure complete writes
        if let Err(e) = stream.set_nonblocking(false) {
            debug!("Failed to set socket back to blocking mode: {}", e);
            // Continue anyway
        }

        // Deserialize message
        let message: DaemonMessage = match bincode::deserialize(&message_bytes) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to deserialize message: {}", e);
                return Err(DaemonError::SerializationError(e));
            }
        };

        debug!("Received message: {:?}", message);

        // Process message and get response
        let response = match self.process_message(message) {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Error processing message: {}", e);
                return Err(e);
            }
        };

        // Serialize response with error handling
        let response_bytes = match bincode::serialize(&response) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Failed to serialize response: {}", e);
                return Err(DaemonError::SerializationError(e));
            }
        };

        // Send response length with better error handling
        let response_len = response_bytes.len() as u32;
        match write_with_timeout(
            &mut stream,
            &response_len.to_be_bytes(),
            Duration::from_millis(2000),
        ) {
            Ok(_) => {
                // Successfully wrote length
            }
            Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                debug!("Client disconnected before response could be sent");
                return Ok(());
            }
            Err(e) => {
                return Err(DaemonError::IpcError(e));
            }
        }

        // Send response with better error handling
        match write_with_timeout(&mut stream, &response_bytes, Duration::from_millis(2000)) {
            Ok(_) => {
                // Successfully wrote response
            }
            Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                debug!("Client disconnected before response body could be sent");
                return Ok(());
            }
            Err(e) => {
                return Err(DaemonError::IpcError(e));
            }
        }

        Ok(())
    }

    /// Processes an incoming message and returns a response
    fn process_message(&mut self, message: DaemonMessage) -> Result<DaemonMessage, DaemonError> {
        match message {
            DaemonMessage::StatusRequest => {
                // Get uptime in seconds
                let uptime = self.start_time.elapsed().as_secs();

                Ok(DaemonMessage::StatusResponse {
                    running: true,
                    pid: Some(std::process::id()),
                    uptime_seconds: Some(uptime),
                    monitored_repos: Some(self.repositories.len()),
                })
            }

            DaemonMessage::AddRepositoryRequest { path, name } => {
                // In a real implementation, we would actually set up monitoring for this repository
                // For now, just add it to our list
                let repo_id = name.unwrap_or_else(|| {
                    Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unnamed")
                        .to_string()
                });

                info!("Adding repository: {} at {}", repo_id, path);

                // Check if already monitoring this repository
                if self.repositories.contains(&path) {
                    return Ok(DaemonMessage::AddRepositoryResponse {
                        success: false,
                        error: Some(format!("Already monitoring repository: {}", path)),
                    });
                }

                // Add repository to list
                self.repositories.push(path);

                Ok(DaemonMessage::AddRepositoryResponse { success: true, error: None })
            }

            DaemonMessage::RemoveRepositoryRequest { identifier } => {
                // Remove repository from list
                let initial_len = self.repositories.len();
                self.repositories.retain(|r| r != &identifier);

                let removed = self.repositories.len() < initial_len;

                if removed {
                    info!("Removed repository: {}", identifier);
                    Ok(DaemonMessage::RemoveRepositoryResponse { success: true, error: None })
                } else {
                    warn!("Repository not found: {}", identifier);
                    Ok(DaemonMessage::RemoveRepositoryResponse {
                        success: false,
                        error: Some(format!("Repository not found: {}", identifier)),
                    })
                }
            }

            DaemonMessage::CommandRequest { command, args: _ } => {
                // Handle custom commands
                match command.as_str() {
                    "ping" => Ok(DaemonMessage::CommandResponse {
                        success: true,
                        message: "pong".to_string(),
                        data: None,
                    }),
                    "list-repos" => {
                        let repos_list = self.repositories.join("\n");
                        Ok(DaemonMessage::CommandResponse {
                            success: true,
                            message: format!("Found {} repositories", self.repositories.len()),
                            data: Some(repos_list),
                        })
                    }
                    "uptime" => {
                        let uptime = self.start_time.elapsed().as_secs();
                        Ok(DaemonMessage::CommandResponse {
                            success: true,
                            message: format!("Uptime: {} seconds", uptime),
                            data: Some(uptime.to_string()),
                        })
                    }
                    // Add our new health command following the existing pattern
                    "health" => {
                        let uptime = self.start_time.elapsed().as_secs();
                        let repo_count = self.repositories.len();

                        // Get memory usage
                        let memory_usage = get_process_memory_usage();

                        // Add formatted memory usage info to the message
                        let memory_str = if memory_usage > 1024 {
                            format!("{:.2} MB", memory_usage as f64 / 1024.0)
                        } else if memory_usage > 0 {
                            format!("{} KB", memory_usage)
                        } else {
                            "unknown".to_string()
                        };

                        // Create a structured health response
                        let health_data = serde_json::json!({
                            "status": "healthy",
                            "uptime_seconds": uptime,
                            "repository_count": repo_count,
                            "memory_usage_kb": memory_usage,
                            "memory_usage_formatted": memory_str,
                        });

                        Ok(DaemonMessage::CommandResponse {
                            success: true,
                            message: format!(
                                "Daemon healthy, uptime: {}s, repos: {}, memory: {}",
                                uptime, repo_count, memory_str
                            ),
                            data: Some(health_data.to_string()),
                        })
                    }
                    _ => {
                        warn!("Unknown command: {}", command);
                        Ok(DaemonMessage::CommandResponse {
                            success: false,
                            message: format!("Unknown command: {}", command),
                            data: None,
                        })
                    }
                }
            }

            DaemonMessage::ShutdownRequest => {
                info!("Received shutdown request, shutting down daemon...");
                self.running.store(false, Ordering::SeqCst);

                Ok(DaemonMessage::ShutdownResponse { success: true })
            }

            _ => {
                warn!("Received unexpected message type");
                Ok(DaemonMessage::CommandResponse {
                    success: false,
                    message: "Unexpected message type".to_string(),
                    data: None,
                })
            }
        }
    }

    // Add log rotation method
    fn check_and_rotate_logs(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_log_check).as_millis() < self.log_check_interval_ms as u128
        {
            return; // Not time to check yet
        }

        // Update last check time
        self.last_log_check = now;

        // Get log file path
        let log_path = if let Some(pid_file) = self.daemon_pid_file() {
            pid_file.with_file_name("daemon.log")
        } else {
            return; // Can't determine log path
        };

        // Constants for log rotation
        const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
        const MAX_LOG_FILES: usize = 5; // 5 rotated files

        // Check if rotation is needed
        if !log_path.exists() {
            return; // No log file yet
        }

        match metadata(&log_path) {
            Ok(metadata) => {
                if metadata.len() > MAX_LOG_SIZE {
                    info!("Log file exceeds max size ({}), rotating...", MAX_LOG_SIZE);
                    self.rotate_log_file(&log_path, MAX_LOG_FILES);
                }
            }
            Err(e) => warn!("Failed to check log file size: {}", e),
        }
    }

    // Get the daemon PID file path
    fn daemon_pid_file(&self) -> Option<PathBuf> {
        // Try environment variable first
        if let Ok(path) = std::env::var("WORKSPACE_PID_FILE") {
            return Some(PathBuf::from(path));
        }

        // Try to derive from config
        if let Some(ref config_path) = self.config_path {
            if let Ok(config) = Config::load(config_path) {
                if let Some(daemon_config) = config.daemon {
                    if let Some(pid_file) = daemon_config.pid_file {
                        return Some(PathBuf::from(pid_file));
                    }
                }
            }
        }

        // Fallback to standard location
        let mut path = dirs::runtime_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")))
            .join("workspace-cli");
        path.push("daemon.pid");
        Some(path)
    }

    // Rotate the log file
    fn rotate_log_file(&self, log_path: &Path, max_files: usize) {
        // Create backup filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup_path = log_path.with_file_name(format!("daemon-{}.log", timestamp));

        // Open a new log file for writing
        match File::create(&backup_path) {
            Ok(mut backup_file) => {
                // Open current log for reading
                match File::open(log_path) {
                    Ok(mut current_log) => {
                        // Copy current log to backup
                        if let Err(e) = std::io::copy(&mut current_log, &mut backup_file) {
                            warn!("Failed to copy log file during rotation: {}", e);
                            return;
                        }

                        // Truncate current log
                        if let Ok(mut file) =
                            OpenOptions::new().write(true).truncate(true).open(log_path)
                        {
                            // Write rotation message to new log
                            let _ = writeln!(
                                file,
                                "Log rotated at {}, previous log saved to {}",
                                chrono::Local::now().to_rfc3339(),
                                backup_path.display()
                            );
                        }

                        info!("Rotated log to {}", backup_path.display());
                    }
                    Err(e) => warn!("Failed to open current log for rotation: {}", e),
                }
            }
            Err(e) => warn!("Failed to create backup log file: {}", e),
        }

        // Cleanup old log files
        if let Some(parent) = log_path.parent() {
            if parent.exists() {
                if let Ok(entries) = fs::read_dir(parent) {
                    let mut log_files = Vec::new();

                    // Collect all rotated log files
                    for entry in entries.filter_map(Result::ok) {
                        let path = entry.path();
                        if let Some(filename) = path.file_name() {
                            let filename = filename.to_string_lossy();
                            if filename.starts_with("daemon-") && filename.ends_with(".log") {
                                log_files.push(path);
                            }
                        }
                    }

                    // Sort by modification time (newest first)
                    log_files.sort_by(|a, b| {
                        // Get modification time for first file, with fallback to UNIX_EPOCH if we can't get metadata
                        let time_a = metadata(a)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                        // Get modification time for second file
                        let time_b = metadata(b)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                        // Compare times (newest first)
                        time_b.cmp(&time_a)
                    });

                    // Remove older logs beyond our limit
                    for old_log in log_files.iter().skip(max_files) {
                        info!("Removing old log file: {}", old_log.display());
                        let _ = fs::remove_file(old_log);
                    }
                }
            }
        }
    }
}

/// Helper function for non-blocking reads with timeout
fn read_with_timeout(stream: &mut UnixStream, buf: &mut [u8], timeout: Duration) -> io::Result<()> {
    let start = std::time::Instant::now();
    let mut bytes_read = 0;

    while bytes_read < buf.len() {
        // Check if we've exceeded the timeout
        if start.elapsed() > timeout {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "Read operation timed out"));
        }

        match stream.read(&mut buf[bytes_read..]) {
            Ok(0) => {
                // End of file
                if bytes_read == 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"));
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Connection closed before completing read",
                    ));
                }
            }
            Ok(n) => {
                bytes_read += n;
                if bytes_read == buf.len() {
                    return Ok(());
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Socket would block, wait a bit and retry
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Helper function for non-blocking writes with timeout
fn write_with_timeout(stream: &mut UnixStream, buf: &[u8], timeout: Duration) -> io::Result<()> {
    let start = std::time::Instant::now();
    let mut bytes_written = 0;

    while bytes_written < buf.len() {
        // Check if we've exceeded the timeout
        if start.elapsed() > timeout {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "Write operation timed out"));
        }

        match stream.write(&buf[bytes_written..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Failed to write data to socket",
                ));
            }
            Ok(n) => {
                bytes_written += n;
                if bytes_written == buf.len() {
                    return Ok(());
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Socket would block, wait a bit and retry
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Gets the current memory usage of the process in kilobytes.
///
/// Returns the resident set size (RSS), which represents the portion of process memory
/// held in RAM (physical memory) rather than swap or disk.
///
/// # Returns
///
/// * The memory usage in kilobytes, or 0 if the information couldn't be obtained
///
/// # Platform Support
///
/// * Linux: Uses /proc/self/status (VmRSS field)
/// * macOS: Uses Mach task_info API
/// * Windows: Uses Windows GetProcessMemoryInfo API
/// * Other platforms: Returns 0 with a warning
fn get_process_memory_usage() -> u64 {
    #[cfg(target_os = "linux")]
    {
        // Linux implementation - read from /proc/self/status
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        match File::open("/proc/self/status") {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        // VmRSS gives the resident set size (physical memory used)
                        if line.starts_with("VmRSS:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                if let Ok(kb) = parts[1].parse::<u64>() {
                                    // Already in KB on Linux
                                    return kb;
                                } else {
                                    log::warn!(
                                        "Failed to parse VmRSS value from /proc/self/status"
                                    );
                                }
                            }
                        }
                    }
                }

                // Fallback if we can't find VmRSS
                log::warn!("Failed to find VmRSS in /proc/self/status");
                0
            }
            Err(e) => {
                log::warn!("Failed to open /proc/self/status: {}", e);
                0
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Define constants from mach API
        const TASK_VM_INFO: i32 = 22;
        const TASK_VM_INFO_COUNT: u32 = 40; // varies by macOS version

        // Simplified version of task_vm_info structure
        #[repr(C)]
        struct TaskVMInfo {
            virtual_size: u64, // virtual memory size (bytes)
            region_count: u32, // number of memory regions
            page_size: u32,
            resident_size: u64,      // resident memory size (bytes)
            resident_size_peak: u64, // peak resident size (bytes)
            // Additional fields omitted - using a buffer instead
            _unused: [u64; 33],
        }

        extern "C" {
            fn mach_task_self() -> u32;
            fn task_info(
                task: u32,
                flavor: i32,
                task_info: *mut TaskVMInfo,
                task_info_count: *mut u32,
            ) -> i32;
        }

        // Initialize the task info struct
        let mut task_info_data = TaskVMInfo {
            virtual_size: 0,
            region_count: 0,
            page_size: 0,
            resident_size: 0,
            resident_size_peak: 0,
            _unused: [0; 33],
        };

        // Get the info count - note this varies by macOS version
        let mut count = TASK_VM_INFO_COUNT;

        // Call the task_info function
        unsafe {
            let kr = task_info(mach_task_self(), TASK_VM_INFO, &mut task_info_data, &mut count);

            if kr == 0 {
                // Convert bytes to kilobytes
                task_info_data.resident_size / 1024
            } else {
                log::warn!("Failed to get macOS task info, error code: {}", kr);
                0
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows implementation - use GetProcessMemoryInfo API
        use std::mem::size_of;
        use std::ptr;

        // Define Windows API types
        type DWORD = u32;
        type HANDLE = *mut std::ffi::c_void;
        type SIZE_T = usize;
        type BOOL = i32;

        // Process memory counters structure - PROCESS_MEMORY_COUNTERS
        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: DWORD,
            page_fault_count: DWORD,
            peak_working_set_size: SIZE_T,
            working_set_size: SIZE_T, // This is what we want (RSS equivalent)
            quota_peak_paged_pool_usage: SIZE_T,
            quota_paged_pool_usage: SIZE_T,
            quota_peak_non_paged_pool_usage: SIZE_T,
            quota_non_paged_pool_usage: SIZE_T,
            page_file_usage: SIZE_T,
            peak_page_file_usage: SIZE_T,
        }

        // Windows API functions
        extern "system" {
            fn GetCurrentProcess() -> HANDLE;
            fn GetProcessMemoryInfo(
                process: HANDLE,
                ppsmemCounters: *mut ProcessMemoryCounters,
                cb: DWORD,
            ) -> BOOL;
        }

        // Initialize the memory counters structure
        let mut pmc = ProcessMemoryCounters {
            cb: size_of::<ProcessMemoryCounters>() as DWORD,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            page_file_usage: 0,
            peak_page_file_usage: 0,
        };

        unsafe {
            // Get current process handle (doesn't need to be closed)
            let process_handle = GetCurrentProcess();

            if process_handle != ptr::null_mut() {
                // Get memory info
                let result = GetProcessMemoryInfo(
                    process_handle,
                    &mut pmc,
                    size_of::<ProcessMemoryCounters>() as DWORD,
                );

                if result != 0 {
                    // Convert bytes to kilobytes
                    return (pmc.working_set_size / 1024) as u64;
                } else {
                    log::warn!("Windows GetProcessMemoryInfo failed");
                    return 0;
                }
            } else {
                log::warn!("Windows GetCurrentProcess failed");
                return 0;
            }
        }
    }

    // If we're on a different platform or all the platform-specific methods failed
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        log::info!("Process memory usage reporting not implemented for this platform");
        0
    }
}

// Helper function to check if a process is running
#[cfg(unix)]
pub fn check_process_running(pid: u32) -> bool {
    use std::process::Command;

    // Different command for different Unix variants
    #[cfg(target_os = "macos")]
    let output = Command::new("ps").args(["-p", &pid.to_string()]).output();

    #[cfg(all(unix, not(target_os = "macos")))]
    let output = Command::new("ps").args(&["-p", &pid.to_string()]).output();

    match output {
        Ok(output) => {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

#[cfg(windows)]
pub fn check_process_running(pid: u32) -> bool {
    // For Windows, we'd need to use the Windows API
    // This is a simplified implementation
    type DWORD = u32;
    type HANDLE = *mut std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: i32, dwProcessId: DWORD) -> HANDLE;
        fn CloseHandle(hObject: HANDLE) -> i32;
    }

    const PROCESS_QUERY_INFORMATION: DWORD = 0x0400;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        let exists = !handle.is_null();

        if !handle.is_null() {
            CloseHandle(handle);
        }

        exists
    }
}
