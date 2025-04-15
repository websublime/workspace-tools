use anyhow::{Context, Result};
use clap::{ArgMatches, Command};
use colored::Colorize;
use log::{error, info};
use std::env;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use sublime_workspace_cli::common::config::Config;
use sublime_workspace_cli::common::paths;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let matches = build_cli().get_matches();

    info!("Starting workspace CLI v{}", VERSION);

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    // Process subcommand
    match matches.subcommand() {
        Some((subcommand, args)) => run_subcommand(subcommand, args, &config),
        None => {
            println!("{} v{}", PACKAGE_NAME.bright_green(), VERSION);
            println!("Use {} to see available commands", "--help".bright_blue());
            Ok(())
        }
    }
}

fn build_cli() -> Command {
    Command::new("workspace")
        .version(VERSION)
        .about("Monorepo management tool")
        .subcommand_required(false)
        .subcommand(
            Command::new("daemon").about("Run the workspace daemon").arg_required_else_help(false),
        )
        .subcommand(
            Command::new("monitor")
                .about("Open the interactive workspace monitor")
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new("changes").about("Manage package changes").arg_required_else_help(true),
        )
        .subcommand(
            Command::new("version").about("Manage package versions").arg_required_else_help(true),
        )
        .subcommand(
            Command::new("graph").about("Visualize dependency graphs").arg_required_else_help(true),
        )
}

fn run_subcommand(subcommand: &str, args: &ArgMatches, config: &Config) -> Result<()> {
    let binary_name = format!("workspace-{}", subcommand);

    // Build args to pass to the subcommand
    let mut sub_args = Vec::new();
    for (name, value) in args.ids() {
        // Extract all arguments to pass along
        if let Some(val) = value.raw() {
            sub_args.push(format!("--{}", name));
            sub_args.push(val.to_string());
        } else {
            sub_args.push(format!("--{}", name));
        }
    }

    // Get current executable directory
    let current_exe = env::current_exe().context("Failed to get current executable path")?;
    let exe_dir = current_exe.parent().context("Failed to get executable directory")?;

    // Look for subcommand in same directory as main binary
    let subcommand_path = exe_dir.join(&binary_name);

    info!("Running subcommand: {}", binary_name);

    // If subcommand binary exists, execute it
    if subcommand_path.exists() {
        let status = ProcessCommand::new(subcommand_path)
            .args(&sub_args)
            .status()
            .with_context(|| format!("Failed to execute subcommand: {}", binary_name))?;

        if !status.success() {
            error!("Subcommand {} exited with status: {}", binary_name, status);
            anyhow::bail!("Subcommand execution failed");
        }

        Ok(())
    } else {
        error!("Subcommand binary not found: {}", subcommand_path.display());
        anyhow::bail!("Subcommand binary not found: {}", binary_name)
    }
}
