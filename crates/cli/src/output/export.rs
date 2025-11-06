//! Export functionality for CLI reports.
//!
//! This module provides export capabilities for converting command results
//! and reports into various formats (HTML, Markdown) for sharing, archiving,
//! and documentation purposes.
//!
//! # What
//!
//! Provides:
//! - `ExportFormat` enum for different export formats
//! - `Exporter` trait for implementing format-specific exporters
//! - `HtmlExporter` for generating HTML reports
//! - `MarkdownExporter` for generating Markdown documents
//! - `export_data` convenience function for quick exports
//! - Support for structured data via serde serialization
//!
//! # How
//!
//! The export system works by:
//! 1. Taking serializable data structures from commands
//! 2. Converting them to the requested format (HTML or Markdown)
//! 3. Writing the formatted output to a file
//! 4. Preserving structure and formatting appropriate to each format
//!
//! Each exporter implements the `Exporter` trait which defines how to convert
//! data into the target format. The trait uses `serde_json::Value` as an
//! intermediate representation, allowing any serializable type to be exported.
//!
//! # Why
//!
//! Export functionality is essential for:
//! - Sharing audit reports with team members
//! - Archiving release information
//! - Generating documentation from command output
//! - Integrating CLI results into external systems
//! - Creating portable, human-readable reports
//!
//! # Examples
//!
//! Basic export usage:
//!
//! ```rust,no_run
//! use sublime_cli_tools::output::export::{export_data, ExportFormat};
//! use serde::Serialize;
//! use std::path::Path;
//!
//! #[derive(Serialize)]
//! struct Report {
//!     title: String,
//!     items: Vec<String>,
//! }
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let report = Report {
//!     title: "Audit Report".to_string(),
//!     items: vec!["Item 1".to_string(), "Item 2".to_string()],
//! };
//!
//! export_data(&report, ExportFormat::Html, Path::new("report.html"))?;
//! export_data(&report, ExportFormat::Markdown, Path::new("report.md"))?;
//! # Ok(())
//! # }
//! ```
//!
//! Using exporters directly:
//!
//! ```rust,no_run
//! use sublime_cli_tools::output::export::{HtmlExporter, Exporter};
//! use serde_json::json;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let exporter = HtmlExporter::new("My Report");
//! let data = json!({
//!     "packages": ["@org/core", "@org/utils"],
//!     "count": 2
//! });
//!
//! let html = exporter.export(&data)?;
//! std::fs::write("output.html", html)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Export format for command output.
///
/// Specifies the target format for exporting command results and reports.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::export::ExportFormat;
///
/// let format = ExportFormat::Html;
/// assert_eq!(format.extension(), "html");
///
/// let format = ExportFormat::Markdown;
/// assert_eq!(format.extension(), "md");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// HTML format with styling and structure.
    ///
    /// Produces a complete HTML document with embedded CSS for styling.
    /// Suitable for viewing in browsers and sharing via web.
    Html,

    /// Markdown format for documentation.
    ///
    /// Produces a Markdown document that can be viewed in any Markdown
    /// renderer, committed to repositories, or converted to other formats.
    Markdown,
}

impl ExportFormat {
    /// Returns the file extension for this format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::export::ExportFormat;
    ///
    /// assert_eq!(ExportFormat::Html.extension(), "html");
    /// assert_eq!(ExportFormat::Markdown.extension(), "md");
    /// ```
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Markdown => "md",
        }
    }

    /// Returns the MIME type for this format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::export::ExportFormat;
    ///
    /// assert_eq!(ExportFormat::Html.mime_type(), "text/html");
    /// assert_eq!(ExportFormat::Markdown.mime_type(), "text/markdown");
    /// ```
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Html => "text/html",
            Self::Markdown => "text/markdown",
        }
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Html => write!(f, "HTML"),
            Self::Markdown => write!(f, "Markdown"),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "html" | "htm" => Ok(Self::Html),
            "markdown" | "md" => Ok(Self::Markdown),
            _ => Err(format!("Invalid export format '{s}'. Valid options: html, markdown")),
        }
    }
}

