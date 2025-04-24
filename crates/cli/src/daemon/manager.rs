use log::{debug, error, info, warn};
#[cfg(unix)]
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
    env,
};
#[cfg(unix)]
use nix::Error as NixError; // Import NixError explicitly needed for ESRCH comparison

// Adjust imports based on the new structure
use crate::{
    common::{
        errors::{CliError, CliResult},
        utils::check_process_running, // Assuming check_process_running moved to common::utils
    },
    config::{DaemonConfig, get_config_path}, // Assuming get_config_path is in config module
    ipc::{DaemonMessage, IpcMessage, IpcTransport, IpcConnection, DEFAULT_TIMEOUT}, // Use constants from ipc
};
use super::{DaemonControl, DaemonStatus, HealthInfo}; // Local imports from daemon module

// Constants
const MAX_CONNECT_RETRIES: usize = 3;
const RETRY_DELAY_MS: u64 = 500;
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

pub struct DaemonManager {
    config: DaemonConfig,
    // Store the transport layer; Unix specific for now
    #[cfg(unix)]
    transport: crate::ipc::UnixSocketTransport,
    // #[cfg(windows)]
    // transport: crate::ipc::NamedPipeTransport, // Placeholder for Windows
}

impl DaemonManager {
    pub fn new(config: DaemonConfig) -> CliResult<Self> {
        // Validate configuration paths
        let socket_path_str = config.socket_path.as_ref()
            .ok_or_else(|| CliError::Config("Missing socket_path in daemon configuration".to_string()))?;
        let _pid_file_str = config.pid_file.as_ref()
            .ok_or_else(|| CliError::Config("Missing pid_file in daemon configuration".to_string()))?;

        let socket_path = PathBuf::from(socket_path_str);

        // Initialize the correct transport based on the OS
        #[cfg(unix)]
        let transport = crate::ipc::UnixSocketTransport::new(&socket_path);
        #[cfg(windows)]
        {
             // Placeholder: Initialize Windows transport when available
             // let transport = crate::ipc::NamedPipeTransport::new(&socket_path)?; // Example
             return Err(CliError::InvalidOperation("Windows daemon transport not yet implemented".to_string()));
        }


        Ok(Self { config, transport })
    }

    pub fn socket_path(&self) -> PathBuf {
        PathBuf::from(self.config.socket_path.as_ref().unwrap()) // Unwrap is safe due to check in new()
    }

    pub fn pid_file_path(&self) -> PathBuf {
        PathBuf::from(self.config.pid_file.as_ref().unwrap()) // Unwrap is safe due to check in new()
    }

    /// Checks if the daemon process appears to be running (PID exists and process active).
    pub fn is_process_running(&self) -> CliResult<bool> {
        let pid_file = self.pid_file_path();
        if !pid_file.exists() {
            return Ok(false);
        }
        match self.read_pid() {
            Ok(pid) => Ok(check_process_running(pid)),
            Err(_) => Ok(false), // Couldn't read PID, assume not running
        }
    }

    pub fn read_pid(&self) -> CliResult<u32> {
        let pid_file = self.pid_file_path();
        if !pid_file.exists() {
            return Err(CliError::Daemon("Daemon PID file not found".to_string()));
        }
        let mut file = File::open(&pid_file).map_err(CliError::Io)?;
        let mut pid_str = String::new();
        file.read_to_string(&mut pid_str).map_err(CliError::Io)?;
        pid_str.trim().parse::<u32>().map_err(|_| {
            CliError::Daemon(format!("Invalid PID in PID file: {}", pid_file.display()))
        })
    }

    fn write_pid(&self, pid: u32) -> CliResult<()> {
        let pid_file = self.pid_file_path();
        if let Some(parent) = pid_file.parent() {
            fs::create_dir_all(parent).map_err(CliError::Io)?;
        }
        let mut file = File::create(&pid_file).map_err(CliError::Io)?;
        file.write_all(pid.to_string().as_bytes()).map_err(CliError::Io)?;
        info!("Wrote PID {} to file {}", pid, pid_file.display());
        Ok(())
    }

