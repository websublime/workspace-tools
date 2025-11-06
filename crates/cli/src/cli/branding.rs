//! Branding and visual styling for the CLI.
//!
//! This module provides ASCII art, custom styling, and visual branding
//! elements for the Workspace Tools CLI.
//!
//! # What
//!
//! Provides:
//! - ASCII art header/logo for the CLI
//! - Custom clap styling for modern, minimal appearance
//! - Separator lines and visual elements
//! - Version and branding information formatting
//!
//! # How
//!
//! Uses:
//! - Raw string literals for ASCII art
//! - `clap::builder::styling::Styles` for CLI output customization
//! - ANSI color codes for terminal styling
//! - Unicode box-drawing characters for separators
//!
//! # Why
//!
//! Creates a professional, distinctive brand identity for the CLI tool,
//! improving user experience and making the tool more memorable and
//! recognizable in the terminal.
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::cli::branding::{print_header, CLAP_STYLING};
//!
//! // Print the ASCII art header
//! print_header("1.0.0");
//!
//! // Use custom styling in clap
//! # use clap::Parser;
//! #[derive(Parser)]
//! #[command(styles = CLAP_STYLING)]
//! struct MyCli {
//!     // ...
//! }
//! ```

use clap::builder::styling::{AnsiColor, Color, Style, Styles};

/// ASCII art logo for Workspace Tools.
///
/// This is displayed at the top of help messages and when explicitly
/// requested. The art spells out "WORKSPACE" in a distinctive
/// ASCII font style.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::cli::branding::ASCII_LOGO;
///
/// println!("{}", ASCII_LOGO);
/// ```
pub const ASCII_LOGO: &str = r"
░█░█░█▀█░█▀▄░█░█░█▀▀░█▀█░█▀█░█▀▀░█▀▀
░█▄█░█░█░█▀▄░█▀▄░▀▀█░█▀▀░█▀█░█░█░█▀▀
░▀░▀░▀▀▀░▀░▀░▀░▀░▀▀▀░▀░░░▀░▀░▀▀▀░▀▀▀
";

/// Separator line using box-drawing characters.
///
/// Creates a horizontal line for visual separation in terminal output.
/// Uses Unicode box-drawing characters for a clean, modern appearance.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::cli::branding::SEPARATOR;
///
/// println!("{}", SEPARATOR);
/// ```
pub const SEPARATOR: &str = "⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯";

/// Short name abbreviation for the CLI.
pub const SHORT_NAME: &str = "workspace";

/// Full name of the tool.
pub const FULL_NAME: &str = "Workspace Tools";

/// Custom clap styling for modern, minimal appearance.
pub const CLAP_STYLING: Styles = Styles::styled()
    .header(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
    .usage(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
    .literal(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
    .error(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Red))))
    .valid(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .invalid(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Red))));

/// Prints the CLI header with ASCII art, separator, and version.
#[allow(clippy::print_stdout)]
pub fn print_header(version: &str) {
    println!("{ASCII_LOGO}");
    println!("{SEPARATOR}");
    println!("{SHORT_NAME}: {version}");
    println!();
}

/// Prints just the separator line.
#[allow(clippy::print_stdout)]
pub fn print_separator() {
    println!("{SEPARATOR}");
}

/// Formats a version string with the short name.
#[must_use]
pub fn format_version(version: &str) -> String {
    format!("{SHORT_NAME}: {version}")
}

/// Formats a full title with name and version.
#[must_use]
pub fn format_full_title(version: &str) -> String {
    format!("{FULL_NAME} v{version}")
}

/// Returns a styled header string without printing it.
#[must_use]
pub fn get_header(version: &str) -> String {
    format!("{ASCII_LOGO}\n{SEPARATOR}\n{SHORT_NAME}: {version}\n")
}