/// Trait for implementing format-specific exporters.
///
/// This trait defines the interface for converting structured data into
/// various export formats. Implementors take a `serde_json::Value` and
/// produce a formatted string representation.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::export::Exporter;
/// use serde_json::json;
///
/// # fn example<E: Exporter>(exporter: &E) -> Result<(), Box<dyn std::error::Error>> {
/// let data = json!({
///     "name": "test-package",
///     "version": "1.0.0"
/// });
///
/// let exported = exporter.export(&data)?;
/// println!("Exported: {}", exported);
/// # Ok(())
/// # }
/// ```
pub trait Exporter {
    /// Exports the given data to the target format.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to export as a JSON value
    ///
    /// # Returns
    ///
    /// A formatted string representation of the data in the target format.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be exported (e.g., invalid structure).
    fn export(&self, data: &Value) -> Result<String>;
}

/// HTML exporter for generating styled HTML reports.
///
/// Produces complete HTML documents with embedded CSS styling, making them
/// suitable for viewing in any web browser without external dependencies.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::export::{HtmlExporter, Exporter};
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let exporter = HtmlExporter::new("Audit Report");
/// let data = json!({
///     "packages": 5,
///     "issues": 2
/// });
///
/// let html = exporter.export(&data)?;
/// std::fs::write("report.html", html)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct HtmlExporter {
    /// Title for the HTML document.
    title: String,
}

impl HtmlExporter {
    /// Creates a new HTML exporter with the given title.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to use for the HTML document
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::export::HtmlExporter;
    ///
    /// let exporter = HtmlExporter::new("My Report");
    /// ```
    #[must_use]
    pub fn new<S: Into<String>>(title: S) -> Self {
        Self { title: title.into() }
    }

    /// Generates CSS styles for the HTML document.
    ///
    /// Returns embedded CSS that provides professional styling for the report.
    #[allow(clippy::too_many_lines)]
    fn generate_styles() -> &'static str {
        r"
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
                line-height: 1.6;
                color: #333;
                background: #f5f5f5;
                padding: 20px;
            }
            .container {
                max-width: 1200px;
                margin: 0 auto;
                background: white;
                border-radius: 8px;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                overflow: hidden;
            }
            header {
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                padding: 30px;
                border-bottom: 3px solid #5568d3;
            }
            h1 {
                font-size: 28px;
                font-weight: 600;
                margin-bottom: 5px;
            }
            .timestamp {
                opacity: 0.9;
                font-size: 14px;
            }
            .content {
                padding: 30px;
            }
            h2 {
                font-size: 20px;
                color: #667eea;
                margin: 25px 0 15px 0;
                padding-bottom: 8px;
                border-bottom: 2px solid #e0e0e0;
            }
            h3 {
                font-size: 16px;
                color: #555;
                margin: 15px 0 10px 0;
            }
            table {
                width: 100%;
                border-collapse: collapse;
                margin: 15px 0;
                background: white;
                border-radius: 4px;
                overflow: hidden;
            }
            th, td {
                padding: 12px;
                text-align: left;
                border-bottom: 1px solid #e0e0e0;
            }
            th {
                background: #f8f9fa;
                font-weight: 600;
                color: #555;
                text-transform: uppercase;
                font-size: 12px;
                letter-spacing: 0.5px;
            }
            tr:hover {
                background: #f8f9fa;
            }
            .badge {
                display: inline-block;
                padding: 4px 8px;
                border-radius: 3px;
                font-size: 12px;
                font-weight: 600;
            }
            .badge-success {
                background: #d4edda;
                color: #155724;
            }
            .badge-warning {
                background: #fff3cd;
                color: #856404;
            }
            .badge-danger {
                background: #f8d7da;
                color: #721c24;
            }
            .badge-info {
                background: #d1ecf1;
                color: #0c5460;
            }
            code {
                background: #f4f4f4;
                padding: 2px 6px;
                border-radius: 3px;
                font-family: 'Courier New', monospace;
                font-size: 13px;
            }
            pre {
                background: #f8f9fa;
                padding: 15px;
                border-radius: 4px;
                overflow-x: auto;
                margin: 10px 0;
            }
            .key-value {
                display: flex;
                padding: 8px 0;
                border-bottom: 1px solid #f0f0f0;
            }
            .key-value:last-child {
                border-bottom: none;
            }
            .key {
                font-weight: 600;
                color: #555;
                min-width: 200px;
            }
            .value {
                color: #333;
                flex: 1;
            }
            ul {
                margin: 10px 0;
                padding-left: 25px;
            }
            li {
                margin: 5px 0;
            }
            footer {
                padding: 20px 30px;
                background: #f8f9fa;
                border-top: 1px solid #e0e0e0;
                text-align: center;
                color: #666;
                font-size: 13px;
            }
        </style>
        "
    }

    /// Converts a JSON value to HTML representation.
    fn value_to_html(&self, value: &Value, depth: usize) -> String {
        match value {
            Value::Null => String::from("<span class=\"value\">null</span>"),
            Value::Bool(b) => format!("<span class=\"value\">{b}</span>"),
            Value::Number(n) => format!("<span class=\"value\">{n}</span>"),
            Value::String(s) => format!("<span class=\"value\">{}</span>", html_escape(s)),
            Value::Array(arr) => {
                if arr.is_empty() {
                    return String::from("<span class=\"value\">[]</span>");
                }

                let items: Vec<String> = arr
                    .iter()
                    .map(|item| format!("<li>{}</li>", self.value_to_html(item, depth + 1)))
                    .collect();
                format!("<ul>{}</ul>", items.join(""))
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    return String::from("<span class=\"value\">{}</span>");
                }

                if depth == 0 { self.object_to_sections(obj) } else { self.object_to_table(obj) }
            }
        }
    }

    /// Converts an object to HTML sections (for top-level objects).
    fn object_to_sections(&self, obj: &serde_json::Map<String, Value>) -> String {
        use std::fmt::Write;
        let mut html = String::new();

        for (key, value) in obj {
            let _ = write!(html, "<h2>{}</h2>", html_escape(key));
            html.push_str(&self.value_to_html(value, 1));
        }

        html
    }

    /// Converts an object to an HTML table or key-value pairs.
    fn object_to_table(&self, obj: &serde_json::Map<String, Value>) -> String {
        use std::fmt::Write;
        let mut html = String::from("<div style=\"margin: 15px 0;\">");

        for (key, value) in obj {
            html.push_str("<div class=\"key-value\">");
            let _ = write!(html, "<div class=\"key\">{}</div>", html_escape(key));
            let value_html = self.value_to_html(value, 2);
            let _ = write!(html, "<div class=\"value\">{value_html}</div>");
            html.push_str("</div>");
        }

        html.push_str("</div>");
        html
    }
}

