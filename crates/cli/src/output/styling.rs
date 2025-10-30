//! Output styling and formatting utilities for modern CLI display.
//!
//! This module provides styling utilities for creating modern, visually
//! appealing CLI output using box-drawing characters, Unicode symbols,
//! and colors.
//!
//! # What
//!
//! Provides:
//! - Box-drawing characters for structured output
//! - Status symbols (success, warning, error, info)
//! - Section headers with visual hierarchy
//! - Styled text output with consistent formatting
//! - Color schemes for different message types
//!
//! # How
//!
//! Uses:
//! - Unicode box-drawing characters (U+2500 series)
//! - Console crate for colored output
//! - Consistent spacing and alignment
//! - Visual indicators (bullets, icons)
//!
//! # Why
//!
//! Creates a professional, modern appearance that:
//! - Improves readability and scannability
//! - Provides visual hierarchy
//! - Makes important information stand out
//! - Follows modern CLI design patterns
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::output::styling::{Section, StatusSymbol};
//!
//! // Create a section header
//! let section = Section::new("Configuration");
//! section.print();
//!
//! // Print status messages
//! StatusSymbol::Success.print_line("Operation completed");
//! StatusSymbol::Warning.print_line("No config file found");
//! StatusSymbol::Error.print_line("Invalid configuration");
//! ```

use console::{Color, style};

/// Box-drawing characters for structured output.
///
/// Provides Unicode characters for creating visual structure in terminal output.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::styling::BoxChars;
///
/// println!("{} Section Title", BoxChars::TOP_LEFT);
/// println!("{} Item 1", BoxChars::VERTICAL_RIGHT);
/// println!("{} Item 2", BoxChars::VERTICAL_RIGHT);
/// ```
pub struct BoxChars;

impl BoxChars {
    /// Top-left corner: ⎡
    pub const TOP_LEFT: &'static str = "⎡";

    /// Vertical line: ⎟
    pub const VERTICAL: &'static str = "⎟";

    /// Vertical line with right branch: ├
    pub const VERTICAL_RIGHT: &'static str = "├";

    /// Bottom-left corner: ⎣
    pub const BOTTOM_LEFT: &'static str = "⎣";

    /// Horizontal line: ━
    pub const HORIZONTAL: &'static str = "━";

    /// Heavy horizontal line: ━━━
    pub const HORIZONTAL_HEAVY: &'static str = "━━━━━━━━━━━━━━━━";

    /// Bullet point: ●
    pub const BULLET: &'static str = "●";

    /// Empty bullet: ○
    pub const BULLET_EMPTY: &'static str = "○";

    /// Arrow right: ➜
    pub const ARROW_RIGHT: &'static str = "➜";

    /// Checkmark: ✓
    pub const CHECK: &'static str = "✓";

    /// Cross mark: ✗
    pub const CROSS: &'static str = "✗";
}

/// Status symbols with associated colors.
///
/// Provides visual indicators for different message types with appropriate
/// colors and symbols.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::styling::StatusSymbol;
///
/// StatusSymbol::Success.print_line("Configuration valid");
/// StatusSymbol::Warning.print_line("Using default values");
/// StatusSymbol::Error.print_line("Failed to load config");
/// StatusSymbol::Info.print_line("Processing 5 packages");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSymbol {
    /// Success indicator (green checkmark)
    Success,

    /// Warning indicator (yellow warning sign)
    Warning,

    /// Error indicator (red cross)
    Error,

    /// Info indicator (cyan info)
    Info,

    /// Question indicator (blue question mark)
    Question,
}

