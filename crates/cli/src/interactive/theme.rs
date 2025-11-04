//! Custom theme for interactive prompts.
//!
//! This module provides a custom theme that enhances the visual appearance
//! of interactive prompts with consistent styling, colors, and indicators.
//!
//! # What
//!
//! Provides:
//! - `WntTheme` - A custom theme with enhanced visual styling
//! - Consistent color scheme across all prompts
//! - Visual indicators (checkboxes, arrows, bullets)
//! - Support for color and no-color modes
//!
//! # How
//!
//! Implements the `dialoguer::theme::Theme` trait to customize:
//! - Prompt formatting with colors and symbols
//! - Selection indicators (active, inactive, selected)
//! - Input field styling
//! - Success/error message formatting
//! - Help text styling
//!
//! The theme automatically respects the NO_COLOR environment variable
//! and the `no_color` flag passed to prompt functions.
//!
//! # Why
//!
//! A custom theme provides:
//! - Consistent visual identity across the CLI
//! - Better user experience with clear visual feedback
//! - Professional appearance matching modern CLI standards
//! - Accessibility through clear indicators even without colors
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::interactive::theme::WntTheme;
//! use dialoguer::Select;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let theme = WntTheme::new(false);
//! let items = vec!["Option 1", "Option 2", "Option 3"];
//!
//! let selection = Select::with_theme(&theme)
//!     .with_prompt("Choose an option")
//!     .items(&items)
//!     .default(0)
//!     .interact()?;
//!
//! println!("You selected: {}", items[selection]);
//! # Ok(())
//! # }
//! ```

use console::{Style, style};
use dialoguer::theme::Theme;
use std::fmt;

/// Custom theme for workspace node tools CLI prompts.
///
/// Provides enhanced visual styling with:
/// - Color-coded elements (green for success, blue for info, red for errors)
/// - Clear visual indicators (✓, ✗, ›, •)
/// - Consistent formatting across all prompt types
/// - Support for both color and no-color modes
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::interactive::theme::WntTheme;
///
/// // Create theme with colors enabled
/// let theme = WntTheme::new(false);
///
/// // Create theme with colors disabled
/// let theme_no_color = WntTheme::new(true);
/// ```
#[derive(Clone)]
pub struct WntTheme {
    /// Whether to disable colors
    no_color: bool,
    /// Style for prompts and questions
    prompt_style: Style,
    /// Style for error messages
    error_style: Style,
    /// Style for hints and help text
    hint_style: Style,
    /// Style for values and user input
    values_style: Style,
    /// Style for active selections
    active_style: Style,
    /// Style for inactive selections
    inactive_style: Style,
}

impl WntTheme {
    /// Creates a new WntTheme instance.
    ///
    /// # Arguments
    ///
    /// * `no_color` - Whether to disable colored output
    ///
    /// # Returns
    ///
    /// A new `WntTheme` instance with appropriate styling
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::interactive::theme::WntTheme;
    ///
    /// let theme = WntTheme::new(false);
    /// let theme_no_color = WntTheme::new(true);
    /// ```
    pub fn new(no_color: bool) -> Self {
        if no_color {
            Self {
                no_color,
                prompt_style: Style::new(),
                error_style: Style::new(),
                hint_style: Style::new(),
                values_style: Style::new(),
                active_style: Style::new(),
                inactive_style: Style::new(),
            }
        } else {
            Self {
                no_color,
                prompt_style: Style::new().cyan().bold(),
                error_style: Style::new().red().bold(),
                hint_style: Style::new().dim(),
                values_style: Style::new().yellow(),
                active_style: Style::new().cyan(),
                inactive_style: Style::new().dim(),
            }
        }
    }

    /// Formats a prompt prefix with an icon.
    ///
    /// Returns "?" for colored mode or "?" for no-color mode.
    fn prompt_prefix(&self) -> String {
        if self.no_color { "?".to_string() } else { style("?").cyan().bold().to_string() }
    }

    /// Formats a success indicator.
    ///
    /// Returns "✓" for colored mode or "[✓]" for no-color mode.
    fn success_indicator(&self) -> String {
        if self.no_color { "[✓]".to_string() } else { style("✓").green().bold().to_string() }
    }

    /// Formats an error indicator.
    ///
    /// Returns "✗" for colored mode or "[✗]" for no-color mode.
    fn error_indicator(&self) -> String {
        if self.no_color { "[✗]".to_string() } else { style("✗").red().bold().to_string() }
    }

    /// Formats an active selection indicator (arrow).
    ///
    /// Returns "›" for colored mode or ">" for no-color mode.
    fn active_indicator(&self) -> String {
        if self.no_color { ">".to_string() } else { style("›").cyan().bold().to_string() }
    }

    /// Formats an inactive selection indicator (space).
    fn inactive_indicator() -> String {
        " ".to_string()
    }

    /// Returns whether colors are disabled.
    ///
    /// # Returns
    ///
    /// `true` if colors are disabled, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::interactive::theme::WntTheme;
    ///
    /// let theme = WntTheme::new(true);
    /// assert!(theme.is_no_color());
    ///
    /// let theme_color = WntTheme::new(false);
    /// assert!(!theme_color.is_no_color());
    /// ```
    pub fn is_no_color(&self) -> bool {
        self.no_color
    }

