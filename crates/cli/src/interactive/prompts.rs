//! Interactive prompts for CLI commands.
//!
//! This module provides reusable interactive prompt functions with enhanced
//! features including fuzzy search, better validation, and improved visual feedback.
//!
//! # What
//!
//! Provides interactive prompts for:
//! - Package selection (multi-select with fuzzy search)
//! - Bump type selection (single select with descriptions)
//! - Environment selection (multi-select with fuzzy search)
//! - Summary/message text input (with validation)
//! - Confirmation dialogs (with context and warnings)
//!
//! # How
//!
//! Uses enhanced modules:
//! - `select` for fuzzy search and multi-select
//! - `confirm` for styled confirmation dialogs
//! - `validation` for input validation with helpful messages
//! - `theme` for consistent visual styling
//!
//! All prompts are designed to be:
//! - User-friendly with clear instructions
//! - Fast and responsive with fuzzy search
//! - Visually appealing with custom theme
//! - Accessible (work in various terminal environments)
//! - Cancelable (Ctrl+C returns error)
//!
//! # Why
//!
//! Centralizing interactive prompts provides:
//! - Consistent UX across all interactive commands
//! - Reusable prompt implementations
//! - Easy testing with mock implementations
//! - Proper error handling for user cancellation
//! - Better user experience through enhanced features
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
use crate::interactive::{confirm, select, theme::WntTheme, validation};
use dialoguer::Input;

/// Prompts user to select a bump type with enhanced UI.
///
/// Displays a single-select menu with the three bump types: patch, minor, and major.
/// Includes helpful descriptions for each type following semantic versioning.
/// Uses the custom theme for consistent styling.
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

    let prompt_text = "Select bump type";
    let selection = select::simple_select(prompt_text, &items, Some(0), no_color)?;

    match selection {
        0 => Ok("patch".to_string()),
        1 => Ok("minor".to_string()),
        2 => Ok("major".to_string()),
        _ => Err(CliError::user("Invalid selection")),
    }
}

/// Prompts user to select packages from a list with fuzzy search.
///
/// Displays an enhanced multi-select menu with fuzzy search capability,
/// allowing users to quickly find and select packages. Pre-selects packages
/// that are detected as changed from git if provided.
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
///
/// Returns `CliError::Validation` if:
/// - No packages are available
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

    let prompt_text = if detected_packages.is_empty() {
        "Select packages (space to select, enter to confirm)".to_string()
    } else {
        format!(
            "Select packages (detected {} from git changes, space to toggle, enter to confirm)",
            detected_packages.len()
        )
    };

    let selected = select::select_packages(&prompt_text, packages, detected_packages, no_color)?;

    // Validate selection
    validation::validate_at_least_one_selected(&(0..selected.len()).collect::<Vec<_>>())?;

    Ok(selected)
}

/// Prompts user to select environments from a list with fuzzy search.
///
/// Displays an enhanced multi-select menu with fuzzy search capability.
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
///
/// Returns `CliError::Validation` if:
/// - No environments are available
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

    let prompt_text = if default_environments.is_empty() {
        "Select environments (space to select, enter to confirm)".to_string()
    } else {
        format!(
            "Select environments (defaults: {}, space to toggle, enter to confirm)",
            default_environments.join(", ")
        )
    };

    let selected =
        select::select_environments(&prompt_text, environments, default_environments, no_color)?;

    // Validate selection
    validation::validate_at_least_one_selected(&(0..selected.len()).collect::<Vec<_>>())?;

    Ok(selected)
}

/// Prompts user to enter a summary message with validation.
///
/// Displays a text input prompt for entering a changeset summary or message.
/// Provides a helpful placeholder and validates that input is not empty.
/// The input is validated in real-time to provide immediate feedback.
///
/// # Arguments
///
/// * `placeholder` - Optional placeholder text to show in the input
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<String>` - The entered summary text (trimmed)
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
///
/// Returns `CliError::Validation` if:
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
    let theme = WntTheme::new(no_color);
    let prompt_text = "Enter changeset summary";
    let default_placeholder = "Brief description of changes";

    let input = Input::<String>::with_theme(&theme)
        .with_prompt(prompt_text)
        .with_initial_text(placeholder.unwrap_or(default_placeholder))
        .allow_empty(false)
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            validation::validate_non_empty(input).map_err(|e| e.to_string())
        })
        .interact_text()
        .map_err(|e| CliError::user(format!("Prompt cancelled: {e}")))?;

    let trimmed = input.trim().to_string();

    // Final validation
    validation::validate_non_empty(&trimmed)?;

    Ok(trimmed)
}

/// Prompts user for a yes/no confirmation with enhanced styling.
///
/// Displays a confirmation prompt with a default value using the custom theme.
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
    confirm::confirm(message, default, no_color)
}

/// Prompts user for confirmation with additional context.
///
/// Displays contextual information before asking for confirmation.
///
/// # Arguments
///
/// * `message` - The main confirmation message
/// * `context` - Additional context or explanation
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
/// use sublime_cli_tools::interactive::prompts::prompt_confirm_with_context;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let confirmed = prompt_confirm_with_context(
///     "Apply upgrades?",
///     "This will modify 3 package.json files",
///     false,
///     false,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn prompt_confirm_with_context(
    message: &str,
    context: &str,
    default: bool,
    no_color: bool,
) -> Result<bool> {
    confirm::confirm_with_context(message, context, default, no_color)
}

/// Prompts user for confirmation of a dangerous operation.
///
/// Displays a prominent warning before asking for confirmation.
///
/// # Arguments
///
/// * `message` - The confirmation message
/// * `warning` - The warning text to display
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
/// use sublime_cli_tools::interactive::prompts::prompt_confirm_dangerous;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let confirmed = prompt_confirm_dangerous(
///     "Delete all changesets?",
///     "This action cannot be undone",
///     false,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn prompt_confirm_dangerous(message: &str, warning: &str, no_color: bool) -> Result<bool> {
    confirm::confirm_dangerous(message, warning, no_color)
}
