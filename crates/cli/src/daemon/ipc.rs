//! IPC (Inter-Process Communication) for the daemon service

use anyhow::{Context, Result};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::daemon::event::Event;
use crate::daemon::status::DaemonStatus;

/// IPC message from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Check if the daemon is running
    Ping,
    /// Get daemon status
    Status,
    /// Add a repository to monitor
    AddRepository { path: PathBuf, name: Option<String> },
    /// Remove a repository from monitoring
    RemoveRepository { name: String },
    /// List monitored repositories
    ListRepositories,
    /// Get changes for a repository
    GetChanges { repository: String },
    /// Get events
    GetEvents { since: Option<String>, limit: Option<usize> },
    /// Shutdown the daemon
    Shutdown,
}

/// IPC response from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    /// Success with no data
    Ok,
    /// Success with string data
    String(String),
    /// Success with daemon status
    Status(DaemonStatus),
    /// Success with repository list
    Repositories(Vec<(String, PathBuf)>),
    /// Success with events
    Events(Vec<Event>),
    /// Error
    Error(String),
}

impl IpcResponse {
    /// Create a success response
    pub fn ok() -> Self {
        Self::Ok
    }

    /// Create a string response
    pub fn string<S: Into<String>>(s: S) -> Self {
        Self::String(s.into())
    }

    /// Create an error response
    pub fn error<S: Into<String>>(s: S) -> Self {
        Self::Error(s.into())
    }
}

/// IPC server for the daemon service
pub struct IpcServer {
    /// Socket path
    socket_path: PathBuf,
    /// Callback for processing messages
    message_handler: Arc<dyn Fn(IpcMessage) -> IpcResponse + Send + Sync>,
    /// Whether the server is running
    running: Arc<Mutex<bool>>,
}

/// IPC client for connecting to the daemon
pub struct IpcClient {
    /// Socket path
    socket_path: PathBuf,
    /// Timeout for operations
    timeout: Duration,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new<P: AsRef<Path>, F>(socket_path: P, message_handler: F) -> Self
    where
        F: Fn(IpcMessage) -> IpcResponse + Send + Sync + 'static,
    {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            message_handler: Arc::new(message_handler),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the IPC server
    pub fn start(&self) -> Result<()> {
        let socket_path = self.socket_path.clone();

        // Create parent directory if it doesn't exist
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Remove socket file if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).with_context(|| {
                format!("Failed to remove existing socket: {}", socket_path.display())
            })?;
        }

        // Create the listener
        let listener = UnixListener::bind(&socket_path)
            .with_context(|| format!("Failed to bind to socket: {}", socket_path.display()))?;

        // Mark server as running
        *self.running.lock().unwrap() = true;

        let running = self.running.clone();
        let message_handler = self.message_handler.clone();

