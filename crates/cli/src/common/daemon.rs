use crate::common::config::DaemonConfig;
use anyhow::Result;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thiserror::Error;

use super::config::Config;

/// Maximum message size for IPC communications
const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB

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
        // Check if socket exists and is valid
        let socket_path = self.socket_path();
        if socket_path.exists() {
            // Try to connect to the socket to verify it's active
            match UnixStream::connect(&socket_path) {
                Ok(_) => {
                    // Socket exists and is connectable, daemon is running
                    return Ok(true);
                }
                Err(e) => {
                    debug!("Socket exists but connection failed: {}", e);
                    // Socket exists but can't connect, might be stale
                    // Continue checking PID file
                }
            }
        }

        // Check if PID file exists
        let pid_file = self.pid_file_path();
        if !pid_file.exists() {
            return Ok(false);
        }

        // Read PID from file
        let pid = self.read_pid()?;

        // On Unix-like systems, we can check if the process exists
        #[cfg(unix)]
        {
            use std::process::Command;
            let output = Command::new("ps")
                .arg("-p")
                .arg(pid.to_string())
                .output()
                .map_err(DaemonError::StatusReadError)?;

            // If ps returns with output containing the PID, the process exists
            return Ok(output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()));
        }

        // On Windows, we would need a different approach
        #[cfg(windows)]
        {
            // Windows implementation would be different and use different APIs
            // For now, just check if we can connect to the socket
            match UnixStream::connect(self.socket_path()) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }

        // For other platforms, just check if we can connect to the socket
        #[cfg(not(any(unix, windows)))]
        {
            match UnixStream::connect(self.socket_path()) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
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

        // Set up command to run the daemon in detached mode
        let mut cmd = Command::new(daemon_bin);
        cmd.arg("run");

        // Add socket path and pid file arguments
        cmd.arg("--socket-path").arg(self.socket_path().to_string_lossy().to_string());
        cmd.arg("--pid-file").arg(self.pid_file_path().to_string_lossy().to_string());

        // Set log level if provided
        if let Some(level) = log_level {
            cmd.arg("--log-level").arg(level);
        }

        // Detach the process
        cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

        // Start daemon process
        info!("Starting daemon process...");
        let child = cmd.spawn().map_err(DaemonError::StartError)?;

        // Write PID to file
        self.write_pid(child.id() as u32)?;

        // Wait a bit for the daemon to initialize
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_millis(DEFAULT_OPERATION_TIMEOUT_MS);

        while start_time.elapsed() < timeout {
            if self.is_running()? {
                // Try to connect to socket to ensure it's actually running
                match UnixStream::connect(self.socket_path()) {
                    Ok(_) => {
                        info!("Daemon started successfully with PID {}", child.id());
                        return Ok(());
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        // If we got here, the process started but isn't responding
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

        // Create an add repository request
        let request = DaemonMessage::AddRepositoryRequest {
            path: path.to_string(),
            name: name.map(String::from),
        };

        // Send the request and get the response
        match self.send_message(&request) {
            Ok(DaemonMessage::AddRepositoryResponse { success, error }) => {
                if success {
                    Ok(())
                } else {
                    Err(DaemonError::CommandError(
                        error.unwrap_or_else(|| "Failed to add repository".to_string()),
                    ))
                }
            }
            Ok(_) => Err(DaemonError::InvalidMessage(
                "Unexpected response to add repository request".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Removes a repository from monitoring
    pub fn remove_repository(&self, identifier: &str) -> Result<(), DaemonError> {
        // Check if the daemon is running
        if !self.is_running()? {
            return Err(DaemonError::NotRunning);
        }

        // Create a remove repository request
        let request = DaemonMessage::RemoveRepositoryRequest { identifier: identifier.to_string() };

        // Send the request and get the response
        match self.send_message(&request) {
            Ok(DaemonMessage::RemoveRepositoryResponse { success, error }) => {
                if success {
                    Ok(())
                } else {
                    Err(DaemonError::CommandError(
                        error.unwrap_or_else(|| "Failed to remove repository".to_string()),
                    ))
                }
            }
            Ok(_) => Err(DaemonError::InvalidMessage(
                "Unexpected response to remove repository request".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    // Private helper methods

    /// Sends a message to the daemon and receives a response
    fn send_message(&self, message: &DaemonMessage) -> Result<DaemonMessage, DaemonError> {
        // Connect to the daemon socket
        let mut stream =
            UnixStream::connect(self.socket_path()).map_err(DaemonError::SocketError)?;

        // Set a read timeout
        stream
            .set_read_timeout(Some(Duration::from_millis(DEFAULT_OPERATION_TIMEOUT_MS)))
            .map_err(DaemonError::SocketError)?;

        stream
            .set_write_timeout(Some(Duration::from_millis(DEFAULT_OPERATION_TIMEOUT_MS)))
            .map_err(DaemonError::SocketError)?;

        // Serialize the message
        let serialized = bincode::serialize(message).map_err(DaemonError::SerializationError)?;

        // Send the message length first
        let len = serialized.len() as u32;
        let len_bytes = len.to_be_bytes();
        stream.write_all(&len_bytes).map_err(DaemonError::IpcError)?;

        // Then send the message itself
        stream.write_all(&serialized).map_err(DaemonError::IpcError)?;

        // Read the response length
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).map_err(DaemonError::IpcError)?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        // Check for message size limits
        if len > MAX_MESSAGE_SIZE {
            return Err(DaemonError::InvalidMessage(format!(
                "Response message too large: {} bytes",
                len
            )));
        }

        // Read the response
        let mut response_bytes = vec![0u8; len];
        stream.read_exact(&mut response_bytes).map_err(DaemonError::IpcError)?;

        // Deserialize the response
        let response: DaemonMessage =
            bincode::deserialize(&response_bytes).map_err(DaemonError::SerializationError)?;

        Ok(response)
    }

    /// Reads the daemon PID from the PID file
    fn read_pid(&self) -> Result<u32, DaemonError> {
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
}

/// Daemon server for handling IPC communication in the daemon process
pub struct DaemonServer {
    socket_path: PathBuf,
    running: Arc<AtomicBool>,
    start_time: std::time::Instant,
    repositories: Vec<String>, // Placeholder for repository tracking
}

impl DaemonServer {
    /// Creates a new daemon server
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            running: Arc::new(AtomicBool::new(true)),
            start_time: std::time::Instant::now(),
            repositories: Vec::new(),
        }
    }

    /// Runs the daemon server
    pub fn run(&mut self) -> Result<(), DaemonError> {
        // Remove socket file if it already exists
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path).map_err(DaemonError::SocketError)?;
        }

        // Create directory for socket if it doesn't exist
        if let Some(parent) = self.socket_path.parent() {
            fs::create_dir_all(parent).map_err(DaemonError::SocketError)?;
        }

        // Create the Unix socket
        let listener = UnixListener::bind(&self.socket_path).map_err(DaemonError::SocketError)?;

        info!("Daemon server started at {:?}", self.socket_path);

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
        while self.running.load(Ordering::SeqCst) {
            // Set the listener to non-blocking mode
            if let Err(e) = listener.set_nonblocking(true) {
                error!("Failed to set non-blocking mode: {}", e);
                return Err(DaemonError::SocketError(e));
            }

            // Accept connections with timeout
            match listener.accept() {
                Ok((stream, _)) => {
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

    /// Handles a client connection
    fn handle_client(&mut self, mut stream: UnixStream) -> Result<(), DaemonError> {
        // Read message length
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).map_err(DaemonError::IpcError)?;

        let len = u32::from_be_bytes(len_bytes) as usize;

        // Check for message size limits
        if len > MAX_MESSAGE_SIZE {
            return Err(DaemonError::InvalidMessage(format!("Message too large: {} bytes", len)));
        }

        // Read message
        let mut message_bytes = vec![0u8; len];
        stream.read_exact(&mut message_bytes).map_err(DaemonError::IpcError)?;

        // Deserialize message
        let message: DaemonMessage =
            bincode::deserialize(&message_bytes).map_err(DaemonError::SerializationError)?;

        debug!("Received message: {:?}", message);

        // Process message and get response
        let response = self.process_message(message)?;

        // Serialize response
        let response_bytes =
            bincode::serialize(&response).map_err(DaemonError::SerializationError)?;

        // Send response length
        let response_len = response_bytes.len() as u32;
        stream.write_all(&response_len.to_be_bytes()).map_err(DaemonError::IpcError)?;

        // Send response
        stream.write_all(&response_bytes).map_err(DaemonError::IpcError)?;

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
}
