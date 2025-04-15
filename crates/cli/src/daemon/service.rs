//! Main daemon service implementation

use anyhow::{anyhow, Context, Result};
use log::{error, info};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use crate::common::config::Config;
use crate::common::paths;
use crate::daemon::event::{Event, EventType};
use crate::daemon::ipc::{IpcMessage, IpcResponse, IpcServer};
use crate::daemon::status::{DaemonStatus, RepositoryStatus};
use crate::daemon::watcher::RepositoryWatcher;

/// Maximum number of events to store
const MAX_EVENTS: usize = 1000;

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Path to the socket file
    pub socket_path: PathBuf,
    /// Path to the PID file
    pub pid_file: PathBuf,
    /// Polling interval in milliseconds
    pub polling_interval_ms: u64,
    /// Inactive polling interval in milliseconds
    pub inactive_polling_ms: u64,
}

/// Commands for the daemon service
#[derive(Debug, Clone)]
pub enum ServiceCommand {
    /// Add a repository to monitor
    AddRepository { path: PathBuf, name: Option<String> },
    /// Remove a repository from monitoring
    RemoveRepository { name: String },
    /// Get daemon status
    GetStatus,
    /// List monitored repositories
    ListRepositories,
    /// Shutdown the daemon
    Shutdown,
}

/// Main daemon service
pub struct DaemonService {
    /// Service configuration
    config: ServiceConfig,
    /// Repository watchers
    watchers: HashMap<String, RepositoryWatcher>,
    /// Event queue
    events: Arc<Mutex<VecDeque<Event>>>,
    /// IPC server
    ipc_server: Option<IpcServer>,
    /// Running flag
    running: Arc<Mutex<bool>>,
    /// Thread handle
    thread_handle: Option<JoinHandle<()>>,
    /// Start time
    start_time: SystemTime,
    /// Process ID
    pid: u32,
}

