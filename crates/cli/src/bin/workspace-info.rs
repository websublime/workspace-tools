//! Info subcommand implementation
//! Displays information about the workspace

use anyhow::Result;
use clap::Command;
use colored::Colorize;

use sublime_workspace_cli::common::config::Config;
use sublime_workspace_cli::common::help;
use sublime_workspace_cli::common::paths;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let matches = build_cli().get_matches();

    // Load configuration
    let _config = Config::load()?;

    match matches.subcommand() {
        Some(("path", _)) => show_workspace_path(),
        Some(("env", _)) => show_environment(),
        Some(("commands", _)) => show_available_commands(),
        _ => show_general_info(),
    }
}

fn build_cli() -> Command {
    Command::new("info")
        .about("Display information about the workspace")
        .subcommand(Command::new("path").about("Show workspace path information"))
        .subcommand(Command::new("env").about("Show environment information"))
        .subcommand(Command::new("commands").about("Show information about available commands"))
}

fn show_general_info() -> Result<()> {
    // Print formatting helper for custom help
    help::print_custom_help(
        "Workspace Information",
        "Display information about the current workspace and environment.",
        "workspace info [SUBCOMMAND]",
        &[
            ("path", "Show workspace path information"),
            ("env", "Show environment information"),
            ("commands", "Show information about available commands"),
        ],
        &[
            ("workspace info path", "Show workspace path information"),
            ("workspace info env", "Show environment information"),
        ],
    )?;

    // Try to find the project root
    if let Ok(root) = paths::find_project_root(None) {
        println!("Current workspace: {}", root.display().to_string().bright_green());

        // Detect package manager
        if let Some(pkg_manager) = sublime_standard_tools::detect_package_manager(&root) {
            println!("Package manager: {}", pkg_manager.to_string().bright_yellow());
        } else {
            println!("Package manager: {}", "none detected".bright_red());
        }
    } else {
        println!("No workspace root detected");
    }

    Ok(())
}

fn show_workspace_path() -> Result<()> {
    println!("{}", "Workspace Paths".bright_green().bold());
    println!("=============");

    // Try to find the project root
    match paths::find_project_root(None) {
        Ok(root) => {
            println!("Workspace root: {}", root.display());

            // Check for common workspace files
            let package_json = root.join("package.json");
            if package_json.exists() {
                println!("package.json: {}", "✓".bright_green());
            } else {
                println!("package.json: {}", "✗".bright_red());
            }

            let npm_lock = root.join("package-lock.json");
            let yarn_lock = root.join("yarn.lock");
            let pnpm_lock = root.join("pnpm-lock.yaml");

            if npm_lock.exists() {
                println!("Lock file: {} (npm)", npm_lock.display());
            } else if yarn_lock.exists() {
                println!("Lock file: {} (yarn)", yarn_lock.display());
            } else if pnpm_lock.exists() {
                println!("Lock file: {} (pnpm)", pnpm_lock.display());
            } else {
                println!("Lock file: {}", "none found".bright_red());
            }

            // Check for workspaces property
            if package_json.exists() {
                if let Ok(content) = std::fs::read_to_string(&package_json) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(workspaces) = json.get("workspaces") {
                            match workspaces {
                                serde_json::Value::Array(arr) => {
                                    println!("Workspace patterns:");
                                    for pattern in arr {
                                        if let Some(pattern_str) = pattern.as_str() {
                                            println!("  - {}", pattern_str);
                                        }
                                    }
                                }
                                serde_json::Value::Object(obj) => {
                                    if let Some(packages) = obj.get("packages") {
                                        if let Some(arr) = packages.as_array() {
                                            println!("Workspace patterns:");
                                            for pattern in arr {
                                                if let Some(pattern_str) = pattern.as_str() {
                                                    println!("  - {}", pattern_str);
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    println!(
                                        "Workspace patterns: {}",
                                        "invalid format".bright_yellow()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            println!("Failed to find workspace root: {}", err);
        }
    }

    println!();
    println!("Configuration paths:");
    println!("  Config file: {}", paths::get_config_path()?.display());
    println!("  Data directory: {}", paths::get_data_dir()?.display());

    Ok(())
}

fn show_environment() -> Result<()> {
    println!("{}", "Environment Information".bright_green().bold());
    println!("======================");

    // System information
    println!("System:");
    println!("  OS: {}", std::env::consts::OS);
    println!("  Arch: {}", std::env::consts::ARCH);

    // Current directory
    println!("Current directory: {}", std::env::current_dir()?.display());

    // Environment variables
    println!("Environment variables:");
    let mut envs: Vec<(String, String)> = std::env::vars().collect();
    envs.sort_by(|a, b| a.0.cmp(&b.0));

    let workspace_vars: Vec<_> = envs.iter().filter(|(k, _)| k.starts_with("WORKSPACE_")).collect();

    if workspace_vars.is_empty() {
        println!("  No WORKSPACE_* variables set");
    } else {
        for (key, value) in workspace_vars {
            println!("  {}={}", key, value);
        }
    }

    // Path
    if let Ok(path) = std::env::var("PATH") {
        println!("PATH:");
        for path_entry in std::env::split_paths(&path) {
            println!("  {}", path_entry.display());
        }
    }

    Ok(())
}

fn show_available_commands() -> Result<()> {
    println!("{}", "Available Commands".bright_green().bold());
    println!("=================");

    let commands = sublime_workspace_cli::available_commands()?;

    let mut max_len = 0;
    for cmd in &commands {
        max_len = max_len.max(cmd.name.len());
    }

    println!("Built-in commands:");
    for cmd in &commands {
        if cmd.built_in {
            let padding = " ".repeat(max_len - cmd.name.len() + 2);
            println!(
                "  {}{}{}",
                cmd.name.bright_blue(),
                padding,
                cmd.description.as_deref().unwrap_or("No description available")
            );
        }
    }

    println!();
    println!("External commands:");
    let external_commands: Vec<_> = commands.iter().filter(|c| !c.built_in).collect();

    if external_commands.is_empty() {
        println!("  No external commands found");
    } else {
        for cmd in external_commands {
            let padding = " ".repeat(max_len - cmd.name.len() + 2);
            println!(
                "  {}{}{} ({})",
                cmd.name.bright_yellow(),
                padding,
                cmd.description.as_deref().unwrap_or("No description available"),
                cmd.path.display()
            );
        }
    }

    Ok(())
}
