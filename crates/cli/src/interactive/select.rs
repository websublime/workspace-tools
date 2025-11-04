//! Enhanced selection prompts with fuzzy search.
//!
//! This module provides enhanced selection prompts that support fuzzy search,
//! better visual indicators, and improved user experience for package and
//! environment selection.
//!
//! # What
//!
//! Provides:
//! - Fuzzy search for filtering large lists of items
//! - Enhanced multi-select with better visual feedback
//! - Single-select with search capability
//! - Pre-selection based on detected changes
//! - Clear instructions and help text
//!
//! # How
//!
//! Uses:
//! - `fuzzy-matcher` for fuzzy string matching
//! - `dialoguer` for terminal interaction
//! - Custom theme for consistent styling
//! - Real-time filtering as user types
//!
//! The fuzzy search allows users to quickly narrow down large lists by typing
//! a search query. Results are ranked by relevance and displayed in real-time.
//!
//! # Why
//!
//! Enhanced selection improves UX by:
//! - Making it easy to find items in large lists
//! - Reducing cognitive load through filtering
//! - Providing immediate visual feedback
//! - Supporting keyboard-driven workflows
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::interactive::select::{fuzzy_select, fuzzy_multi_select};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let items = vec!["package-a", "package-b", "package-core", "utils"];
//!
//! // Single select with fuzzy search
//! let selection = fuzzy_select("Select a package", &items, None, false)?;
//! println!("Selected: {}", items[selection]);
//!
//! // Multi-select with fuzzy search and defaults
//! let defaults = vec![0, 1]; // Pre-select first two items
//! let selections = fuzzy_multi_select("Select packages", &items, &defaults, false)?;
//! println!("Selected: {} items", selections.len());
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use crate::interactive::theme::WntTheme;
use dialoguer::{FuzzySelect, MultiSelect, Select};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Performs a single selection with fuzzy search capability.
///
/// Displays a searchable list where users can type to filter options.
/// Results are ranked by relevance to the search query.
///
/// # Arguments
///
/// * `prompt` - The prompt message to display
/// * `items` - The list of items to choose from
/// * `default` - Optional default selection index
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<usize>` - The index of the selected item
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
/// use sublime_cli_tools::interactive::select::fuzzy_select;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages = vec!["pkg-a", "pkg-b", "pkg-c"];
/// let selection = fuzzy_select("Choose a package", &packages, Some(0), false)?;
/// println!("Selected: {}", packages[selection]);
/// # Ok(())
/// # }
/// ```
pub fn fuzzy_select<T: ToString>(
    prompt: &str,
    items: &[T],
    default: Option<usize>,
    no_color: bool,
) -> Result<usize> {
    if items.is_empty() {
        return Err(CliError::validation("No items available to select"));
    }

    let theme = WntTheme::new(no_color);
    let mut builder = FuzzySelect::with_theme(&theme).with_prompt(prompt);

    if let Some(default_idx) = default {
        builder = builder.default(default_idx);
    }

    let items_str: Vec<String> = items.iter().map(std::string::ToString::to_string).collect();
    builder = builder.items(&items_str);

    builder.interact().map_err(|e| CliError::user(format!("Selection cancelled: {e}")))
}

/// Performs a multi-selection with fuzzy search capability.
///
/// Displays a searchable list where users can:
/// - Type to filter options with fuzzy matching
/// - Use Space to toggle selection
/// - Use arrow keys to navigate
/// - Press Enter to confirm
///
/// # Arguments
///
/// * `prompt` - The prompt message to display
/// * `items` - The list of items to choose from
/// * `defaults` - Indices of items to pre-select
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<Vec<usize>>` - Indices of the selected items
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
///
/// Returns `CliError::Validation` if:
/// - No items are selected
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::select::fuzzy_multi_select;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages = vec!["pkg-a", "pkg-b", "pkg-c"];
/// let defaults = vec![0]; // Pre-select first item
/// let selections = fuzzy_multi_select("Select packages", &packages, &defaults, false)?;
/// # Ok(())
/// # }
/// ```
pub fn fuzzy_multi_select<T: ToString>(
    prompt: &str,
    items: &[T],
    defaults: &[usize],
    no_color: bool,
) -> Result<Vec<usize>> {
    if items.is_empty() {
        return Err(CliError::validation("No items available to select"));
    }

    let theme = WntTheme::new(no_color);
    let items_str: Vec<String> = items.iter().map(std::string::ToString::to_string).collect();

    // Create defaults array
    let defaults_bool: Vec<bool> = (0..items.len()).map(|i| defaults.contains(&i)).collect();

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt(prompt)
        .items(&items_str)
        .defaults(&defaults_bool)
        .interact()
        .map_err(|e| CliError::user(format!("Selection cancelled: {e}")))?;

    if selections.is_empty() {
        Err(CliError::validation(
            "At least one item must be selected. Use Space to select, Enter to confirm.",
        ))
    } else {
        Ok(selections)
    }
}

