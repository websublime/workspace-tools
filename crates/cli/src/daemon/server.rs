
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use log::{debug, error, info, warn};

use crate::common::errors::{CliError, CliResult};
use crate::config::DaemonConfig;
use crate::ipc::{DaemonMessage, IpcMessage, IpcTransport};
use super::health::{HealthInfo, HealthStatus};
use super::handlers::CommandHandler;

pub struct DaemonServer {
    socket_path: PathBuf,
    config_path: Option<PathBuf>,
    running: Arc<AtomicBool>,
    start_time: Instant,
    command_handler: CommandHandler,
}

impl DaemonServer {
    pub fn new(socket_path: PathBuf, config_path: Option<PathBuf>) -> Self {
        Self {
            socket_path,
            config_path,
            running: Arc::new(AtomicBool::new(true)),
            start_time: Instant::now(),
            command_handler: CommandHandler::new(),
        }
    }

    pub fn running(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    pub fn run(&mut self) -> CliResult<()> {
        info!("Starting daemon server at {}", self.socket_path.display());

        // Create transport
        #[cfg(unix)]
        let transport = crate::ipc::UnixSocketTransport::new(&self.socket_path);
        
        #[cfg(windows)]
        let transport = crate::ipc::NamedPipeTransport::new(&self.socket_path);

        // Create listener
        let mut listener = transport.bind()?;
        listener.set_nonblocking(true)?;

        info!("Daemon server initialized, entering main loop");

        while self.running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok(mut connection) => {
                    debug!("Accepted new client connection");
                    
                    // Handle client connection
                    match self.handle_client(&mut *connection) {
                        Ok(()) => debug!("Client connection handled successfully"),
                        Err(e) => warn!("Error handling client connection: {}", e),
                    }
                }
                Err(e) => {
                    if let CliError::Io(io_err) = &e {
                        if io_err.kind() == std::io::ErrorKind::WouldBlock {
                            // No connection available, sleep and continue
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                    }
                    error!("Error accepting connection: {}", e);
                    return Err(e);
                }
            }
        }

        info!("Daemon server shutting down");
        Ok(())
    }

    fn handle_client(&mut self, connection: &mut dyn crate::ipc::IpcConnection) -> CliResult<()> {
        // Set timeouts
        connection.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
        connection.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

        // Receive message
        let message = connection.receive_message()?;

        // Process message
        let response = match message.payload {
            DaemonMessage::StatusRequest => {
                let health = HealthInfo::new(
                    self.start_time.elapsed(),
                    self.command_handler.repository_count(),
                );

                DaemonMessage::StatusResponse {
                    running: true,
                    pid: Some(std::process::id()),
                    uptime_seconds: Some(health.uptime_seconds),
                    monitored_repos: Some(health.repository_count),
                }
            }
            DaemonMessage::ShutdownRequest => {
                info!("Received shutdown request");
                self.running.store(false, Ordering::SeqCst);
                DaemonMessage::ShutdownResponse { success: true }
            }
            message => self.command_handler.handle_message(message)?,
        };

        // Send response
        let response = IpcMessage::new(response)?;
        connection.send_message(&response)?;

        Ok(())
    }
}

