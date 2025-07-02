//! CLI utility functions and helpers
//!
//! Common utility functions used across CLI commands including
//! user interaction, file operations, and formatting helpers.

use crate::error::{CliError, CliResult};
use std::io::{self, Write};
use std::path::Path;

/// Prompt user for confirmation
///
/// # Arguments
///
/// * `message` - The confirmation message to display
/// * `default` - Default response if user just presses Enter
///
/// # Returns
///
/// True if user confirms, false otherwise
pub fn confirm(message: &str, default: bool) -> CliResult<bool> {
    let default_indicator = if default { "[Y/n]" } else { "[y/N]" };
    print!("{} {}: ", message, default_indicator);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();

    match input.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        "" => Ok(default),
        _ => {
            println!("Please answer yes or no.");
            confirm(message, default)
        }
    }
}

/// Prompt user for input with validation
///
/// # Arguments
///
/// * `message` - The prompt message
/// * `validator` - Function to validate the input
///
/// # Returns
///
/// The validated user input
pub fn prompt_with_validation<F>(message: &str, validator: F) -> CliResult<String>
where
    F: Fn(&str) -> Result<(), String>,
{
    loop {
        print!("{}: ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match validator(input) {
            Ok(()) => return Ok(input.to_string()),
            Err(error) => println!("Invalid input: {}", error),
        }
    }
}

/// Check if a path is within the current working directory or its subdirectories
pub fn is_safe_path(path: &Path) -> bool {
    if path.is_absolute() {
        return false;
    }

    // Check for path traversal attempts
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }

    true
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let unit_index = (bytes as f64).log10() as usize / 3;
    let unit_index = unit_index.min(UNITS.len() - 1);

    let size = bytes as f64 / 1000_f64.powi(unit_index as i32);

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();

    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        if seconds == 0 {
            format!("{}m", minutes)
        } else {
            format!("{}m {}s", minutes, seconds)
        }
    } else {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        if minutes == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, minutes)
        }
    }
}

/// Truncate text to a maximum width, adding ellipsis if needed
pub fn truncate_text(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &text[..max_width - 3])
    }
}

/// Detect terminal width for formatting
pub fn terminal_width() -> usize {
    // Try to get terminal width from environment or use default
    if let Some((width, _)) = term_size::dimensions() {
        width
    } else {
        80 // Default fallback width
    }
}

/// Create a progress bar string
pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "░".repeat(width);
    }

    let filled = (current * width) / total;
    let empty = width - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Validate that a string is a valid package name
pub fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Package name cannot be empty".to_string());
    }

    if name.starts_with('.') || name.starts_with('_') {
        return Err("Package name cannot start with '.' or '_'".to_string());
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(
            "Package name can only contain alphanumeric characters, hyphens, underscores, and dots"
                .to_string(),
        );
    }

    Ok(())
}

/// Validate that a string is a valid version
pub fn validate_version(version: &str) -> Result<(), String> {
    // Basic semver validation
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        return Err("Version must be in format MAJOR.MINOR.PATCH".to_string());
    }

    for part in parts {
        if part.parse::<u32>().is_err() {
            return Err("Version parts must be numbers".to_string());
        }
    }

    Ok(())
}

/// Escape shell arguments
pub fn escape_shell_arg(arg: &str) -> String {
    if arg.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/') {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\"'\"'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path(Path::new("src/lib.rs")));
        assert!(is_safe_path(Path::new("relative/path")));
        assert!(!is_safe_path(Path::new("/absolute/path")));
        assert!(!is_safe_path(Path::new("../parent")));
        assert!(!is_safe_path(Path::new("src/../parent")));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1_000_000), "1.0 MB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(std::time::Duration::from_secs(3900)), "1h 5m");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(truncate_text("this is a long text", 10), "this is...");
        assert_eq!(truncate_text("tiny", 3), "...");
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(0, 10, 5), "░░░░░");
        assert_eq!(progress_bar(5, 10, 5), "██░░░");
        assert_eq!(progress_bar(10, 10, 5), "█████");
        assert_eq!(progress_bar(1, 0, 5), "░░░░░"); // Edge case: total = 0
    }

    #[test]
    fn test_validate_package_name() {
        assert!(validate_package_name("valid-name").is_ok());
        assert!(validate_package_name("valid_name").is_ok());
        assert!(validate_package_name("valid.name").is_ok());
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name(".invalid").is_err());
        assert!(validate_package_name("_invalid").is_err());
        assert!(validate_package_name("invalid@name").is_err());
    }

    #[test]
    fn test_validate_version() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("10.5.2").is_ok());
        assert!(validate_version("1.0").is_err());
        assert!(validate_version("1.0.0.0").is_err());
        assert!(validate_version("1.a.0").is_err());
    }

    #[test]
    fn test_escape_shell_arg() {
        assert_eq!(escape_shell_arg("simple"), "simple");
        assert_eq!(escape_shell_arg("with-dash"), "with-dash");
        assert_eq!(escape_shell_arg("with space"), "'with space'");
        assert_eq!(escape_shell_arg("with'quote"), "'with'\"'\"'quote'");
    }
}
