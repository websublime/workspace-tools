//! CLI Application structure and main entry point
//!
//! Defines the main CLI application structure using Clap for argument parsing
//! and command dispatch. Integrates with the sublime-monorepo-tools library
//! for all monorepo operations.

use crate::commands::{Commands, OutputFormat};
use crate::config::CliConfig;
use crate::error::CliResult;
use crate::output::OutputManager;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use sublime_monorepo_tools::MonorepoTools;

/// Sublime Monorepo CLI - Advanced monorepo management and automation
///
/// A comprehensive command-line tool for managing monorepos with features including:
/// - Intelligent change detection and package analysis
/// - Automated versioning with dependency propagation
/// - Task execution with conditional logic
/// - Workflow automation (development, integration, release)
/// - Git hooks integration and validation
/// - Plugin system for extensibility
///
/// Examples:
///   monorepo analyze                    # Analyze monorepo structure
///   monorepo tasks run --affected       # Run tasks for changed packages
///   monorepo version bump minor         # Bump version for affected packages
///   monorepo release --environment prod # Release workflow to production
#[derive(Parser, Debug)]
#[command(name = "monorepo")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct MonorepoCliApp {
    /// Working directory (defaults to current directory)
    #[arg(short = 'C', long, value_name = "PATH", global = true)]
    pub directory: Option<PathBuf>,

    /// Output format for results
    #[arg(long, value_enum, default_value = "human", global = true)]
    pub output: OutputFormat,

    /// Verbose logging output
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Configuration file path
    #[arg(long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Enable debug mode with detailed information
    #[arg(long, global = true)]
    pub debug: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

impl MonorepoCliApp {
    /// Run the CLI application with the parsed arguments
    ///
    /// # Returns
    ///
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The working directory is invalid
    /// - MonorepoTools initialization fails
    /// - Command execution fails
    pub async fn run(self) -> Result<()> {
        // Set up working directory
        let working_dir = self.get_working_directory()?;
        
        // Initialize CLI configuration
        let config = CliConfig::new(
            self.config.as_deref(),
            self.verbose,
            self.quiet,
            !self.no_color,
            self.debug,
        )?;

        // Initialize output manager
        let output = OutputManager::new(self.output, config.use_color);

        // Initialize monorepo tools
        let tools = MonorepoTools::initialize(&working_dir)?;

        // Execute the command
        self.command.execute(tools, config, output).await
    }

    /// Get the working directory, defaulting to current directory
    fn get_working_directory(&self) -> CliResult<PathBuf> {
        match &self.directory {
            Some(dir) => {
                if dir.exists() && dir.is_dir() {
                    Ok(dir.canonicalize()?)
                } else {
                    Err(crate::error::CliError::InvalidDirectory(dir.clone()))
                }
            }
            None => Ok(std::env::current_dir()?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::TempDir;

    #[test]
    fn test_cli_app_parsing() {
        let app = MonorepoCliApp::try_parse_from(&["monorepo", "analyze"]);
        assert!(app.is_ok());
    }

    #[test]
    fn test_working_directory_validation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        
        let app = MonorepoCliApp {
            directory: Some(temp_path),
            output: OutputFormat::Human,
            verbose: 0,
            quiet: false,
            config: None,
            no_color: false,
            debug: false,
            command: Commands::Analyze { detailed: false },
        };

        let result = app.get_working_directory();
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_directory() {
        let invalid_path = PathBuf::from("/nonexistent/directory");
        
        let app = MonorepoCliApp {
            directory: Some(invalid_path.clone()),
            output: OutputFormat::Human,
            verbose: 0,
            quiet: false,
            config: None,
            no_color: false,
            debug: false,
            command: Commands::Analyze { detailed: false },
        };

        let result = app.get_working_directory();
        assert!(result.is_err());
        
        if let Err(crate::error::CliError::InvalidDirectory(path)) = result {
            assert_eq!(path, invalid_path);
        } else {
            panic!("Expected InvalidDirectory error");
        }
    }
}