impl DaemonService {
    /// Create a new daemon service
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            watchers: HashMap::new(),
            events: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_EVENTS))),
            ipc_server: None,
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
            start_time: SystemTime::now(),
            pid: std::process::id(),
        }
    }

    /// Start the daemon service
    pub fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        info!("Starting daemon service");

        // Create PID file
        self.create_pid_file()?;

        // Create IPC server
        let events = self.events.clone();
        let _running = self.running.clone();
        let socket_path = self.config.socket_path.clone();
        let service_arc = Arc::new(Mutex::new(self.clone_inner()));

        let ipc_server = IpcServer::new(socket_path, move |message| {
            handle_ipc_message(message, service_arc.clone(), events.clone())
        });

        // Start IPC server
        ipc_server.start()?;
        self.ipc_server = Some(ipc_server);

        // Mark as running
        *self.running.lock().unwrap() = true;

        // Store references for thread
        let running = self.running.clone();
        let config = self.config.clone();
        let _events = self.events.clone();

        // Spawn main thread
        let thread_handle = thread::spawn(move || {
            info!("Daemon service thread started");

            let mut last_active_check = SystemTime::now();

            while *running.lock().unwrap() {
                // Sleep between polling
                thread::sleep(Duration::from_millis(config.polling_interval_ms));

                // Check for activity
                let now = SystemTime::now();
                let inactive_duration = Duration::from_millis(config.inactive_polling_ms);
                let _polling_duration = Duration::from_millis(config.polling_interval_ms);

                if now.duration_since(last_active_check).unwrap_or_default() > inactive_duration {
                    // Perform background tasks when idle
                    last_active_check = now;
                }
            }

            info!("Daemon service thread stopped");
        });

        // Store thread handle
        self.thread_handle = Some(thread_handle);

        info!("Daemon service started");
        Ok(())
    }

    /// Stop the daemon service
    pub fn stop(&mut self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        info!("Stopping daemon service");

        // Mark as not running
        *self.running.lock().unwrap() = false;

        // Stop IPC server
        if let Some(ipc_server) = &self.ipc_server {
            ipc_server.stop();
        }

        // Stop all watchers
        for (name, watcher) in self.watchers.iter_mut() {
            if let Err(e) = watcher.stop() {
                error!("Error stopping watcher for {}: {}", name, e);
            }
        }

        // Wait for thread to finish
        if let Some(thread_handle) = self.thread_handle.take() {
            if let Err(e) = thread_handle.join() {
                error!("Error joining daemon thread: {:?}", e);
            }
        }

        // Remove PID file
        self.remove_pid_file()?;

        info!("Daemon service stopped");
        Ok(())
    }

    /// Check if the daemon is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Add a repository to monitor
    pub fn add_repository<P: AsRef<Path>, S: Into<String>>(
        &mut self,
        path: P,
        name: Option<S>,
    ) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        // Canonicalize path
        let path = fs::canonicalize(&path)
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

        // Generate name if not provided
        let name = match name {
            Some(name) => name.into(),
            None => path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
                .unwrap_or_else(|| format!("repo_{}", self.watchers.len())),
        };

        // Check if repository already exists
        if self.watchers.contains_key(&name) {
            return Err(anyhow!("Repository with name '{}' already exists", name));
        }

        // Create watcher
        let events = self.events.clone();
        let mut watcher = RepositoryWatcher::new(&path, &name).with_callback(move |event| {
            // Add event to queue
            let mut events = events.lock().unwrap();
            events.push_back(event);

            // Trim queue if needed
            while events.len() > MAX_EVENTS {
                events.pop_front();
            }
        });

        // Start watcher
        watcher.start()?;

        // Add to watchers
        self.watchers.insert(name.clone(), watcher);

        info!("Added repository: {} at {}", name, path.display());

        // Add event
        let event = Event::new(EventType::RepositoryAdded { name: name.clone(), path });

        let mut events = self.events.lock().unwrap();
        events.push_back(event);

        Ok(())
    }

    /// Remove a repository from monitoring
    pub fn remove_repository(&mut self, name: &str) -> Result<()> {
        // Check if repository exists
        if let Some(mut watcher) = self.watchers.remove(name) {
            // Stop watcher
            watcher.stop()?;

            info!("Removed repository: {}", name);

            // Add event
            let event = Event::new(EventType::RepositoryRemoved { name: name.to_string() });

            let mut events = self.events.lock().unwrap();
            events.push_back(event);

            Ok(())
        } else {
            Err(anyhow!("Repository '{}' not found", name))
        }
    }

    /// List monitored repositories
    pub fn list_repositories(&self) -> Vec<(String, PathBuf)> {
        self.watchers.iter().map(|(name, watcher)| (name.clone(), watcher.path().clone())).collect()
    }

    /// Get repository status
    pub fn repository_status(&self, name: &str) -> Option<RepositoryStatus> {
        self.watchers.get(name).map(|watcher| {
            RepositoryStatus {
                name: name.to_string(),
                path: watcher.path().clone(),
                branch: None,      // TODO: Get from Git
                last_commit: None, // TODO: Get from Git
                last_checked: watcher.last_event_time(),
                pending_changes: 0, // TODO: Get from Git
                active: watcher.is_running(),
                repo_type: "git".to_string(), // TODO: Detect repo type
            }
        })
    }

    /// Get daemon status
    pub fn status(&self) -> DaemonStatus {
        let uptime = SystemTime::now().duration_since(self.start_time).map(|d| d.as_secs()).ok();

        let repositories =
            self.watchers.keys().filter_map(|name| self.repository_status(name)).collect();

        DaemonStatus {
            running: self.is_running(),
            pid: Some(self.pid),
            uptime,
            repository_count: self.watchers.len(),
            repositories,
            started_at: Some(self.start_time),
            memory_usage: None, // TODO: Get memory usage
            cpu_usage: None,    // TODO: Get CPU usage
            socket_path: Some(self.config.socket_path.clone()),
        }
    }

    /// Create PID file
    fn create_pid_file(&self) -> Result<()> {
        let pid = std::process::id().to_string();

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config.pid_file.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create PID file directory: {}", parent.display())
            })?;
        }

        // Write PID file
        fs::write(&self.config.pid_file, pid).with_context(|| {
            format!("Failed to write PID file: {}", self.config.pid_file.display())
        })?;

        Ok(())
    }

    /// Remove PID file
    fn remove_pid_file(&self) -> Result<()> {
        if self.config.pid_file.exists() {
            fs::remove_file(&self.config.pid_file).with_context(|| {
                format!("Failed to remove PID file: {}", self.config.pid_file.display())
            })?;
        }

        Ok(())
    }

    /// Check if a daemon is already running
    pub fn is_another_daemon_running() -> bool {
        match paths::get_default_pid_path() {
            Ok(pid_path) => {
                let pid_path = paths::expand_path(&pid_path).unwrap_or_default();
                if !pid_path.exists() {
                    return false;
                }

                // Read PID file
                let pid = match fs::read_to_string(&pid_path) {
                    Ok(pid) => pid.trim().parse::<u32>().ok(),
                    Err(_) => return false,
                };

                // Check if process is running
                if let Some(pid) = pid {
                    #[cfg(unix)]
                    {
                        unsafe {
                            // Send signal 0 to process to check if it exists
                            libc::kill(pid as i32, 0) == 0
                        }
                    }

                    #[cfg(windows)]
                    {
                        // Try to open process to check if it exists
                        use std::os::windows::process::CommandExt;
                        use std::process::Command;

                        Command::new("cmd")
                            .args(&[
                                "/c",
                                &format!("tasklist /fi \"PID eq {}\" | find \"{}\"", pid, pid),
                            ])
                            .creation_flags(0x08000000) // CREATE_NO_WINDOW
                            .output()
                            .map(|output| output.status.success() && !output.stdout.is_empty())
                            .unwrap_or(false)
                    }

                    #[cfg(not(any(unix, windows)))]
                    {
                        false
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    /// Get daemon by process ID
    pub fn get_daemon_by_pid(_pid: u32) -> Option<Self> {
        None // TODO: Implement
    }

    /// Load from configuration
    pub fn from_config(config: &Config) -> Result<Self> {
        let socket_path = paths::expand_path(&config.daemon.socket_path)?;
        let pid_file = paths::expand_path(&config.daemon.pid_file)?;

        let service_config = ServiceConfig {
            socket_path,
            pid_file,
            polling_interval_ms: config.daemon.polling_interval_ms,
            inactive_polling_ms: config.daemon.inactive_polling_ms,
        };

        Ok(Self::new(service_config))
    }

    /// Clone internal state for IPC handler
    fn clone_inner(&self) -> DaemonServiceInner {
        DaemonServiceInner {
            watchers: self.watchers.keys().cloned().collect(),
            running: self.is_running(),
            start_time: self.start_time,
            pid: self.pid,
        }
    }
}

/// Internal state for IPC handler
#[derive(Debug, Clone)]
struct DaemonServiceInner {
    /// Names of repositories being watched
    watchers: Vec<String>,
    /// Whether the daemon is running
    running: bool,
    /// Start time
    start_time: SystemTime,
    /// Process ID
    pid: u32,
}

/// Handle IPC messages
fn handle_ipc_message(
    message: IpcMessage,
    service: Arc<Mutex<DaemonServiceInner>>,
    events: Arc<Mutex<VecDeque<Event>>>,
) -> IpcResponse {
    match message {
        IpcMessage::Ping => IpcResponse::Ok,
        IpcMessage::Status => {
            let service = service.lock().unwrap();

            let uptime =
                SystemTime::now().duration_since(service.start_time).map(|d| d.as_secs()).ok();

            let status = DaemonStatus {
                running: service.running,
                pid: Some(service.pid),
                uptime,
                repository_count: service.watchers.len(),
                repositories: Vec::new(), // TODO: Fill repositories
                started_at: Some(service.start_time),
                memory_usage: None,
                cpu_usage: None,
                socket_path: None,
            };

            IpcResponse::Status(status)
        }
        IpcMessage::AddRepository { path: _, name: _ } => {
            // This would be handled by the main service
            IpcResponse::error("Not implemented in IPC handler")
        }
        IpcMessage::RemoveRepository { name: _ } => {
            // This would be handled by the main service
            IpcResponse::error("Not implemented in IPC handler")
        }
        IpcMessage::ListRepositories => {
            let service = service.lock().unwrap();

            // In the real implementation, this would include paths
            let repositories =
                service.watchers.iter().map(|name| (name.clone(), PathBuf::new())).collect();

            IpcResponse::Repositories(repositories)
        }
        IpcMessage::GetChanges { repository: _ } => {
            // This would be handled by the main service
            IpcResponse::error("Not implemented in IPC handler")
        }
        IpcMessage::GetEvents { since: _, limit: _ } => {
            let events_guard = events.lock().unwrap();

            // Convert to vector for serialization
            let events_vec: Vec<Event> = events_guard.iter().cloned().collect();

            IpcResponse::Events(events_vec)
        }
        IpcMessage::Shutdown => {
            // This would be handled by the main service
            IpcResponse::error("Not implemented in IPC handler")
        }
    }
}

impl Drop for DaemonService {
    fn drop(&mut self) {
        if self.is_running() {
            if let Err(e) = self.stop() {
                error!("Error stopping daemon service: {}", e);
            }
        }
    }
}
