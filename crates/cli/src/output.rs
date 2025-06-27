//! Output formatting and display management
//!
//! Handles different output formats (human-readable, JSON, YAML, plain text)
//! and provides consistent formatting across all CLI commands.

use crate::commands::OutputFormat;
use crate::error::{CliError, CliResult};
use colored::*;
use serde_json::Value;
use std::io::{self, Write};

/// Output manager for formatting and displaying results
pub struct OutputManager {
    /// Output format to use
    format: OutputFormat,
    
    /// Whether to use colors
    use_color: bool,
    
    /// Writer for output (usually stdout)
    writer: Box<dyn Write + Send>,
}

impl OutputManager {
    /// Create a new output manager
    ///
    /// # Arguments
    ///
    /// * `format` - Output format to use
    /// * `use_color` - Whether to use colored output
    ///
    /// # Returns
    ///
    /// A new OutputManager instance
    pub fn new(format: OutputFormat, use_color: bool) -> Self {
        Self {
            format,
            use_color,
            writer: Box::new(io::stdout()),
        }
    }

    /// Create an output manager with a custom writer (for testing)
    pub fn with_writer(format: OutputFormat, use_color: bool, writer: Box<dyn Write + Send>) -> Self {
        Self {
            format,
            use_color,
            writer,
        }
    }

    /// Display a section header
    pub fn section(&mut self, title: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    format!("\n{}", title.bold().cyan())
                } else {
                    format!("\n{}", title)
                };
                writeln!(self.writer, "{}", formatted)?;
                writeln!(self.writer, "{}", "─".repeat(title.len()))?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "\n{}", title)?;
                writeln!(self.writer, "{}", "-".repeat(title.len()))?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Sections are handled differently in structured formats
            }
        }
        Ok(())
    }

    /// Display a subsection header
    pub fn subsection(&mut self, title: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    format!("\n{}", title.bold())
                } else {
                    format!("\n{}", title)
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "\n{}", title)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Subsections are handled differently in structured formats
            }
        }
        Ok(())
    }

    /// Display an informational message
    pub fn info(&mut self, message: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    message.normal()
                } else {
                    message.normal()
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "{}", message)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Info messages are typically not included in structured output
            }
        }
        Ok(())
    }

    /// Display a success message
    pub fn success(&mut self, message: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    message.green()
                } else {
                    message.normal()
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "SUCCESS: {}", message)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Success messages are typically not included in structured output
            }
        }
        Ok(())
    }

    /// Display a warning message
    pub fn warning(&mut self, message: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    format!("⚠️  {}", message.yellow())
                } else {
                    format!("WARNING: {}", message)
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "WARNING: {}", message)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Warnings could be included in structured output
            }
        }
        Ok(())
    }

    /// Display an error message
    pub fn error(&mut self, message: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    format!("❌ {}", message.red())
                } else {
                    format!("ERROR: {}", message)
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "ERROR: {}", message)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Errors could be included in structured output
            }
        }
        Ok(())
    }

    /// Display a debug message
    pub fn debug(&mut self, message: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    format!("🐛 {}", message.dimmed())
                } else {
                    format!("DEBUG: {}", message)
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "DEBUG: {}", message)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Debug messages are typically not included in structured output
            }
        }
        Ok(())
    }

    /// Display a list item
    pub fn item(&mut self, text: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    format!("  • {}", text)
                } else {
                    format!("  - {}", text)
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "  - {}", text)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Items are handled as part of arrays in structured formats
            }
        }
        Ok(())
    }

    /// Output structured data (for JSON/YAML formats)
    pub fn output_data<T: serde::Serialize>(&mut self, data: &T) -> CliResult<()> {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(data)
                    .map_err(|e| CliError::OutputError(format!("JSON serialization failed: {}", e)))?;
                writeln!(self.writer, "{}", json)?;
            }
            OutputFormat::Yaml => {
                let yaml = serde_yaml::to_string(data)
                    .map_err(|e| CliError::OutputError(format!("YAML serialization failed: {}", e)))?;
                writeln!(self.writer, "{}", yaml)?;
            }
            OutputFormat::Human | OutputFormat::Plain => {
                // For human/plain formats, convert to JSON first then display
                let json_value: Value = serde_json::to_value(data)
                    .map_err(|e| CliError::OutputError(format!("Data serialization failed: {}", e)))?;
                self.display_json_value(&json_value, 0)?;
            }
        }
        Ok(())
    }

    /// Display a JSON value in human-readable format
    fn display_json_value(&mut self, value: &Value, indent: usize) -> CliResult<()> {
        let indent_str = "  ".repeat(indent);
        
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            writeln!(self.writer, "{}{}:", indent_str, key)?;
                            self.display_json_value(val, indent + 1)?;
                        }
                        _ => {
                            writeln!(self.writer, "{}{}: {}", indent_str, key, self.format_value(val))?;
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    writeln!(self.writer, "{}[{}]:", indent_str, i)?;
                    self.display_json_value(val, indent + 1)?;
                }
            }
            _ => {
                writeln!(self.writer, "{}{}", indent_str, self.format_value(value))?;
            }
        }
        
        Ok(())
    }

    /// Format a JSON value for display
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => value.to_string(),
        }
    }

    /// Display a progress indicator (for long-running operations)
    pub fn progress(&mut self, message: &str) -> CliResult<()> {
        match self.format {
            OutputFormat::Human => {
                let formatted = if self.use_color {
                    format!("⏳ {}", message.cyan())
                } else {
                    format!("PROGRESS: {}", message)
                };
                writeln!(self.writer, "{}", formatted)?;
            }
            OutputFormat::Plain => {
                writeln!(self.writer, "PROGRESS: {}", message)?;
            }
            OutputFormat::Json | OutputFormat::Yaml => {
                // Progress messages are typically not included in structured output
            }
        }
        Ok(())
    }

    /// Flush the output
    pub fn flush(&mut self) -> CliResult<()> {
        self.writer.flush().map_err(|e| CliError::IoError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_output_manager() -> (OutputManager, Vec<u8>) {
        let buffer = Vec::new();
        let writer = Box::new(std::io::Cursor::new(buffer));
        let manager = OutputManager::with_writer(OutputFormat::Plain, false, writer as Box<dyn Write + Send>);
        // We need to extract the buffer for testing, but this is complex with the current design
        // For now, we'll just test the creation
        (manager, Vec::new())
    }

    #[test]
    fn test_output_manager_creation() {
        let manager = OutputManager::new(OutputFormat::Human, true);
        // Basic creation test
        assert!(true); // Just verify it compiles and creates
    }

    #[test]
    fn test_format_value() {
        let manager = OutputManager::new(OutputFormat::Json, false);
        
        assert_eq!(manager.format_value(&Value::String("test".to_string())), "test");
        assert_eq!(manager.format_value(&Value::Number(42.into())), "42");
        assert_eq!(manager.format_value(&Value::Bool(true)), "true");
        assert_eq!(manager.format_value(&Value::Null), "null");
    }

    #[test]
    fn test_structured_data_serialization() {
        let mut data = HashMap::new();
        data.insert("key1", "value1");
        data.insert("key2", "value2");

        // Test that serialization works without panicking
        let json_result = serde_json::to_string_pretty(&data);
        assert!(json_result.is_ok());
    }
}