    /// Sends a message via IPC, handles connection and retries.
    fn send_message(&self, message: DaemonMessage) -> CliResult<DaemonMessage> {
        let ipc_msg = IpcMessage::new(message)?;
        let mut last_error: Option<CliError> = None;

        for attempt in 1..=MAX_CONNECT_RETRIES {
             match self.transport.connect() {
                 Ok(mut connection) => {
                      connection.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
                      connection.set_write_timeout(Some(DEFAULT_TIMEOUT))?;

                      if let Err(e) = connection.send_message(&ipc_msg) {
                          warn!("Failed to send message on attempt {}: {}", attempt, e);
                          last_error = Some(e);
                          thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
                          continue; // Retry connection and send
                      }

                      match connection.receive_message() {
                          Ok(response) => return Ok(response.payload), // Success
                          Err(e) => {
                              warn!("Failed to receive response on attempt {}: {}", attempt, e);
                              last_error = Some(e);
                              thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
                              continue; // Retry connection and send/receive
                          }
                      }
                 }
                 Err(e) => {
                      warn!("Failed to connect on attempt {}: {}", attempt, e);
                      last_error = Some(e);
                      thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
                 }
            }
        }
        Err(last_error.unwrap_or_else(|| CliError::Ipc("Failed to communicate with daemon after multiple attempts".to_string())))
    }

    fn cleanup_files(&self) -> CliResult<()> {
        let socket_path = self.socket_path();
        if socket_path.exists() {
            // Use map_err for cleaner error handling and context
            fs::remove_file(&socket_path).map_err(|e| {
                warn!("Failed to remove socket file {}: {}", socket_path.display(), e);
                CliError::Io(io::Error::new(e.kind(), format!("Failed to remove socket file '{}': {}", socket_path.display(), e)))
            })?;
        }
        let pid_file = self.pid_file_path();
        if pid_file.exists() {
             fs::remove_file(&pid_file).map_err(|e| {
                 warn!("Failed to remove PID file {}: {}", pid_file.display(), e);
                  CliError::Io(io::Error::new(e.kind(), format!("Failed to remove PID file '{}': {}", pid_file.display(), e)))
             })?;
        }
        Ok(())
    }

    fn force_kill_daemon(&self, pid: u32) -> CliResult<()> {
        warn!("Forcefully terminating daemon process (PID: {})", pid);
        #[cfg(unix)]
        {
             match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                 Ok(_) => {
                      // Wait briefly for termination
                      thread::sleep(Duration::from_millis(500));
                      if check_process_running(pid) {
                          warn!("Process did not respond to SIGTERM, sending SIGKILL");
                          match kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                              Ok(_) => (),
                              Err(e) => warn!("Failed to send SIGKILL: {}", e), // Log but don't error out
                          }
                      }
                 }
                 Err(e) => {
                      // Process might already be gone (ESRCH: No such process)
                      if e != NixError::ESRCH {
                           warn!("Failed to send SIGTERM: {}", e); // Log other errors
                      } else {
                           info!("Process {} already terminated before SIGTERM.", pid);
                      }
                 }
            }
        }
        #[cfg(windows)]
        {
            // Add Windows force kill implementation (e.g., TerminateProcess)
            warn!("Force kill not implemented for Windows yet");
            // To make it compile, return Ok(()) or an unimplemented error
             return Err(CliError::InvalidOperation("Force kill not implemented for Windows.".to_string()));

        }
        Ok(())
    }
}