impl Exporter for HtmlExporter {
    fn export(&self, data: &Value) -> Result<String> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

        let content = self.value_to_html(data, 0);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {styles}
</head>
<body>
    <div class="container">
        <header>
            <h1>{title}</h1>
            <div class="timestamp">Generated on {timestamp}</div>
        </header>
        <div class="content">
            {content}
        </div>
        <footer>
            Generated by Workspace Tools CLI
        </footer>
    </div>
</body>
</html>"#,
            title = html_escape(&self.title),
            styles = Self::generate_styles(),
            timestamp = timestamp,
            content = content
        );

        Ok(html)
    }
}

/// Markdown exporter for generating Markdown documents.
///
/// Produces clean, readable Markdown that can be viewed in any Markdown
/// renderer, committed to version control, or converted to other formats.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::export::{MarkdownExporter, Exporter};
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let exporter = MarkdownExporter::new("Audit Report");
/// let data = json!({
///     "packages": 5,
///     "issues": 2
/// });
///
/// let markdown = exporter.export(&data)?;
/// std::fs::write("report.md", markdown)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MarkdownExporter {
    /// Title for the Markdown document.
    title: String,
}

impl MarkdownExporter {
    /// Creates a new Markdown exporter with the given title.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to use for the Markdown document
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::export::MarkdownExporter;
    ///
    /// let exporter = MarkdownExporter::new("My Report");
    /// ```
    #[must_use]
    pub fn new<S: Into<String>>(title: S) -> Self {
        Self { title: title.into() }
    }

