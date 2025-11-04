//! Enhanced confirmation prompts.
//!
//! This module provides enhanced confirmation dialogs with better context,
//! visual indicators, and user-friendly prompts.

#![allow(clippy::print_stdout)]
//!
//! # What
//!
//! Provides:
//! - Simple yes/no confirmation prompts
//! - Contextual confirmations with details
//! - Dangerous operation confirmations with warnings
//! - Custom styled confirmation dialogs
//!
//! # How
//!
//! Uses:
//! - `dialoguer::Confirm` for yes/no prompts
//! - Custom theme for consistent styling
//! - Contextual information to help users make informed decisions
//! - Clear default values to prevent accidental actions
//!
//! The confirmation functions provide clear visual feedback and sensible
//! defaults to help users make informed decisions, especially for potentially
//! dangerous operations.
//!
//! # Why
//!
//! Enhanced confirmations improve safety and UX by:
//! - Preventing accidental destructive actions
//! - Providing context for the confirmation
//! - Using clear visual indicators
//! - Following consistent patterns
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::interactive::confirm::{confirm, confirm_dangerous};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Simple confirmation
//! if confirm("Proceed with operation?", true, false)? {
//!     println!("Proceeding...");
//! }
//!
//! // Dangerous operation confirmation
//! if confirm_dangerous("Delete all changesets?", "This action cannot be undone", false)? {
//!     println!("Deleting...");
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use crate::interactive::theme::WntTheme;
use dialoguer::Confirm;

/// Prompts user for a yes/no confirmation.
///
/// Displays a simple confirmation prompt with a default value.
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
/// use sublime_cli_tools::interactive::confirm::confirm;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// if confirm("Continue?", true, false)? {
///     println!("Continuing...");
/// } else {
///     println!("Cancelled");
/// }
/// # Ok(())
/// # }
/// ```
pub fn confirm(message: &str, default: bool, no_color: bool) -> Result<bool> {
    let theme = WntTheme::new(no_color);

    Confirm::with_theme(&theme)
        .with_prompt(message)
        .default(default)
        .interact()
        .map_err(|e| CliError::user(format!("Confirmation cancelled: {e}")))
}

/// Prompts user for confirmation with additional context.
///
/// Displays a confirmation prompt with explanatory text to help users
/// make informed decisions.
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
/// use sublime_cli_tools::interactive::confirm::confirm_with_context;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let confirmed = confirm_with_context(
///     "Apply upgrades?",
///     "This will modify 3 package.json files",
///     false,
///     false,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn confirm_with_context(
    message: &str,
    context: &str,
    default: bool,
    no_color: bool,
) -> Result<bool> {
    // Print context before showing confirmation
    if no_color {
        println!("{context}");
    } else {
        use console::style;
        println!("{}", style(context).dim());
    }
    println!();

    confirm(message, default, no_color)
}

/// Prompts user for confirmation of a dangerous operation.
///
/// Displays a prominent warning before asking for confirmation. The default
/// is always `false` to prevent accidental destructive actions.
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
/// use sublime_cli_tools::interactive::confirm::confirm_dangerous;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let confirmed = confirm_dangerous(
///     "Delete all changesets?",
///     "This action cannot be undone. All changeset data will be permanently removed.",
///     false,
/// )?;
///
/// if confirmed {
///     println!("Deleting changesets...");
/// }
/// # Ok(())
/// # }
/// ```
pub fn confirm_dangerous(message: &str, warning: &str, no_color: bool) -> Result<bool> {
    // Print warning prominently
    if no_color {
        println!("WARNING: {warning}");
    } else {
        use console::style;
        println!("{} {}", style("⚠").red().bold(), style(warning).red().bold());
    }
    println!();

    // Always default to false for dangerous operations
    confirm(message, false, no_color)
}

/// Prompts user to confirm an operation that will affect multiple items.
///
/// Displays a summary of what will be affected and asks for confirmation.
///
/// # Arguments
///
/// * `action` - The action to perform (e.g., "upgrade", "delete")
/// * `items` - The items that will be affected
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
/// use sublime_cli_tools::interactive::confirm::confirm_with_items;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages = vec!["package-a", "package-b", "package-c"];
/// let confirmed = confirm_with_items("upgrade", &packages, true, false)?;
///
/// if confirmed {
///     println!("Upgrading packages...");
/// }
/// # Ok(())
/// # }
/// ```
pub fn confirm_with_items<T: std::fmt::Display>(
    action: &str,
    items: &[T],
    default: bool,
    no_color: bool,
) -> Result<bool> {
    // Print items summary
    if no_color {
        println!("This will {} {} item(s):", action, items.len());
    } else {
        use console::style;
        println!(
            "This will {} {} item(s):",
            style(action).yellow(),
            style(items.len()).cyan().bold()
        );
    }

    for item in items.iter().take(10) {
        println!("  - {item}");
    }

    if items.len() > 10 {
        println!("  ... and {} more", items.len() - 10);
    }
    println!();

    let message = format!("Proceed with {action}?");
    confirm(&message, default, no_color)
}

/// Prompts user to confirm an operation with a detailed summary.
///
/// Displays detailed information about what will happen and asks for confirmation.
///
/// # Arguments
///
/// * `title` - The title of the operation
/// * `details` - A list of detail strings to display
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
/// use sublime_cli_tools::interactive::confirm::confirm_with_details;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let details = vec![
///     "3 packages will be upgraded".to_string(),
///     "2 breaking changes detected".to_string(),
///     "5 files will be modified".to_string(),
/// ];
///
/// let confirmed = confirm_with_details("Apply upgrades", &details, false, false)?;
/// # Ok(())
/// # }
/// ```
pub fn confirm_with_details(
    title: &str,
    details: &[String],
    default: bool,
    no_color: bool,
) -> Result<bool> {
    // Print title
    if no_color {
        println!("{title}:");
    } else {
        use console::style;
        println!("{}:", style(title).cyan().bold());
    }
    println!();

    // Print details
    for detail in details {
        if no_color {
            println!("  • {detail}");
        } else {
            use console::style;
            println!("  {} {detail}", style("•").dim());
        }
    }
    println!();

    let message = format!("Proceed with {}?", title.to_lowercase());
    confirm(&message, default, no_color)
}