impl DaemonControl for DaemonManager {
    fn start(&self, log_level: Option<&str>) -> CliResult<()> {
         // Check status first using the status() method which handles IPC check
         match self.status() {
            Ok(status) if status.running => {
                 let pid = status.pid.unwrap_or(0); // PID should be available if running
                 return Err(CliError::Daemon(format!("Daemon is already running (PID: {})", pid)));
            }
            Ok(_) => {} // Not running, proceed
            Err(e) => {
                 warn!("Could not reliably determine daemon status before start: {}. Proceeding with start attempt.", e);
                 // Attempt cleanup just in case, ignore error if it happens
                 let _ = self.cleanup_files();
            }
         }

        // If we reached here, daemon is likely not running, or status check failed.
        // Ensure cleanup happened if status check failed.
        if let Err(e) = self.cleanup_files() {
            warn!("Pre-start cleanup failed: {}. Proceeding cautiously.", e);
            // Depending on the error, might want to return Err here.
            // If cleanup fails, starting might lead to issues.
        }

        // Create directories
        if let Some(parent) = self.socket_path().parent() {
            fs::create_dir_all(parent).map_err(CliError::Io)?;
        }
        if let Some(parent) = self.pid_file_path().parent() {
            fs::create_dir_all(parent).map_err(CliError::Io)?;
        }

        // Find the workspace-daemon binary relative to the current executable
        let current_exe = env::current_exe()
            .map_err(|e| CliError::Daemon(format!("Failed to get current executable path: {}", e)))?;
        // Assuming daemon binary is named 'workspace-daemon' and located alongside the main CLI binary
        let daemon_bin = current_exe.with_file_name("workspace-daemon");

        if !daemon_bin.exists() {
             // Maybe check standard install locations too? For now, require it to be colocated.
            return Err(CliError::Daemon(format!("Daemon binary not found at {}", daemon_bin.display())));
        }

        let log_path = self.pid_file_path().with_file_name("daemon.log");
        info!("Daemon output will be captured to: {}", log_path.display());
        // Ensure log file can be created/opened
        let log_file = File::create(&log_path).map_err(CliError::Io)?;

        // Build command
        let mut cmd = Command::new(&daemon_bin);
        cmd.arg("run") // Assuming 'run' is the foreground command for the daemon binary
            .env("RUST_LOG", log_level.unwrap_or("info"))
            .env("WORKSPACE_SOCKET_PATH", self.socket_path().to_string_lossy().to_string())
            .env("WORKSPACE_PID_FILE", self.pid_file_path().to_string_lossy().to_string())
            .env("RUST_BACKTRACE", "1")
            .stdin(Stdio::null())
            // Use try_clone() for File handles passed to stdio
            .stdout(Stdio::from(log_file.try_clone().map_err(CliError::Io)?))
            .stderr(Stdio::from(log_file));

        // Pass config path if available and canonicalized
         match get_config_path().canonicalize() {
            Ok(config_path) => {
                 cmd.env("WORKSPACE_CONFIG_PATH", config_path.to_string_lossy().to_string());
            }
            Err(e) => {
                 log::warn!("Could not canonicalize config path {}: {}", get_config_path().display(), e);
                 // Decide if this is critical. Maybe the daemon can find it anyway?
            }
         }

        // Spawn the process
        info!("Starting daemon process: {:?}", cmd);
        let child = cmd.spawn().map_err(|e| CliError::Daemon(format!("Failed to spawn daemon process: {}", e)))?;
        let pid = child.id();
        info!("Daemon process spawned with PID {}", pid);

        // Write PID immediately after successful spawn
        self.write_pid(pid)?;

        // Wait for daemon to initialize and become responsive via IPC
        let start_time = std::time::Instant::now();
        info!("Waiting for daemon to initialize (timeout: {}s)...", DAEMON_STARTUP_TIMEOUT.as_secs());

        while start_time.elapsed() < DAEMON_STARTUP_TIMEOUT {
             // Check if process died unexpectedly
             if !check_process_running(pid) {
                 error!("Daemon process terminated unexpectedly during startup.");
                 // Try reading log file for clues
                 match fs::read_to_string(&log_path) {
                    Ok(log_content) if !log_content.is_empty() => error!("Daemon log output:\n{}", log_content),
                    Ok(_) => error!("Daemon log file is empty."),
                    Err(e) => error!("Could not read daemon log file {}: {}", log_path.display(), e),
                 }
                 self.cleanup_files()?; // Clean up potentially inconsistent state
                 return Err(CliError::Daemon("Daemon process terminated unexpectedly during startup".to_string()));
             }

             // Check if socket is connectable using the transport layer
             match self.transport.connect() {
                  Ok(_) => {
                       // Additionally, send a simple ping or status request to confirm responsiveness
                       match self.send_message(DaemonMessage::StatusRequest) {
                            Ok(_) => {
                                info!("Daemon started successfully and is responsive.");
                                return Ok(()); // Daemon started successfully
                            }
                            Err(e) => {
                                debug!("Daemon socket connected, but status request failed: {}. Waiting...", e);
                                // Continue waiting, maybe it's still initializing IPC handler
                            }
                       }
                  }
                  Err(e) => {
                      debug!("Waiting for daemon socket connection... ({})", e);
                  }
             }
             thread::sleep(Duration::from_millis(RETRY_DELAY_MS)); // Wait before next check
        }

        // Timeout occurred
        warn!("Daemon startup timed out after {} seconds.", DAEMON_STARTUP_TIMEOUT.as_secs());
        // Try reading log file for clues
         match fs::read_to_string(&log_path) {
            Ok(log_content) if !log_content.is_empty() => warn!("Daemon log output:\n{}", log_content),
            Ok(_) => warn!("Daemon log file is empty."),
            Err(e) => warn!("Could not read daemon log file {}: {}", log_path.display(), e),
         }


        // Check if process is still running but unresponsive
        if check_process_running(pid) {
             warn!("Daemon process (PID {}) is running but did not become responsive via IPC.", pid);
             // Don't clean up yet, user might want to inspect or kill manually.
             // Consider adding a command to force cleanup later.
             Err(CliError::Timeout("Daemon process started but did not respond within timeout.".to_string()))
        } else {
             error!("Daemon process terminated after timeout without becoming responsive.");
             self.cleanup_files()?; // Clean up state
             Err(CliError::Daemon("Daemon process terminated after timeout.".to_string()))
        }
    }

