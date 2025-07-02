//! Command definitions and implementations
//!
//! This module contains all CLI commands and their implementations,
//! organized by functional areas: analysis, tasks, versioning, workflows, etc.

use crate::config::CliConfig;
use crate::output::OutputManager;
use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use sublime_monorepo_tools::MonorepoTools;

mod analyze;
mod tasks;
mod version;
mod workflows;

pub use analyze::*;
pub use tasks::*;
pub use version::*;
pub use workflows::*;

/// Output format options for CLI results
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable format with colors and formatting
    Human,
    /// JSON format for machine consumption
    Json,
    /// Plain text format without colors
    Plain,
    /// YAML format for configuration
    Yaml,
}

/// All available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze monorepo structure and dependencies
    #[command(alias = "info")]
    Analyze {
        /// Show detailed analysis including dependency graphs
        #[arg(short, long)]
        detailed: bool,
    },

    /// Task management and execution
    #[command(subcommand)]
    Tasks(TasksCommands),

    /// Version management and bumping
    #[command(subcommand)]
    Version(VersionCommands),

    /// Workflow automation (development, integration, release)
    #[command(subcommand)]
    Workflows(WorkflowCommands),

    /// Configuration management
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,

        /// Validate configuration file
        #[arg(short, long)]
        validate: bool,

        /// Initialize default configuration
        #[arg(short, long)]
        init: bool,
    },

    /// Plugin management
    Plugin {
        /// List available plugins
        #[arg(short, long)]
        list: bool,

        /// Load a specific plugin
        #[arg(short = 'L', long)]
        load: Option<String>,

        /// Plugin command to execute
        #[arg(short, long)]
        command: Option<String>,

        /// Arguments for plugin command
        args: Vec<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

impl Commands {
    /// Execute the command with the provided tools and configuration
    ///
    /// # Arguments
    ///
    /// * `tools` - Initialized MonorepoTools instance
    /// * `config` - CLI configuration
    /// * `output` - Output manager for formatting results
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn execute(
        self,
        tools: MonorepoTools,
        config: CliConfig,
        output: OutputManager,
    ) -> Result<()> {
        match self {
            Commands::Analyze { detailed } => {
                execute_analyze(tools, config, output, detailed).await
            }
            Commands::Tasks(cmd) => cmd.execute(tools, config, output).await,
            Commands::Version(cmd) => cmd.execute(tools, config, output).await,
            Commands::Workflows(cmd) => cmd.execute(tools, config, output).await,
            Commands::Config { show, validate, init } => {
                execute_config(tools, config, output, show, validate, init).await
            }
            Commands::Plugin { list, load, command, args } => {
                execute_plugin(tools, config, output, list, load, command, args).await
            }
            Commands::Completions { shell } => {
                execute_completions(shell);
                Ok(())
            }
        }
    }
}

/// Execute config command
async fn execute_config(
    _tools: MonorepoTools,
    _config: CliConfig,
    mut output: OutputManager,
    show: bool,
    validate: bool,
    init: bool,
) -> Result<()> {
    if init {
        output.info("Initializing default configuration...")?;
        // TODO: Implement config initialization
        output.success("Configuration initialized successfully")?;
    } else if validate {
        output.info("Validating configuration...")?;
        // TODO: Implement config validation
        output.success("Configuration is valid")?;
    } else if show {
        output.info("Current configuration:")?;
        // TODO: Implement config display
    } else {
        output.error("No config action specified. Use --help for options.")?;
    }

    Ok(())
}

/// Execute plugin command
async fn execute_plugin(
    tools: MonorepoTools<'_>,
    _config: CliConfig,
    mut output: OutputManager,
    list: bool,
    load: Option<String>,
    command: Option<String>,
    args: Vec<String>,
) -> Result<()> {
    if list {
        output.info("Available plugins:")?;
        
        // Create plugin manager and load plugins
        let mut plugin_manager = tools.plugin_manager()
            .map_err(|e| anyhow::anyhow!("Failed to create plugin manager: {}", e))?;
        
        // Load built-in plugins
        plugin_manager.load_builtin_plugins()
            .map_err(|e| anyhow::anyhow!("Failed to load built-in plugins: {}", e))?;
        
        // List all available plugins
        let plugins = plugin_manager.list_plugins();
        
        if plugins.is_empty() {
            output.warning("No plugins found.")?;
        } else {
            for plugin_info in plugins {
                output.success(&format!(
                    "  {} v{} - {}",
                    plugin_info.name,
                    plugin_info.version,
                    plugin_info.description
                ))?;
                
                // Show available commands
                for cmd in &plugin_info.capabilities.commands {
                    output.info(&format!("    - {}: {}", cmd.name, cmd.description))?;
                }
            }
        }
    } else if let Some(plugin_name) = load {
        output.info(&format!("Loading plugin: {}", plugin_name))?;
        
        let mut plugin_manager = tools.plugin_manager()
            .map_err(|e| anyhow::anyhow!("Failed to create plugin manager: {}", e))?;
        
        // Load built-in plugins
        plugin_manager.load_builtin_plugins()
            .map_err(|e| anyhow::anyhow!("Failed to load built-in plugins: {}", e))?;
        
        // Check if plugin exists
        if plugin_manager.is_plugin_loaded(&plugin_name) {
            output.success(&format!("Plugin '{}' is already loaded", plugin_name))?;
        } else {
            output.error(&format!("Plugin '{}' not found", plugin_name))?;
        }
    } else if let Some(cmd) = command {
        // Parse command format: plugin_name:command_name
        let parts: Vec<&str> = cmd.split(':').collect();
        if parts.len() != 2 {
            output.error("Command format should be 'plugin_name:command_name'")?;
            return Ok(());
        }
        
        let plugin_name = parts[0];
        let command_name = parts[1];
        
        output.info(&format!("Executing plugin command: {}:{}", plugin_name, command_name))?;
        
        let mut plugin_manager = tools.plugin_manager()
            .map_err(|e| anyhow::anyhow!("Failed to create plugin manager: {}", e))?;
        
        // Load built-in plugins
        plugin_manager.load_builtin_plugins()
            .map_err(|e| anyhow::anyhow!("Failed to load built-in plugins: {}", e))?;
        
        // Execute the plugin command
        match plugin_manager.execute_plugin_command(plugin_name, command_name, &args) {
            Ok(result) => {
                if result.success {
                    output.success("Plugin command executed successfully")?;
                    
                    // Display result data if available
                    if !result.data.is_null() {
                        let formatted_output = serde_json::to_string_pretty(&result.data)
                            .unwrap_or_else(|_| result.data.to_string());
                        output.info(&format!("Result:\n{}", formatted_output))?;
                    }
                    
                    // Show execution time if available
                    if let Some(execution_time) = result.execution_time {
                        output.info(&format!("Execution time: {}ms", execution_time))?;
                    }
                } else {
                    output.error("Plugin command failed")?;
                    if let Some(error) = result.error {
                        output.error(&format!("Error: {}", error))?;
                    }
                }
            }
            Err(e) => {
                output.error(&format!("Failed to execute plugin command: {}", e))?;
            }
        }
    } else {
        output.error("No plugin action specified. Use --help for options.")?;
    }

    Ok(())
}

/// Execute completions generation
fn execute_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::{generate, Generator};
    use std::io;

    fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
        generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
    }

    let mut cmd = crate::app::MonorepoCliApp::command();
    print_completions(shell, &mut cmd);
}