impl StatusSymbol {
    /// Returns the symbol character for this status.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::StatusSymbol;
    ///
    /// assert_eq!(StatusSymbol::Success.symbol(), "✓");
    /// assert_eq!(StatusSymbol::Warning.symbol(), "⚠");
    /// ```
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Error => "✗",
            Self::Info => "ℹ",
            Self::Question => "?",
        }
    }

    /// Returns the color for this status.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::StatusSymbol;
    /// use console::Color;
    ///
    /// assert_eq!(StatusSymbol::Success.color(), Color::Green);
    /// assert_eq!(StatusSymbol::Error.color(), Color::Red);
    /// ```
    #[must_use]
    pub const fn color(&self) -> Color {
        match self {
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
            Self::Info => Color::Cyan,
            Self::Question => Color::Blue,
        }
    }

    /// Prints the symbol with appropriate styling.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_cli_tools::output::styling::StatusSymbol;
    ///
    /// StatusSymbol::Success.print();
    /// println!(" Configuration loaded");
    /// ```
    #[allow(clippy::print_stdout)]
    pub fn print(&self) {
        print!("{}", style(self.symbol()).fg(self.color()).bold());
    }

    /// Prints the symbol followed by a message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print after the symbol
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_cli_tools::output::styling::StatusSymbol;
    ///
    /// StatusSymbol::Success.print_line("All tests passed");
    /// StatusSymbol::Warning.print_line("Deprecated feature used");
    /// ```
    #[allow(clippy::print_stdout)]
    pub fn print_line(&self, message: &str) {
        println!("{} {}", style(self.symbol()).fg(self.color()).bold(), message);
    }

    /// Returns a styled string with the symbol and message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to include
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::StatusSymbol;
    ///
    /// let msg = StatusSymbol::Success.format("Done");
    /// assert!(msg.contains("✓"));
    /// ```
    #[must_use]
    pub fn format(&self, message: &str) -> String {
        format!("{} {}", style(self.symbol()).fg(self.color()).bold(), message)
    }
}

/// Section header with visual styling.
///
/// Creates a visually distinct section header for organizing output.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::styling::Section;
///
/// let section = Section::new("Configuration");
/// section.print();
///
/// println!("{} Strategy: independent", Section::ITEM_PREFIX);
/// println!("{} Path: .changesets", Section::ITEM_PREFIX);
/// ```
pub struct Section {
    title: String,
}

impl Section {
    /// Item prefix for items within a section.
    pub const ITEM_PREFIX: &'static str = "├";

    /// Last item prefix for the final item in a section.
    pub const LAST_ITEM_PREFIX: &'static str = "└";

    /// Vertical line for section continuation.
    pub const VERTICAL: &'static str = "⎟";

    /// Creates a new section with the given title.
    ///
    /// # Arguments
    ///
    /// * `title` - The section title
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::Section;
    ///
    /// let section = Section::new("Build Information");
    /// ```
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into() }
    }

    /// Prints the section header.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_cli_tools::output::styling::Section;
    ///
    /// let section = Section::new("Dependencies");
    /// section.print();
    /// ```
    #[allow(clippy::print_stdout)]
    pub fn print(&self) {
        println!();
        println!(
            "{} {}",
            style(BoxChars::TOP_LEFT).cyan().bold(),
            style(&self.title).cyan().bold()
        );
        println!(
            "{}{}",
            style(BoxChars::VERTICAL).cyan(),
            style(BoxChars::HORIZONTAL_HEAVY).cyan()
        );
        println!("{}", style(BoxChars::VERTICAL).cyan());
    }

    /// Prints the section header with a custom style.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to use for the section
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_cli_tools::output::styling::Section;
    /// use console::Color;
    ///
    /// let section = Section::new("Errors");
    /// section.print_with_color(Color::Red);
    /// ```
    #[allow(clippy::print_stdout)]
    pub fn print_with_color(&self, color: Color) {
        println!();
        println!(
            "{} {}",
            style(BoxChars::TOP_LEFT).fg(color).bold(),
            style(&self.title).fg(color).bold()
        );
        println!(
            "{}{}",
            style(BoxChars::VERTICAL).fg(color),
            style(BoxChars::HORIZONTAL_HEAVY).fg(color)
        );
        println!("{}", style(BoxChars::VERTICAL).fg(color));
    }
}

/// Prints a styled item within a section.
///
/// # Arguments
///
/// * `label` - The item label
/// * `value` - The item value
/// * `is_last` - Whether this is the last item in the section
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::styling::print_item;
///
/// print_item("Name", "my-package", false);
/// print_item("Version", "1.0.0", true);
/// ```
#[allow(clippy::print_stdout)]
pub fn print_item(label: &str, value: &str, is_last: bool) {
    let prefix = if is_last { Section::LAST_ITEM_PREFIX } else { Section::ITEM_PREFIX };

    println!("{} {} {}", style(prefix).cyan(), style(format!("{label}:")).bold(), value);
}

/// Prints a styled bullet item.
///
/// # Arguments
///
/// * `message` - The message to display
/// * `color` - The color for the bullet
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::styling::print_bullet;
/// use console::Color;
///
/// print_bullet("Strategy: independent", Color::Green);
/// print_bullet("Using default config", Color::Yellow);
/// ```
#[allow(clippy::print_stdout)]
pub fn print_bullet(message: &str, color: Color) {
    println!(
        "{} {} {}",
        style(Section::ITEM_PREFIX).cyan(),
        style(BoxChars::BULLET).fg(color).bold(),
        message
    );
}

