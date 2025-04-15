//! Daemon subcommand implementation
//! Manages the background service that monitors repositories

use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use log::{error, warn};
use std::env;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sublime_workspace_cli::common::config::{Config, ConfigSource};
use sublime_workspace_cli::common::paths;
use sublime_workspace_cli::daemon::{DaemonService, DaemonStatus, IpcClient};

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let matches = build_cli().get_matches();

    // Load configuration with appropriate sources
    let config = Config::load_with_sources(vec![
        ConfigSource::Defaults,
        ConfigSource::SystemConfig,
        ConfigSource::UserConfig,
        ConfigSource::ProjectConfig,
        ConfigSource::Environment,
    ])?;

    // Create client
    let socket_path = paths::expand_path(&config.daemon.socket_path)?;
    let client = IpcClient::new(socket_path);

    if matches.get_flag("start") {
        start_daemon(&config, matches.get_flag("foreground"))
    } else if matches.get_flag("stop") {
        stop_daemon(&client)
    } else if matches.get_flag("restart") {
        restart_daemon(&client, &config, matches.get_flag("foreground"))
    } else if matches.get_flag("status") {
        show_status(&client)
    } else {
        // Default action based on config
        if config.general.auto_start_daemon {
            // Check if daemon is running
            match client.ping() {
                Ok(true) => {
                    // Daemon is running, show status
                    show_status(&client)
                }
                _ => {
                    // Daemon is not running, start it
                    start_daemon(&config, matches.get_flag("foreground"))
                }
            }
        } else {
            // Just show status
            show_status(&client)
        }
    }
}

fn build_cli() -> Command {
    Command::new("daemon")
        .about("Run the workspace daemon")
        .arg(
            Arg::new("start")
                .long("start")
                .action(ArgAction::SetTrue)
                .help("Start the daemon")
                .conflicts_with_all(["stop", "restart", "status"]),
        )
        .arg(
            Arg::new("stop")
                .long("stop")
                .action(ArgAction::SetTrue)
                .help("Stop the daemon")
                .conflicts_with_all(["start", "restart", "status"]),
        )
        .arg(
            Arg::new("restart")
                .long("restart")
                .action(ArgAction::SetTrue)
                .help("Restart the daemon")
                .conflicts_with_all(["start", "stop", "status"]),
        )
        .arg(
            Arg::new("status")
                .long("status")
                .action(ArgAction::SetTrue)
                .help("Check daemon status")
                .conflicts_with_all(["start", "stop", "restart"]),
        )
        .arg(
            Arg::new("foreground")
                .long("foreground")
                .short('f')
                .action(ArgAction::SetTrue)
                .help("Run in foreground (don't daemonize)"),
        )
}

/// Start the daemon
fn start_daemon(config: &Config, foreground: bool) -> Result<()> {
    // Check if daemon is already running
    let socket_path = paths::expand_path(&config.daemon.socket_path)?;
    let client = IpcClient::new(&socket_path);

    match client.ping() {
        Ok(true) => {
            println!("{}", "Daemon is already running".bright_yellow());
            return Ok(());
        }
        _ => {
            // Daemon is not running, start it
        }
    }

    // Check if we should daemonize
    if !foreground {
        // Fork a new process
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            // Get current executable
            let current_exe = env::current_exe()?;

            // Prepare args for the new process
            let mut args: Vec<String> = env::args().collect();
            args.push("--foreground".to_string());

            // Remove the first arg (executable name)
            args.remove(0);

            // Spawn a new process
            let mut cmd = process::Command::new(current_exe);
            cmd.args(args);
            cmd.stdin(process::Stdio::null());
            cmd.stdout(process::Stdio::null());
            cmd.stderr(process::Stdio::null());

            // Fork and exit parent
            unsafe {
                cmd.pre_exec(|| {
                    // Create a new session
                    if libc::setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }

            let child = cmd.spawn().context("Failed to spawn daemon process")?;

            println!("Daemon started with PID {}", child.id());
            return Ok(());
        }

        #[cfg(not(unix))]
        {
            // On non-Unix platforms, just run in foreground
            warn!("Daemonization not supported on this platform, running in foreground");
        }
    }

    // Running in foreground or on a platform that doesn't support daemonization

    // Create daemon service
    let mut daemon_service = DaemonService::from_config(config)?;

    // Start the daemon
    daemon_service.start()?;

    println!("Daemon started. Press Ctrl+C to stop.");

    // Wait for Ctrl+C
    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\nReceived Ctrl+C, shutting down...");
    })
    .expect("Error setting Ctrl+C handler");

    // Wait while running
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("Stopping daemon...");
    daemon_service.stop()?;
    println!("Daemon stopped.");

    Ok(())
}