    fn stop(&self) -> CliResult<()> {
        let status = match self.status() {
             Ok(s) => s,
             Err(e) => {
                 warn!("Could not determine daemon status before stop: {}", e);
                 // Attempt to read PID file directly if status failed
                 if let Ok(pid) = self.read_pid() {
                     if check_process_running(pid) {
                         warn!("Process {} seems to be running despite status check failure. Attempting force kill.", pid);
                         self.force_kill_daemon(pid)?;
                         self.cleanup_files()?;
                         return Ok(());
                     }
                 }
                 // If PID read fails or process not running, assume it's stopped.
                 info!("Assuming daemon is stopped due to status check failure.");
                 // Attempt cleanup, ignore error
                 let _ = self.cleanup_files();
                 return Ok(());
             }
        };

        if !status.running {
            info!("Daemon is not running.");
            // Ensure cleanup if process isn't actually running
            if !self.is_process_running()? {
                 let _ = self.cleanup_files();
            }
            return Ok(());
        }

        let pid = status.pid.ok_or_else(|| CliError::Daemon("Cannot stop daemon: PID unknown".to_string()))?;

        info!("Sending shutdown request to daemon (PID: {})...", pid);
        match self.send_message(DaemonMessage::ShutdownRequest) {
             Ok(DaemonMessage::ShutdownResponse { success: true }) => {
                 info!("Daemon acknowledged shutdown request.");
                 // Wait for process to terminate gracefully
                 let start_time = std::time::Instant::now();
                 while start_time.elapsed() < DEFAULT_TIMEOUT { // Use IPC timeout as grace period
                      if !check_process_running(pid) {
                           info!("Daemon process terminated gracefully.");
                           self.cleanup_files()?;
                           return Ok(());
                      }
                      thread::sleep(Duration::from_millis(100));
                 }
                 warn!("Daemon did not terminate within grace period after shutdown request.");
                 self.force_kill_daemon(pid)?; // Force kill if it didn't stop
             }
             Ok(_) => {
                  warn!("Received unexpected response to shutdown request. Attempting force kill.");
                  self.force_kill_daemon(pid)?;
             }
             Err(e) => {
                  warn!("Failed to send shutdown request: {}. Attempting force kill.", e);
                  // Check if the error indicates the daemon already stopped
                  if let CliError::Ipc(_) | CliError::Io(_) = e {
                       if !check_process_running(pid) {
                            info!("Daemon seems to have stopped before force kill attempt.");
                       } else {
                            self.force_kill_daemon(pid)?;
                       }
                  } else {
                       self.force_kill_daemon(pid)?;
                  }
             }
        }

        // Cleanup files after attempting stop/kill
        self.cleanup_files()?;
        info!("Daemon stopped.");
        Ok(())
    }

    fn restart(&self, log_level: Option<&str>) -> CliResult<()> {
        info!("Restarting daemon...");
        match self.stop() {
             Ok(()) => info!("Daemon stopped successfully."),
             // If stop() indicates it wasn't running, that's fine.
             Err(CliError::Daemon(msg)) if msg.contains("not running") || msg.contains("PID file not found") => {
                 info!("Daemon was not running, starting it now.");
             }
             Err(e) => {
                 // Log other errors during stop but attempt to start anyway
                 warn!("Failed to stop daemon during restart: {}. Attempting to start anyway.", e);
                 // Attempt to clean up potentially stale files before starting
                 if let Err(cleanup_err) = self.cleanup_files() {
                     warn!("Cleanup before restart failed: {}. Proceeding with caution.", cleanup_err);
                 }
             }
        }
        // Add a small delay before starting again, especially after potential force kill
        thread::sleep(Duration::from_millis(500));
        self.start(log_level)
    }

