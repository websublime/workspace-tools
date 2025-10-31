//! Interactive prompts for CLI commands.
//!
//! This module provides reusable interactive prompt functions using dialoguer
//! for user input in interactive mode.
//!
//! # What
//!
//! Provides interactive prompts for:
//! - Package selection (multi-select)
//! - Bump type selection (single select)
//! - Environment selection (multi-select)
//! - Summary/message text input
//! - Confirmation dialogs
//!
//! # How
//!
//! Uses the `dialoguer` crate to create interactive prompts with:
//! - Clear labels and help text
//! - Default values where appropriate
//! - Input validation
//! - Keyboard navigation support
//! - Color-coded display (respecting NO_COLOR)
//!
//! All prompts are designed to be:
//! - User-friendly with clear instructions
//! - Cancelable (Ctrl+C returns error)
//! - Accessible (work in various terminal environments)
//!
//! # Why
//!
//! Centralizing interactive prompts provides:
//! - Consistent UX across all interactive commands
//! - Reusable prompt implementations
//! - Easy testing with mock implementations
//! - Proper error handling for user cancellation
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::interactive::prompts::{prompt_bump_type, prompt_summary};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Prompt for bump type
//! let bump = prompt_bump_type(false)?;
//! println!("Selected bump type: {}", bump);
//!
//! // Prompt for summary
//! let summary = prompt_summary(None, false)?;
//! println!("Summary: {}", summary);
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};

/// Prompts user to select a bump type.
///
/// Displays a single-select menu with the three bump types: patch, minor, and major.
/// Includes helpful descriptions for each type following semantic versioning.
///
/// # Arguments
///
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<String>` - The selected bump type ("patch", "minor", or "major")
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::prompts::prompt_bump_type;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let bump = prompt_bump_type(false)?;
/// assert!(["patch", "minor", "major"].contains(&bump.as_str()));
/// # Ok(())
/// # }
/// ```
pub fn prompt_bump_type(no_color: bool) -> Result<String> {
    let items = vec![
        "patch - Bug fixes and small changes (0.0.X)",
        "minor - New features, backwards compatible (0.X.0)",
        "major - Breaking changes (X.0.0)",
    ];

    let selection = if no_color {
        Select::new().with_prompt("Select bump type").items(&items).default(0).interact()
    } else {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select bump type")
            .items(&items)
            .default(0)
            .interact()
    };

    match selection {
        Ok(0) => Ok("patch".to_string()),
        Ok(1) => Ok("minor".to_string()),
        Ok(2) => Ok("major".to_string()),
        Ok(_) => Err(CliError::user("Invalid selection")),
        Err(e) => Err(CliError::user(format!("Prompt cancelled: {e}"))),
    }
}