    /// Formats a checked checkbox indicator.
    ///
    /// Returns "◉" for colored mode or "[x]" for no-color mode.
    fn checked_indicator(&self) -> String {
        if self.no_color { "[x]".to_string() } else { style("◉").cyan().to_string() }
    }

    /// Formats an unchecked checkbox indicator.
    ///
    /// Returns "◯" for colored mode or "[ ]" for no-color mode.
    fn unchecked_indicator(&self) -> String {
        if self.no_color { "[ ]".to_string() } else { style("◯").dim().to_string() }
    }
}

impl Theme for WntTheme {
    /// Formats a prompt with styling.
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        write!(f, "{} {}", self.prompt_prefix(), self.prompt_style.apply_to(prompt))
    }

    /// Formats an error message.
    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        write!(f, "{} {}", self.error_indicator(), self.error_style.apply_to(err))
    }

    /// Formats a success message.
    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        if let Some(default) = default {
            let hint = if default { "(Y/n)" } else { "(y/N)" };
            write!(
                f,
                "{} {} {}",
                self.prompt_prefix(),
                self.prompt_style.apply_to(prompt),
                self.hint_style.apply_to(hint)
            )
        } else {
            write!(f, "{} {} (y/n)", self.prompt_prefix(), self.prompt_style.apply_to(prompt))
        }
    }

    /// Formats a confirmation with the default value selected.
    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        let selection_str = match selection {
            Some(true) => "yes",
            Some(false) => "no",
            None => "",
        };

        write!(
            f,
            "{} {} {}",
            self.success_indicator(),
            self.prompt_style.apply_to(prompt),
            self.values_style.apply_to(selection_str)
        )
    }

    /// Formats an input prompt.
    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        if let Some(default) = default {
            write!(
                f,
                "{} {} {}",
                self.prompt_prefix(),
                self.prompt_style.apply_to(prompt),
                self.hint_style.apply_to(format!("({default})"))
            )
        } else {
            write!(f, "{} {}", self.prompt_prefix(), self.prompt_style.apply_to(prompt))
        }
    }

    /// Formats the user's input after submission.
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.success_indicator(),
            self.prompt_style.apply_to(prompt),
            self.values_style.apply_to(sel)
        )
    }

    /// Formats a password prompt.
    fn format_password_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        write!(f, "{} {}", self.prompt_prefix(), self.prompt_style.apply_to(prompt))
    }

    /// Formats the password prompt after submission.
    fn format_password_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
    ) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.success_indicator(),
            self.prompt_style.apply_to(prompt),
            self.values_style.apply_to("[hidden]")
        )
    }

    /// Formats a select prompt item.
    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        if active {
            write!(f, "{} {}", self.active_indicator(), self.active_style.apply_to(text))
        } else {
            write!(f, "{} {}", Self::inactive_indicator(), self.inactive_style.apply_to(text))
        }
    }

    /// Formats a multi-select prompt item.
    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        let checkbox = if checked { self.checked_indicator() } else { self.unchecked_indicator() };

        if active {
            write!(
                f,
                "{} {} {}",
                self.active_indicator(),
                checkbox,
                self.active_style.apply_to(text)
            )
        } else {
            write!(
                f,
                "{} {} {}",
                Self::inactive_indicator(),
                checkbox,
                self.inactive_style.apply_to(text)
            )
        }
    }

    /// Formats the selection after a select prompt completes.
    fn format_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.success_indicator(),
            self.prompt_style.apply_to(prompt),
            self.values_style.apply_to(sel)
        )
    }

    /// Formats the selection after a multi-select prompt completes.
    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.success_indicator(),
            self.prompt_style.apply_to(prompt),
            self.values_style.apply_to(format!("{} selected", selections.len()))
        )
    }

    /// Formats a sort prompt item.
    fn format_sort_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        picked: bool,
        active: bool,
    ) -> fmt::Result {
        if picked {
            if active {
                write!(
                    f,
                    "{} {} {}",
                    self.active_indicator(),
                    self.checked_indicator(),
                    self.active_style.apply_to(text)
                )
            } else {
                write!(
                    f,
                    "{} {} {}",
                    Self::inactive_indicator(),
                    self.checked_indicator(),
                    self.inactive_style.apply_to(text)
                )
            }
        } else if active {
            write!(
                f,
                "{} {} {}",
                self.active_indicator(),
                self.unchecked_indicator(),
                self.active_style.apply_to(text)
            )
        } else {
            write!(
                f,
                "{} {} {}",
                Self::inactive_indicator(),
                self.unchecked_indicator(),
                self.inactive_style.apply_to(text)
            )
        }
    }
}

impl Default for WntTheme {
    /// Creates a default WntTheme with colors enabled.
    ///
    /// Respects the NO_COLOR environment variable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::interactive::theme::WntTheme;
    ///
    /// let theme = WntTheme::default();
    /// ```
    fn default() -> Self {
        let no_color = std::env::var("NO_COLOR").is_ok();
        Self::new(no_color)
    }
}