    fn status(&self) -> CliResult<DaemonStatus> {
        // More robust status check:
        // 1. Check PID file.
        // 2. If exists, read PID.
        // 3. If readable, check if process with PID is running.
        // 4. If running, try IPC status request.

        let pid_file = self.pid_file_path();
        if !pid_file.exists() {
             // No PID file, definitely not running managed by us.
             return Ok(DaemonStatus { running: false, pid: None, socket_path: None, uptime_seconds: None, monitored_repos: None, health: None });
        }

        let pid = match self.read_pid() {
            Ok(p) => p,
            Err(e) => { // PID file exists but unreadable or invalid content
                warn!("PID file {} exists but is unreadable/invalid: {}. Assuming daemon is not running correctly.", pid_file.display(), e);
                // Consider cleaning up the bad PID file? Risky if another process created it.
                // For now, report as not running.
                 let _ = self.cleanup_files(); // Attempt cleanup of potentially bad state
                return Ok(DaemonStatus { running: false, pid: None, socket_path: None, uptime_seconds: None, monitored_repos: None, health: None });
            }
        };

        if !check_process_running(pid) {
             // Process is not running, but files might exist. Clean them up.
             warn!("Daemon process (PID {} from file) not running, but PID file exists. Cleaning up stale files.", pid);
             let _ = self.cleanup_files(); // Attempt cleanup, ignore error
             return Ok(DaemonStatus { running: false, pid: None, socket_path: None, uptime_seconds: None, monitored_repos: None, health: None });
        }

        // Process seems to be running, try IPC status request for confirmation and details
        match self.send_message(DaemonMessage::StatusRequest) {
            Ok(DaemonMessage::StatusResponse { running, pid: response_pid, uptime_seconds, monitored_repos }) => {
                 if !running {
                     // This is inconsistent state. Process running but IPC says not running?
                     warn!("Inconsistent State: Process PID {} is running, but daemon IPC reported running=false.", pid);
                     // Report based on process check, but log inconsistency.
                 }
                 // Verify PID matches if daemon provides it
                 if response_pid.is_some() && response_pid != Some(pid) {
                     warn!("PID mismatch: PID file has {}, daemon responded with {:?}. PID file might be stale.", pid, response_pid);
                     // Trust the process check PID for now.
                 }

                 Ok(DaemonStatus {
                     running: true, // Based on process check and IPC response received
                     pid: Some(pid), // Use PID from file as the primary identifier
                     socket_path: Some(self.socket_path()),
                     uptime_seconds,
                     monitored_repos,
                     health: None, // Health info would require another message or be part of status response
                 })
            }
            Err(e) => {
                 warn!("Daemon process (PID {}) is running but failed to respond to status request: {}", pid, e);
                 // Report as running based on process check, but indicate communication failure.
                 Ok(DaemonStatus {
                     running: true,
                     pid: Some(pid),
                     socket_path: Some(self.socket_path()),
                     uptime_seconds: None,
                     monitored_repos: None,
                     health: None, // Indicate health unknown / communication failed
                 })
            }
            // Handle unexpected response types
            Ok(other_msg) => {
                 error!("Received unexpected IPC response type for StatusRequest: {:?}", other_msg);
                 Err(CliError::Ipc("Received unexpected response type for StatusRequest".to_string()))
            }
        }
    }
}

