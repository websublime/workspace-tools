use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use log::{debug, info, trace};
use std::collections::HashMap;
use std::env;

use sublime_workspace_cli::common::commands;
use sublime_workspace_cli::common::config::{Config, ConfigSource};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> Result<()> {
    // Parse command line arguments
    let matches = build_cli().get_matches();

    // Configure logging based on verbosity
    let log_level = match matches.get_count("verbose") {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };

    env::set_var("RUST_LOG", format!("{}={}", PACKAGE_NAME, log_level));
    env_logger::init();

    info!("Starting workspace CLI v{}", VERSION);
    trace!("Command line arguments: {:?}", env::args().collect::<Vec<_>>());

    // Extract global config overrides
    let config_overrides = extract_config_overrides(&matches);

    // Load configuration with appropriate sources
    let mut config = Config::load_with_sources(vec![
        ConfigSource::Defaults,
        ConfigSource::SystemConfig,
        ConfigSource::UserConfig,
        ConfigSource::ProjectConfig,
        ConfigSource::Environment,
    ])?;

    // Apply command-line overrides
    if !config_overrides.is_empty() {
        debug!("Applying command-line configuration overrides");
        config.apply_cli_overrides(config_overrides)?;
    }

    // Process subcommand
    match matches.subcommand() {
        Some((subcommand, args)) => commands::run_subcommand(subcommand, args, &config),
        None => {
            if matches.get_flag("list") {
                list_commands()?;
                Ok(())
            } else if matches.get_flag("version") {
                print_version();
                Ok(())
            } else {
                print_banner();
                println!("Use {} to see available commands", "--help".bright_blue());
                Ok(())
            }
        }
    }
}

fn build_cli() -> Command {
    Command::new("workspace")
        .version(VERSION)
        .author(AUTHOR)
        .about("Monorepo management tool")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count)
                .help("Increase output verbosity (can be used multiple times)"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Specify an alternative config file")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("set")
                .long("set")
                .help("Override a configuration value (format: key=value)")
                .value_name("KEY=VALUE")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .action(ArgAction::SetTrue)
                .help("List available commands")
                .conflicts_with("help"),
        )
        .subcommand_required(false)
    // We'll dynamically discover subcommands rather than hardcoding them
}

fn print_banner() {
    let version_str = format!("v{}", VERSION);
    println!("{} {}", PACKAGE_NAME.bright_green().bold(), version_str.bright_yellow());
    println!("Monorepo management tools for Node.js projects");
    println!("--------------------------------------------");
}

fn print_version() {
    println!("{} v{}", PACKAGE_NAME, VERSION);
    println!("Author: {}", AUTHOR);
    println!();

    // Print information about the build
    println!("Build Information:");
    println!("  Rust Version: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!("  Profile: {}", if cfg!(debug_assertions) { "debug" } else { "release" });
    match std::env::var("TARGET") {
        Ok(target) => println!("  Target: {}", target),
        Err(_) => println!("  Target: {} {}", std::env::consts::OS, std::env::consts::ARCH),
    };
    println!();

    // Print all available commands
    if let Ok(commands) = commands::get_available_commands() {
        println!("Available Commands:");
        for cmd in commands {
            println!("  {}", cmd.name);
        }
    }
}

fn list_commands() -> Result<()> {
    let commands = commands::get_available_commands()?;

    println!("Available commands:");
    println!();

    let mut max_len = 0;
    for cmd in &commands {
        max_len = max_len.max(cmd.name.len());
    }

    for cmd in commands {
        let padding = " ".repeat(max_len - cmd.name.len() + 2);
        let name = if cmd.built_in { cmd.name.bright_green() } else { cmd.name.bright_yellow() };

        println!(
            "  {}{}{}",
            name,
            padding,
            cmd.description.unwrap_or_else(|| "No description available".to_string())
        );
    }

    println!();
    println!("Run '{} <command> --help' to see command-specific help", PACKAGE_NAME);

    Ok(())
}

fn extract_config_overrides(matches: &ArgMatches) -> HashMap<String, String> {
    let mut overrides = HashMap::new();

    if let Some(values) = matches.get_many::<String>("set") {
        for value in values {
            if let Some((key, val)) = value.split_once('=') {
                overrides.insert(key.to_string(), val.to_string());
            }
        }
    }

    overrides
}
