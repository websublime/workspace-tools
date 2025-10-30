//! Styling and color utilities for terminal output.
//!
//! This module provides styling helpers for colorizing and formatting terminal output.
//! It respects the NO_COLOR environment variable and terminal capabilities to ensure
//! colors are only used when appropriate.
//!
//! # What
//!
//! Provides:
//! - `Style` struct with color and formatting methods
//! - `StyledText` for building complex styled output
//! - Automatic detection of terminal capabilities
//! - NO_COLOR environment variable support
//! - Cross-platform color support
//!
//! # How
//!
//! Uses the `console` crate for terminal styling and capability detection.
//! Automatically disables colors when:
//! - NO_COLOR environment variable is set
//! - Output is not a TTY
//! - Terminal doesn't support colors
//! - User explicitly requests no color
//!
//! # Why
//!
//! Consistent styling improves readability and user experience while ensuring
//! compatibility with different terminal environments and respecting user preferences.
//!
//! # Examples
//!
//! Basic styling:
//!
//! ```rust
//! use sublime_cli_tools::output::Style;
//!
//! let text = Style::success("Operation completed");
//! let error = Style::error("Something went wrong");
//! let warning = Style::warning("Deprecated option used");
//! ```
//!
//! Building complex styled text:
//!
//! ```rust
//! use sublime_cli_tools::output::StyledText;
//!
//! let text = StyledText::new()
//!     .text("Found ")
//!     .bold("5")
//!     .text(" packages in ")
//!     .cyan("workspace")
//!     .build();
//! ```

use console::{Color, Term, style};
use std::env;

/// Styling utilities for terminal output.
///
/// Provides methods for applying colors and styles to text while respecting
/// terminal capabilities and user preferences.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::Style;
///
/// // Colored output
/// println!("{}", Style::success("✓ Done"));
/// println!("{}", Style::error("✗ Failed"));
/// println!("{}", Style::warning("⚠ Warning"));
/// println!("{}", Style::info("ℹ Info"));
/// ```
pub struct Style;

impl Style {
    /// Returns true if colors should be enabled.
    ///
    /// Colors are disabled if:
    /// - NO_COLOR environment variable is set
    /// - Output is not a TTY
    /// - Terminal doesn't support colors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// if Style::colors_enabled() {
    ///     println!("Colors are supported");
    /// }
    /// ```
    pub fn colors_enabled() -> bool {
        // Check NO_COLOR environment variable
        if env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Check if stdout is a TTY and supports colors
        Term::stdout().is_term() && console::colors_enabled()
    }

    /// Applies success styling (green).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::success("Operation completed"));
    /// ```
    pub fn success(text: &str) -> String {
        if Self::colors_enabled() { style(text).green().to_string() } else { text.to_string() }
    }

    /// Applies error styling (red).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::error("Operation failed"));
    /// ```
    pub fn error(text: &str) -> String {
        if Self::colors_enabled() { style(text).red().to_string() } else { text.to_string() }
    }

    /// Applies warning styling (yellow).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::warning("Deprecated option"));
    /// ```
    pub fn warning(text: &str) -> String {
        if Self::colors_enabled() { style(text).yellow().to_string() } else { text.to_string() }
    }

    /// Applies info styling (blue/cyan).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::info("Found 3 packages"));
    /// ```
    pub fn info(text: &str) -> String {
        if Self::colors_enabled() { style(text).cyan().to_string() } else { text.to_string() }
    }

    /// Applies bold styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::bold("Important"));
    /// ```
    pub fn bold(text: &str) -> String {
        if Self::colors_enabled() { style(text).bold().to_string() } else { text.to_string() }
    }

    /// Applies dim styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::dim("Secondary info"));
    /// ```
    pub fn dim(text: &str) -> String {
        if Self::colors_enabled() { style(text).dim().to_string() } else { text.to_string() }
    }