    /// Converts a JSON value to Markdown representation.
    fn value_to_markdown(&self, value: &Value, depth: usize) -> String {
        match value {
            Value::Null => String::from("null"),
            Value::Bool(b) => format!("`{b}`"),
            Value::Number(n) => format!("`{n}`"),
            Value::String(s) => markdown_escape(s),
            Value::Array(arr) => {
                if arr.is_empty() {
                    return String::from("_No items_");
                }

                let items: Vec<String> = arr
                    .iter()
                    .map(|item| format!("- {}", self.value_to_markdown(item, depth + 1)))
                    .collect();
                format!("\n{}\n", items.join("\n"))
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    return String::from("_Empty_");
                }

                if depth == 0 {
                    self.object_to_sections(obj)
                } else {
                    self.object_to_list(obj, depth)
                }
            }
        }
    }

    /// Converts an object to Markdown sections (for top-level objects).
    fn object_to_sections(&self, obj: &serde_json::Map<String, Value>) -> String {
        use std::fmt::Write;
        let mut md = String::new();

        for (key, value) in obj {
            let _ = write!(md, "\n## {key}\n\n");
            md.push_str(&self.value_to_markdown(value, 1));
            md.push('\n');
        }

        md
    }

    /// Converts an object to a Markdown list.
    fn object_to_list(&self, obj: &serde_json::Map<String, Value>, depth: usize) -> String {
        use std::fmt::Write;
        let mut md = String::new();

        for (key, value) in obj {
            let indent = "  ".repeat(depth.saturating_sub(1));
            let _ = write!(md, "\n{indent}- **{key}**: ");
            md.push_str(&self.value_to_markdown(value, depth + 1));
        }

        md
    }
}

impl Exporter for MarkdownExporter {
    fn export(&self, data: &Value) -> Result<String> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

        let content = self.value_to_markdown(data, 0);

        let markdown = format!(
            "# {title}\n\n_Generated on {timestamp}_\n\n{content}\n\n---\n\n_Generated by Workspace Tools CLI_\n",
            title = self.title,
            timestamp = timestamp,
            content = content
        );

        Ok(markdown)
    }
}

/// Exports data to a file in the specified format.
///
/// This is a convenience function that handles serialization, format conversion,
/// and file writing in a single call.
///
/// # Arguments
///
/// * `data` - The data to export (must implement Serialize)
/// * `format` - The target export format
/// * `output_path` - Path where the exported file will be written
///
/// # Returns
///
/// Returns `Ok(())` if the export was successful.
///
/// # Errors
///
/// Returns an error if:
/// - The data cannot be serialized
/// - The export conversion fails
/// - The file cannot be written
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::output::export::{export_data, ExportFormat};
/// use serde::Serialize;
/// use std::path::Path;
///
/// #[derive(Serialize)]
/// struct Report {
///     title: String,
///     count: usize,
/// }
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let report = Report {
///     title: "Test Report".to_string(),
///     count: 42,
/// };
///
/// export_data(&report, ExportFormat::Html, Path::new("report.html"))?;
/// # Ok(())
/// # }
/// ```
pub fn export_data<T: Serialize>(data: &T, format: ExportFormat, output_path: &Path) -> Result<()> {
    // Serialize data to JSON value
    let json_value = serde_json::to_value(data)
        .map_err(|e| CliError::execution(format!("Failed to serialize data: {e}")))?;

    // Extract title from data if available, otherwise use filename
    let title = extract_title(&json_value, output_path);

    // Create exporter based on format
    let content = match format {
        ExportFormat::Html => {
            let exporter = HtmlExporter::new(title);
            exporter.export(&json_value)?
        }
        ExportFormat::Markdown => {
            let exporter = MarkdownExporter::new(title);
            exporter.export(&json_value)?
        }
    };

    // Write to file
    fs::write(output_path, content)
        .map_err(|e| CliError::io(format!("Failed to write export file: {e}")))?;

    Ok(())
}

/// Extracts a title from the data or generates one from the filename.
fn extract_title(data: &Value, path: &Path) -> String {
    // Try to extract title from common field names
    if let Value::Object(obj) = data {
        for field in &["title", "name", "type", "command"] {
            if let Some(Value::String(s)) = obj.get(*field) {
                return s.clone();
            }
        }
    }

    // Fall back to filename
    path.file_stem()
        .and_then(|s| s.to_str())
        .map_or_else(|| String::from("Report"), |s| s.replace(['_', '-'], " "))
}

/// Escapes HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Escapes Markdown special characters.
fn markdown_escape(s: &str) -> String {
    // Only escape characters that would break the formatting
    // Don't escape if already in code or if it's a deliberate markdown
    // Currently just returns the string as-is since we're not doing complex escaping
    s.to_string()
}
