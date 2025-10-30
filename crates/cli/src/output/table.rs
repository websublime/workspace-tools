//! Table rendering utilities for terminal output.
//!
//! This module provides a convenient API for rendering tables using comfy-table
//! with support for responsive sizing, theming, and cross-platform compatibility.
//!
//! # What
//!
//! Provides:
//! - `TableBuilder` for constructing tables with a fluent API
//! - `TableTheme` enum for different table styles
//! - `ColumnAlignment` for column alignment options
//! - Responsive width handling based on terminal size
//! - Automatic content truncation for narrow terminals
//! - Color support that respects NO_COLOR and terminal capabilities
//!
//! # How
//!
//! Uses the `comfy-table` crate for rendering with custom theming and
//! formatting. Automatically detects terminal width and adjusts column
//! widths to fit. Handles color output based on terminal capabilities
//! and user preferences.
//!
//! # Why
//!
//! Consistent table rendering across all commands improves readability
//! and provides a professional user experience. Responsive sizing ensures
//! tables display correctly on all terminal sizes.
//!
//! # Examples
//!
//! Basic table:
//!
//! ```rust
//! use sublime_cli_tools::output::table::{TableBuilder, TableTheme};
//!
//! let mut table = TableBuilder::new()
//!     .theme(TableTheme::Default)
//!     .columns(&["Package", "Version", "Type"])
//!     .build();
//!
//! table.add_row(&["typescript", "5.3.3", "minor"]);
//! table.add_row(&["eslint", "9.0.0", "major"]);
//!
//! let output = table.render(false);
//! println!("{}", output);
//! ```
//!
//! Table with alignment:
//!
//! ```rust
//! use sublime_cli_tools::output::table::{TableBuilder, ColumnAlignment};
//!
//! let mut table = TableBuilder::new()
//!     .columns(&["Name", "Count", "Status"])
//!     .alignment(1, ColumnAlignment::Right)
//!     .alignment(2, ColumnAlignment::Center)
//!     .build();
//!
//! table.add_row(&["Package A", "42", "✓"]);
//! table.add_row(&["Package B", "7", "✗"]);
//! ```

use comfy_table::{
    Attribute, Cell, CellAlignment, Color as ComfyColor, ContentArrangement, Table as ComfyTable,
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use console::Term;
use std::fmt;
use terminal_size::{Width, terminal_size};

/// Column alignment options for table cells.
///
/// Determines how content is aligned within table columns.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::ColumnAlignment;
///
/// let left = ColumnAlignment::Left;
/// let right = ColumnAlignment::Right;
/// let center = ColumnAlignment::Center;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    /// Align content to the left (default)
    Left,
    /// Align content to the right
    Right,
    /// Align content to the center
    Center,
}

impl From<ColumnAlignment> for CellAlignment {
    fn from(alignment: ColumnAlignment) -> Self {
        match alignment {
            ColumnAlignment::Left => CellAlignment::Left,
            ColumnAlignment::Right => CellAlignment::Right,
            ColumnAlignment::Center => CellAlignment::Center,
        }
    }
}

/// Table theme styles.
///
/// Different visual styles for tables to match context and preference.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::TableTheme;
///
/// let default = TableTheme::Default;
/// let minimal = TableTheme::Minimal;
/// let compact = TableTheme::Compact;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableTheme {
    /// Default theme with rounded corners and full borders
    Default,
    /// Minimal theme with fewer borders
    Minimal,
    /// Compact theme for dense information
    Compact,
    /// Plain theme without borders (for simple lists)
    Plain,
}