impl DaemonControl for DaemonManager {
    fn start(&self, log_level: Option<&str>) -> CliResult<()> {
        if self.status()?.running {
            let pid = self.read_pid()?; // Should succeed if status() was true
            return Err(CliError::Daemon(format!("Daemon is already running (PID: {})", pid)));
        }

        self.cleanup_files()?; // Clean up any stale files before starting

        // Create directories
        if let Some(parent) = self.socket_path().parent() {
            fs::create_dir_all(parent).map_err(|e| CliError::Io(e))?;
        }
        if let Some(parent) = self.pid_file_path().parent() {
            fs::create_dir_all(parent).map_err(|e| CliError::Io(e))?;
        }

        // Find the workspace-daemon binary relative to the current executable
        let current_exe = env::current_exe().map_err(|e| {
            CliError::Daemon(format!("Failed to get current executable path: {}", e))
        })?;
        let daemon_bin = current_exe.with_file_name("workspace-daemon");

        if !daemon_bin.exists() {
            return Err(CliError::Daemon(format!(
                "Daemon binary not found at {}",
                daemon_bin.display()
            )));
        }

        let log_path = self.pid_file_path().with_file_name("daemon.log");
        info!("Daemon output will be captured to: {}", log_path.display());
        let log_file = File::create(&log_path).map_err(|e| CliError::Io(e))?;

        // Build command
        let mut cmd = Command::new(&daemon_bin);
        cmd.arg("run") // Assuming 'run' is the foreground command for the daemon binary
            .env("RUST_LOG", log_level.unwrap_or("info"))
            .env("WORKSPACE_SOCKET_PATH", self.socket_path().to_string_lossy().to_string())
            .env("WORKSPACE_PID_FILE", self.pid_file_path().to_string_lossy().to_string())
            .env("RUST_BACKTRACE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::from(log_file.try_clone().map_err(|e| CliError::Io(e))?))
            .stderr(Stdio::from(log_file));

        // Pass config path if available
        if let Ok(config_path) = get_config_path().canonicalize() {
            cmd.env("WORKSPACE_CONFIG_PATH", config_path.to_string_lossy().to_string());
        }

        // Spawn the process
        info!("Starting daemon process: {:?}", cmd);
        let child = cmd
            .spawn()
            .map_err(|e| CliError::Daemon(format!("Failed to spawn daemon process: {}", e)))?;
        let pid = child.id();
        info!("Daemon process spawned with PID {}", pid);

        // Write PID immediately
        self.write_pid(pid)?;

        // Wait for daemon to initialize
        let start_time = std::time::Instant::now();
        info!(
            "Waiting for daemon to initialize (timeout: {}s)...",
            DAEMON_STARTUP_TIMEOUT.as_secs()
        );

        while start_time.elapsed() < DAEMON_STARTUP_TIMEOUT {
            // Check if process died unexpectedly
            if !check_process_running(pid) {
                error!("Daemon process terminated unexpectedly during startup.");
                // Try reading log file for clues
                if let Ok(log_content) = fs::read_to_string(&log_path) {
                    error!("Daemon log output:\n{}", log_content);
                }
                self.cleanup_files()?; // Clean up potentially inconsistent state
                return Err(CliError::Daemon("Daemon process terminated unexpectedly".to_string()));
            }

            // Check if socket is connectable
            match self.transport.connect() {
                Ok(_) => {
                    info!("Successfully connected to daemon socket.");
                    return Ok(()); // Daemon started successfully
                }
                Err(e) => {
                    debug!("Waiting for daemon socket connection... ({})", e);
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }

        // Timeout occurred
        warn!("Daemon startup timed out after {} seconds.", DAEMON_STARTUP_TIMEOUT.as_secs());
        // Try reading log file for clues
        if let Ok(log_content) = fs::read_to_string(&log_path) {
            warn!("Daemon log output:\n{}", log_content);
        }

        // Check if process is still running but unresponsive
        if check_process_running(pid) {
            warn!("Daemon process (PID {}) is running but did not become responsive.", pid);
            // Don't clean up yet, maybe it will recover? Or user might want to inspect.
            Err(CliError::Timeout(
                "Daemon process started but did not respond within timeout.".to_string(),
            ))
        } else {
            error!("Daemon process terminated after timeout.");
            self.cleanup_files()?;
            Err(CliError::Daemon("Daemon process terminated after timeout.".to_string()))
        }
    }

    fn stop(&self) -> CliResult<()> {
        let status = self.status()?; // Check status first
        if !status.running {
            info!("Daemon is not running.");
            // Clean up potentially stale files if process is definitely not running
            if !self.is_process_running()? {
                self.cleanup_files()?;
            }
            return Ok(());
        }

        let pid = status
            .pid
            .ok_or_else(|| CliError::Daemon("Cannot stop daemon: PID unknown".to_string()))?;

        info!("Sending shutdown request to daemon (PID: {})...", pid);
        match self.send_message(DaemonMessage::ShutdownRequest) {
            Ok(DaemonMessage::ShutdownResponse { success: true }) => {
                info!("Daemon acknowledged shutdown request.");
                // Wait for process to terminate gracefully
                let start_time = std::time::Instant::now();
                while start_time.elapsed() < DEFAULT_TIMEOUT {
                    if !check_process_running(pid) {
                        info!("Daemon process terminated gracefully.");
                        self.cleanup_files()?;
                        return Ok(());
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                warn!("Daemon did not terminate within grace period after shutdown request.");
                self.force_kill_daemon(pid)?; // Force kill if it didn't stop
            }
            Ok(_) => {
                warn!("Received unexpected response to shutdown request.");
                self.force_kill_daemon(pid)?; // Force kill
            }
            Err(e) => {
                warn!("Failed to send shutdown request: {}. Attempting force kill.", e);
                self.force_kill_daemon(pid)?; // Force kill
            }
        }

        self.cleanup_files()?; // Ensure files are cleaned up after force kill attempt
        info!("Daemon stopped.");
        Ok(())
    }

    fn restart(&self, log_level: Option<&str>) -> CliResult<()> {
        info!("Restarting daemon...");
        match self.stop() {
            Ok(()) => info!("Daemon stopped successfully."),
            Err(CliError::Daemon(msg)) if msg.contains("not running") => {
                info!("Daemon was not running, starting it now.");
            }
            Err(e) => {
                warn!("Failed to stop daemon during restart: {}. Attempting to start anyway.", e);
                // Attempt to clean up potentially stale files before starting
                if let Err(cleanup_err) = self.cleanup_files() {
                    warn!("Failed during cleanup before restart: {}", cleanup_err);
                }
            }
        }
        // Add a small delay before starting again
        thread::sleep(Duration::from_millis(500));
        self.start(log_level)
    }

    fn status(&self) -> CliResult<DaemonStatus> {
        // First, check basic process running state
        let pid_file = self.pid_file_path();
        if !pid_file.exists() {
            return Ok(DaemonStatus {
                running: false,
                pid: None,
                socket_path: None,
                uptime_seconds: None,
                monitored_repos: None,
                health: None,
            });
        }

        let pid = match self.read_pid() {
            Ok(p) => p,
            Err(_) => {
                // PID file exists but unreadable
                return Ok(DaemonStatus {
                    running: false,
                    pid: None,
                    socket_path: None,
                    uptime_seconds: None,
                    monitored_repos: None,
                    health: None,
                });
            }
        };

        if !check_process_running(pid) {
            // Process is not running, but files might exist. Clean them up.
            warn!("Daemon process (PID {}) not running, but PID file exists. Cleaning up.", pid);
            self.cleanup_files()?;
            return Ok(DaemonStatus {
                running: false,
                pid: None,
                socket_path: None,
                uptime_seconds: None,
                monitored_repos: None,
                health: None,
            });
        }

        // Process seems to be running, try IPC status request
        match self.send_message(DaemonMessage::StatusRequest) {
            Ok(DaemonMessage::StatusResponse {
                running,
                pid: response_pid,
                uptime_seconds,
                monitored_repos,
            }) => {
                // Should always be running if we get a response
                if !running {
                    warn!("Daemon responded with running=false, which is unexpected.");
                }
                // Verify PID matches
                if response_pid.is_some() && response_pid != Some(pid) {
                    warn!("PID mismatch: PID file has {}, daemon responded with {:?}. PID file might be stale.", pid, response_pid);
                    // Trust the response PID more? Or flag as inconsistent?
                }

                Ok(DaemonStatus {
                    running: true,
                    pid: Some(pid), // Use PID from file as the primary identifier
                    socket_path: Some(self.socket_path()),
                    uptime_seconds,
                    monitored_repos,
                    health: None, // Health info would require another message or be part of status
                })
            }
            Err(e) => {
                warn!("Daemon process (PID {}) is running but failed to respond to status request: {}", pid, e);
                // Return running=true, but without details from IPC
                Ok(DaemonStatus {
                    running: true,
                    pid: Some(pid),
                    socket_path: Some(self.socket_path()),
                    uptime_seconds: None,
                    monitored_repos: None,
                    health: None,
                })
            }
            // Handle unexpected response types
            Ok(_) => Err(CliError::Ipc(
                "Received unexpected response type for StatusRequest".to_string(),
            )),
        }
    }
}