/// Prints an indented message within a section.
///
/// # Arguments
///
/// * `message` - The message to display
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::styling::print_indented;
///
/// print_indented("Additional information here");
/// ```
#[allow(clippy::print_stdout)]
pub fn print_indented(message: &str) {
    println!("{} {}", style(Section::VERTICAL).cyan(), message);
}

/// Prints a section separator.
///
/// Creates a visual break between sections.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::styling::print_separator;
///
/// print_separator();
/// ```
#[allow(clippy::print_stdout)]
pub fn print_separator() {
    println!("{}", style(Section::VERTICAL).cyan());
}

/// Styles for different text types.
///
/// Provides consistent styling for different types of text output.
pub struct TextStyle;

impl TextStyle {
    /// Styles a key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to style
    /// * `value` - The value to display
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::TextStyle;
    ///
    /// let styled = TextStyle::key_value("Version", "1.0.0");
    /// ```
    #[must_use]
    pub fn key_value(key: &str, value: &str) -> String {
        format!("{}: {}", style(key).bold(), value)
    }

    /// Styles a success message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to style
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::TextStyle;
    ///
    /// let msg = TextStyle::success("Configuration loaded");
    /// ```
    #[must_use]
    pub fn success(message: &str) -> String {
        style(message).green().to_string()
    }

    /// Styles a warning message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to style
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::TextStyle;
    ///
    /// let msg = TextStyle::warning("Using defaults");
    /// ```
    #[must_use]
    pub fn warning(message: &str) -> String {
        style(message).yellow().to_string()
    }

    /// Styles an error message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to style
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::TextStyle;
    ///
    /// let msg = TextStyle::error("Invalid config");
    /// ```
    #[must_use]
    pub fn error(message: &str) -> String {
        style(message).red().bold().to_string()
    }

    /// Styles an info message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to style
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::TextStyle;
    ///
    /// let msg = TextStyle::info("Processing packages");
    /// ```
    #[must_use]
    pub fn info(message: &str) -> String {
        style(message).cyan().to_string()
    }

    /// Styles a dimmed/secondary message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to style
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::styling::TextStyle;
    ///
    /// let msg = TextStyle::dim("(optional)");
    /// ```
    #[must_use]
    pub fn dim(message: &str) -> String {
        style(message).dim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_symbol_symbol() {
        assert_eq!(StatusSymbol::Success.symbol(), "✓");
        assert_eq!(StatusSymbol::Warning.symbol(), "⚠");
        assert_eq!(StatusSymbol::Error.symbol(), "✗");
        assert_eq!(StatusSymbol::Info.symbol(), "ℹ");
        assert_eq!(StatusSymbol::Question.symbol(), "?");
    }

    #[test]
    fn test_status_symbol_color() {
        assert_eq!(StatusSymbol::Success.color(), Color::Green);
        assert_eq!(StatusSymbol::Warning.color(), Color::Yellow);
        assert_eq!(StatusSymbol::Error.color(), Color::Red);
        assert_eq!(StatusSymbol::Info.color(), Color::Cyan);
        assert_eq!(StatusSymbol::Question.color(), Color::Blue);
    }

    #[test]
    fn test_status_symbol_format() {
        let msg = StatusSymbol::Success.format("Done");
        assert!(msg.contains("✓"));
        assert!(msg.contains("Done"));
    }

    #[test]
    fn test_section_creation() {
        let section = Section::new("Test");
        assert_eq!(section.title, "Test");
    }

    #[test]
    fn test_text_style_key_value() {
        let styled = TextStyle::key_value("Name", "value");
        assert!(styled.contains("Name"));
        assert!(styled.contains("value"));
    }

    #[allow(clippy::const_is_empty)]
    #[test]
    fn test_box_chars_constants() {
        assert!(!BoxChars::TOP_LEFT.is_empty());
        assert!(!BoxChars::VERTICAL.is_empty());
        assert!(!BoxChars::VERTICAL_RIGHT.is_empty());
        assert!(!BoxChars::BOTTOM_LEFT.is_empty());
        assert!(!BoxChars::HORIZONTAL.is_empty());
        assert!(!BoxChars::BULLET.is_empty());
    }
}