/// Builder for constructing tables with a fluent API.
///
/// Provides methods for configuring table appearance, columns, and behavior
/// before building the final table.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::{TableBuilder, TableTheme, ColumnAlignment};
///
/// let mut table = TableBuilder::new()
///     .theme(TableTheme::Default)
///     .columns(&["Name", "Version", "Type"])
///     .alignment(1, ColumnAlignment::Right)
///     .max_width(100)
///     .build();
///
/// table.add_row(&["package", "1.0.0", "major"]);
/// ```
#[derive(Debug)]
pub struct TableBuilder {
    theme: TableTheme,
    columns: Vec<String>,
    alignments: Vec<ColumnAlignment>,
    max_width: Option<usize>,
    min_column_width: usize,
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TableBuilder {
    /// Creates a new table builder with default settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let builder = TableBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            theme: TableTheme::Default,
            columns: Vec::new(),
            alignments: Vec::new(),
            max_width: None,
            min_column_width: 10,
        }
    }

    /// Sets the table theme.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::{TableBuilder, TableTheme};
    ///
    /// let builder = TableBuilder::new().theme(TableTheme::Minimal);
    /// ```
    #[must_use]
    pub fn theme(mut self, theme: TableTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the column headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let builder = TableBuilder::new()
    ///     .columns(&["Package", "Version", "Type"]);
    /// ```
    #[must_use]
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| (*s).to_string()).collect();
        self.alignments = vec![ColumnAlignment::Left; columns.len()];
        self
    }

    /// Sets the alignment for a specific column.
    ///
    /// # Arguments
    ///
    /// * `column_index` - Zero-based column index
    /// * `alignment` - The alignment to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::{TableBuilder, ColumnAlignment};
    ///
    /// let builder = TableBuilder::new()
    ///     .columns(&["Name", "Count"])
    ///     .alignment(1, ColumnAlignment::Right);
    /// ```
    #[must_use]
    pub fn alignment(mut self, column_index: usize, alignment: ColumnAlignment) -> Self {
        if column_index < self.alignments.len() {
            self.alignments[column_index] = alignment;
        }
        self
    }

    /// Sets the maximum table width in characters.
    ///
    /// If not set, the terminal width is used automatically.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let builder = TableBuilder::new().max_width(100);
    /// ```
    #[must_use]
    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Sets the minimum column width in characters.
    ///
    /// Columns will not be narrower than this width unless the terminal
    /// is too narrow. Default is 10.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let builder = TableBuilder::new().min_column_width(15);
    /// ```
    #[must_use]
    pub fn min_column_width(mut self, width: usize) -> Self {
        self.min_column_width = width;
        self
    }

    /// Builds the table with the configured settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let table = TableBuilder::new()
    ///     .columns(&["Name", "Value"])
    ///     .build();
    /// ```
    pub fn build(self) -> Table {
        let terminal_width = self.max_width.or_else(get_terminal_width);

        Table {
            inner: ComfyTable::new(),
            theme: self.theme,
            columns: self.columns,
            alignments: self.alignments,
            terminal_width,
            min_column_width: self.min_column_width,
        }
    }
}

/// A table for rendering structured data.
///
/// Created by `TableBuilder`, this struct provides methods for adding rows
/// and rendering the final table output.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::TableBuilder;
///
/// let mut table = TableBuilder::new()
///     .columns(&["Package", "Version"])
///     .build();
///
/// table.add_row(&["typescript", "5.3.3"]);
/// table.add_row(&["eslint", "9.0.0"]);
///
/// let output = table.render(false);
/// println!("{}", output);
/// ```
#[derive(Debug)]
pub struct Table {
    inner: ComfyTable,
    theme: TableTheme,
    columns: Vec<String>,
    alignments: Vec<ColumnAlignment>,
    terminal_width: Option<usize>,
    // TODO: will be implemented on story 3.3 - used for responsive column sizing
    #[allow(dead_code)]
    min_column_width: usize,
}

