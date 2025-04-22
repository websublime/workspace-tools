use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{error, info, warn};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use sublime_workspace_cli::common::config::{get_config_path, Config};
use sublime_workspace_cli::common::daemon::{DaemonManager, DaemonServer};
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
    Run,

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
        }
    } else {
        sublime_workspace_cli::common::config::DaemonConfig {
            socket_path: Some(socket_path),
            pid_file: Some(pid_file),
            polling_interval_ms: Some(500),
            inactive_polling_ms: Some(5000),
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
        Commands::Run => run_daemon(&daemon_manager, daemon_config, Some(config_path), log_level)?,
        Commands::ListRepos => list_repositories(&daemon_manager)?,
        Commands::AddRepo { path, name } => {
            add_repository(&daemon_manager, &path, name.as_deref())?
        }
        Commands::RemoveRepo { identifier } => remove_repository(&daemon_manager, &identifier)?,
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

fn run_daemon(
    daemon_manager: &DaemonManager,
    daemon_config: sublime_workspace_cli::common::config::DaemonConfig,
    config_path: Option<PathBuf>,
    log_level: &str,
) -> Result<()> {
    // This function is called when the daemon is started in the background
    // It should detach from the parent process and run until terminated

    // Make sure we're not already running
    if daemon_manager.is_running()? {
        error!("Daemon is already running");
        return Err(anyhow::anyhow!("Daemon is already running"));
    }

    info!("Starting daemon process with log level: {}", log_level);

    // Create socket path
    let socket_path = PathBuf::from(daemon_config.socket_path.unwrap());

    // Create parent directories if they don't exist
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    // Write the PID file
    let pid_file_path = PathBuf::from(daemon_config.pid_file.unwrap());
    if let Some(parent) = pid_file_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create pid directory")?;
    }
    let mut pid_file = File::create(&pid_file_path).context("Failed to create PID file")?;
    pid_file
        .write_all(std::process::id().to_string().as_bytes())
        .context("Failed to write PID file")?;

    info!("Daemon PID file created at {:?}", pid_file_path);

    // Create a new daemon server
    let mut server = DaemonServer::new(socket_path, config_path);

    // Run the server
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