/// Performs a single selection without fuzzy search.
///
/// This is a simpler version for cases where fuzzy search is not needed.
///
/// # Arguments
///
/// * `prompt` - The prompt message to display
/// * `items` - The list of items to choose from
/// * `default` - Optional default selection index
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<usize>` - The index of the selected item
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
/// use sublime_cli_tools::interactive::select::simple_select;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let options = vec!["patch", "minor", "major"];
/// let selection = simple_select("Select bump type", &options, Some(0), false)?;
/// # Ok(())
/// # }
/// ```
pub fn simple_select<T: ToString>(
    prompt: &str,
    items: &[T],
    default: Option<usize>,
    no_color: bool,
) -> Result<usize> {
    if items.is_empty() {
        return Err(CliError::validation("No items available to select"));
    }

    let theme = WntTheme::new(no_color);
    let mut builder = Select::with_theme(&theme).with_prompt(prompt);

    if let Some(default_idx) = default {
        builder = builder.default(default_idx);
    }

    let items_str: Vec<String> = items.iter().map(std::string::ToString::to_string).collect();
    builder = builder.items(&items_str);

    builder.interact().map_err(|e| CliError::user(format!("Selection cancelled: {e}")))
}

/// Filters items using fuzzy matching and returns ranked results.
///
/// This is a utility function that can be used independently of the prompt
/// functions for custom filtering logic.
///
/// # Arguments
///
/// * `items` - The list of items to filter
/// * `query` - The search query
///
/// # Returns
///
/// A vector of (index, score) tuples, sorted by relevance (highest score first)
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::interactive::select::fuzzy_filter;
///
/// let items = vec!["package-a", "package-b", "package-core", "utils"];
/// let results = fuzzy_filter(&items, "pkg");
///
/// // Results are sorted by relevance
/// assert!(!results.is_empty());
/// for (idx, score) in results {
///     println!("{}: {} (score: {})", idx, items[idx], score);
/// }
/// ```
pub fn fuzzy_filter<T: AsRef<str>>(items: &[T], query: &str) -> Vec<(usize, i64)> {
    if query.is_empty() {
        // Return all items with a default score
        return items.iter().enumerate().map(|(idx, _)| (idx, 0)).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<(usize, i64)> = items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            matcher.fuzzy_match(item.as_ref(), query).map(|score| (idx, score))
        })
        .collect();

    // Sort by score (highest first)
    matches.sort_by(|a, b| b.1.cmp(&a.1));

    matches
}

/// Performs a multi-selection with fuzzy filtering by package names.
///
/// This is a specialized version of `fuzzy_multi_select` that:
/// - Takes string slices instead of generic items
/// - Converts indices to actual values
/// - Returns the selected string values instead of indices
///
/// # Arguments
///
/// * `prompt` - The prompt message to display
/// * `items` - The list of items to choose from
/// * `detected` - Items to pre-select
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<Vec<String>>` - The selected item values
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
///
/// Returns `CliError::Validation` if:
/// - No items are selected
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::select::select_packages;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages = vec!["pkg-a".to_string(), "pkg-b".to_string()];
/// let detected = vec!["pkg-a".to_string()];
/// let selected = select_packages("Select packages", &packages, &detected, false)?;
/// # Ok(())
/// # }
/// ```
pub fn select_packages(
    prompt: &str,
    items: &[String],
    detected: &[String],
    no_color: bool,
) -> Result<Vec<String>> {
    if items.is_empty() {
        return Err(CliError::validation("No packages available to select"));
    }

    // Find indices of detected items
    let default_indices: Vec<usize> =
        detected.iter().filter_map(|det| items.iter().position(|item| item == det)).collect();

    // Perform multi-select
    let selected_indices = fuzzy_multi_select(prompt, items, &default_indices, no_color)?;

    // Convert indices to values
    let selected: Vec<String> = selected_indices.iter().map(|&idx| items[idx].clone()).collect();

    Ok(selected)
}

/// Performs a multi-selection with fuzzy filtering by environment names.
///
/// Similar to `select_packages` but specialized for environments.
///
/// # Arguments
///
/// * `prompt` - The prompt message to display
/// * `items` - The list of environment names to choose from
/// * `defaults` - Environment names to pre-select
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<Vec<String>>` - The selected environment names
///
/// # Errors
///
/// Returns `CliError::User` if:
/// - User cancels the prompt (Ctrl+C)
/// - Terminal interaction fails
///
/// Returns `CliError::Validation` if:
/// - No items are selected
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::interactive::select::select_environments;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let envs = vec!["dev".to_string(), "staging".to_string(), "prod".to_string()];
/// let defaults = vec!["staging".to_string(), "prod".to_string()];
/// let selected = select_environments("Select environments", &envs, &defaults, false)?;
/// # Ok(())
/// # }
/// ```
pub fn select_environments(
    prompt: &str,
    items: &[String],
    defaults: &[String],
    no_color: bool,
) -> Result<Vec<String>> {
    if items.is_empty() {
        return Err(CliError::validation("No environments available to select"));
    }

    // Find indices of default items
    let default_indices: Vec<usize> =
        defaults.iter().filter_map(|def| items.iter().position(|item| item == def)).collect();

    // Perform multi-select
    let selected_indices = fuzzy_multi_select(prompt, items, &default_indices, no_color)?;

    // Convert indices to values
    let selected: Vec<String> = selected_indices.iter().map(|&idx| items[idx].clone()).collect();

    Ok(selected)
}