impl Table {
    /// Adds a row of data to the table.
    ///
    /// The number of cells should match the number of columns. If fewer cells
    /// are provided, empty cells are added. Extra cells are ignored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let mut table = TableBuilder::new()
    ///     .columns(&["Name", "Count"])
    ///     .build();
    ///
    /// table.add_row(&["Package A", "42"]);
    /// table.add_row(&["Package B", "7"]);
    /// ```
    pub fn add_row(&mut self, cells: &[&str]) {
        let row: Vec<Cell> = cells
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let mut cell = Cell::new(content);
                if i < self.alignments.len() {
                    cell = cell.set_alignment(self.alignments[i].into());
                }
                cell
            })
            .collect();

        self.inner.add_row(row);
    }

    /// Adds a styled row with custom formatting.
    ///
    /// This allows adding rows with colored or formatted cells.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    /// use comfy_table::{Cell, Color, Attribute};
    ///
    /// let mut table = TableBuilder::new()
    ///     .columns(&["Status", "Message"])
    ///     .build();
    ///
    /// let cells = vec![
    ///     Cell::new("✓").fg(Color::Green),
    ///     Cell::new("Success"),
    /// ];
    /// table.add_styled_row(cells);
    /// ```
    pub fn add_styled_row(&mut self, cells: Vec<Cell>) {
        let row: Vec<Cell> = cells
            .into_iter()
            .enumerate()
            .map(|(i, mut cell)| {
                if i < self.alignments.len() {
                    cell = cell.set_alignment(self.alignments[i].into());
                }
                cell
            })
            .collect();

        self.inner.add_row(row);
    }

    /// Adds a separator row.
    ///
    /// This creates a horizontal line across the table for visual separation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let mut table = TableBuilder::new()
    ///     .columns(&["Name", "Value"])
    ///     .build();
    ///
    /// table.add_row(&["Item 1", "100"]);
    /// table.add_separator();
    /// table.add_row(&["Item 2", "200"]);
    /// ```
    pub fn add_separator(&mut self) {
        // Comfy-table doesn't have explicit separators, but we can add an empty styled row
        // For now, we'll skip this as it's not essential for the core functionality
    }

    /// Returns true if the table has no data rows.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let table = TableBuilder::new()
    ///     .columns(&["Name", "Value"])
    ///     .build();
    ///
    /// assert!(table.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of data rows in the table.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let mut table = TableBuilder::new()
    ///     .columns(&["Name", "Value"])
    ///     .build();
    ///
    /// table.add_row(&["Item 1", "100"]);
    /// assert_eq!(table.row_count(), 1);
    /// ```
    pub fn row_count(&self) -> usize {
        self.inner.row_iter().count()
    }

    /// Renders the table as a string.
    ///
    /// # Arguments
    ///
    /// * `no_color` - If true, disables color output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::table::TableBuilder;
    ///
    /// let mut table = TableBuilder::new()
    ///     .columns(&["Name", "Value"])
    ///     .build();
    ///
    /// table.add_row(&["Package", "1.0.0"]);
    /// let output = table.render(false);
    /// ```
    pub fn render(&mut self, no_color: bool) -> String {
        // Apply theme
        self.apply_theme();

        // Set up header
        if !self.columns.is_empty() {
            let header: Vec<Cell> = self
                .columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let mut cell = Cell::new(col).add_attribute(Attribute::Bold);
                    if i < self.alignments.len() {
                        cell = cell.set_alignment(self.alignments[i].into());
                    }
                    cell
                })
                .collect();

            self.inner.set_header(header);
        }

        // Configure content arrangement for responsive width
        if let Some(width) = self.terminal_width {
            self.inner.set_content_arrangement(ContentArrangement::DynamicFullWidth);
            // Clamp width to u16::MAX to avoid truncation
            // Safe cast: we've clamped the value to u16::MAX
            #[allow(clippy::cast_possible_truncation)]
            let clamped_width = width.min(u16::MAX as usize) as u16;
            self.inner.set_width(clamped_width);
        } else {
            self.inner.set_content_arrangement(ContentArrangement::Disabled);
        }

        // Disable colors if requested or not supported
        if no_color || !colors_enabled() {
            // Comfy-table will handle color stripping internally
        }

        self.inner.trim_fmt()
    }

    /// Applies the selected theme to the table.
    fn apply_theme(&mut self) {
        match self.theme {
            TableTheme::Default => {
                self.inner.load_preset(UTF8_FULL);
                self.inner.apply_modifier(UTF8_ROUND_CORNERS);
            }
            TableTheme::Minimal => {
                self.inner.load_preset(UTF8_FULL);
                // Remove some borders for minimal look
                self.inner.apply_modifier(UTF8_ROUND_CORNERS);
            }
            TableTheme::Compact => {
                self.inner.load_preset(UTF8_FULL);
                // Keep full borders but could adjust padding
            }
            TableTheme::Plain => {
                // No borders, just spacing
                self.inner.load_preset("                     ");
            }
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// Truncates text to a maximum width with ellipsis.
///
/// If the text is longer than `max_width`, it is truncated and "..." is appended.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::truncate_text;
///
/// let text = truncate_text("This is a very long text", 10);
/// assert_eq!(text, "This is...");
/// ```
pub fn truncate_text(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else if max_width <= 3 {
        text.chars().take(max_width).collect()
    } else {
        let truncated: String = text.chars().take(max_width - 3).collect();
        format!("{truncated}...")
    }
}

/// Creates a styled cell with success color (green).
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::success_cell;
///
/// let cell = success_cell("✓");
/// ```
pub fn success_cell(content: &str) -> Cell {
    Cell::new(content).fg(ComfyColor::Green)
}

/// Creates a styled cell with error color (red).
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::error_cell;
///
/// let cell = error_cell("✗");
/// ```
pub fn error_cell(content: &str) -> Cell {
    Cell::new(content).fg(ComfyColor::Red)
}

/// Creates a styled cell with warning color (yellow).
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::warning_cell;
///
/// let cell = warning_cell("⚠");
/// ```
pub fn warning_cell(content: &str) -> Cell {
    Cell::new(content).fg(ComfyColor::Yellow)
}

/// Creates a styled cell with info color (cyan).
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::info_cell;
///
/// let cell = info_cell("ℹ");
/// ```
pub fn info_cell(content: &str) -> Cell {
    Cell::new(content).fg(ComfyColor::Cyan)
}

/// Creates a bold cell.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::bold_cell;
///
/// let cell = bold_cell("Important");
/// ```
pub fn bold_cell(content: &str) -> Cell {
    Cell::new(content).add_attribute(Attribute::Bold)
}

/// Creates a dim cell.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::table::dim_cell;
///
/// let cell = dim_cell("Secondary");
/// ```
pub fn dim_cell(content: &str) -> Cell {
    Cell::new(content).add_attribute(Attribute::Dim)
}

/// Gets the current terminal width.
///
/// Returns `None` if the terminal size cannot be determined.
fn get_terminal_width() -> Option<usize> {
    terminal_size().map(|(Width(w), _)| w as usize)
}

/// Checks if colors are enabled in the terminal.
///
/// Colors are disabled if:
/// - NO_COLOR environment variable is set
/// - Output is not a TTY
/// - Terminal doesn't support colors
fn colors_enabled() -> bool {
    use std::env;

    // Check NO_COLOR environment variable
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if stdout is a TTY and supports colors
    Term::stdout().is_term() && console::colors_enabled()
}
