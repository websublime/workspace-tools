//! Command discovery and execution

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::common::config::Config;

/// Represents a discovered command
#[derive(Debug, Clone)]
pub struct DiscoveredCommand {
    /// Name of the command
    pub name: String,
    /// Path to the command executable
    pub path: PathBuf,
    /// Whether the command is built-in
    pub built_in: bool,
    /// Description of the command (if available)
    pub description: Option<String>,
}

/// Discovers available commands
pub fn discover_commands() -> Result<HashMap<String, DiscoveredCommand>> {
    let mut commands = HashMap::new();

    // Get current executable directory
    let current_exe = env::current_exe().context("Failed to get current executable path")?;
    let exe_dir = current_exe.parent().context("Failed to get executable directory")?;

    // Discover built-in commands
    discover_built_in_commands(exe_dir, &mut commands)?;

    // Discover external commands
    discover_external_commands(&mut commands)?;

    Ok(commands)
}

/// Discovers built-in commands (workspace-*)
fn discover_built_in_commands(
    exe_dir: &Path,
    commands: &mut HashMap<String, DiscoveredCommand>,
) -> Result<()> {
    // Standard built-in commands
    let built_ins = ["daemon", "monitor", "changes", "version", "graph", "config"];

    for cmd in built_ins {
        let cmd_name = format!("workspace-{}", cmd);
        let cmd_path = exe_dir.join(&cmd_name);

        if cmd_path.exists() {
            let description = get_command_description(&cmd_path)?;
            commands.insert(
                cmd.to_string(),
                DiscoveredCommand {
                    name: cmd.to_string(),
                    path: cmd_path,
                    built_in: true,
                    description,
                },
            );
            debug!("Discovered built-in command: {}", cmd);
        } else {
            warn!("Built-in command binary not found: {}", cmd_path.display());
        }
    }

    Ok(())
}

/// Discovers external commands (workspace-*)
fn discover_external_commands(commands: &mut HashMap<String, DiscoveredCommand>) -> Result<()> {
    // Get PATH directories
    let path_var = env::var_os("PATH").unwrap_or_default();
    let paths = env::split_paths(&path_var);

    for path in paths {
        if !path.exists() || !path.is_dir() {
            continue;
        }

        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(err) => {
                warn!("Failed to read directory {}: {}", path.display(), err);
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    warn!("Failed to read directory entry: {}", err);
                    continue;
                }
            };

            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if !file_name_str.starts_with("workspace-") {
                continue;
            }

            let cmd_path = entry.path();
            if !is_executable(&cmd_path) {
                continue;
            }

            // Extract the command name after "workspace-"
            let cmd_name = file_name_str.strip_prefix("workspace-").unwrap().to_string();

            // Skip if it's a built-in command that we already discovered
            if commands.contains_key(&cmd_name) && commands[&cmd_name].built_in {
                continue;
            }

            let description = get_command_description(&cmd_path)?;
            commands.insert(
                cmd_name.clone(),
                DiscoveredCommand {
                    name: cmd_name.clone(),
                    path: cmd_path.clone(),
                    built_in: false,
                    description,
                },
            );
            debug!("Discovered external command: {}", cmd_name);
        }
    }

    Ok(())
}

/// Checks if a file is executable
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    match fs::metadata(path) {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            permissions.mode() & 0o111 != 0
        }
        Err(_) => false,
    }
}

/// Checks if a file is executable (Windows version)
#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    path.exists()
        && path.extension().map_or(false, |ext| {
            ext.eq_ignore_ascii_case("exe")
                || ext.eq_ignore_ascii_case("cmd")
                || ext.eq_ignore_ascii_case("bat")
        })
}

/// Gets the description of a command by running it with --help
fn get_command_description(cmd_path: &Path) -> Result<Option<String>> {
    // Try to run the command with --help to get its description
    let output = match ProcessCommand::new(cmd_path).arg("--help").output() {
        Ok(output) => output,
        Err(err) => {
            warn!("Failed to get description for {}: {}", cmd_path.display(), err);
            return Ok(None);
        }
    };

    if !output.status.success() {
        warn!("Command {} returned non-zero status with --help", cmd_path.display());
        return Ok(None);
    }

    let help_text = String::from_utf8_lossy(&output.stdout);

    // Parse the description from the help text
    // Typically, the description is on the first line after the command name
    let mut lines = help_text.lines();
    let _first_line = lines.next(); // Skip the first line (command name)

    // The description is usually the second line or empty line + next non-empty line
    let mut description = None;
    for line in lines {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with("Usage:") && !line.starts_with('-') {
            description = Some(line.to_string());
            break;
        }
    }

    Ok(description)
}

/// Executes a command with the given arguments
pub fn execute_command(cmd: &DiscoveredCommand, args: &ArgMatches, _config: &Config) -> Result<()> {
    // Build args to pass to the subcommand
    let mut sub_args = Vec::new();

    // Add configuration value if it exists
    if let Some(config_path) = args.get_one::<String>("config") {
        sub_args.push("--config".to_string());
        sub_args.push(config_path.clone());
    }

    // Add verbosity if it exists
    if let Some(count) = args.get_one::<u8>("verbose") {
        for _ in 0..*count {
            sub_args.push("-v".to_string());
        }
    }

    // Add all other arguments
    for id in args.ids() {
        let name = id.as_str();
        if name == "verbose" || name == "config" || name == "command" {
            continue; // Skip these as we've already handled them
        }

        // Handle flags and options differently
        if args.contains_id(id.as_str()) {
            if args.get_flag(id.as_str()) {
                // This is a flag-type argument
                sub_args.push(format!("--{}", name));
            } else if let Some(vals) = args.get_many::<String>(id.as_str()) {
                // This is a value-type argument
                for val in vals {
                    sub_args.push(format!("--{}", name));
                    sub_args.push(val.clone());
                }
            } else if let Some(raw_vals) = args.get_raw(id.as_str()) {
                // Raw values
                for val in raw_vals {
                    sub_args.push(format!("--{}", name));
                    sub_args.push(val.to_string_lossy().to_string());
                }
            }
        }
    }

    info!("Executing command: {}", cmd.name);
    debug!("Command path: {}", cmd.path.display());
    debug!("Command args: {:?}", sub_args);

    // Execute the command
    let status = ProcessCommand::new(&cmd.path)
        .args(&sub_args)
        .status()
        .with_context(|| format!("Failed to execute command: {}", cmd.name))?;

    if !status.success() {
        error!("Command {} exited with status: {}", cmd.name, status);

        // If we have a specific exit code, return it
        if let Some(code) = status.code() {
            // Exit with the same code
            std::process::exit(code);
        }

        return Err(anyhow!("Command execution failed"));
    }

    Ok(())
}

/// Get a list of available commands, sorted alphabetically
pub fn get_available_commands() -> Result<Vec<DiscoveredCommand>> {
    let commands = discover_commands()?;
    let mut command_list: Vec<DiscoveredCommand> = commands.values().cloned().collect();
    command_list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(command_list)
}

/// Run a specific subcommand by name
pub fn run_subcommand(subcommand: &str, args: &ArgMatches, config: &Config) -> Result<()> {
    let commands = discover_commands()?;

    if let Some(cmd) = commands.get(subcommand) {
        execute_command(cmd, args, config)
    } else {
        Err(anyhow!("Unknown command: {}", subcommand))
    }
}