        // Spawn a thread to handle connections
        thread::spawn(move || {
            info!("IPC server started at {}", socket_path.display());

            listener.set_nonblocking(true).unwrap_or_else(|e| {
                error!("Failed to set non-blocking mode: {}", e);
            });

            while *running.lock().unwrap() {
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        debug!("Accepted connection");
                        let message_handler_clone = Arc::clone(&message_handler);

                        thread::spawn(move || {
                            // Handle client in a separate thread
                            if let Err(e) = handle_client(stream, message_handler_clone) {
                                error!("Error handling client: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection available, sleep a bit
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        error!("Error accepting connection: {}", e);
                        // Don't break on transient errors
                        if e.kind() != std::io::ErrorKind::Interrupted {
                            break;
                        }
                    }
                }
            }

            info!("IPC server stopping");

            // Clean up socket file
            if socket_path.exists() {
                if let Err(e) = std::fs::remove_file(&socket_path) {
                    error!("Failed to remove socket file: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Stop the IPC server
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self { socket_path: socket_path.as_ref().to_path_buf(), timeout: Duration::from_secs(5) }
    }

    /// Connect to the IPC server
    pub fn connect(&self) -> Result<UnixStream> {
        UnixStream::connect(&self.socket_path)
            .with_context(|| format!("Failed to connect to socket: {}", self.socket_path.display()))
    }

    /// Set timeout for operations
    pub fn set_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    /// Send a message to the server
    pub fn send_message(&self, message: IpcMessage) -> Result<IpcResponse> {
        let mut stream = self.connect()?;

        // Set read timeout
        stream.set_read_timeout(Some(self.timeout)).context("Failed to set read timeout")?;

        // Serialize message
        let message_bytes = bincode::serialize(&message).context("Failed to serialize message")?;

        // Send message size first (u32 as 4 bytes)
        let size = message_bytes.len() as u32;
        stream.write_all(&size.to_le_bytes()).context("Failed to send message size")?;

        // Send message
        stream.write_all(&message_bytes).context("Failed to send message")?;

        // Read response size
        let mut size_buf = [0u8; 4];
        stream.read_exact(&mut size_buf).context("Failed to read response size")?;
        let response_size = u32::from_le_bytes(size_buf) as usize;

        // Read response
        let mut response_buf = vec![0u8; response_size];
        stream.read_exact(&mut response_buf).context("Failed to read response")?;

        // Deserialize response
        let response =
            bincode::deserialize(&response_buf).context("Failed to deserialize response")?;

        Ok(response)
    }

    /// Check if the daemon is running
    pub fn ping(&self) -> Result<bool> {
        match self.send_message(IpcMessage::Ping) {
            Ok(IpcResponse::Ok) => Ok(true),
            Ok(_) => Ok(true), // Any response means the daemon is running
            Err(_) => Ok(false),
        }
    }

    /// Get daemon status
    pub fn status(&self) -> Result<DaemonStatus> {
        match self.send_message(IpcMessage::Status) {
            Ok(IpcResponse::Status(status)) => Ok(status),
            Ok(IpcResponse::Error(msg)) => Err(anyhow::anyhow!("Error getting status: {}", msg)),
            Ok(_) => Err(anyhow::anyhow!("Unexpected response")),
            Err(e) => Err(e),
        }
    }

    /// Add a repository to monitor
    pub fn add_repository<P: AsRef<Path>, S: Into<String>>(
        &self,
        path: P,
        name: Option<S>,
    ) -> Result<()> {
        let name = name.map(|s| s.into());
        match self
            .send_message(IpcMessage::AddRepository { path: path.as_ref().to_path_buf(), name })
        {
            Ok(IpcResponse::Ok) => Ok(()),
            Ok(IpcResponse::Error(msg)) => Err(anyhow::anyhow!("Error adding repository: {}", msg)),
            Ok(_) => Err(anyhow::anyhow!("Unexpected response")),
            Err(e) => Err(e),
        }
    }

    /// Remove a repository from monitoring
    pub fn remove_repository<S: Into<String>>(&self, name: S) -> Result<()> {
        match self.send_message(IpcMessage::RemoveRepository { name: name.into() }) {
            Ok(IpcResponse::Ok) => Ok(()),
            Ok(IpcResponse::Error(msg)) => {
                Err(anyhow::anyhow!("Error removing repository: {}", msg))
            }
            Ok(_) => Err(anyhow::anyhow!("Unexpected response")),
            Err(e) => Err(e),
        }
    }

    /// List monitored repositories
    pub fn list_repositories(&self) -> Result<Vec<(String, PathBuf)>> {
        match self.send_message(IpcMessage::ListRepositories) {
            Ok(IpcResponse::Repositories(repos)) => Ok(repos),
            Ok(IpcResponse::Error(msg)) => {
                Err(anyhow::anyhow!("Error listing repositories: {}", msg))
            }
            Ok(_) => Err(anyhow::anyhow!("Unexpected response")),
            Err(e) => Err(e),
        }
    }

    /// Shutdown the daemon
    pub fn shutdown(&self) -> Result<()> {
        match self.send_message(IpcMessage::Shutdown) {
            Ok(IpcResponse::Ok) => Ok(()),
            Ok(IpcResponse::Error(msg)) => Err(anyhow::anyhow!("Error shutting down: {}", msg)),
            Ok(_) => Err(anyhow::anyhow!("Unexpected response")),
            Err(e) => Err(e),
        }
    }
}

/// Handle a client connection
fn handle_client(
    mut stream: UnixStream,
    message_handler: Arc<dyn Fn(IpcMessage) -> IpcResponse + Send + Sync>,
) -> Result<()> {
    // Read message size
    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf).context("Failed to read message size")?;
    let message_size = u32::from_le_bytes(size_buf) as usize;

    // Read message
    let mut message_buf = vec![0u8; message_size];
    stream.read_exact(&mut message_buf).context("Failed to read message")?;

    // Deserialize message
    let message: IpcMessage =
        bincode::deserialize(&message_buf).context("Failed to deserialize message")?;

    debug!("Received message: {:?}", message);

    // Process message
    let response = message_handler(message);

    // Serialize response
    let response_bytes = bincode::serialize(&response).context("Failed to serialize response")?;

    // Send response size
    let size = response_bytes.len() as u32;
    stream.write_all(&size.to_le_bytes()).context("Failed to send response size")?;

    // Send response
    stream.write_all(&response_bytes).context("Failed to send response")?;

    Ok(())
}
