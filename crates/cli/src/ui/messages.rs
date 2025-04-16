//! Inline message formatting utilities.
//!
//! Provides functions for formatting inline messages with appropriate
//! styling and prefixes.

use super::symbols::Symbol;
use super::theme;

/// Format an info message
pub fn info(message: &str) -> String {
    format!("{} {}", theme::info_style(&Symbol::info().to_string()), message)
}

/// Format a success message
pub fn success(message: &str) -> String {
    format!("{} {}", theme::success_style(&Symbol::success().to_string()), message)
}

/// Format a warning message
pub fn warning(message: &str) -> String {
    format!("{} {}", theme::warning_style(&Symbol::warning().to_string()), message)
}

/// Format an error message
pub fn error(message: &str) -> String {
    format!("{} {}", theme::error_style(&Symbol::error().to_string()), message)
}

/// Format a message with the primary color
pub fn primary(message: &str) -> String {
    theme::primary_style(message).to_string()
}

/// Format a message with the secondary color
pub fn secondary(message: &str) -> String {
    theme::secondary_style(message).to_string()
}

/// Format a message with the highlight color
pub fn highlight(message: &str) -> String {
    theme::highlight_style(message).to_string()
}

/// Format a message with the muted color
pub fn muted(message: &str) -> String {
    theme::muted_style(message).to_string()
}

/// Format a message for command example
pub fn command_example(command: &str) -> String {
    format!("$ {}", theme::primary_style(command))
}

/// Format a file path
pub fn file_path(path: &str) -> String {
    theme::secondary_style(path).to_string()
}

/// Format a key-value pair
pub fn key_value(key: &str, value: &str) -> String {
    format!("{}: {}", theme::primary_style(key), value)
}

/// Format a section header
pub fn section_header(title: &str) -> String {
    format!(
        "\n{}\n{}\n",
        theme::highlight_style(title),
        theme::muted_style(&"─".repeat(title.len()))
    )
}
