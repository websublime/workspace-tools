//! Interactive prompt module for CLI commands.
//!
//! This module provides enhanced interactive user input functionality with
//! fuzzy search, better validation, custom themes, and improved user experience.
//!
//! # What
//!
//! Provides:
//! - Interactive prompts for user input (multi-select, single-select, text input)
//! - Fuzzy search for package and environment selection
//! - Enhanced visual feedback with custom theme
//! - Real-time validation with helpful error messages
//! - Confirmation dialogs with context and warnings
//! - Package selection with git-based detection
//! - Bump type selection with semantic versioning explanations
//! - Environment selection with defaults
//! - Summary/message text input
//!
//! # How
//!
//! Uses multiple specialized modules:
//! - `theme`: Custom styled theme for consistent visual appearance
//! - `validation`: Enhanced validation logic with helpful suggestions
//! - `select`: Fuzzy search and enhanced selection capabilities
//! - `confirm`: Enhanced confirmation dialogs with context
//! - `prompts`: High-level prompt functions combining all features
//!
//! The module is built on:
//! - `dialoguer` crate for terminal-based interactive prompts
//! - `fuzzy-matcher` for fuzzy string matching
//! - `console` for terminal styling
//!
//! All prompts are designed to be:
//! - User-friendly with clear instructions
//! - Fast and responsive with fuzzy search
//! - Accessible across different terminal environments
//! - Cancelable with proper error handling (Ctrl+C)
//! - Consistent in appearance and behavior
//!
//! # Why
//!
//! Separating interactive functionality into specialized modules allows:
//! - Reusable components across different commands
//! - Consistent UX throughout the CLI
//! - Easy testing with mock implementations
//! - Clear separation between interactive and non-interactive modes
//! - Better maintainability and extensibility
//! - Enhanced user experience with modern features
//!
//! # Examples
//!
//! ## Using High-Level Prompts
//!
//! ```rust,no_run
//! use sublime_cli_tools::interactive::prompts::{
//!     prompt_bump_type,
//!     prompt_packages,
//!     prompt_environments,
//!     prompt_summary,
//!     prompt_confirm,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Interactive bump type selection
//! let bump = prompt_bump_type(false)?;
//! println!("User selected: {}", bump);
//!
//! // Interactive package selection with fuzzy search
//! let packages = vec!["pkg-a".to_string(), "pkg-b".to_string()];
//! let detected = vec!["pkg-a".to_string()];
//! let selected = prompt_packages(&packages, &detected, false)?;
//! println!("Selected packages: {:?}", selected);
//!
//! // Interactive environment selection
//! let envs = vec!["dev".to_string(), "staging".to_string(), "prod".to_string()];
//! let defaults = vec!["staging".to_string()];
//! let selected_envs = prompt_environments(&envs, &defaults, false)?;
//!
//! // Text input with validation
//! let summary = prompt_summary(None, false)?;
//!
//! // Confirmation with context
//! if prompt_confirm("Proceed?", true, false)? {
//!     println!("Proceeding...");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Low-Level Components
//!
//! ```rust,no_run
//! use sublime_cli_tools::interactive::{select, confirm, validation, theme};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use fuzzy select directly
//! let items = vec!["item1", "item2", "item3"];
//! let selection = select::fuzzy_select("Choose", &items, Some(0), false)?;
//!
//! // Use validation directly
//! validation::validate_non_empty("user input")?;
//!
//! // Use confirm with items
//! let packages = vec!["pkg-a", "pkg-b"];
//! confirm::confirm_with_items("upgrade", &packages, true, false)?;
//!
//! // Use custom theme
//! let theme = theme::WntTheme::new(false);
//! # Ok(())
//! # }
//! ```

// Module declarations
pub mod confirm;
pub mod prompts;
pub mod select;
pub mod theme;
pub mod validation;

#[cfg(test)]
mod tests;

// Re-export commonly used prompt functions for convenience
pub use prompts::{
    prompt_bump_type, prompt_confirm, prompt_confirm_dangerous, prompt_confirm_with_context,
    prompt_environments, prompt_packages, prompt_summary,
};

// Re-export theme for external use
pub use theme::WntTheme;

// Re-export select functions for advanced usage
pub use select::{
    fuzzy_filter, fuzzy_multi_select, fuzzy_select, select_environments, select_packages,
};

// Re-export confirm functions for advanced usage
pub use confirm::{
    confirm, confirm_dangerous, confirm_with_context, confirm_with_details, confirm_with_items,
};

// Re-export validation functions for external use
pub use validation::{
    validate_at_least_one_selected, validate_bump_type, validate_environment_names,
    validate_non_empty, validate_package_names,
};
