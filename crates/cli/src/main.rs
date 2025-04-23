use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use sublime_workspace_cli::ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Log level (error, warn, info, debug, trace)
    #[arg(long, global = true)]
    log_level: Option<String>,

    /// Path to configuration file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Path to the workspace (defaults to current directory)
    #[arg(long, global = true)]
    workspace: Option<PathBuf>,

    /// Use JSON output format where supported
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure workspace settings
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },

    /// Manage the monitoring daemon
    Daemon {
        /// Subcommand to execute
        command: Option<String>,

        /// Additional arguments to pass to the subcommand
        args: Vec<String>,
    },

    /// Show detailed workspace information
    Info {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Monitor changes in real-time (interactive UI)
    Monitor {
        /// Subcommand to execute
        command: Option<String>,

        /// Additional arguments to pass to the subcommand
        args: Vec<String>,
    },

    /// Show recent changes in the workspace
    Changes {
        /// Subcommand to execute
        command: Option<String>,

        /// Additional arguments to pass to the subcommand
        args: Vec<String>,
    },

    /// Manage package versions
    Version {
        /// Subcommand to execute
        command: Option<String>,

        /// Additional arguments to pass to the subcommand
        args: Vec<String>,
    },

    /// Visualize workspace dependency graph
    Graph {
        /// Subcommand to execute
        command: Option<String>,

        /// Additional arguments to pass to the subcommand
        args: Vec<String>,
    },

    /// Debug and troubleshoot the daemon connection
    Debug {
        /// Subcommand to execute
        command: Option<String>,

        /// Additional arguments to pass to the subcommand
        args: Vec<String>,
    },

    /// Execute a custom subcommand
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Initialize configuration file
    Init {
        /// Force overwrite if configuration already exists
        #[arg(short, long)]
        force: bool,
    },

    /// Show configuration path
    Path,

    /// View current configuration
    View {
        /// Show raw TOML format
        #[arg(short, long)]
        raw: bool,
    },

    /// Edit configuration file
    Edit,

    /// Get a specific configuration value
    Get {
        /// Configuration key (e.g., "daemon.socket_path")
        key: String,
    },

    /// Set a specific configuration value
    Set {
        /// Configuration key (e.g., "daemon.socket_path")
        key: String,

        /// Configuration value
        value: String,
    },
}

