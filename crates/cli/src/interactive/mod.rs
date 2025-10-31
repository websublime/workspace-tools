//! Interactive prompt module for CLI commands.
//!
//! This module provides interactive user input functionality for CLI commands
//! that support interactive mode.
//!
//! # What
//!
//! Provides:
//! - Interactive prompts for user input (multi-select, single-select, text input)
//! - Package selection with git-based detection
//! - Bump type selection with semantic versioning explanations
//! - Environment selection with defaults
//! - Summary/message text input
//! - Confirmation dialogs
//!
//! # How
//!
//! Uses the `dialoguer` crate to create terminal-based interactive prompts with:
//! - Keyboard navigation (arrow keys, space, enter)
//! - Color-coded display (respecting NO_COLOR environment variable)
//! - Clear instructions and help text
//! - Input validation
//! - Cancellation support (Ctrl+C)
//!
//! All prompts are designed to be:
//! - User-friendly with clear instructions
//! - Accessible across different terminal environments
//! - Cancelable with proper error handling
//! - Consistent in appearance and behavior
//!
//! # Why
//!
//! Separating interactive functionality allows:
//! - Reusable prompts across different commands
//! - Consistent UX throughout the CLI
//! - Easy testing with mock implementations
//! - Clear separation between interactive and non-interactive modes
//! - Better maintainability
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::interactive::prompts::{prompt_bump_type, prompt_packages};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Interactive bump type selection
//! let bump = prompt_bump_type(false)?;
//! println!("User selected: {}", bump);
//!
//! // Interactive package selection
//! let packages = vec!["pkg-a".to_string(), "pkg-b".to_string()];
//! let detected = vec!["pkg-a".to_string()];
//! let selected = prompt_packages(&packages, &detected, false)?;
//! println!("Selected packages: {:?}", selected);
//! # Ok(())
//! # }
//! ```

pub mod prompts;

#[cfg(test)]
mod tests;

// Re-export commonly used prompt functions
pub use prompts::{
    prompt_bump_type, prompt_confirm, prompt_environments, prompt_packages, prompt_summary,
};