/// Prompts user to select packages from a list.
///
/// Displays a multi-select menu allowing the user to choose one or more packages.
/// Pre-selects packages that are detected as changed from git if provided.
///
/// # Arguments
///
/// * `packages` - List of all available package names
/// * `detected_packages` - List of packages detected as changed (will be pre-selected)
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<Vec<String>>` - List of selected package names
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
/// - No packages are selected
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::prompts::prompt_packages;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let all_packages = vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-c".to_string()];
/// let detected = vec!["pkg-a".to_string()];
/// let selected = prompt_packages(&all_packages, &detected, false)?;
/// assert!(!selected.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn prompt_packages(
    packages: &[String],
    detected_packages: &[String],
    no_color: bool,
) -> Result<Vec<String>> {
    if packages.is_empty() {
        return Err(CliError::validation("No packages available to select"));
    }

    // Create defaults array - mark detected packages as selected
    let defaults: Vec<bool> = packages.iter().map(|pkg| detected_packages.contains(pkg)).collect();

    let prompt_text = if detected_packages.is_empty() {
        "Select packages (space to select, enter to confirm)".to_string()
    } else {
        format!(
            "Select packages (detected {} from git changes, space to toggle, enter to confirm)",
            detected_packages.len()
        )
    };

    let selections = if no_color {
        MultiSelect::new().with_prompt(&prompt_text).items(packages).defaults(&defaults).interact()
    } else {
        MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(&prompt_text)
            .items(packages)
            .defaults(&defaults)
            .interact()
    };

    match selections {
        Ok(indices) => {
            if indices.is_empty() {
                Err(CliError::validation("At least one package must be selected"))
            } else {
                Ok(indices.iter().map(|&i| packages[i].clone()).collect())
            }
        }
        Err(e) => Err(CliError::user(format!("Prompt cancelled: {e}"))),
    }
}

/// Prompts user to select environments from a list.
///
/// Displays a multi-select menu allowing the user to choose one or more environments.
/// Pre-selects default environments if provided.
///
/// # Arguments
///
/// * `environments` - List of all available environment names
/// * `default_environments` - List of default environments (will be pre-selected)
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<Vec<String>>` - List of selected environment names
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
/// - No environments are selected
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::prompts::prompt_environments;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let envs = vec!["dev".to_string(), "staging".to_string(), "prod".to_string()];
/// let defaults = vec!["staging".to_string(), "prod".to_string()];
/// let selected = prompt_environments(&envs, &defaults, false)?;
/// assert!(!selected.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn prompt_environments(
    environments: &[String],
    default_environments: &[String],
    no_color: bool,
) -> Result<Vec<String>> {
    if environments.is_empty() {
        return Err(CliError::validation("No environments available to select"));
    }

    // Create defaults array - mark default environments as selected
    let defaults: Vec<bool> =
        environments.iter().map(|env| default_environments.contains(env)).collect();

    let prompt_text = if default_environments.is_empty() {
        "Select environments (space to select, enter to confirm)".to_string()
    } else {
        format!(
            "Select environments (defaults: {}, space to toggle, enter to confirm)",
            default_environments.join(", ")
        )
    };

    let selections = if no_color {
        MultiSelect::new()
            .with_prompt(&prompt_text)
            .items(environments)
            .defaults(&defaults)
            .interact()
    } else {
        MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(&prompt_text)
            .items(environments)
            .defaults(&defaults)
            .interact()
    };

    match selections {
        Ok(indices) => {
            if indices.is_empty() {
                Err(CliError::validation("At least one environment must be selected"))
            } else {
                Ok(indices.iter().map(|&i| environments[i].clone()).collect())
            }
        }
        Err(e) => Err(CliError::user(format!("Prompt cancelled: {e}"))),
    }
}

/// Prompts user to enter a summary message.
///
/// Displays a text input prompt for entering a changeset summary or message.
/// Provides a helpful placeholder and validates that input is not empty.
///
/// # Arguments
///
/// * `placeholder` - Optional placeholder text to show in the input
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<String>` - The entered summary text
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
/// - Input is empty after trimming
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::prompts::prompt_summary;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let summary = prompt_summary(Some("Describe your changes"), false)?;
/// assert!(!summary.trim().is_empty());
/// # Ok(())
/// # }
/// ```
pub fn prompt_summary(placeholder: Option<&str>, no_color: bool) -> Result<String> {
    let prompt_text = "Enter changeset summary";
    let default_placeholder = "Brief description of changes";

    let input = if no_color {
        Input::<String>::new()
            .with_prompt(prompt_text)
            .with_initial_text(placeholder.unwrap_or(default_placeholder))
            .allow_empty(false)
            .interact_text()
    } else {
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt_text)
            .with_initial_text(placeholder.unwrap_or(default_placeholder))
            .allow_empty(false)
            .interact_text()
    };

    match input {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                Err(CliError::validation("Summary cannot be empty"))
            } else {
                Ok(trimmed)
            }
        }
        Err(e) => Err(CliError::user(format!("Prompt cancelled: {e}"))),
    }
}

/// Prompts user for a yes/no confirmation.
///
/// Displays a confirmation prompt with a default value.
///
/// # Arguments
///
/// * `message` - The confirmation message to display
/// * `default` - The default value if user just presses enter
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<bool>` - True if user confirmed, false otherwise
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::prompts::prompt_confirm;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let confirmed = prompt_confirm("Proceed with operation?", true, false)?;
/// if confirmed {
///     println!("Proceeding...");
/// }
/// # Ok(())
/// # }
/// ```
pub fn prompt_confirm(message: &str, default: bool, no_color: bool) -> Result<bool> {
    let result = if no_color {
        Confirm::new().with_prompt(message).default(default).interact()
    } else {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .default(default)
            .interact()
    };

    result.map_err(|e| CliError::user(format!("Prompt cancelled: {e}")))
}
