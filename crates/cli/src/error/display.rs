//! Enhanced error display with colors and suggestions.
//!
//! This module provides enhanced formatting for CLI errors, including
//! color-coded output, contextual suggestions, and helpful next steps.
//!
//! # What
//!
//! Provides:
//! - Color-coded error messages
//! - Contextual suggestions for common errors
//! - Helpful next steps
//! - Error formatting utilities
//!
//! # How
//!
//! Uses the `console` crate for terminal styling and provides helpers
//! to format errors with appropriate colors and suggestions based on
//! the error type.
//!
//! # Why
//!
//! Enhanced error display improves user experience by making errors
//! more readable and providing actionable guidance for resolution.
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::error::{CliError, ErrorDisplay};
//!
//! let error = CliError::configuration("Config file not found");
//! let display = ErrorDisplay::new(&error, true);
//! println!("{}", display.format());
//! ```

use crate::error::CliError;
use console::style;
use std::fmt::Write;

/// Enhanced error display formatter.
///
/// Provides formatted error output with colors and suggestions.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::error::{CliError, ErrorDisplay};
///
/// let error = CliError::configuration("Invalid config");
/// let display = ErrorDisplay::new(&error, true);
/// let formatted = display.format();
/// ```
pub struct ErrorDisplay<'a> {
    error: &'a CliError,
    use_colors: bool,
}

impl<'a> ErrorDisplay<'a> {
    /// Creates a new error display formatter.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to format
    /// * `use_colors` - Whether to use colors in output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ErrorDisplay};
    ///
    /// let error = CliError::configuration("Test");
    /// let display = ErrorDisplay::new(&error, true);
    /// ```
    pub fn new(error: &'a CliError, use_colors: bool) -> Self {
        Self { error, use_colors }
    }

    /// Formats the error with colors and suggestions.
    ///
    /// # Returns
    ///
    /// A formatted string with the error message, suggestions, and next steps.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ErrorDisplay};
    ///
    /// let error = CliError::configuration("Config not found");
    /// let display = ErrorDisplay::new(&error, true);
    /// let formatted = display.format();
    /// assert!(formatted.contains("Configuration error"));
    /// ```
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Error type and message
        output.push_str(&self.format_header());
        output.push('\n');
        output.push_str(&self.format_message());
        output.push('\n');

        // Suggestions if available
        if let Some(suggestion) = self.get_suggestion() {
            output.push('\n');
            output.push_str(&self.format_suggestion(&suggestion));
            output.push('\n');
        }

        // Next steps if available
        if let Some(next_steps) = self.get_next_steps() {
            output.push('\n');
            output.push_str(&self.format_next_steps(&next_steps));
        }

