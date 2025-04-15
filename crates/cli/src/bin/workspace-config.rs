//! Config subcommand implementation
//! Manages workspace CLI configuration

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use colored::Colorize;

use sublime_workspace_cli::common::config::{Config, ConfigSource};
use sublime_workspace_cli::common::paths;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("init", args)) => {
            let force = args.get_flag("force");
            init_config(force)
        }
        Some(("get", args)) => {
            let key = args.get_one::<String>("key").expect("key is required");
            get_config_value(key)
        }
        Some(("set", args)) => {
            let key = args.get_one::<String>("key").expect("key is required");
            let value = args.get_one::<String>("value").expect("value is required");
            set_config_value(key, value)
        }
        Some(("edit", _)) => edit_config(),
        Some(("path", _)) => show_config_path(),
        Some(("validate", _)) => validate_config(),
        _ => show_config(),
    }
}

fn build_cli() -> Command {
    Command::new("config")
        .about("Manage workspace CLI configuration")
        .subcommand_required(false)
        .subcommand(
            Command::new("init").about("Initialize default configuration").arg(
                Arg::new("force")
                    .short('f')
                    .long("force")
                    .help("Overwrite existing configuration")
                    .action(ArgAction::SetTrue),
            ),
        )
        .subcommand(Command::new("get").about("Get a configuration value").arg(
            Arg::new("key").help("Configuration key (e.g., 'general.log_level')").required(true),
        ))
        .subcommand(
            Command::new("set")
                .about("Set a configuration value")
                .arg(
                    Arg::new("key")
                        .help("Configuration key (e.g., 'general.log_level')")
                        .required(true),
                )
                .arg(Arg::new("value").help("Value to set").required(true)),
        )
        .subcommand(Command::new("edit").about("Open configuration in editor"))
        .subcommand(Command::new("path").about("Show configuration file path"))
        .subcommand(Command::new("validate").about("Validate configuration"))
}

fn init_config(force: bool) -> Result<()> {
    let config_path = paths::get_config_path()?;

    if config_path.exists() && !force {
        println!("Configuration file already exists at: {}", config_path.display());
        println!("Use {} to overwrite", "--force".bright_yellow());
        return Ok(());
    }

    // Create default configuration
    let config = Config::default();

    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    // Save configuration
    config
        .save_to(&config_path)
        .with_context(|| format!("Failed to save configuration to {}", config_path.display()))?;

    println!("Created configuration at: {}", config_path.display().to_string().bright_green());
    Ok(())
}

fn get_config_value(key: &str) -> Result<()> {
    let config = Config::load_with_sources(vec![ConfigSource::UserConfig])?;

    let keys: Vec<&str> = key.split('.').collect();
    if keys.is_empty() || keys.len() > 3 {
        anyhow::bail!("Invalid key format. Use section.key or section.subsection.key");
    }

    let value = config.get_value(key)?;
    println!("{} = {}", key.bright_blue(), value);

    Ok(())
}

fn set_config_value(key: &str, value: &str) -> Result<()> {
    // Load existing config
    let mut config = Config::load_with_sources(vec![ConfigSource::UserConfig])?;

    // Update value
    config.set_value(key, value)?;

    // Save updated config
    config.save()?;

    println!("Updated configuration: {} = {}", key.bright_blue(), value);
    Ok(())
}

fn edit_config() -> Result<()> {
    let config_path = paths::get_config_path()?;

    if !config_path.exists() {
        // Initialize config if it doesn't exist
        init_config(false)?;
    }

    // Determine editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    // Open editor
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .with_context(|| format!("Failed to open editor {}", editor))?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status: {}", status);
    }

    // Validate the edited config
    validate_config()?;

    Ok(())
}

fn show_config_path() -> Result<()> {
    let config_path = paths::get_config_path()?;
    println!("{}", config_path.display());
    Ok(())
}

fn validate_config() -> Result<()> {
    // Load config and validate it
    let config = Config::load()?;
    let validation_result = config.validate();

    match validation_result {
        Ok(_) => {
            println!("{}", "Configuration is valid".bright_green());
            Ok(())
        }
        Err(errors) => {
            println!("{}", "Configuration validation failed:".bright_red());
            for error in errors {
                println!("- {}", error);
            }
            anyhow::bail!("Configuration validation failed");
        }
    }
}

fn show_config() -> Result<()> {
    // Load config
    let config = Config::load()?;

    // Format config for display
    let config_str =
        toml::to_string_pretty(&config).context("Failed to serialize configuration")?;

    println!("{}", config_str);
    Ok(())
}
