//! CLI-specific error types and handling
//!
//! Defines error types specific to CLI operations and provides
//! user-friendly error messages and suggestions.

use std::path::PathBuf;
use thiserror::Error;

/// CLI-specific result type
pub type CliResult<T> = Result<T, CliError>;

/// Errors that can occur during CLI operations
#[derive(Error, Debug)]
pub enum CliError {
    /// Invalid working directory specified
    #[error("Invalid directory: {0:?}")]
    InvalidDirectory(PathBuf),

    /// Configuration file error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Command execution failed
    #[error("Command failed: {0}")]
    CommandFailed(String),

    /// Output formatting error
    #[error("Output error: {0}")]
    OutputError(String),

    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// IO operation failed
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Monorepo tools error
    #[error("Monorepo error: {0}")]
    MonorepoError(#[from] sublime_monorepo_tools::Error),

    /// Generic error for other cases
    #[error("Error: {0}")]
    Other(String),
}

impl CliError {
    /// Get a user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            CliError::InvalidDirectory(path) => {
                format!(
                    "The directory '{}' does not exist or is not accessible.\n\
                     Try:\n\
                     • Check that the path is correct\n\
                     • Ensure you have read permissions\n\
                     • Use an absolute path if relative paths aren't working",
                    path.display()
                )
            }
            CliError::ConfigError(msg) => {
                format!(
                    "Configuration error: {}\n\
                     Try:\n\
                     • Check the configuration file syntax\n\
                     • Validate the configuration with 'monorepo config --validate'\n\
                     • Initialize a new config with 'monorepo config --init'",
                    msg
                )
            }
            CliError::CommandFailed(msg) => {
                format!(
                    "Command execution failed: {}\n\
                     Try:\n\
                     • Run with --verbose for more details\n\
                     • Check that all required dependencies are installed\n\
                     • Verify the monorepo structure is valid",
                    msg
                )
            }
            CliError::OutputError(msg) => {
                format!(
                    "Output formatting failed: {}\n\
                     Try:\n\
                     • Use a different output format (--output json/yaml/plain)\n\
                     • Check that the output destination is writable",
                    msg
                )
            }
            CliError::PluginError(msg) => {
                format!(
                    "Plugin error: {}\n\
                     Try:\n\
                     • Check that the plugin is properly installed\n\
                     • Verify plugin compatibility\n\
                     • Use 'monorepo plugin --list' to see available plugins",
                    msg
                )
            }
            CliError::ValidationError(msg) => {
                format!(
                    "Validation failed: {}\n\
                     Try:\n\
                     • Check the monorepo configuration\n\
                     • Ensure all required files are present\n\
                     • Run 'monorepo analyze' to check the structure",
                    msg
                )
            }
            CliError::IoError(e) => {
                format!(
                    "File system error: {}\n\
                     Try:\n\
                     • Check file permissions\n\
                     • Ensure the disk is not full\n\
                     • Verify the path exists and is accessible",
                    e
                )
            }
            CliError::MonorepoError(e) => {
                format!(
                    "Monorepo operation failed: {}\n\
                     Try:\n\
                     • Run 'monorepo analyze' to check the structure\n\
                     • Ensure you're in a valid monorepo directory\n\
                     • Check that all package.json files are valid",
                    e
                )
            }
            CliError::Other(msg) => {
                format!(
                    "An error occurred: {}\n\
                     Try:\n\
                     • Run with --debug for more information\n\
                     • Check the documentation for help\n\
                     • Report this issue if it persists",
                    msg
                )
            }
        }
    }

    /// Get error category for exit codes
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::InvalidDirectory(_) => 2,
            CliError::ConfigError(_) => 3,
            CliError::CommandFailed(_) => 4,
            CliError::OutputError(_) => 5,
            CliError::PluginError(_) => 6,
            CliError::ValidationError(_) => 7,
            CliError::IoError(_) => 8,
            CliError::MonorepoError(_) => 9,
            CliError::Other(_) => 1,
        }
    }

    /// Check if this error suggests using debug mode
    pub fn suggests_debug(&self) -> bool {
        matches!(self, CliError::CommandFailed(_) | CliError::MonorepoError(_) | CliError::Other(_))
    }

    /// Check if this error suggests using verbose mode
    pub fn suggests_verbose(&self) -> bool {
        matches!(self, CliError::CommandFailed(_) | CliError::MonorepoError(_))
    }
}

/// Helper trait for converting results to CLI results with context
pub trait CliResultExt<T> {
    /// Convert to CLI result with additional context
    fn cli_context(self, context: &str) -> CliResult<T>;
}

impl<T, E: std::fmt::Display> CliResultExt<T> for Result<T, E> {
    fn cli_context(self, context: &str) -> CliResult<T> {
        self.map_err(|e| CliError::Other(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let error = CliError::InvalidDirectory(PathBuf::from("/nonexistent"));
        let message = error.user_message();
        assert!(message.contains("does not exist"));
        assert!(message.contains("Try:"));
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(CliError::InvalidDirectory(PathBuf::new()).exit_code(), 2);
        assert_eq!(CliError::ConfigError("test".to_string()).exit_code(), 3);
        assert_eq!(CliError::CommandFailed("test".to_string()).exit_code(), 4);
    }

    #[test]
    fn test_suggestions() {
        let error = CliError::CommandFailed("test".to_string());
        assert!(error.suggests_debug());
        assert!(error.suggests_verbose());

        let error = CliError::OutputError("test".to_string());
        assert!(!error.suggests_debug());
        assert!(!error.suggests_verbose());
    }

    #[test]
    fn test_cli_context() {
        let result: Result<(), &str> = Err("original error");
        let cli_result = result.cli_context("test context");

        assert!(cli_result.is_err());
        let error_msg = cli_result.unwrap_err().to_string();
        assert!(error_msg.contains("test context"));
        assert!(error_msg.contains("original error"));
    }
}