        output
    }

    /// Formats the error header with icon and type.
    fn format_header(&self) -> String {
        let icon = "✖";
        let error_type = self.error.kind();

        if self.use_colors {
            format!("{} {} Error", style(icon).red().bold(), style(error_type).red().bold())
        } else {
            format!("{icon} {error_type} Error")
        }
    }

    /// Formats the error message.
    fn format_message(&self) -> String {
        let message = self.error.user_message();

        if self.use_colors {
            format!("  {}", style(message).white())
        } else {
            format!("  {message}")
        }
    }

    /// Formats a suggestion with appropriate styling.
    fn format_suggestion(&self, suggestion: &str) -> String {
        let icon = "ℹ";

        if self.use_colors {
            format!(
                "{} {}\n  {}",
                style(icon).cyan().bold(),
                style("Suggestion:").cyan().bold(),
                style(suggestion).cyan()
            )
        } else {
            format!("{icon} Suggestion:\n  {suggestion}")
        }
    }

    /// Formats next steps with appropriate styling.
    fn format_next_steps(&self, steps: &[String]) -> String {
        let icon = "→";
        let mut output = String::new();

        if self.use_colors {
            let _ = writeln!(
                output,
                "{} {}",
                style(icon).yellow().bold(),
                style("Next steps:").yellow().bold()
            );

            for (i, step) in steps.iter().enumerate() {
                let _ = writeln!(output, "  {}. {}", i + 1, style(step).yellow());
            }
        } else {
            let _ = writeln!(output, "{icon} Next steps:");

            for (i, step) in steps.iter().enumerate() {
                let _ = writeln!(output, "  {}. {step}", i + 1);
            }
        }

        output.trim_end().to_string()
    }

    /// Gets a contextual suggestion based on the error type.
    fn get_suggestion(&self) -> Option<String> {
        match self.error {
            CliError::Configuration(msg) if msg.contains("not found") => {
                Some("Run 'wnt init' to create a new configuration file.".to_string())
            }
            CliError::Configuration(msg)
                if msg.to_lowercase().contains("invalid")
                    || msg.to_lowercase().contains("parsing") =>
            {
                Some(
                    "Run 'wnt config validate' to check your configuration for errors.".to_string(),
                )
            }
            CliError::Git(msg)
                if msg.contains("not found") || msg.contains("not a git repository") =>
            {
                Some("Make sure you're running this command in a git repository.".to_string())
            }
            CliError::Package(msg) if msg.contains("package.json") && msg.contains("not found") => {
                Some(
                    "Make sure you're in a Node.js project directory with a package.json file."
                        .to_string(),
                )
            }
            CliError::Network(msg) if msg.contains("registry") || msg.contains("unreachable") => {
                Some("Check your internet connection and try again.".to_string())
            }
            CliError::Io(msg) if msg.contains("permission") || msg.contains("denied") => Some(
                "Check file permissions or try running with appropriate privileges.".to_string(),
            ),
            CliError::Validation(msg) if msg.contains("version") => {
                Some("Version must be in semver format (e.g., 1.0.0).".to_string())
            }
            _ => None,
        }
    }

    /// Gets next steps based on the error type.
    fn get_next_steps(&self) -> Option<Vec<String>> {
        match self.error {
            CliError::Configuration(msg) if msg.contains("not found") => Some(vec![
                "Run 'wnt init' to create a configuration file".to_string(),
                "Or specify a config path with --config".to_string(),
            ]),
            CliError::Configuration(msg)
                if msg.to_lowercase().contains("invalid")
                    || msg.to_lowercase().contains("parsing") =>
            {
                Some(vec![
                    "Check the configuration file syntax".to_string(),
                    "Run 'wnt config validate' for details".to_string(),
                    "See the documentation for valid configuration options".to_string(),
                ])
            }
            CliError::Git(msg)
                if msg.contains("not found") || msg.contains("not a git repository") =>
            {
                Some(vec![
                    "Initialize a git repository with 'git init'".to_string(),
                    "Or ensure you're in the correct directory".to_string(),
                ])
            }
            CliError::Package(msg) if msg.contains("package.json") && msg.contains("not found") => {
                Some(vec![
                    "Initialize a Node.js project with 'npm init'".to_string(),
                    "Or ensure you're in the correct directory".to_string(),
                ])
            }
            CliError::Network(_) => Some(vec![
                "Check your internet connection".to_string(),
                "Verify the registry URL in your configuration".to_string(),
                "Try again later if the registry is temporarily unavailable".to_string(),
            ]),
            CliError::Validation(msg) if msg.contains("version") => Some(vec![
                "Use semver format: MAJOR.MINOR.PATCH (e.g., 1.0.0)".to_string(),
                "See https://semver.org for more information".to_string(),
            ]),
            _ => None,
        }
    }
}

impl std::fmt::Display for ErrorDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_without_colors() {
        let error = CliError::configuration("Test error");
        let display = ErrorDisplay::new(&error, false);
        let formatted = display.format();