fn main() -> Result<()> {
    // Initialize the UI system
    ui::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Setup logging
    if let Some(log_level) = &cli.log_level {
        std::env::set_var("RUST_LOG", log_level);
    } else {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Set workspace path in environment if provided
    if let Some(workspace) = &cli.workspace {
        std::env::set_var("WORKSPACE_PATH", workspace.to_string_lossy().to_string());
    }

    // Set config path in environment if provided
    if let Some(config) = &cli.config {
        std::env::set_var("WORKSPACE_CONFIG_PATH", config.to_string_lossy().to_string());
    }

    // Set JSON output flag if provided
    if cli.json {
        std::env::set_var("WORKSPACE_JSON_OUTPUT", "true");
    }

    // Dispatch to appropriate subcommand
    match &cli.command {
        Some(Commands::Config { command }) => {
            let mut args = vec!["config"];

            match command {
                Some(ConfigCommands::Init { force }) => {
                    args.push("init");
                    if *force {
                        args.push("--force");
                    }
                }
                Some(ConfigCommands::Path) => {
                    args.push("path");
                }
                Some(ConfigCommands::View { raw }) => {
                    args.push("view");
                    if *raw {
                        args.push("--raw");
                    }
                }
                Some(ConfigCommands::Edit) => {
                    args.push("edit");
                }
                Some(ConfigCommands::Get { key }) => {
                    args.push("get");
                    args.push(key);
                }
                Some(ConfigCommands::Set { key, value }) => {
                    args.push("set");
                    args.push(key);
                    args.push(value);
                }
                None => {}
            }

            run_subcommand("workspace-config", &args)?;
        }
        Some(Commands::Daemon { command, args }) => {
            let mut full_args = Vec::new();
            if let Some(cmd) = command {
                full_args.push(cmd.as_str());
            }
            full_args.extend(args.iter().map(String::as_str));

            run_subcommand("workspace-daemon", &full_args)?;
        }
        Some(Commands::Info { verbose }) => {
            let mut args = Vec::<String>::new();

            if *verbose {
                args.push("--verbose".to_string());
            }
            if cli.json {
                args.push("--json".to_string());
            }
            if let Some(workspace) = &cli.workspace {
                args.push("--path".to_string());
                args.push(workspace.to_string_lossy().to_string());
            }

            // Convert String to &str for the function call
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_subcommand("workspace-info", &args_refs)?;
        }
        Some(Commands::Monitor { command, args }) => {
            let mut full_args = Vec::new();
            if let Some(cmd) = command {
                full_args.push(cmd.as_str());
            }
            full_args.extend(args.iter().map(String::as_str));

            run_subcommand("workspace-monitor", &full_args)?;
        }
        Some(Commands::Changes { command, args }) => {
            let mut full_args = Vec::new();
            if let Some(cmd) = command {
                full_args.push(cmd.as_str());
            }
            full_args.extend(args.iter().map(String::as_str));

            run_subcommand("workspace-changes", &full_args)?;
        }
        Some(Commands::Version { command, args }) => {
            let mut full_args = Vec::new();
            if let Some(cmd) = command {
                full_args.push(cmd.as_str());
            }
            full_args.extend(args.iter().map(String::as_str));

            run_subcommand("workspace-version", &full_args)?;
        }
        Some(Commands::Graph { command, args }) => {
            let mut full_args = Vec::new();
            if let Some(cmd) = command {
                full_args.push(cmd.as_str());
            }
            full_args.extend(args.iter().map(String::as_str));

            run_subcommand("workspace-graph", &full_args)?;
        }
        Some(Commands::Debug { command, args }) => {
            let mut full_args = Vec::new();
            if let Some(cmd) = command {
                full_args.push(cmd.as_str());
            }
            full_args.extend(args.iter().map(String::as_str));

            run_subcommand("workspace-debug", &full_args)?;
        }
        Some(Commands::External(args)) => {
            if args.is_empty() {
                return Err(anyhow::anyhow!("No external command provided"));
            }
            let cmd = format!("workspace-{}", args[0]);
            let external_args: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();

            run_subcommand(&cmd, &external_args)?;
        }
        None => {
            // No subcommand provided, show main help
            print_main_help();
        }
    }

    Ok(())
}

fn run_subcommand(command: &str, args: &[&str]) -> Result<()> {
    log::debug!("Running subcommand: {} {:?}", command, args);

    let status = Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute subcommand: {}", command))?;

    if !status.success() {
        if let Some(code) = status.code() {
            return Err(anyhow::anyhow!(
                "Subcommand '{}' failed with exit code: {}",
                command,
                code
            ));
        } else {
            return Err(anyhow::anyhow!("Subcommand '{}' terminated by signal", command));
        }
    }

    Ok(())
}

fn print_main_help() {
    println!("{}", ui::section_header("Workspace CLI"));
    println!("A comprehensive tool for managing monorepo workspaces\n");

    println!("{}", ui::highlight("USAGE:"));
    println!("  {} [OPTIONS] [COMMAND]\n", ui::command_example("workspace"));

    println!("{}", ui::highlight("COMMANDS:"));
    println!("  {} - Configure workspace settings", ui::primary_style("config"));
    println!("  {} - Manage the monitoring daemon", ui::primary_style("daemon"));
    println!("  {} - Show detailed workspace information", ui::primary_style("info"));
    println!("  {} - Monitor changes in real-time", ui::primary_style("monitor"));
    println!("  {} - Show recent changes in the workspace", ui::primary_style("changes"));
    println!("  {} - Manage package versions", ui::primary_style("version"));
    println!("  {} - Visualize workspace dependency graph", ui::primary_style("graph"));
    println!(
        "\n  Run {} for command-specific help",
        ui::command_example("workspace <COMMAND> --help")
    );

    println!("\n{}", ui::highlight("GLOBAL OPTIONS:"));
    println!("  {} - Set the logging level", ui::primary_style("--log-level <LEVEL>"));
    println!("  {} - Path to configuration file", ui::primary_style("--config <FILE>"));
    println!("  {} - Path to workspace directory", ui::primary_style("--workspace <PATH>"));
    println!("  {} - Use JSON output format", ui::primary_style("--json"));

    println!("\n{}", ui::highlight("EXAMPLES:"));
    println!("  {}", ui::command_example("workspace info"));
    println!("  {}", ui::command_example("workspace monitor"));
    println!("  {}", ui::command_example("workspace changes --json"));
    println!("  {}", ui::command_example("workspace version bump minor"));
}