    /// Applies italic styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::italic("Note"));
    /// ```
    pub fn italic(text: &str) -> String {
        if Self::colors_enabled() { style(text).italic().to_string() } else { text.to_string() }
    }

    /// Applies underline styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// println!("{}", Style::underline("Link"));
    /// ```
    pub fn underline(text: &str) -> String {
        if Self::colors_enabled() { style(text).underlined().to_string() } else { text.to_string() }
    }

    /// Applies a specific color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    /// use console::Color;
    ///
    /// println!("{}", Style::color(Color::Magenta, "Custom color"));
    /// ```
    pub fn color(color: Color, text: &str) -> String {
        if Self::colors_enabled() { style(text).fg(color).to_string() } else { text.to_string() }
    }

    /// Creates a styled text builder for complex formatting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::Style;
    ///
    /// let builder = Style::builder();
    /// ```
    pub fn builder() -> StyledText {
        StyledText::new()
    }
}

/// Builder for creating complex styled text.
///
/// Allows chaining multiple styles and text segments to create
/// rich formatted output.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::StyledText;
///
/// let text = StyledText::new()
///     .text("Found ")
///     .green("5")
///     .text(" packages: ")
///     .bold("@org/core")
///     .build();
///
/// println!("{}", text);
/// ```
#[derive(Debug, Default)]
pub struct StyledText {
    parts: Vec<String>,
}

impl StyledText {
    /// Creates a new styled text builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let builder = StyledText::new();
    /// ```
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Adds plain text without styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new()
    ///     .text("Hello ")
    ///     .text("world")
    ///     .build();
    /// ```
    #[must_use]
    pub fn text(mut self, text: &str) -> Self {
        self.parts.push(text.to_string());
        self
    }

    /// Adds green text (success).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().green("Success").build();
    /// ```
    #[must_use]
    pub fn green(mut self, text: &str) -> Self {
        self.parts.push(Style::success(text));
        self
    }

    /// Adds red text (error).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().red("Error").build();
    /// ```
    #[must_use]
    pub fn red(mut self, text: &str) -> Self {
        self.parts.push(Style::error(text));
        self
    }

    /// Adds yellow text (warning).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().yellow("Warning").build();
    /// ```
    #[must_use]
    pub fn yellow(mut self, text: &str) -> Self {
        self.parts.push(Style::warning(text));
        self
    }

    /// Adds cyan text (info).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().cyan("Info").build();
    /// ```
    #[must_use]
    pub fn cyan(mut self, text: &str) -> Self {
        self.parts.push(Style::info(text));
        self
    }

    /// Adds bold text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().bold("Important").build();
    /// ```
    #[must_use]
    pub fn bold(mut self, text: &str) -> Self {
        self.parts.push(Style::bold(text));
        self
    }

    /// Adds dim text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().dim("Secondary").build();
    /// ```
    #[must_use]
    pub fn dim(mut self, text: &str) -> Self {
        self.parts.push(Style::dim(text));
        self
    }

    /// Adds italic text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().italic("Note").build();
    /// ```
    #[must_use]
    pub fn italic(mut self, text: &str) -> Self {
        self.parts.push(Style::italic(text));
        self
    }

    /// Adds underlined text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new().underline("Link").build();
    /// ```
    #[must_use]
    pub fn underline(mut self, text: &str) -> Self {
        self.parts.push(Style::underline(text));
        self
    }

    /// Adds text with a custom color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    /// use console::Color;
    ///
    /// let text = StyledText::new()
    ///     .color(Color::Magenta, "Custom")
    ///     .build();
    /// ```
    #[must_use]
    pub fn color(mut self, color: Color, text: &str) -> Self {
        self.parts.push(Style::color(color, text));
        self
    }

    /// Builds the final styled string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::StyledText;
    ///
    /// let text = StyledText::new()
    ///     .text("Hello ")
    ///     .bold("world")
    ///     .build();
    ///
    /// assert_eq!(text.len() > 0, true);
    /// ```
    pub fn build(self) -> String {
        self.parts.join("")
    }
}