        assert!(formatted.contains("Configuration Error"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_format_with_colors() {
        let error = CliError::configuration("Test error");
        let display = ErrorDisplay::new(&error, true);
        let formatted = display.format();

        // When colors are enabled, ANSI codes are included
        // We verify that the output is not empty and contains key text
        assert!(!formatted.is_empty());
        assert!(formatted.contains("Configuration"));
        assert!(formatted.contains("Error"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_suggestion_for_config_not_found() {
        let error = CliError::configuration("Config file not found");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_some());
        if let Some(msg) = suggestion {
            assert!(msg.contains("wnt init"));
        }
    }

    #[test]
    fn test_suggestion_for_invalid_config() {
        let error = CliError::configuration("Invalid configuration");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_some());
        if let Some(msg) = suggestion {
            assert!(msg.contains("wnt config validate"));
        }
    }

    #[test]
    fn test_suggestion_for_git_not_found() {
        let error = CliError::git("not a git repository");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_some());
        if let Some(msg) = suggestion {
            assert!(msg.contains("git repository"));
        }
    }

    #[test]
    fn test_suggestion_for_package_not_found() {
        let error = CliError::package("package.json not found");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_some());
        if let Some(msg) = suggestion {
            assert!(msg.contains("package.json"));
        }
    }

    #[test]
    fn test_suggestion_for_network_error() {
        let error = CliError::network("Registry unreachable");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_some());
        if let Some(msg) = suggestion {
            assert!(msg.contains("internet connection"));
        }
    }

    #[test]
    fn test_suggestion_for_permission_error() {
        let error = CliError::io("Permission denied");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_some());
        if let Some(msg) = suggestion {
            assert!(msg.contains("permission"));
        }
    }

    #[test]
    fn test_suggestion_for_version_validation() {
        let error = CliError::validation("Invalid version format");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_some());
        if let Some(msg) = suggestion {
            assert!(msg.contains("semver"));
        }
    }

    #[test]
    fn test_no_suggestion_for_generic_error() {
        let error = CliError::execution("Something failed");
        let display = ErrorDisplay::new(&error, false);
        let suggestion = display.get_suggestion();

        assert!(suggestion.is_none());
    }

    #[test]
    fn test_next_steps_for_config_not_found() {
        let error = CliError::configuration("Config file not found");
        let display = ErrorDisplay::new(&error, false);
        let steps = display.get_next_steps();

        assert!(steps.is_some());
        if let Some(steps_vec) = steps {
            assert!(!steps_vec.is_empty());
            assert!(steps_vec[0].contains("wnt init"));
        }
    }

    #[test]
    fn test_next_steps_for_git_not_found() {
        let error = CliError::git("not a git repository");
        let display = ErrorDisplay::new(&error, false);
        let steps = display.get_next_steps();

        assert!(steps.is_some());
        if let Some(steps_vec) = steps {
            assert!(!steps_vec.is_empty());
            assert!(steps_vec[0].contains("git init"));
        }
    }

    #[test]
    fn test_next_steps_for_network_error() {
        let error = CliError::network("Connection failed");
        let display = ErrorDisplay::new(&error, false);
        let steps = display.get_next_steps();

        assert!(steps.is_some());
        if let Some(steps_vec) = steps {
            assert!(!steps_vec.is_empty());
        }
    }

    #[test]
    fn test_no_next_steps_for_generic_error() {
        let error = CliError::execution("Something failed");
        let display = ErrorDisplay::new(&error, false);
        let steps = display.get_next_steps();

        assert!(steps.is_none());
    }

    #[test]
    fn test_format_header() {
        let error = CliError::configuration("Test");
        let display = ErrorDisplay::new(&error, false);
        let header = display.format_header();

        assert!(header.contains("Configuration Error"));
    }

    #[test]
    fn test_format_message() {
        let error = CliError::configuration("Test message");
        let display = ErrorDisplay::new(&error, false);
        let message = display.format_message();

        assert!(message.contains("Test message"));
    }

    #[test]
    fn test_display_trait() {
        let error = CliError::configuration("Test");
        let display = ErrorDisplay::new(&error, false);
        let output = format!("{display}");

        assert!(output.contains("Configuration Error"));
        assert!(output.contains("Test"));
    }
}