/// Stop the daemon
fn stop_daemon(client: &IpcClient) -> Result<()> {
    match client.ping() {
        Ok(true) => {
            // Daemon is running, shut it down
            match client.shutdown() {
                Ok(_) => {
                    println!("Daemon successfully stopped");
                    Ok(())
                }
                Err(e) => {
                    error!("Error shutting down daemon: {}", e);
                    Err(anyhow!("Failed to stop daemon: {}", e))
                }
            }
        }
        _ => {
            println!("Daemon is not running");
            Ok(())
        }
    }
}

/// Restart the daemon
fn restart_daemon(client: &IpcClient, config: &Config, foreground: bool) -> Result<()> {
    // Try to stop the daemon
    if client.ping().unwrap_or(false) {
        match client.shutdown() {
            Ok(_) => {
                println!("Daemon stopped");
                // Sleep to ensure the socket is closed
                std::thread::sleep(Duration::from_secs(1));
            }
            Err(e) => {
                warn!("Error stopping daemon: {}", e);
                // Continue anyway to start a new instance
            }
        }
    }

    // Start the daemon
    start_daemon(config, foreground)
}

/// Show daemon status
fn show_status(client: &IpcClient) -> Result<()> {
    match client.ping() {
        Ok(true) => {
            // Daemon is running, get status
            match client.status() {
                Ok(status) => {
                    print_status(&status);
                    Ok(())
                }
                Err(e) => {
                    error!("Error getting daemon status: {}", e);
                    Err(anyhow!("Failed to get daemon status: {}", e))
                }
            }
        }
        _ => {
            println!("{}", "Daemon is not running".bright_red());
            Ok(())
        }
    }
}

/// Print daemon status
fn print_status(status: &DaemonStatus) {
    println!("{}", "Daemon Status".bright_green().bold());
    println!("=============");

    println!(
        "Status: {}",
        if status.running { "Running".bright_green() } else { "Stopped".bright_red() }
    );

    if let Some(pid) = status.pid {
        println!("PID: {}", pid);
    }

    if let Some(uptime) = status.uptime {
        // Format uptime nicely
        let days = uptime / (24 * 60 * 60);
        let hours = (uptime % (24 * 60 * 60)) / (60 * 60);
        let minutes = (uptime % (60 * 60)) / 60;
        let seconds = uptime % 60;

        if days > 0 {
            println!("Uptime: {}d {}h {}m {}s", days, hours, minutes, seconds);
        } else if hours > 0 {
            println!("Uptime: {}h {}m {}s", hours, minutes, seconds);
        } else if minutes > 0 {
            println!("Uptime: {}m {}s", minutes, seconds);
        } else {
            println!("Uptime: {}s", seconds);
        }
    }

    if let Some(started_at) = status.started_at {
        // Format start time
        let start_time = chrono::DateTime::<chrono::Utc>::from(started_at);
        println!("Started at: {}", start_time.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    println!("Repositories: {}", status.repository_count);

    if !status.repositories.is_empty() {
        println!("\nMonitored Repositories:");
        for repo in &status.repositories {
            println!("  {} ({})", repo.name.bright_blue(), repo.path.display());

            if let Some(branch) = &repo.branch {
                println!("    Branch: {}", branch);
            }

            if let Some(last_commit) = &repo.last_commit {
                println!("    Last commit: {}", last_commit);
            }

            println!(
                "    Status: {}",
                if repo.active { "Active".bright_green() } else { "Inactive".bright_yellow() }
            );

            if let Some(last_checked) = repo.last_checked {
                // Format last checked time
                let last_checked_time = chrono::DateTime::<chrono::Utc>::from(last_checked);
                println!("    Last checked: {}", last_checked_time.format("%Y-%m-%d %H:%M:%S UTC"));
            }

            if repo.pending_changes > 0 {
                println!("    Pending changes: {}", repo.pending_changes);
            }
        }
    }

    // System resources
    if let Some(memory) = status.memory_usage {
        println!("\nMemory usage: {} bytes", memory);
    }

    if let Some(cpu) = status.cpu_usage {
        println!("CPU usage: {:.2}%", cpu);
    }

    if let Some(socket_path) = &status.socket_path {
        println!("Socket path: {}", socket_path.display());
    }
}
