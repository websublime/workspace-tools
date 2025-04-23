use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{error, info, warn};
use std::fs::File;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;
use sublime_workspace_cli::common::config::{get_config_path, Config};
use sublime_workspace_cli::common::daemon::{
    check_process_running, DaemonError, DaemonManager, DaemonServer,
};
use sublime_workspace_cli::ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to socket file (overrides config)
    #[arg(long)]
    socket_path: Option<String>,

    /// Path to PID file (overrides config)
    #[arg(long)]
    pid_file: Option<String>,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long)]
    log_level: Option<String>,

    /// Path to configuration file
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Start,

    /// Stop the daemon
    Stop,

    /// Restart the daemon
    Restart,

    /// Check daemon status
    Status,

    /// Run the daemon in foreground mode (used internally)
    Run {
        /// Path to socket file (overrides config)
        #[arg(long)]
        socket_path: Option<String>,

        /// Path to PID file (overrides config)
        #[arg(long)]
        pid_file: Option<String>,

        /// Log level (error, warn, info, debug, trace)
        #[arg(long)]
        log_level: Option<String>,

        /// Path to configuration file
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// List repositories being monitored by the daemon
    ListRepos,

    /// Add a repository to be monitored
    AddRepo {
        /// Path to the repository
        path: String,

        /// Optional name for the repository
        #[arg(long)]
        name: Option<String>,
    },

    /// Remove a repository from monitoring
    RemoveRepo {
        /// Repository path or name
        identifier: String,
    },
    /// Run the daemon in foreground diagnostic mode
    RunDiagnostic {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    // Initialize UI
    ui::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Setup logging
    let log_level = cli.log_level.as_deref().unwrap_or("info");
    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();

    // Load configuration
    let config_path = cli.config.clone().unwrap_or_else(get_config_path);
    let config = Config::load(&config_path).unwrap_or(Config::default());

    // Override config with CLI arguments
    let socket_path = cli
        .socket_path
        .clone()
        .or_else(|| config.daemon.as_ref().and_then(|d| d.socket_path.clone()))
        .unwrap_or_else(|| {
            let path = dirs::runtime_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")))
                .join("workspace-cli")
                .join("daemon.sock");
            path.to_string_lossy().to_string()
        });

    let pid_file = cli
        .pid_file
        .clone()
        .or_else(|| config.daemon.as_ref().and_then(|d| d.pid_file.clone()))
        .unwrap_or_else(|| {
            let path = dirs::runtime_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")))
                .join("workspace-cli")
                .join("daemon.pid");
            path.to_string_lossy().to_string()
        });

    // Create a daemon config with overridden values
    let daemon_config = if let Some(daemon_config) = &config.daemon {
        sublime_workspace_cli::common::config::DaemonConfig {
            socket_path: Some(socket_path),
            pid_file: Some(pid_file),
            polling_interval_ms: daemon_config.polling_interval_ms,
            inactive_polling_ms: daemon_config.inactive_polling_ms,
            // Set log rotation values from config or use defaults
            log_max_size_bytes: daemon_config.log_max_size_bytes.or(Some(10 * 1024 * 1024)), // 10 MB default
            log_max_files: daemon_config.log_max_files.or(Some(5)), // 5 files default
            log_check_interval_ms: daemon_config.log_check_interval_ms.or(Some(3600000)), // 1 hour default
        }
    } else {
        sublime_workspace_cli::common::config::DaemonConfig {
            socket_path: Some(socket_path),
            pid_file: Some(pid_file),
            polling_interval_ms: Some(500),
            inactive_polling_ms: Some(5000),
            // Set default log rotation values
            log_max_size_bytes: Some(10 * 1024 * 1024), // 10 MB
            log_max_files: Some(5),                     // 5 files
            log_check_interval_ms: Some(3600000),       // 1 hour
        }
    };

    // Create daemon manager
    let daemon_manager =
        DaemonManager::new(daemon_config.clone()).context("Failed to create daemon manager")?;

    // Process commands
    match cli.command {
        Commands::Start => start_daemon(&daemon_manager, cli.log_level.as_deref())?,
        Commands::Stop => stop_daemon(&daemon_manager)?,
        Commands::Restart => restart_daemon(&daemon_manager, cli.log_level.as_deref())?,
        Commands::Status => show_daemon_status(&daemon_manager)?,
        Commands::Run { socket_path, pid_file, log_level: cmd_log_level, config: cmd_config } => {
            // Create a modified config with the command-line overrides if provided
            let modified_config = sublime_workspace_cli::common::config::DaemonConfig {
                socket_path: socket_path.or(daemon_config.socket_path),
                pid_file: pid_file.or(daemon_config.pid_file),
                polling_interval_ms: daemon_config.polling_interval_ms,
                inactive_polling_ms: daemon_config.inactive_polling_ms,
                // Preserve log rotation settings
                log_max_size_bytes: daemon_config.log_max_size_bytes,
                log_max_files: daemon_config.log_max_files,
                log_check_interval_ms: daemon_config.log_check_interval_ms,
            };

            // Use the command-specific log level if provided, otherwise fall back to the global one
            let effective_log_level = cmd_log_level.as_deref().unwrap_or(log_level);

            // Use command-specific config path if provided
            let config_path_to_use = cmd_config.or(Some(config_path));

            run_daemon(&daemon_manager, modified_config, config_path_to_use, effective_log_level)?
        }
        Commands::ListRepos => list_repositories(&daemon_manager)?,
        Commands::AddRepo { path, name } => {
            add_repository(&daemon_manager, &path, name.as_deref())?
        }
        Commands::RemoveRepo { identifier } => remove_repository(&daemon_manager, &identifier)?,
        Commands::RunDiagnostic { verbose } => run_daemon_diagnostic(
            &daemon_manager,
            daemon_config,
            Some(config_path),
            log_level,
            verbose,
        )?,
    }

    Ok(())
}

fn start_daemon(daemon_manager: &DaemonManager, log_level: Option<&str>) -> Result<()> {
    println!("{}", ui::section_header("Starting Workspace Daemon"));

    // Check if daemon is already running
    if daemon_manager.is_running()? {
        let status = daemon_manager.status()?;
        if let Some(pid) = status.pid {
            println!("{}", ui::warning(&format!("Daemon is already running (PID: {})", pid)));
            return Ok(());
        }
    }

    // Check for stale socket and PID files
    let socket_path = daemon_manager.socket_path();
    let pid_file_path = daemon_manager.pid_file_path();

    // Remove stale socket file if it exists
    if socket_path.exists() {
        println!(
            "{}",
            ui::warning(&format!("Found stale socket file at {}", socket_path.display()))
        );
        println!("{}", ui::info("Removing stale socket file..."));

        if let Err(e) = std::fs::remove_file(&socket_path) {
            println!("{}", ui::error(&format!("Failed to remove stale socket file: {}", e)));
            println!("{}", ui::info("You may need to manually remove it or check permissions"));
            return Err(anyhow::anyhow!("Failed to remove stale socket file"));
        }
    }

    // Remove stale PID file if it exists
    if pid_file_path.exists() {
        println!(
            "{}",
            ui::warning(&format!("Found stale PID file at {}", pid_file_path.display()))
        );
        println!("{}", ui::info("Removing stale PID file..."));

        if let Err(e) = std::fs::remove_file(&pid_file_path) {
            println!("{}", ui::error(&format!("Failed to remove stale PID file: {}", e)));
            println!("{}", ui::info("You may need to manually remove it or check permissions"));
            return Err(anyhow::anyhow!("Failed to remove stale PID file"));
        }
    }

    // Start the daemon
    match daemon_manager.start(log_level) {
        Ok(()) => {
            println!("{}", ui::success("Daemon started successfully"));
            let status = daemon_manager.status()?;
            if let Some(pid) = status.pid {
                println!("{}", ui::key_value("PID", &pid.to_string()));
            }
            if let Some(socket) = status.socket_path {
                println!("{}", ui::key_value("Socket", &socket.to_string_lossy()));
            }
        }
        Err(DaemonError::Timeout) => {
            println!(
                "{}",
                ui::warning("Daemon process started but didn't respond within the timeout period")
            );

            // Check if the process actually exists
            if let Ok(pid) = daemon_manager.read_pid() {
                if check_process_running(pid) {
                    println!(
                        "{}",
                        ui::info(&format!(
                            "Process with PID {} is running but not yet responsive",
                            pid
                        ))
                    );
                    println!(
                        "{}",
                        ui::info("The daemon might still be initializing. Check status with:")
                    );
                    println!("{}", ui::command_example("workspace daemon status"));
                    return Ok(());
                } else {
                    println!("{}", ui::error(&format!("Process with PID {} is not running - daemon may have crashed during startup", pid)));
                    println!("{}", ui::info("Check logs for errors and try again"));

                    // Clean up PID and socket files
                    if pid_file_path.exists() {
                        let _ = std::fs::remove_file(&pid_file_path);
                    }
                    if socket_path.exists() {
                        let _ = std::fs::remove_file(&socket_path);
                    }

                    return Err(anyhow::anyhow!("Daemon failed to start properly"));
                }
            } else {
                println!(
                    "{}",
                    ui::error("Cannot read PID file - daemon may have crashed during startup")
                );
                println!("{}", ui::info("Check logs for errors and try again"));
                return Err(anyhow::anyhow!("Daemon failed to start properly"));
            }
        }
        Err(e) => {
            println!("{}", ui::error(&format!("Failed to start daemon: {}", e)));
            return Err(e.into());
        }
    }

    Ok(())
}

fn stop_daemon(daemon_manager: &DaemonManager) -> Result<()> {
    println!("{}", ui::section_header("Stopping Workspace Daemon"));

    // Check if daemon is running
    if !daemon_manager.is_running()? {
        println!("{}", ui::warning("Daemon is not running"));
        return Ok(());
    }

    // Show current status before stopping
    let status = daemon_manager.status()?;
    if let Some(pid) = status.pid {
        println!("{}", ui::info(&format!("Stopping daemon with PID: {}", pid)));
    }

    // Stop the daemon
    match daemon_manager.stop() {
        Ok(()) => {
            println!("{}", ui::success("Daemon stopped successfully"));
        }
        Err(e) => {
            println!("{}", ui::error(&format!("Failed to stop daemon: {}", e)));
            return Err(e.into());
        }
    }

    Ok(())
}

fn restart_daemon(daemon_manager: &DaemonManager, log_level: Option<&str>) -> Result<()> {
    println!("{}", ui::section_header("Restarting Workspace Daemon"));

    // Check if daemon is running
    let was_running = daemon_manager.is_running()?;

    if was_running {
        let status = daemon_manager.status()?;
        if let Some(pid) = status.pid {
            println!("{}", ui::info(&format!("Stopping daemon with PID: {}", pid)));
        }

        // Stop the daemon
        match daemon_manager.stop() {
            Ok(()) => {
                println!("{}", ui::success("Daemon stopped successfully"));
            }
            Err(e) => {
                println!("{}", ui::error(&format!("Failed to stop daemon: {}", e)));
                return Err(e.into());
            }
        }
    } else {
        println!("{}", ui::info("Daemon was not running"));
    }

    // Start the daemon
    match daemon_manager.start(log_level) {
        Ok(()) => {
            println!("{}", ui::success("Daemon started successfully"));
            let status = daemon_manager.status()?;
            if let Some(pid) = status.pid {
                println!("{}", ui::key_value("PID", &pid.to_string()));
            }
        }
        Err(e) => {
            println!("{}", ui::error(&format!("Failed to start daemon: {}", e)));
            return Err(e.into());
        }
    }

    Ok(())
}

fn show_daemon_status(daemon_manager: &DaemonManager) -> Result<()> {
    println!("{}", ui::section_header("Daemon Status"));

    // Check if daemon is running
    let is_running = daemon_manager.is_running()?;

    if is_running {
        println!("{}", ui::success_style("Daemon is running"));

        // Get detailed status
        let status = daemon_manager.status()?;

        // Display status information in a table
        let mut items = Vec::new();

        if let Some(pid) = status.pid {
            items.push(("PID", pid.to_string()));
        }

        if let Some(socket) = status.socket_path {
            items.push(("Socket", socket.to_string_lossy().to_string()));
        }

        if let Some(uptime) = status.uptime_seconds {
            // Format uptime nicely
            let days = uptime / 86400;
            let hours = (uptime % 86400) / 3600;
            let minutes = (uptime % 3600) / 60;
            let seconds = uptime % 60;

            let uptime_str = match (days, hours, minutes) {
                (0, 0, 0) => format!("{} seconds", seconds),
                (0, 0, _) => format!("{} minutes, {} seconds", minutes, seconds),
                (0, _, _) => format!("{} hours, {} minutes", hours, minutes),
                (_, _, _) => format!("{} days, {} hours", days, hours),
            };

            items.push(("Uptime", uptime_str));
        }

        if let Some(repos) = status.monitored_repos {
            items.push(("Monitored Repositories", repos.to_string()));
        }

        // Add health check information
        match daemon_manager.send_command("health", &[]) {
            Ok(output) => {
                // Try to parse the JSON data if available
                if let Some(data_start) = output.find('{') {
                    if let Ok(health_data) =
                        serde_json::from_str::<serde_json::Value>(&output[data_start..])
                    {
                        // Add health data to items
                        if let Some(status) = health_data.get("status") {
                            if let Some(status_str) = status.as_str() {
                                items.push(("Health Status", status_str.to_string()));
                            }
                        }

                        // Use formatted memory if available
                        if let Some(memory) = health_data.get("memory_usage_formatted") {
                            if let Some(memory_str) = memory.as_str() {
                                items.push(("Memory Usage", memory_str.to_string()));
                            }
                        } else if let Some(memory) = health_data.get("memory_usage_kb") {
                            if let Some(memory_kb) = memory.as_u64() {
                                let memory_str = if memory_kb > 1024 {
                                    format!("{:.2} MB", memory_kb as f64 / 1024.0)
                                } else {
                                    format!("{} KB", memory_kb)
                                };
                                items.push(("Memory Usage", memory_str));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Health check failed, add a note about it
                items.push(("Health Check", format!("Failed: {}", e)));
            }
        }

        // Display options for tabular output
        let options = ui::TabularOptions {
            title: Some("Daemon Information".to_string()),
            headers_in_columns: true,
            border_color: Some(tabled::settings::Color::FG_GREEN),
            header_color: Some(tabled::settings::Color::FG_CYAN),
            header_title: None,
            footer_title: None,
        };

        // Create tabular display
        let tabular = ui::Tabular {
            headers: vec!["Property".to_string(), "Value".to_string()],
            rows: items.iter().map(|(k, v)| vec![k.to_string(), v.to_string()]).collect(),
        };

        println!("{}", ui::create_tabular(&tabular, &options));

        // List available commands
        println!("\n{}", ui::highlight("Available commands:"));
        println!("{}", ui::command_example("workspace daemon stop"));
        println!("{}", ui::command_example("workspace daemon restart"));
        println!("{}", ui::command_example("workspace daemon list-repos"));
    } else {
        println!("{}", ui::warning_style("Daemon is not running"));
        println!("\n{}", ui::highlight("Commands to start the daemon:"));
        println!("{}", ui::command_example("workspace daemon start"));
    }

    Ok(())
}

fn list_repositories(daemon_manager: &DaemonManager) -> Result<()> {
    // Query the daemon for repositories
    match daemon_manager.send_command("list-repos", &[]) {
        Ok(output) => {
            let repos: Vec<&str> = output.split('\n').filter(|s| !s.is_empty()).collect();

            println!("{}", ui::section_header("Monitored Repositories"));

            if repos.is_empty() {
                println!("{}", ui::muted("No repositories are currently being monitored."));
                println!("\nTo add a repository:");
                println!("{}", ui::command_example("workspace daemon add-repo <path>"));
            } else {
                // Create repository table
                let mut table_rows = Vec::new();

                for (i, repo) in repos.iter().enumerate() {
                    table_rows.push(vec![(i + 1).to_string(), repo.to_string()]);
                }

                let headers = vec!["#".to_string(), "Repository Path".to_string()];

                let tabular = ui::Tabular { headers, rows: table_rows };

                let options = ui::TabularOptions {
                    title: None,
                    headers_in_columns: true,
                    border_color: Some(tabled::settings::Color::FG_CYAN),
                    header_color: Some(tabled::settings::Color::FG_YELLOW),
                    header_title: None,
                    footer_title: Some(format!("Total: {} repositories", repos.len())),
                };

                println!("{}", ui::create_tabular(&tabular, &options));
            }
        }
        Err(e) => {
            println!("{}", ui::error(&format!("Failed to list repositories: {}", e)));
            return Err(e.into());
        }
    }

    Ok(())
}

fn add_repository(daemon_manager: &DaemonManager, path: &str, name: Option<&str>) -> Result<()> {
    match daemon_manager.add_repository(path, name) {
        Ok(()) => {
            println!("{}", ui::success(&format!("Repository '{}' added for monitoring", path)));
        }
        Err(e) => {
            println!("{}", ui::error(&format!("Failed to add repository: {}", e)));
            return Err(e.into());
        }
    }

    Ok(())
}

fn remove_repository(daemon_manager: &DaemonManager, identifier: &str) -> Result<()> {
    match daemon_manager.remove_repository(identifier) {
        Ok(()) => {
            println!(
                "{}",
                ui::success(&format!("Repository '{}' removed from monitoring", identifier))
            );
        }
        Err(e) => {
            println!("{}", ui::error(&format!("Failed to remove repository: {}", e)));
            return Err(e.into());
        }
    }

    Ok(())
}

// Add the diagnostic run function
fn run_daemon_diagnostic(
    daemon_manager: &DaemonManager,
    daemon_config: sublime_workspace_cli::common::config::DaemonConfig,
    config_path: Option<PathBuf>,
    log_level: &str,
    verbose: bool,
) -> Result<()> {
    println!("{}", ui::section_header("Daemon Diagnostic Mode"));
    println!(
        "{}",
        ui::warning("This mode runs the daemon in the foreground for diagnostic purposes.")
    );
    println!("{}", ui::info("Press Ctrl+C to exit."));
    println!();

    // Make sure we're not already running
    if daemon_manager.is_running()? {
        println!("{}", ui::error("Daemon is already running. Stop it first with:"));
        println!("{}", ui::command_example("workspace daemon stop"));
        return Err(anyhow::anyhow!("Daemon is already running"));
    }

    // Print configuration details
    println!("{}", ui::highlight("Configuration:"));
    println!("{}", ui::key_value("Log Level", log_level));
    println!(
        "{}",
        ui::key_value("Socket Path", &daemon_config.socket_path.as_ref().unwrap().to_string())
    );
    println!(
        "{}",
        ui::key_value("PID File", &daemon_config.pid_file.as_ref().unwrap().to_string())
    );
    if let Some(poll) = daemon_config.polling_interval_ms {
        println!("{}", ui::key_value("Poll Interval", &format!("{}ms", poll)));
    }
    if let Some(inactive) = daemon_config.inactive_polling_ms {
        println!("{}", ui::key_value("Inactive Poll", &format!("{}ms", inactive)));
    }

    // Check for existing files
    let socket_path = PathBuf::from(daemon_config.socket_path.as_ref().unwrap());
    if socket_path.exists() {
        println!(
            "{}",
            ui::warning(&format!("Socket file already exists at {}", socket_path.display()))
        );
        println!("{}", ui::info("Removing existing socket file..."));
        if let Err(e) = std::fs::remove_file(&socket_path) {
            println!("{}", ui::error(&format!("Failed to remove socket file: {}", e)));
            return Err(anyhow::anyhow!("Failed to remove socket file"));
        }
        println!("{}", ui::success("Socket file removed."));
    }

    // Create parent directories
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            println!("{}", ui::info(&format!("Creating socket directory at {}", parent.display())));
            if let Err(e) = std::fs::create_dir_all(parent) {
                println!("{}", ui::error(&format!("Failed to create socket directory: {}", e)));
                return Err(anyhow::anyhow!("Failed to create socket directory"));
            }
            println!("{}", ui::success("Socket directory created."));
        }
    }

    // Create daemon server
    println!("{}", ui::info("Creating daemon server..."));
    let mut server =
        sublime_workspace_cli::common::daemon::DaemonServer::new(socket_path.clone(), config_path);
    println!("{}", ui::success("Daemon server created."));

    // Run the server
    println!("{}", ui::info("Starting server loop..."));
    println!("{}", ui::info("Press Ctrl+C to exit."));
    println!();

    // Run the server in a separate thread so we can monitor it
    let running = server.get_running();
    let server_handle = std::thread::spawn(move || match server.run() {
        Ok(()) => println!("{}", ui::success("Server exited normally.")),
        Err(e) => println!("{}", ui::error(&format!("Server error: {}", e))),
    });

    // Monitor the socket file
    println!("{}", ui::info("Monitoring socket..."));
    let start_time = std::time::Instant::now();
    let mut socket_created = false;
    let mut socket_bound = false;

    while running.load(Ordering::SeqCst) && start_time.elapsed() < Duration::from_secs(30) {
        // Check if socket file exists
        if !socket_created && socket_path.exists() {
            println!(
                "{}",
                ui::success(&format!("Socket file created at {}", socket_path.display()))
            );
            socket_created = true;
        }

        // Try connecting to the socket
        if socket_created && !socket_bound {
            match UnixStream::connect(&socket_path) {
                Ok(_) => {
                    println!("{}", ui::success("Successfully connected to socket."));
                    socket_bound = true;
                    // Once we've confirmed the socket is working, break the monitoring loop
                    break;
                }
                Err(e) => {
                    if verbose {
                        println!(
                            "{}",
                            ui::warning(&format!("Socket connection attempt failed: {}", e))
                        );
                    }
                }
            }
        }

        // Pause before next check
        std::thread::sleep(Duration::from_millis(500));
    }

    if socket_bound {
        println!("{}", ui::success("Daemon is running and accepting connections!"));
        println!("{}", ui::info("Press Ctrl+C to exit diagnostic mode."));

        // Wait for server thread to complete (should be when user presses Ctrl+C)
        server_handle.join().unwrap();
    } else {
        // If we haven't bound to the socket within the timeout, something is wrong
        println!("{}", ui::error("Failed to establish socket connection within timeout."));
        println!("{}", ui::info("Terminating daemon process..."));

        // Force termination of daemon loop
        running.store(false, Ordering::SeqCst);
        server_handle.join().unwrap();
    }

    // Clean up
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            println!("{}", ui::warning(&format!("Failed to remove socket file: {}", e)));
        }
    }

    println!("{}", ui::info("Diagnostic run completed."));
    Ok(())
}

fn run_daemon(
    daemon_manager: &DaemonManager,
    daemon_config: sublime_workspace_cli::common::config::DaemonConfig,
    config_path: Option<PathBuf>,
    log_level: &str,
) -> Result<()> {
    // Check if daemon is already running
    if daemon_manager.is_running()? {
        error!("Daemon is already running");
        return Err(anyhow::anyhow!("Daemon is already running"));
    }

    std::env::set_var("RUST_LOG", log_level);

    // Output debug information
    info!("Starting daemon process with log level: {}", log_level);

    // Read configuration from environment variables or params
    let socket_path = match std::env::var("WORKSPACE_SOCKET_PATH") {
        Ok(path) => {
            info!("Using socket path from environment: {}", path);
            PathBuf::from(path)
        }
        Err(_) => {
            let path = PathBuf::from(daemon_config.socket_path.as_ref().unwrap_or_else(|| {
                error!("No socket path defined in config");
                std::process::exit(1);
            }));
            info!("Using socket path from config: {}", path.display());
            path
        }
    };

    let pid_file_path = match std::env::var("WORKSPACE_PID_FILE") {
        Ok(path) => {
            info!("Using PID file path from environment: {}", path);
            PathBuf::from(path)
        }
        Err(_) => {
            let path = PathBuf::from(daemon_config.pid_file.as_ref().unwrap_or_else(|| {
                error!("No PID file path defined in config");
                std::process::exit(1);
            }));
            info!("Using PID file path from config: {}", path.display());
            path
        }
    };

    // Get effective config path
    let effective_config_path = match std::env::var("WORKSPACE_CONFIG_PATH") {
        Ok(path) => {
            info!("Using config path from environment: {}", path);
            Some(PathBuf::from(path))
        }
        Err(_) => {
            if let Some(ref path) = config_path {
                info!("Using config path from arguments: {}", path.display());
                Some(path.clone())
            } else {
                info!("No config path specified, using default location");
                None
            }
        }
    };

    // Report polling configuration
    if let Some(polling) = daemon_config.polling_interval_ms {
        info!("Active polling interval: {}ms", polling);
    }
    if let Some(inactive) = daemon_config.inactive_polling_ms {
        info!("Inactive polling interval: {}ms", inactive);
    }

    // Clean up existing socket file if present
    if socket_path.exists() {
        info!("Removing existing socket file at {}", socket_path.display());
        if let Err(e) = std::fs::remove_file(&socket_path) {
            warn!("Failed to remove existing socket file: {}", e);
            // Continue anyway, the bind might still work
        }
    }

    // Create parent directories if they don't exist
    if let Some(parent) = socket_path.parent() {
        info!("Ensuring socket directory exists at {}", parent.display());
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("Failed to create socket directory: {}", e);
            return Err(e.into());
        }
    }

    // Write the PID file
    info!("Using PID file path: {}", pid_file_path.display());
    if let Some(parent) = pid_file_path.parent() {
        info!("Ensuring PID directory exists at {}", parent.display());
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("Failed to create pid directory: {}", e);
            return Err(e.into());
        }
    }

    let process_id = std::process::id();
    let mut pid_file = match File::create(&pid_file_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create PID file: {}", e);
            return Err(e.into());
        }
    };

    if let Err(e) = pid_file.write_all(process_id.to_string().as_bytes()) {
        error!("Failed to write PID file: {}", e);
        return Err(e.into());
    }

    info!("Daemon PID file created at {:?} with PID {}", pid_file_path, process_id);

    // Create a new daemon server instance
    info!("Creating daemon server...");
    let mut server = DaemonServer::new(socket_path, effective_config_path);

    // Run the server
    info!("Starting daemon server...");
    match server.run() {
        Ok(()) => {
            info!("Daemon server exited cleanly");
        }
        Err(e) => {
            error!("Daemon server error: {}", e);
            return Err(e.into());
        }
    }

    // Clean up PID file on exit
    if pid_file_path.exists() {
        if let Err(e) = std::fs::remove_file(&pid_file_path) {
            warn!("Failed to remove PID file: {}", e);
        }
    }

    Ok(())
}
