//! List components for CLI output.
//!
//! Provides functions for rendering formatted lists with bullets,
//! numbering, and horizontal separators.

use super::symbols::Symbol;
use super::theme;
use unicode_width::UnicodeWidthStr;

/// Types of list markers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMarkerType {
    Bullet,
    Number,
    None,
}

/// A builder for creating consistent lists
pub struct ListBuilder {
    items: Vec<String>,
    marker_type: ListMarkerType,
    indentation: usize,
    separator: bool,
}

#[allow(clippy::new_without_default)]
impl ListBuilder {
    /// Create a new list builder
    pub fn new() -> Self {
        ListBuilder {
            items: Vec::new(),
            marker_type: ListMarkerType::Bullet,
            indentation: 2,
            separator: false,
        }
    }

    /// Add an item to the list
    pub fn add_item<T: Into<String>>(&mut self, item: T) -> &mut Self {
        self.items.push(item.into());
        self
    }

    /// Set multiple items at once
    pub fn items<T, I>(mut self, items: I) -> Self
    where
        T: Into<String>,
        I: IntoIterator<Item = T>,
    {
        self.items = items.into_iter().map(|i| i.into()).collect();
        self
    }

    /// Use bullet markers
    pub fn bullets(mut self) -> Self {
        self.marker_type = ListMarkerType::Bullet;
        self
    }

    /// Use numbered markers
    pub fn numbered(mut self) -> Self {
        self.marker_type = ListMarkerType::Number;
        self
    }

    /// Use no markers
    pub fn no_markers(mut self) -> Self {
        self.marker_type = ListMarkerType::None;
        self
    }

    /// Set custom indentation
    pub fn indent(mut self, spaces: usize) -> Self {
        self.indentation = spaces;
        self
    }

    /// Add horizontal separators between items
    pub fn with_separators(mut self) -> Self {
        self.separator = true;
        self
    }

    /// Build the list string
    pub fn build(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        let term_width = console::Term::stdout().size().1 as usize;
        let max_width = std::cmp::min(term_width.saturating_sub(4), 100);

        let mut result = String::new();
        let indent = " ".repeat(self.indentation);

        for (idx, item) in self.items.iter().enumerate() {
            // Add separator if enabled (except before first item)
            if self.separator && idx > 0 {
                let separator = "─".repeat(max_width.saturating_sub(self.indentation));
                result.push_str(&format!("{}\n", theme::muted_style(&separator)));
            }

            // Add marker based on type
            let marker = match self.marker_type {
                ListMarkerType::Bullet => format!("{} ", Symbol::bullet()),
                ListMarkerType::Number => format!("{}. ", idx + 1),
                ListMarkerType::None => String::new(),
            };

            // Calculate effective indentation based on marker
            let marker_width = UnicodeWidthStr::width(marker.as_str());
            let effective_indent = if self.marker_type == ListMarkerType::None {
                indent.clone()
            } else {
                " ".repeat(self.indentation.saturating_sub(marker_width))
            };

            // Calculate wrap width
            let wrap_width = max_width.saturating_sub(self.indentation + marker_width);

            // Wrap text
            let wrapped_lines = textwrap::wrap(item, wrap_width);

            // Add first line with marker
            if let Some(first_line) = wrapped_lines.first() {
                result.push_str(&format!("{}{}{}\n", effective_indent, marker, first_line));

                // Add continuation lines with proper indentation
                for line in wrapped_lines.iter().skip(1) {
                    let continuation_indent = " ".repeat(self.indentation + marker_width);
                    result.push_str(&format!("{}{}\n", continuation_indent, line));
                }
            }
        }

        // Remove trailing newline
        if !result.is_empty() && result.ends_with('\n') {
            result.pop();
        }

        result
    }

    /// Print the list to stdout
    pub fn print(&self) -> std::io::Result<()> {
        println!("{}", self.build());
        Ok(())
    }
}

/// Create a simple bullet list
pub fn bullet_list<T, I>(items: I) -> String
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    ListBuilder::new().bullets().items(items).build()
}

/// Create a simple numbered list
pub fn numbered_list<T, I>(items: I) -> String
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    ListBuilder::new().numbered().items(items).build()
}

/// Create a list with separators
pub fn separated_list<T, I>(items: I) -> String
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    ListBuilder::new().bullets().with_separators().items(items).build()
}
