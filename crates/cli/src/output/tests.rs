//! Tests for the output module.
//!
//! This module contains comprehensive tests for all output functionality including:
//! - OutputFormat variants and conversions
//! - Output struct methods (success, error, warning, info, etc.)
//! - JSON serialization
//! - Color handling
//! - Different output formats

#![allow(clippy::unwrap_used)]

use super::*;
use std::sync::{Arc, Mutex};

/// Helper to create an Output instance with a shared buffer for testing.
fn create_output_with_buffer(
    format: OutputFormat,
    no_color: bool,
) -> (Output, Arc<Mutex<Vec<u8>>>) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = BufferWriter { buffer: Arc::clone(&buffer) };
    let output = Output::new(format, writer, no_color);
    (output, buffer)
}

/// Helper writer that writes to a shared buffer
#[derive(Clone)]
struct BufferWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.lock().unwrap().flush()
    }
}

/// Helper to get the output string from a shared buffer.
fn get_output_string(buffer: &Arc<Mutex<Vec<u8>>>) -> String {
    String::from_utf8(buffer.lock().unwrap().clone()).unwrap_or_default()
}

#[test]
fn test_output_format_default() {
    let format = OutputFormat::default();
    assert_eq!(format, OutputFormat::Human);
}

#[test]
fn test_output_format_is_json() {
    assert!(OutputFormat::Json.is_json());
    assert!(OutputFormat::JsonCompact.is_json());
    assert!(!OutputFormat::Human.is_json());
    assert!(!OutputFormat::Quiet.is_json());
}

#[test]
fn test_output_format_is_human() {
    assert!(OutputFormat::Human.is_human());
    assert!(!OutputFormat::Json.is_human());
    assert!(!OutputFormat::JsonCompact.is_human());
    assert!(!OutputFormat::Quiet.is_human());
}

#[test]
fn test_output_format_is_quiet() {
    assert!(OutputFormat::Quiet.is_quiet());
    assert!(!OutputFormat::Human.is_quiet());
    assert!(!OutputFormat::Json.is_quiet());
    assert!(!OutputFormat::JsonCompact.is_quiet());
}

#[test]
fn test_output_format_display() {
    assert_eq!(OutputFormat::Human.to_string(), "human");
    assert_eq!(OutputFormat::Json.to_string(), "json");
    assert_eq!(OutputFormat::JsonCompact.to_string(), "json-compact");
    assert_eq!(OutputFormat::Quiet.to_string(), "quiet");
}

#[test]
fn test_output_format_from_str() {
    assert_eq!("human".parse::<OutputFormat>().unwrap(), OutputFormat::Human);
    assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
    assert_eq!("json-compact".parse::<OutputFormat>().unwrap(), OutputFormat::JsonCompact);
    assert_eq!("compact".parse::<OutputFormat>().unwrap(), OutputFormat::JsonCompact);
    assert_eq!("quiet".parse::<OutputFormat>().unwrap(), OutputFormat::Quiet);

    // Case insensitive
    assert_eq!("HUMAN".parse::<OutputFormat>().unwrap(), OutputFormat::Human);
    assert_eq!("Json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);

    // Invalid format
    assert!("invalid".parse::<OutputFormat>().is_err());
}

#[test]
fn test_output_creation() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, false);
    assert_eq!(output.format(), OutputFormat::Human);
    assert!(!output.no_color());
}

#[test]
fn test_output_with_no_color() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, true);
    assert!(output.no_color());
}

#[test]
fn test_success_human_mode() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, false);
    output.success("Test success message").unwrap();
    // Note: actual output might contain color codes, so we just check it doesn't error
}

#[test]
fn test_success_json_mode() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Json, false);
    output.success("Test success message").unwrap();
    // In JSON mode, success messages are ignored
    let output_str = get_output_string(&buffer);
    assert_eq!(output_str, "");
}

#[test]
fn test_success_quiet_mode() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Quiet, false);
    output.success("Test success message").unwrap();
    // In quiet mode, success messages are suppressed
    let output_str = get_output_string(&buffer);
    assert_eq!(output_str, "");
}

#[test]
fn test_error_human_mode() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, false);
    output.error("Test error message").unwrap();
    // Note: actual output might contain color codes
}

#[test]
fn test_error_quiet_mode() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Quiet, false);
    output.error("Test error message").unwrap();
    // Errors are shown even in quiet mode
    let output_str = get_output_string(&buffer);
    assert!(output_str.contains("Error: Test error message"));
}

#[test]
fn test_warning_human_mode() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, false);
    output.warning("Test warning message").unwrap();
}

#[test]
fn test_warning_quiet_mode() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Quiet, false);
    output.warning("Test warning message").unwrap();
    // Warnings are suppressed in quiet mode
    let output_str = get_output_string(&buffer);
    assert_eq!(output_str, "");
}

#[test]
fn test_info_human_mode() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, false);
    output.info("Test info message").unwrap();
}

#[test]
fn test_info_quiet_mode() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Quiet, false);
    output.info("Test info message").unwrap();
    // Info messages are suppressed in quiet mode
    let output_str = get_output_string(&buffer);
    assert_eq!(output_str, "");
}

#[test]
fn test_plain_text_human_mode() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Human, false);
    output.plain("Plain text").unwrap();
    let output_str = get_output_string(&buffer);
    assert!(output_str.contains("Plain text"));
}

#[test]
fn test_plain_text_json_mode() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Json, false);
    output.plain("Plain text").unwrap();
    // Plain text is ignored in JSON mode
    let output_str = get_output_string(&buffer);
    assert_eq!(output_str, "");
}

#[test]
fn test_json_output_pretty() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        count: i32,
    }

    let (output, buffer) = create_output_with_buffer(OutputFormat::Json, false);

    let data = TestData { name: "test".to_string(), count: 42 };
    let response = JsonResponse::success(data);
    output.json(&response).unwrap();

    let output_str = get_output_string(&buffer);
    assert!(output_str.contains("success"));
    assert!(output_str.contains("true"));
    assert!(output_str.contains("test"));
    assert!(output_str.contains("42"));
    // Pretty printed should have newlines
    assert!(output_str.contains('\n'));
}

#[test]
fn test_json_output_compact() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        value: String,
    }

    let (output, buffer) = create_output_with_buffer(OutputFormat::JsonCompact, false);

    let data = TestData { value: "test".to_string() };
    let response = JsonResponse::success(data);
    output.json(&response).unwrap();

    let output_str = get_output_string(&buffer);
    assert!(output_str.contains("success"));
    assert!(output_str.contains("test"));
    // Compact format should be single line (except final newline)
    let lines: Vec<&str> = output_str.trim().lines().collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn test_json_ignored_in_human_mode() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        value: String,
    }

    let (output, buffer) = create_output_with_buffer(OutputFormat::Human, false);

    let data = TestData { value: "test".to_string() };
    let response = JsonResponse::success(data);
    output.json(&response).unwrap();

    // JSON output is ignored in human mode
    let output_str = get_output_string(&buffer);
    assert_eq!(output_str, "");
}

#[test]
fn test_write_raw() {
    let (output, buffer) = create_output_with_buffer(OutputFormat::Human, false);

    output.write_raw(b"Raw data\n").unwrap();

    let output_str = get_output_string(&buffer);
    assert_eq!(output_str, "Raw data\n");
}

#[test]
fn test_flush() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, false);
    // Flush should not error
    output.flush().unwrap();
}

#[test]
fn test_multiple_outputs_human_mode() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, false);

    output.info("Starting operation").unwrap();
    output.success("Step 1 complete").unwrap();
    output.warning("Minor issue detected").unwrap();
    output.success("Operation complete").unwrap();
    // All should succeed without error
}

#[test]
fn test_json_output_with_complex_data() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Package {
        name: String,
        version: String,
    }

    #[derive(Serialize)]
    struct ComplexData {
        packages: Vec<Package>,
        total: usize,
    }

    let (output, buffer) = create_output_with_buffer(OutputFormat::Json, false);

    let data = ComplexData {
        packages: vec![
            Package { name: "@org/core".to_string(), version: "1.0.0".to_string() },
            Package { name: "@org/utils".to_string(), version: "2.0.0".to_string() },
        ],
        total: 2,
    };

    let response = JsonResponse::success(data);
    output.json(&response).unwrap();

    let output_str = get_output_string(&buffer);
    assert!(output_str.contains("@org/core"));
    assert!(output_str.contains("@org/utils"));
    assert!(output_str.contains("1.0.0"));
    assert!(output_str.contains("2.0.0"));
    // Pretty printing adds spaces, so check for "total": 2 instead of "total":2
    assert!(output_str.contains("\"total\"") && output_str.contains('2'));
}

#[test]
fn test_output_format_getter() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Json, false);
    assert_eq!(output.format(), OutputFormat::Json);
}

#[test]
fn test_no_color_getter() {
    let (output, _buffer) = create_output_with_buffer(OutputFormat::Human, true);
    assert!(output.no_color());

    let (output2, _buffer2) = create_output_with_buffer(OutputFormat::Human, false);
    assert!(!output2.no_color());
}

// ============================================================================
// JSON Response Tests
// ============================================================================

#[test]
fn test_json_response_success() {
    let response = JsonResponse::success("test data");
    assert!(response.is_success());
    assert!(!response.is_error());
    assert_eq!(response.data, Some("test data"));
    assert_eq!(response.error, None);
}

#[test]
fn test_json_response_error() {
    let response: JsonResponse<String> = JsonResponse::error("test error".to_string());
    assert!(!response.is_success());
    assert!(response.is_error());
    assert_eq!(response.data, None);
    assert_eq!(response.error, Some("test error".to_string()));
}

#[test]
fn test_json_response_serialization() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        value: i32,
    }

    let response = JsonResponse::success(TestData { value: 42 });
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains(r#""success":true"#));
    assert!(json.contains(r#""value":42"#));
    assert!(!json.contains("error"));
}

#[test]
fn test_json_response_error_serialization() {
    let response: JsonResponse<String> = JsonResponse::error("something failed".to_string());
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains(r#""success":false"#));
    assert!(json.contains(r#""error":"something failed"#));
    assert!(!json.contains("data"));
}

#[test]
fn test_json_response_default() {
    let response: JsonResponse<String> = JsonResponse::default();
    assert!(!response.success);
    assert!(response.error.is_some());
    assert!(response.data.is_none());
}

// ============================================================================
// Style Tests
// ============================================================================

#[test]
fn test_styled_text_builder() {
    let text = StyledText::new().text("hello").text(" world").build();
    assert!(text.contains("hello"));
    assert!(text.contains("world"));
}

#[test]
fn test_styled_text_with_colors() {
    let text = StyledText::new().text("Status: ").green("success").build();
    // Color codes might be present depending on environment
    assert!(!text.is_empty());
}

#[test]
fn test_styled_text_chain() {
    let text =
        StyledText::new().red("Error").text(": ").yellow("warning").text(" ").cyan("info").build();
    assert!(!text.is_empty());
}

#[test]
fn test_style_methods_return_strings() {
    let success = Style::success("test");
    let error = Style::error("test");
    let warning = Style::warning("test");
    let info = Style::info("test");
    let bold = Style::bold("test");
    let dim = Style::dim("test");
    let italic = Style::italic("test");
    let underline = Style::underline("test");

    // All methods should return non-empty strings
    assert!(!success.is_empty());
    assert!(!error.is_empty());
    assert!(!warning.is_empty());
    assert!(!info.is_empty());
    assert!(!bold.is_empty());
    assert!(!dim.is_empty());
    assert!(!italic.is_empty());
    assert!(!underline.is_empty());
}

#[test]
fn test_style_builder_creation() {
    let builder = Style::builder();
    let text = builder.text("test").build();
    assert_eq!(text, "test");
}

// ============================================================================
// Table Tests
// ============================================================================

#[test]
fn test_table_builder_creation() {
    use crate::output::table::TableBuilder;

    let builder = TableBuilder::new();
    let table = builder.build();
    assert!(table.is_empty());
}

#[test]
fn test_table_with_columns() {
    use crate::output::table::TableBuilder;

    let table = TableBuilder::new().columns(&["Name", "Value", "Type"]).build();
    assert!(table.is_empty());
    assert_eq!(table.row_count(), 0);
}

#[test]
fn test_table_add_single_row() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["Name", "Value"]).build();
    table.add_row(&["Item 1", "100"]);
    assert!(!table.is_empty());
    assert_eq!(table.row_count(), 1);
}

#[test]
fn test_table_add_multiple_rows() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["Package", "Version"]).build();
    table.add_row(&["typescript", "5.3.3"]);
    table.add_row(&["eslint", "9.0.0"]);
    table.add_row(&["vitest", "1.2.1"]);
    assert_eq!(table.row_count(), 3);
}

#[test]
fn test_table_render_basic() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["Name", "Value"]).build();
    table.add_row(&["Package", "1.0.0"]);
    let output = table.render(true);
    assert!(!output.is_empty());
    assert!(output.contains("Name"));
    assert!(output.contains("Value"));
    assert!(output.contains("Package"));
    assert!(output.contains("1.0.0"));
}

#[test]
fn test_table_render_with_color() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["Status", "Message"]).build();
    table.add_row(&["OK", "Success"]);
    let output_no_color = table.render(true);
    let output_with_color = table.render(false);

    assert!(!output_no_color.is_empty());
    assert!(!output_with_color.is_empty());
}

#[test]
fn test_table_themes() {
    use crate::output::table::{TableBuilder, TableTheme};

    for theme in &[TableTheme::Default, TableTheme::Minimal, TableTheme::Compact, TableTheme::Plain]
    {
        let mut table = TableBuilder::new().theme(*theme).columns(&["A", "B"]).build();
        table.add_row(&["1", "2"]);
        let output = table.render(true);
        assert!(!output.is_empty());
    }
}

#[test]
fn test_table_column_alignment() {
    use crate::output::table::{ColumnAlignment, TableBuilder};

    let mut table = TableBuilder::new()
        .columns(&["Left", "Center", "Right"])
        .alignment(0, ColumnAlignment::Left)
        .alignment(1, ColumnAlignment::Center)
        .alignment(2, ColumnAlignment::Right)
        .build();

    table.add_row(&["A", "B", "C"]);
    let output = table.render(true);
    assert!(output.contains("Left"));
    assert!(output.contains("Center"));
    assert!(output.contains("Right"));
}

#[test]
fn test_table_max_width() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["Name", "Description"]).max_width(50).build();
    table.add_row(&["Package", "A very long description that might need wrapping"]);
    let output = table.render(true);
    assert!(!output.is_empty());
}

#[test]
fn test_table_min_column_width() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["A", "B"]).min_column_width(20).build();
    table.add_row(&["x", "y"]);
    let output = table.render(true);
    assert!(!output.is_empty());
}

#[test]
fn test_table_styled_rows() {
    use crate::output::table::{TableBuilder, error_cell, success_cell};
    use comfy_table::Cell;

    let mut table = TableBuilder::new().columns(&["Status", "Message"]).build();

    let cells = vec![success_cell("✓"), Cell::new("Success")];
    table.add_styled_row(cells);

    let cells = vec![error_cell("✗"), Cell::new("Failed")];
    table.add_styled_row(cells);

    assert_eq!(table.row_count(), 2);
    let output = table.render(true);
    assert!(output.contains("Status"));
}

#[test]
fn test_table_empty() {
    use crate::output::table::TableBuilder;

    let table = TableBuilder::new().columns(&["Name", "Value"]).build();
    assert!(table.is_empty());
    assert_eq!(table.row_count(), 0);
}

#[test]
fn test_truncate_text_short() {
    use crate::output::table::truncate_text;

    let result = truncate_text("short", 10);
    assert_eq!(result, "short");
}

#[test]
fn test_truncate_text_exact() {
    use crate::output::table::truncate_text;

    let result = truncate_text("exactly ten", 11);
    assert_eq!(result, "exactly ten");
}

#[test]
fn test_truncate_text_long() {
    use crate::output::table::truncate_text;

    let result = truncate_text("this is a very long text", 10);
    assert_eq!(result, "this is...");
}

#[test]
fn test_truncate_text_very_short() {
    use crate::output::table::truncate_text;

    let result = truncate_text("hello", 3);
    assert_eq!(result, "hel");
}

#[test]
fn test_styled_cell_helpers() {
    use crate::output::table::{
        bold_cell, dim_cell, error_cell, info_cell, success_cell, warning_cell,
    };

    let success = success_cell("✓");
    let error = error_cell("✗");
    let warning = warning_cell("⚠");
    let info = info_cell("ℹ");
    let bold = bold_cell("Bold");
    let dim = dim_cell("Dim");

    assert_eq!(success.content(), "✓");
    assert_eq!(error.content(), "✗");
    assert_eq!(warning.content(), "⚠");
    assert_eq!(info.content(), "ℹ");
    assert_eq!(bold.content(), "Bold");
    assert_eq!(dim.content(), "Dim");
}

#[test]
fn test_table_with_output_human_mode() {
    use crate::output::table::TableBuilder;

    let (output, buffer) = create_output_with_buffer(OutputFormat::Human, false);
    let mut table = TableBuilder::new().columns(&["Package", "Version"]).build();
    table.add_row(&["typescript", "5.3.3"]);

    output.table(&mut table).unwrap();
    let output_str = get_output_string(&buffer);
    assert!(output_str.contains("Package"));
    assert!(output_str.contains("typescript"));
}

#[test]
fn test_table_with_output_json_mode() {
    use crate::output::table::TableBuilder;

    let (output, buffer) = create_output_with_buffer(OutputFormat::Json, false);
    let mut table = TableBuilder::new().columns(&["Package", "Version"]).build();
    table.add_row(&["typescript", "5.3.3"]);

    output.table(&mut table).unwrap();
    let output_str = get_output_string(&buffer);
    // Tables are ignored in JSON mode
    assert_eq!(output_str, "");
}

#[test]
fn test_table_with_output_quiet_mode() {
    use crate::output::table::TableBuilder;

    let (output, buffer) = create_output_with_buffer(OutputFormat::Quiet, false);
    let mut table = TableBuilder::new().columns(&["Package", "Version"]).build();
    table.add_row(&["typescript", "5.3.3"]);

    output.table(&mut table).unwrap();
    let output_str = get_output_string(&buffer);
    // Tables are suppressed in quiet mode
    assert_eq!(output_str, "");
}

#[test]
fn test_table_no_color_rendering() {
    use crate::output::table::TableBuilder;

    let (output, buffer) = create_output_with_buffer(OutputFormat::Human, true);
    let mut table = TableBuilder::new().columns(&["Name", "Status"]).build();
    table.add_row(&["Item", "OK"]);

    output.table(&mut table).unwrap();
    let output_str = get_output_string(&buffer);
    assert!(output_str.contains("Name"));
    assert!(output_str.contains("Item"));
}

#[test]
fn test_table_complex_data() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["Package", "Current", "Latest", "Type"]).build();

    table.add_row(&["typescript", "5.0.0", "5.3.3", "minor"]);
    table.add_row(&["eslint", "8.0.0", "9.0.0", "major"]);
    table.add_row(&["vitest", "1.0.0", "1.2.1", "minor"]);

    let output = table.render(true);
    assert!(output.contains("typescript"));
    assert!(output.contains("eslint"));
    assert!(output.contains("vitest"));
    assert!(output.contains("5.3.3"));
    assert!(output.contains("9.0.0"));
}

#[test]
fn test_column_alignment_enum() {
    use crate::output::table::ColumnAlignment;
    use comfy_table::CellAlignment;

    let left: CellAlignment = ColumnAlignment::Left.into();
    let right: CellAlignment = ColumnAlignment::Right.into();
    let center: CellAlignment = ColumnAlignment::Center.into();

    assert_eq!(left, CellAlignment::Left);
    assert_eq!(right, CellAlignment::Right);
    assert_eq!(center, CellAlignment::Center);
}

#[test]
fn test_table_builder_fluent_api() {
    use crate::output::table::{ColumnAlignment, TableBuilder, TableTheme};

    let mut table = TableBuilder::new()
        .theme(TableTheme::Minimal)
        .columns(&["Name", "Count", "Status"])
        .alignment(1, ColumnAlignment::Right)
        .alignment(2, ColumnAlignment::Center)
        .max_width(100)
        .min_column_width(15)
        .build();

    table.add_row(&["Package A", "42", "✓"]);
    table.add_row(&["Package B", "7", "✗"]);

    assert_eq!(table.row_count(), 2);
    let output = table.render(true);
    assert!(!output.is_empty());
}

#[test]
fn test_table_display_trait() {
    use crate::output::table::TableBuilder;

    let mut table = TableBuilder::new().columns(&["A", "B"]).build();
    table.add_row(&["1", "2"]);

    // Test that Display trait works
    let display_output = format!("{table}");
    assert!(!display_output.is_empty());
}

// ============================================================================
// Progress Indicator Tests
// ============================================================================

#[test]
fn test_progress_bar_creation() {
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new(100);
    pb.set_message("test");
    pb.set_position(50);
    pb.inc(1);
    pb.finish();
}

#[test]
fn test_progress_bar_with_json_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new_with_format(100, OutputFormat::Json);
    assert!(!pb.is_active());

    // All operations should be no-ops
    pb.set_message("test");
    pb.set_position(50);
    pb.inc(1);
    pb.finish();
}

#[test]
fn test_progress_bar_with_quiet_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new_with_format(100, OutputFormat::Quiet);
    assert!(!pb.is_active());
}

#[test]
fn test_progress_bar_with_json_compact_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new_with_format(100, OutputFormat::JsonCompact);
    assert!(!pb.is_active());
}

#[test]
fn test_progress_bar_set_length() {
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new(100);
    pb.set_length(200);
    pb.set_position(150);
    pb.finish();
}

#[test]
fn test_progress_bar_finish_with_message() {
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new(100);
    pb.set_position(100);
    pb.finish_with_message("Done");
}

#[test]
fn test_progress_bar_abandon() {
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new(100);
    pb.set_position(50);
    pb.abandon();
}

#[test]
fn test_progress_bar_abandon_with_message() {
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new(100);
    pb.set_position(50);
    pb.abandon_with_message("Failed");
}

#[test]
fn test_progress_bar_complete_workflow() {
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new(10);
    pb.set_message("Processing");

    for i in 0..10 {
        pb.set_position(i);
        pb.inc(1);
    }

    pb.finish_with_message("Complete");
}

#[test]
fn test_spinner_creation() {
    use crate::output::progress::Spinner;

    let spinner = Spinner::new("Loading...");
    spinner.set_message("Still loading...");
    spinner.tick();
    spinner.finish();
}

#[test]
fn test_spinner_with_json_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::Spinner;

    let spinner = Spinner::new_with_format("Loading...", OutputFormat::Json);
    assert!(!spinner.is_active());

    // All operations should be no-ops
    spinner.set_message("test");
    spinner.tick();
    spinner.finish();
}

#[test]
fn test_spinner_with_quiet_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::Spinner;

    let spinner = Spinner::new_with_format("Loading...", OutputFormat::Quiet);
    assert!(!spinner.is_active());
}

#[test]
fn test_spinner_with_json_compact_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::Spinner;

    let spinner = Spinner::new_with_format("Loading...", OutputFormat::JsonCompact);
    assert!(!spinner.is_active());
}

#[test]
fn test_spinner_finish_with_message() {
    use crate::output::progress::Spinner;

    let spinner = Spinner::new("Loading...");
    spinner.finish_with_message("Done");
}

#[test]
fn test_spinner_abandon() {
    use crate::output::progress::Spinner;

    let spinner = Spinner::new("Loading...");
    spinner.abandon();
}

#[test]
fn test_spinner_abandon_with_message() {
    use crate::output::progress::Spinner;

    let spinner = Spinner::new("Loading...");
    spinner.abandon_with_message("Failed");
}

#[test]
fn test_spinner_complete_workflow() {
    use crate::output::progress::Spinner;

    let spinner = Spinner::new("Starting...");
    spinner.set_message("Working...");
    spinner.tick();
    spinner.set_message("Almost done...");
    spinner.finish_with_message("✓ Done");
}

#[test]
fn test_multi_progress_creation() {
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new();
    let _pb1 = multi.add_progress_bar(100);
    let _pb2 = multi.add_spinner("Loading...");
    multi.clear();
}

#[test]
fn test_multi_progress_with_json_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new_with_format(OutputFormat::Json);
    assert!(!multi.is_active());

    let pb = multi.add_progress_bar(100);
    assert!(!pb.is_active());

    let spinner = multi.add_spinner("Loading...");
    assert!(!spinner.is_active());
}

#[test]
fn test_multi_progress_with_quiet_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new_with_format(OutputFormat::Quiet);
    assert!(!multi.is_active());
}

#[test]
fn test_multi_progress_with_json_compact_format() {
    use crate::output::OutputFormat;
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new_with_format(OutputFormat::JsonCompact);
    assert!(!multi.is_active());
}

#[test]
fn test_multi_progress_default() {
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::default();
    let _pb = multi.add_progress_bar(100);
}

#[test]
fn test_multi_progress_add_spinner() {
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new();
    let spinner = multi.add_spinner("Test");
    spinner.set_message("Testing...");
    spinner.finish();
}

#[test]
fn test_multi_progress_multiple_bars() {
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new();
    let pb1 = multi.add_progress_bar(100);
    let pb2 = multi.add_progress_bar(50);

    pb1.set_message("Task 1");
    pb2.set_message("Task 2");

    pb1.inc(10);
    pb2.inc(5);

    pb1.finish();
    pb2.finish();
}

#[test]
fn test_multi_progress_clear() {
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new();
    let _pb1 = multi.add_progress_bar(100);
    let _pb2 = multi.add_spinner("Loading...");
    multi.clear();
}

#[test]
fn test_should_show_progress_json() {
    use crate::output::OutputFormat;
    use crate::output::progress::should_show_progress;

    assert!(!should_show_progress(OutputFormat::Json));
}

#[test]
fn test_should_show_progress_json_compact() {
    use crate::output::OutputFormat;
    use crate::output::progress::should_show_progress;

    assert!(!should_show_progress(OutputFormat::JsonCompact));
}

#[test]
fn test_should_show_progress_quiet() {
    use crate::output::OutputFormat;
    use crate::output::progress::should_show_progress;

    assert!(!should_show_progress(OutputFormat::Quiet));
}

#[test]
fn test_progress_bar_is_active_human_mode() {
    use crate::output::OutputFormat;
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new_with_format(100, OutputFormat::Human);
    // May or may not be active depending on TTY, but should not panic
    let _ = pb.is_active();
}

#[test]
fn test_spinner_is_active_human_mode() {
    use crate::output::OutputFormat;
    use crate::output::progress::Spinner;

    let spinner = Spinner::new_with_format("Loading...", OutputFormat::Human);
    // May or may not be active depending on TTY, but should not panic
    let _ = spinner.is_active();
}

#[test]
fn test_multi_progress_is_active_human_mode() {
    use crate::output::OutputFormat;
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new_with_format(OutputFormat::Human);
    // May or may not be active depending on TTY, but should not panic
    let _ = multi.is_active();
}

// ============================================================================
// Diff Module Tests
// ============================================================================

#[test]
fn test_diff_type_color() {
    use crate::output::diff::DiffType;
    use console::Color;

    assert_eq!(DiffType::Added.color(), Color::Green);
    assert_eq!(DiffType::Modified.color(), Color::Yellow);
    assert_eq!(DiffType::Deleted.color(), Color::Red);
    assert_eq!(DiffType::Unchanged.color(), Color::White);
}

#[test]
fn test_diff_type_symbol() {
    use crate::output::diff::DiffType;

    assert_eq!(DiffType::Added.symbol(), "+");
    assert_eq!(DiffType::Modified.symbol(), "~");
    assert_eq!(DiffType::Deleted.symbol(), "-");
    assert_eq!(DiffType::Unchanged.symbol(), " ");
}

#[test]
fn test_diff_type_label() {
    use crate::output::diff::DiffType;

    assert_eq!(DiffType::Added.label(), "added");
    assert_eq!(DiffType::Modified.label(), "modified");
    assert_eq!(DiffType::Deleted.label(), "deleted");
    assert_eq!(DiffType::Unchanged.label(), "unchanged");
}

#[test]
fn test_diff_type_display() {
    use crate::output::diff::DiffType;

    assert_eq!(DiffType::Added.to_string(), "added");
    assert_eq!(DiffType::Modified.to_string(), "modified");
    assert_eq!(DiffType::Deleted.to_string(), "deleted");
    assert_eq!(DiffType::Unchanged.to_string(), "unchanged");
}

#[test]
fn test_diff_line_new() {
    use crate::output::diff::{DiffLine, DiffType};

    let line = DiffLine::new(DiffType::Added, "test content");
    assert_eq!(line.diff_type, DiffType::Added);
    assert_eq!(line.content, "test content");
    assert_eq!(line.line_number, None);
}

#[test]
fn test_diff_line_with_line_number() {
    use crate::output::diff::{DiffLine, DiffType};

    let line = DiffLine::with_line_number(DiffType::Modified, "test", 42);
    assert_eq!(line.diff_type, DiffType::Modified);
    assert_eq!(line.content, "test");
    assert_eq!(line.line_number, Some(42));
}

#[test]
fn test_diff_line_render_no_color() {
    use crate::output::diff::{DiffLine, DiffType};

    let line = DiffLine::new(DiffType::Added, "new line");
    let rendered = line.render(true);
    assert!(rendered.contains('+'));
    assert!(rendered.contains("new line"));
}

#[test]
fn test_diff_line_render_with_line_number_no_color() {
    use crate::output::diff::{DiffLine, DiffType};

    let line = DiffLine::with_line_number(DiffType::Deleted, "old line", 10);
    let rendered = line.render(true);
    assert!(rendered.contains("10"));
    assert!(rendered.contains('-'));
    assert!(rendered.contains("old line"));
}

#[test]
fn test_diff_line_render_with_color() {
    use crate::output::diff::{DiffLine, DiffType};

    let line = DiffLine::new(DiffType::Modified, "changed");
    let rendered = line.render(false);
    assert!(rendered.contains("changed"));
    // Color codes will be present, but we can't easily test for specific ANSI codes
}

#[test]
fn test_version_diff_new() {
    use crate::output::diff::VersionDiff;

    let diff = VersionDiff::new("my-package", "1.0.0", "2.0.0");
    assert_eq!(diff.package, "my-package");
    assert_eq!(diff.from_version, "1.0.0");
    assert_eq!(diff.to_version, "2.0.0");
    assert_eq!(diff.reason, None);
    assert!(diff.will_change);
}

#[test]
fn test_version_diff_with_reason() {
    use crate::output::diff::VersionDiff;

    let diff = VersionDiff::new("pkg", "1.0.0", "2.0.0").with_reason("Breaking changes");
    assert_eq!(diff.reason, Some("Breaking changes".to_string()));
}

#[test]
fn test_version_diff_with_will_change() {
    use crate::output::diff::VersionDiff;

    let diff = VersionDiff::new("pkg", "1.0.0", "1.0.0").with_will_change(false);
    assert!(!diff.will_change);
}

#[test]
fn test_version_diff_has_change() {
    use crate::output::diff::VersionDiff;

    let diff = VersionDiff::new("pkg", "1.0.0", "2.0.0");
    assert!(diff.has_change());

    let no_change = VersionDiff::new("pkg", "1.0.0", "1.0.0");
    assert!(!no_change.has_change());
}

#[test]
fn test_file_diff_new() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified);
    assert_eq!(diff.path, "test.txt");
    assert_eq!(diff.file_type, DiffType::Modified);
    assert!(diff.lines.is_empty());
    assert_eq!(diff.context, None);
}

#[test]
fn test_file_diff_add_line() {
    use crate::output::diff::{DiffLine, DiffType, FileDiff};

    let mut diff = FileDiff::new("test.txt", DiffType::Modified);
    diff.add_line(DiffLine::new(DiffType::Added, "new"));
    assert_eq!(diff.lines.len(), 1);
}

#[test]
fn test_file_diff_add_line_added() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified)
        .add_line_added("line 1")
        .add_line_added("line 2");
    assert_eq!(diff.lines.len(), 2);
    assert!(diff.lines.iter().all(|l| l.diff_type == DiffType::Added));
}

#[test]
fn test_file_diff_add_line_removed() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified).add_line_removed("old line");
    assert_eq!(diff.lines.len(), 1);
    assert_eq!(diff.lines[0].diff_type, DiffType::Deleted);
}

#[test]
fn test_file_diff_add_line_modified() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified).add_line_modified("changed");
    assert_eq!(diff.lines.len(), 1);
    assert_eq!(diff.lines[0].diff_type, DiffType::Modified);
}

#[test]
fn test_file_diff_add_line_context() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified).add_line_context("unchanged");
    assert_eq!(diff.lines.len(), 1);
    assert_eq!(diff.lines[0].diff_type, DiffType::Unchanged);
}

#[test]
fn test_file_diff_with_context() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified).with_context("Version bump");
    assert_eq!(diff.context, Some("Version bump".to_string()));
}

#[test]
fn test_file_diff_added_count() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified)
        .add_line_added("line 1")
        .add_line_added("line 2")
        .add_line_removed("old");
    assert_eq!(diff.added_count(), 2);
}

#[test]
fn test_file_diff_removed_count() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified)
        .add_line_added("new")
        .add_line_removed("old 1")
        .add_line_removed("old 2");
    assert_eq!(diff.removed_count(), 2);
}

#[test]
fn test_file_diff_modified_count() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified)
        .add_line_modified("changed 1")
        .add_line_modified("changed 2")
        .add_line_added("new");
    assert_eq!(diff.modified_count(), 2);
}

#[test]
fn test_dependency_diff_new() {
    use crate::output::diff::DependencyDiff;

    let diff = DependencyDiff::new("lodash", "^4.0.0", "^5.0.0");
    assert_eq!(diff.name, "lodash");
    assert_eq!(diff.from_version, "^4.0.0");
    assert_eq!(diff.to_version, "^5.0.0");
    assert_eq!(diff.dep_type, None);
    assert_eq!(diff.package_context, None);
}

#[test]
fn test_dependency_diff_with_dep_type() {
    use crate::output::diff::DependencyDiff;

    let diff = DependencyDiff::new("jest", "^27.0.0", "^28.0.0").with_dep_type("devDependencies");
    assert_eq!(diff.dep_type, Some("devDependencies".to_string()));
}

#[test]
fn test_dependency_diff_with_package_context() {
    use crate::output::diff::DependencyDiff;

    let diff =
        DependencyDiff::new("react", "^17.0.0", "^18.0.0").with_package_context("@org/frontend");
    assert_eq!(diff.package_context, Some("@org/frontend".to_string()));
}

#[test]
fn test_dependency_diff_has_change() {
    use crate::output::diff::DependencyDiff;

    let diff = DependencyDiff::new("pkg", "1.0.0", "2.0.0");
    assert!(diff.has_change());

    let no_change = DependencyDiff::new("pkg", "1.0.0", "1.0.0");
    assert!(!no_change.has_change());
}

#[test]
fn test_diff_renderer_new() {
    use crate::output::diff::DiffRenderer;

    let _renderer = DiffRenderer::new(false);
    // Renderer created successfully
}

#[test]
fn test_diff_renderer_with_line_numbers() {
    use crate::output::diff::DiffRenderer;

    let _renderer = DiffRenderer::new(false).with_line_numbers(true);
    // Builder pattern works
}

#[test]
fn test_diff_renderer_with_context_lines() {
    use crate::output::diff::DiffRenderer;

    let _renderer = DiffRenderer::new(false).with_context_lines(5);
    // Builder pattern works
}

#[test]
fn test_diff_renderer_default() {
    use crate::output::diff::DiffRenderer;

    let _renderer = DiffRenderer::default();
    // Default renderer created successfully
}

#[test]
fn test_diff_renderer_render_version_diff() {
    use crate::output::diff::{DiffRenderer, VersionDiff};

    let renderer = DiffRenderer::new(true); // No color for easier testing
    let diff = VersionDiff::new("my-package", "1.0.0", "2.0.0");
    let output = renderer.render_version_diff(&diff);

    assert!(output.contains("my-package"));
    assert!(output.contains("1.0.0"));
    assert!(output.contains("2.0.0"));
    assert!(output.contains('-'));
    assert!(output.contains('+'));
}

#[test]
fn test_diff_renderer_render_version_diff_with_reason() {
    use crate::output::diff::{DiffRenderer, VersionDiff};

    let renderer = DiffRenderer::new(true);
    let diff = VersionDiff::new("pkg", "1.0.0", "2.0.0").with_reason("Major version bump");
    let output = renderer.render_version_diff(&diff);

    assert!(output.contains("pkg"));
    assert!(output.contains("Reason"));
    assert!(output.contains("Major version bump"));
}

#[test]
fn test_diff_renderer_render_version_diff_no_change() {
    use crate::output::diff::{DiffRenderer, VersionDiff};

    let renderer = DiffRenderer::new(true);
    let diff = VersionDiff::new("pkg", "1.0.0", "1.0.0").with_will_change(false);
    let output = renderer.render_version_diff(&diff);

    assert!(output.contains("pkg"));
    assert!(output.contains("unchanged"));
}

#[test]
fn test_diff_renderer_render_file_diff() {
    use crate::output::diff::{DiffRenderer, DiffType, FileDiff};

    let renderer = DiffRenderer::new(true);
    let diff = FileDiff::new("test.txt", DiffType::Modified)
        .add_line_removed("old line")
        .add_line_added("new line");
    let output = renderer.render_file_diff(&diff);

    assert!(output.contains("test.txt"));
    assert!(output.contains("old line"));
    assert!(output.contains("new line"));
    assert!(output.contains("+1"));
    assert!(output.contains("-1"));
}

#[test]
fn test_diff_renderer_render_file_diff_with_context() {
    use crate::output::diff::{DiffRenderer, DiffType, FileDiff};

    let renderer = DiffRenderer::new(true);
    let diff = FileDiff::new("package.json", DiffType::Modified)
        .with_context("Version bump")
        .add_line_removed("  \"version\": \"1.0.0\",")
        .add_line_added("  \"version\": \"2.0.0\",");
    let output = renderer.render_file_diff(&diff);

    assert!(output.contains("package.json"));
    assert!(output.contains("Version bump"));
}

#[test]
fn test_diff_renderer_render_dependency_diff() {
    use crate::output::diff::{DependencyDiff, DiffRenderer};

    let renderer = DiffRenderer::new(true);
    let diff = DependencyDiff::new("lodash", "^4.0.0", "^5.0.0");
    let output = renderer.render_dependency_diff(&diff);

    assert!(output.contains("lodash"));
    assert!(output.contains("^4.0.0"));
    assert!(output.contains("^5.0.0"));
}

#[test]
fn test_diff_renderer_render_dependency_diff_with_package_context() {
    use crate::output::diff::{DependencyDiff, DiffRenderer};

    let renderer = DiffRenderer::new(true);
    let diff =
        DependencyDiff::new("react", "^17.0.0", "^18.0.0").with_package_context("@org/frontend");
    let output = renderer.render_dependency_diff(&diff);

    assert!(output.contains("react"));
    assert!(output.contains("@org/frontend"));
}

#[test]
fn test_diff_renderer_render_dependency_diff_with_dep_type() {
    use crate::output::diff::{DependencyDiff, DiffRenderer};

    let renderer = DiffRenderer::new(true);
    let diff = DependencyDiff::new("jest", "^27.0.0", "^28.0.0").with_dep_type("devDependencies");
    let output = renderer.render_dependency_diff(&diff);

    assert!(output.contains("jest"));
    assert!(output.contains("devDependencies"));
}

#[test]
fn test_diff_renderer_render_version_summary() {
    use crate::output::diff::{DiffRenderer, VersionDiff};

    let renderer = DiffRenderer::new(true);
    let diffs = vec![
        VersionDiff::new("pkg1", "1.0.0", "2.0.0"),
        VersionDiff::new("pkg2", "3.0.0", "3.1.0"),
        VersionDiff::new("pkg3", "5.0.0", "5.0.0").with_will_change(false),
    ];
    let output = renderer.render_version_summary(&diffs);

    assert!(output.contains("Version Changes"));
    assert!(output.contains("pkg1"));
    assert!(output.contains("pkg2"));
    assert!(output.contains("pkg3"));
    assert!(output.contains("Total"));
    assert!(output.contains("3 packages"));
    assert!(output.contains("2 changed"));
    assert!(output.contains("1 unchanged"));
}

#[test]
fn test_diff_renderer_render_version_diff_with_color() {
    use crate::output::diff::{DiffRenderer, VersionDiff};

    let renderer = DiffRenderer::new(false); // Enable colors
    let diff = VersionDiff::new("pkg", "1.0.0", "2.0.0");
    let output = renderer.render_version_diff(&diff);

    // Just ensure it doesn't panic with colors enabled
    assert!(output.contains("pkg"));
}

#[test]
fn test_diff_renderer_render_file_diff_with_color() {
    use crate::output::diff::{DiffRenderer, DiffType, FileDiff};

    let renderer = DiffRenderer::new(false); // Enable colors
    let diff = FileDiff::new("file.txt", DiffType::Modified).add_line_added("new");
    let output = renderer.render_file_diff(&diff);

    // Just ensure it doesn't panic with colors enabled
    assert!(output.contains("file.txt"));
}

#[test]
fn test_version_diff_helper_function() {
    use crate::output::diff::version_diff;

    let diff = version_diff("pkg", "1.0.0", "2.0.0");
    assert_eq!(diff.package, "pkg");
    assert_eq!(diff.from_version, "1.0.0");
    assert_eq!(diff.to_version, "2.0.0");
}

#[test]
fn test_file_diff_modified_helper_function() {
    use crate::output::diff::{DiffType, file_diff_modified};

    let diff = file_diff_modified("test.txt");
    assert_eq!(diff.path, "test.txt");
    assert_eq!(diff.file_type, DiffType::Modified);
}

#[test]
fn test_file_diff_added_helper_function() {
    use crate::output::diff::{DiffType, file_diff_added};

    let diff = file_diff_added("new.txt");
    assert_eq!(diff.path, "new.txt");
    assert_eq!(diff.file_type, DiffType::Added);
}

#[test]
fn test_file_diff_deleted_helper_function() {
    use crate::output::diff::{DiffType, file_diff_deleted};

    let diff = file_diff_deleted("old.txt");
    assert_eq!(diff.path, "old.txt");
    assert_eq!(diff.file_type, DiffType::Deleted);
}

#[test]
fn test_dependency_diff_helper_function() {
    use crate::output::diff::dependency_diff;

    let diff = dependency_diff("lodash", "^4.0.0", "^5.0.0");
    assert_eq!(diff.name, "lodash");
    assert_eq!(diff.from_version, "^4.0.0");
    assert_eq!(diff.to_version, "^5.0.0");
}

#[test]
fn test_diff_line_render_unchanged() {
    use crate::output::diff::{DiffLine, DiffType};

    let line = DiffLine::new(DiffType::Unchanged, "context line");
    let rendered = line.render(true);
    assert!(rendered.contains("context line"));
}

#[test]
fn test_file_diff_empty_stats() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified);
    assert_eq!(diff.added_count(), 0);
    assert_eq!(diff.removed_count(), 0);
    assert_eq!(diff.modified_count(), 0);
}

#[test]
fn test_file_diff_mixed_lines() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified)
        .add_line_context("line 1")
        .add_line_removed("old line 2")
        .add_line_added("new line 2")
        .add_line_modified("changed line 3")
        .add_line_context("line 4");

    assert_eq!(diff.lines.len(), 5);
    assert_eq!(diff.added_count(), 1);
    assert_eq!(diff.removed_count(), 1);
    assert_eq!(diff.modified_count(), 1);
}

#[test]
fn test_version_diff_builder_pattern() {
    use crate::output::diff::VersionDiff;

    let diff = VersionDiff::new("pkg", "1.0.0", "2.0.0")
        .with_reason("Breaking changes")
        .with_will_change(true);

    assert_eq!(diff.package, "pkg");
    assert_eq!(diff.from_version, "1.0.0");
    assert_eq!(diff.to_version, "2.0.0");
    assert_eq!(diff.reason, Some("Breaking changes".to_string()));
    assert!(diff.will_change);
}

#[test]
fn test_dependency_diff_builder_pattern() {
    use crate::output::diff::DependencyDiff;

    let diff = DependencyDiff::new("react", "^17.0.0", "^18.0.0")
        .with_dep_type("dependencies")
        .with_package_context("@org/app");

    assert_eq!(diff.name, "react");
    assert_eq!(diff.dep_type, Some("dependencies".to_string()));
    assert_eq!(diff.package_context, Some("@org/app".to_string()));
}

#[test]
fn test_diff_renderer_builder_pattern() {
    use crate::output::diff::DiffRenderer;

    let _renderer = DiffRenderer::new(true).with_line_numbers(true).with_context_lines(5);

    // Builder pattern chains successfully
}

#[test]
fn test_file_diff_builder_pattern() {
    use crate::output::diff::{DiffType, FileDiff};

    let diff = FileDiff::new("test.txt", DiffType::Modified)
        .with_context("Version update")
        .add_line_removed("old")
        .add_line_added("new");

    assert_eq!(diff.context, Some("Version update".to_string()));
    assert_eq!(diff.lines.len(), 2);
}

#[test]
fn test_progress_bar_inc_multiple_times() {
    use crate::output::progress::ProgressBar;

    let pb = ProgressBar::new(100);
    pb.inc(10);
    pb.inc(20);
    pb.inc(30);
    pb.finish();
}

#[test]
fn test_spinner_tick_multiple_times() {
    use crate::output::progress::Spinner;

    let spinner = Spinner::new("Loading...");
    for _ in 0..5 {
        spinner.tick();
    }
    spinner.finish();
}

#[test]
fn test_progress_suppression_in_json_mode() {
    use crate::output::OutputFormat;
    use crate::output::progress::{MultiProgress, ProgressBar, Spinner};

    // Verify all progress types are suppressed in JSON mode
    let pb = ProgressBar::new_with_format(100, OutputFormat::Json);
    let spinner = Spinner::new_with_format("Test", OutputFormat::Json);
    let multi = MultiProgress::new_with_format(OutputFormat::Json);

    assert!(!pb.is_active());
    assert!(!spinner.is_active());
    assert!(!multi.is_active());
}

#[test]
fn test_progress_suppression_in_quiet_mode() {
    use crate::output::OutputFormat;
    use crate::output::progress::{MultiProgress, ProgressBar, Spinner};

    // Verify all progress types are suppressed in quiet mode
    let pb = ProgressBar::new_with_format(100, OutputFormat::Quiet);
    let spinner = Spinner::new_with_format("Test", OutputFormat::Quiet);
    let multi = MultiProgress::new_with_format(OutputFormat::Quiet);

    assert!(!pb.is_active());
    assert!(!spinner.is_active());
    assert!(!multi.is_active());
}

#[test]
fn test_multi_progress_mixed_indicators() {
    use crate::output::progress::MultiProgress;

    let multi = MultiProgress::new();

    // Add multiple different types of indicators
    let pb1 = multi.add_progress_bar(100);
    let spinner1 = multi.add_spinner("Task 1");
    let pb2 = multi.add_progress_bar(50);
    let spinner2 = multi.add_spinner("Task 2");

    pb1.set_message("Progress 1");
    pb2.set_message("Progress 2");
    spinner1.set_message("Spinner 1");
    spinner2.set_message("Spinner 2");

    pb1.finish();
    spinner1.finish();
    pb2.finish();
    spinner2.finish();

    multi.clear();
}

// ============================================================================
// Logger Module Tests
// ============================================================================

#[test]
fn test_log_level_to_tracing_level_mapping() {
    use crate::cli::LogLevel;

    assert_eq!(LogLevel::Silent.to_tracing_level(), tracing::Level::ERROR);
    assert_eq!(LogLevel::Error.to_tracing_level(), tracing::Level::ERROR);
    assert_eq!(LogLevel::Warn.to_tracing_level(), tracing::Level::WARN);
    assert_eq!(LogLevel::Info.to_tracing_level(), tracing::Level::INFO);
    assert_eq!(LogLevel::Debug.to_tracing_level(), tracing::Level::DEBUG);
    assert_eq!(LogLevel::Trace.to_tracing_level(), tracing::Level::TRACE);
}

#[test]
fn test_log_level_is_silent() {
    use crate::cli::LogLevel;

    assert!(LogLevel::Silent.is_silent());
    assert!(!LogLevel::Error.is_silent());
    assert!(!LogLevel::Warn.is_silent());
    assert!(!LogLevel::Info.is_silent());
    assert!(!LogLevel::Debug.is_silent());
    assert!(!LogLevel::Trace.is_silent());
}

#[test]
fn test_log_level_includes_errors() {
    use crate::cli::LogLevel;

    assert!(!LogLevel::Silent.includes_errors());
    assert!(LogLevel::Error.includes_errors());
    assert!(LogLevel::Warn.includes_errors());
    assert!(LogLevel::Info.includes_errors());
    assert!(LogLevel::Debug.includes_errors());
    assert!(LogLevel::Trace.includes_errors());
}

#[test]
fn test_log_level_includes_warnings() {
    use crate::cli::LogLevel;

    assert!(!LogLevel::Silent.includes_warnings());
    assert!(!LogLevel::Error.includes_warnings());
    assert!(LogLevel::Warn.includes_warnings());
    assert!(LogLevel::Info.includes_warnings());
    assert!(LogLevel::Debug.includes_warnings());
    assert!(LogLevel::Trace.includes_warnings());
}

#[test]
fn test_log_level_includes_info() {
    use crate::cli::LogLevel;

    assert!(!LogLevel::Silent.includes_info());
    assert!(!LogLevel::Error.includes_info());
    assert!(!LogLevel::Warn.includes_info());
    assert!(LogLevel::Info.includes_info());
    assert!(LogLevel::Debug.includes_info());
    assert!(LogLevel::Trace.includes_info());
}

#[test]
fn test_log_level_includes_debug() {
    use crate::cli::LogLevel;

    assert!(!LogLevel::Silent.includes_debug());
    assert!(!LogLevel::Error.includes_debug());
    assert!(!LogLevel::Warn.includes_debug());
    assert!(!LogLevel::Info.includes_debug());
    assert!(LogLevel::Debug.includes_debug());
    assert!(LogLevel::Trace.includes_debug());
}

#[test]
fn test_log_level_includes_trace() {
    use crate::cli::LogLevel;

    assert!(!LogLevel::Silent.includes_trace());
    assert!(!LogLevel::Error.includes_trace());
    assert!(!LogLevel::Warn.includes_trace());
    assert!(!LogLevel::Info.includes_trace());
    assert!(!LogLevel::Debug.includes_trace());
    assert!(LogLevel::Trace.includes_trace());
}

#[test]
fn test_init_logging_silent_mode() {
    use crate::cli::LogLevel;
    use crate::output::logger::init_logging;

    // Silent mode should succeed without actually initializing logging
    let result = init_logging(LogLevel::Silent, false);
    assert!(result.is_ok());
}

#[test]
fn test_init_logging_silent_with_color_disabled() {
    use crate::cli::LogLevel;
    use crate::output::logger::init_logging;

    // Silent mode with no_color should still succeed
    let result = init_logging(LogLevel::Silent, true);
    assert!(result.is_ok());
}

#[test]
fn test_command_span_creation() {
    use crate::output::logger::command_span;

    // Test that span creation doesn't panic
    let span = command_span("test_command");
    // Span created successfully - if we got here, it didn't panic
    drop(span);
}

#[test]
fn test_operation_span_creation() {
    use crate::output::logger::operation_span;

    // Test that span creation doesn't panic
    let span = operation_span("test_operation");
    // Span created successfully - if we got here, it didn't panic
    drop(span);
}

#[test]
fn test_command_span_with_different_names() {
    use crate::output::logger::command_span;

    // Test that spans can be created with different names
    let span1 = command_span("init");
    let span2 = command_span("bump");
    let span3 = command_span("changeset");
    // All spans created successfully - if we got here, it didn't panic
    drop(span1);
    drop(span2);
    drop(span3);
}

#[test]
fn test_operation_span_with_different_names() {
    use crate::output::logger::operation_span;

    // Test that spans can be created with different names
    let span1 = operation_span("load_config");
    let span2 = operation_span("save_changeset");
    let span3 = operation_span("query_registry");
    // All spans created successfully - if we got here, it didn't panic
    drop(span1);
    drop(span2);
    drop(span3);
}

#[test]
fn test_span_can_be_dropped() {
    use crate::output::logger::command_span;

    {
        let _span = command_span("test");
        // Span is active here
    }
    // Span is dropped here - should not panic
}

#[test]
fn test_nested_spans() {
    use crate::output::logger::{command_span, operation_span};

    let command = command_span("test_command");
    {
        let operation = operation_span("nested_operation");
        // Both spans active here
        drop(operation);
    }
    // Inner span dropped, outer still active
    drop(command);
}

#[test]
fn test_multiple_sequential_spans() {
    use crate::output::logger::{command_span, operation_span};

    let span1 = command_span("command1");
    drop(span1);

    let span2 = command_span("command2");
    drop(span2);

    let span3 = operation_span("operation1");
    drop(span3);
    // All should work without issue
}

#[test]
fn test_log_level_default() {
    use crate::cli::LogLevel;

    assert_eq!(LogLevel::default(), LogLevel::Info);
}

#[test]
fn test_log_level_display() {
    use crate::cli::LogLevel;

    assert_eq!(LogLevel::Silent.to_string(), "silent");
    assert_eq!(LogLevel::Error.to_string(), "error");
    assert_eq!(LogLevel::Warn.to_string(), "warn");
    assert_eq!(LogLevel::Info.to_string(), "info");
    assert_eq!(LogLevel::Debug.to_string(), "debug");
    assert_eq!(LogLevel::Trace.to_string(), "trace");
}

#[test]
fn test_log_level_from_str() {
    use crate::cli::LogLevel;

    assert_eq!("silent".parse::<LogLevel>().ok(), Some(LogLevel::Silent));
    assert_eq!("error".parse::<LogLevel>().ok(), Some(LogLevel::Error));
    assert_eq!("warn".parse::<LogLevel>().ok(), Some(LogLevel::Warn));
    assert_eq!("warning".parse::<LogLevel>().ok(), Some(LogLevel::Warn)); // Alias
    assert_eq!("info".parse::<LogLevel>().ok(), Some(LogLevel::Info));
    assert_eq!("debug".parse::<LogLevel>().ok(), Some(LogLevel::Debug));
    assert_eq!("trace".parse::<LogLevel>().ok(), Some(LogLevel::Trace));
}

#[test]
fn test_log_level_from_str_case_insensitive() {
    use crate::cli::LogLevel;

    assert_eq!("SILENT".parse::<LogLevel>().ok(), Some(LogLevel::Silent));
    assert_eq!("Error".parse::<LogLevel>().ok(), Some(LogLevel::Error));
    assert_eq!("WARN".parse::<LogLevel>().ok(), Some(LogLevel::Warn));
    assert_eq!("INFO".parse::<LogLevel>().ok(), Some(LogLevel::Info));
    assert_eq!("DeBuG".parse::<LogLevel>().ok(), Some(LogLevel::Debug));
    assert_eq!("TrAcE".parse::<LogLevel>().ok(), Some(LogLevel::Trace));
}

#[test]
fn test_log_level_from_str_invalid() {
    use crate::cli::LogLevel;

    assert!("invalid".parse::<LogLevel>().is_err());
    assert!("verbose".parse::<LogLevel>().is_err());
    assert!("critical".parse::<LogLevel>().is_err());
    assert!("".parse::<LogLevel>().is_err());
}

#[test]
fn test_command_span_empty_name() {
    use crate::output::logger::command_span;

    // Test that span creation with empty name doesn't panic
    let span = command_span("");
    // If we got here, it didn't panic
    drop(span);
}

#[test]
fn test_operation_span_empty_name() {
    use crate::output::logger::operation_span;

    // Test that span creation with empty name doesn't panic
    let span = operation_span("");
    // If we got here, it didn't panic
    drop(span);
}

#[test]
fn test_command_span_with_special_characters() {
    use crate::output::logger::command_span;

    // Test that span creation with special characters doesn't panic
    let span = command_span("test-command_with.special/chars");
    // If we got here, it didn't panic
    drop(span);
}

#[test]
fn test_operation_span_with_special_characters() {
    use crate::output::logger::operation_span;

    // Test that span creation with special characters doesn't panic
    let span = operation_span("load:config@v1");
    // If we got here, it didn't panic
    drop(span);
}

#[test]
fn test_log_level_ordering() {
    use crate::cli::LogLevel;

    // Test that log levels have the expected ordering
    assert!(LogLevel::Silent < LogLevel::Error);
    assert!(LogLevel::Error < LogLevel::Warn);
    assert!(LogLevel::Warn < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Debug);
    assert!(LogLevel::Debug < LogLevel::Trace);
}

#[test]
fn test_span_functions_are_callable() {
    use crate::output::logger::{command_span, operation_span};

    // Test that both span functions can be called without panicking
    let command = command_span("test");
    let operation = operation_span("test");
    // Both functions executed successfully - if we got here, they didn't panic
    drop(command);
    drop(operation);
}

#[test]
fn test_init_logging_creates_proper_filter() {
    use crate::cli::LogLevel;
    use crate::output::logger::init_logging;

    // This test verifies that init_logging doesn't panic with valid inputs
    // Actual logging behavior is tested in integration tests

    // Test with different log levels
    let levels =
        vec![LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug, LogLevel::Trace];

    for level in levels {
        // Each level should work with both color settings
        let result1 = init_logging(level, false);
        let result2 = init_logging(level, true);

        // Due to global subscriber, only the first will succeed
        // but none should panic
        let _ = result1;
        let _ = result2;
    }
}

#[test]
fn test_module_exports_tracing_macros() {
    // Verify that the re-exported macros are available
    // This is a compile-time test - if it compiles, the exports work

    #[allow(unused_imports)]
    use crate::output::logger::{debug, error, info, trace, warn};

    // If this compiles, the exports are working correctly
    // No assertion needed - compilation is the test
}

// ============================================================================
// Export Tests
// ============================================================================

#[test]
fn test_export_format_extension() {
    assert_eq!(export::ExportFormat::Html.extension(), "html");
    assert_eq!(export::ExportFormat::Markdown.extension(), "md");
}

#[test]
fn test_export_format_mime_type() {
    assert_eq!(export::ExportFormat::Html.mime_type(), "text/html");
    assert_eq!(export::ExportFormat::Markdown.mime_type(), "text/markdown");
}

#[test]
fn test_export_format_display() {
    assert_eq!(export::ExportFormat::Html.to_string(), "HTML");
    assert_eq!(export::ExportFormat::Markdown.to_string(), "Markdown");
}

#[test]
fn test_export_format_from_str() {
    use std::str::FromStr;

    assert_eq!(export::ExportFormat::from_str("html").unwrap(), export::ExportFormat::Html);
    assert_eq!(export::ExportFormat::from_str("htm").unwrap(), export::ExportFormat::Html);
    assert_eq!(export::ExportFormat::from_str("HTML").unwrap(), export::ExportFormat::Html);

    assert_eq!(export::ExportFormat::from_str("markdown").unwrap(), export::ExportFormat::Markdown);
    assert_eq!(export::ExportFormat::from_str("md").unwrap(), export::ExportFormat::Markdown);
    assert_eq!(export::ExportFormat::from_str("MARKDOWN").unwrap(), export::ExportFormat::Markdown);

    assert!(export::ExportFormat::from_str("invalid").is_err());
    assert!(export::ExportFormat::from_str("pdf").is_err());
}

#[test]
fn test_html_exporter_basic() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Test Report");
    let data = json!({
        "test": "value",
        "number": 42
    });

    let html = exporter.export(&data).unwrap();

    // Verify HTML structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("<head>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</html>"));

    // Verify title
    assert!(html.contains("<title>Test Report</title>"));
    assert!(html.contains("<h1>Test Report</h1>"));

    // Verify content
    assert!(html.contains("test"));
    assert!(html.contains("value"));
    assert!(html.contains("42"));

    // Verify CSS is included
    assert!(html.contains("<style>"));
    assert!(html.contains("</style>"));

    // Verify footer
    assert!(html.contains("Generated by Workspace Node Tools CLI"));
}

#[test]
fn test_html_exporter_with_nested_object() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Nested Report");
    let data = json!({
        "section1": {
            "key1": "value1",
            "key2": "value2"
        },
        "section2": {
            "nested": {
                "deep": "value"
            }
        }
    });

    let html = exporter.export(&data).unwrap();

    // Verify sections are present
    assert!(html.contains("section1"));
    assert!(html.contains("section2"));
    assert!(html.contains("key1"));
    assert!(html.contains("value1"));
    assert!(html.contains("nested"));
    assert!(html.contains("deep"));
}

#[test]
fn test_html_exporter_with_array() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Array Report");
    let data = json!({
        "items": ["item1", "item2", "item3"]
    });

    let html = exporter.export(&data).unwrap();

    // Verify array items are rendered
    assert!(html.contains("items"));
    assert!(html.contains("item1"));
    assert!(html.contains("item2"));
    assert!(html.contains("item3"));

    // Arrays should be rendered as lists
    assert!(html.contains("<ul>") || html.contains("<li>"));
}

#[test]
fn test_html_exporter_empty_object() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Empty Report");
    let data = json!({});

    let html = exporter.export(&data).unwrap();

    // Should still generate valid HTML
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Empty Report"));
}

#[test]
fn test_html_exporter_special_characters() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Special <Chars> & \"Quotes\"");
    let data = json!({
        "special": "<script>alert('xss')</script>",
        "ampersand": "A & B",
        "quotes": "\"quoted\""
    });

    let html = exporter.export(&data).unwrap();

    // Verify HTML escaping
    assert!(html.contains("&lt;") || html.contains("&amp;") || html.contains("&quot;"));
    // Should NOT contain unescaped script tags
    assert!(!html.contains("<script>alert"));
}

#[test]
fn test_markdown_exporter_basic() {
    use serde_json::json;

    let exporter = export::MarkdownExporter::new("Test Report");
    let data = json!({
        "test": "value",
        "number": 42
    });

    let markdown = exporter.export(&data).unwrap();

    // Verify markdown structure
    assert!(markdown.starts_with("# Test Report"));

    // Verify content
    assert!(markdown.contains("test"));
    assert!(markdown.contains("value"));
    assert!(markdown.contains("42"));

    // Verify footer
    assert!(markdown.contains("Generated by Workspace Node Tools CLI"));

    // Verify timestamp
    assert!(markdown.contains("Generated on"));
}

#[test]
fn test_markdown_exporter_with_nested_object() {
    use serde_json::json;

    let exporter = export::MarkdownExporter::new("Nested Report");
    let data = json!({
        "section1": {
            "key1": "value1",
            "key2": "value2"
        },
        "section2": {
            "nested": {
                "deep": "value"
            }
        }
    });

    let markdown = exporter.export(&data).unwrap();

    // Verify sections are present as headings
    assert!(markdown.contains("## section1") || markdown.contains("##section1"));
    assert!(markdown.contains("## section2") || markdown.contains("##section2"));
    assert!(markdown.contains("key1"));
    assert!(markdown.contains("value1"));
    assert!(markdown.contains("nested"));
}

#[test]
fn test_markdown_exporter_with_array() {
    use serde_json::json;

    let exporter = export::MarkdownExporter::new("Array Report");
    let data = json!({
        "items": ["item1", "item2", "item3"]
    });

    let markdown = exporter.export(&data).unwrap();

    // Verify array items are rendered
    assert!(markdown.contains("items"));
    assert!(markdown.contains("item1"));
    assert!(markdown.contains("item2"));
    assert!(markdown.contains("item3"));

    // Arrays should be rendered as lists
    assert!(markdown.contains("- "));
}

#[test]
fn test_markdown_exporter_empty_object() {
    use serde_json::json;

    let exporter = export::MarkdownExporter::new("Empty Report");
    let data = json!({});

    let markdown = exporter.export(&data).unwrap();

    // Should still generate valid markdown
    assert!(markdown.starts_with("# Empty Report"));
    assert!(markdown.contains("Generated by Workspace Node Tools CLI"));
}

#[test]
fn test_export_data_html() {
    use serde::Serialize;
    use tempfile::NamedTempFile;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        count: usize,
    }

    let data = TestData { name: "test".to_string(), count: 42 };

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    export::export_data(&data, export::ExportFormat::Html, path).unwrap();

    // Verify file was created and contains HTML
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("test"));
    assert!(content.contains("42"));
}

#[test]
fn test_export_data_markdown() {
    use serde::Serialize;
    use tempfile::NamedTempFile;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        count: usize,
    }

    let data = TestData { name: "test".to_string(), count: 42 };

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    export::export_data(&data, export::ExportFormat::Markdown, path).unwrap();

    // Verify file was created and contains Markdown
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.starts_with('#'));
    assert!(content.contains("test"));
    assert!(content.contains("42"));
}

#[test]
fn test_export_data_complex_structure() {
    use serde::Serialize;
    use tempfile::NamedTempFile;

    #[derive(Serialize)]
    struct Issue {
        severity: String,
        description: String,
    }

    #[derive(Serialize)]
    struct Report {
        title: String,
        issues: Vec<Issue>,
        score: Option<u8>,
    }

    let report = Report {
        title: "Audit Report".to_string(),
        issues: vec![
            Issue { severity: "high".to_string(), description: "Issue 1".to_string() },
            Issue { severity: "low".to_string(), description: "Issue 2".to_string() },
        ],
        score: Some(85),
    };

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Test HTML export
    export::export_data(&report, export::ExportFormat::Html, path).unwrap();
    let html_content = std::fs::read_to_string(path).unwrap();
    assert!(html_content.contains("Audit Report"));
    assert!(html_content.contains("Issue 1"));
    assert!(html_content.contains("Issue 2"));
    assert!(html_content.contains("85"));

    // Test Markdown export
    export::export_data(&report, export::ExportFormat::Markdown, path).unwrap();
    let md_content = std::fs::read_to_string(path).unwrap();
    assert!(md_content.contains("Audit Report"));
    assert!(md_content.contains("Issue 1"));
    assert!(md_content.contains("Issue 2"));
    assert!(md_content.contains("85"));
}

#[test]
fn test_export_data_invalid_path() {
    use serde::Serialize;
    use std::path::Path;

    #[derive(Serialize)]
    struct TestData {
        value: String,
    }

    let data = TestData { value: "test".to_string() };

    // Try to write to invalid path
    let invalid_path = Path::new("/nonexistent/directory/file.html");
    let result = export::export_data(&data, export::ExportFormat::Html, invalid_path);

    // Should fail
    assert!(result.is_err());
}

#[test]
fn test_html_exporter_with_null_values() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Null Test");
    let data = json!({
        "nullValue": null,
        "normalValue": "test"
    });

    let html = exporter.export(&data).unwrap();

    // Should handle null values
    assert!(html.contains("nullValue") || html.contains("null"));
    assert!(html.contains("normalValue"));
    assert!(html.contains("test"));
}

#[test]
fn test_markdown_exporter_with_null_values() {
    use serde_json::json;

    let exporter = export::MarkdownExporter::new("Null Test");
    let data = json!({
        "nullValue": null,
        "normalValue": "test"
    });

    let markdown = exporter.export(&data).unwrap();

    // Should handle null values
    assert!(markdown.contains("nullValue") || markdown.contains("null"));
    assert!(markdown.contains("normalValue"));
    assert!(markdown.contains("test"));
}

#[test]
fn test_html_exporter_with_booleans() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Boolean Test");
    let data = json!({
        "trueValue": true,
        "falseValue": false
    });

    let html = exporter.export(&data).unwrap();

    // Should render boolean values
    assert!(html.contains("true"));
    assert!(html.contains("false"));
}

#[test]
fn test_markdown_exporter_with_booleans() {
    use serde_json::json;

    let exporter = export::MarkdownExporter::new("Boolean Test");
    let data = json!({
        "trueValue": true,
        "falseValue": false
    });

    let markdown = exporter.export(&data).unwrap();

    // Should render boolean values
    assert!(markdown.contains("true"));
    assert!(markdown.contains("false"));
}

#[test]
fn test_html_exporter_large_dataset() {
    use serde_json::json;

    let exporter = export::HtmlExporter::new("Large Dataset");
    let mut items = Vec::new();
    for i in 0..100 {
        items.push(format!("item{i}"));
    }

    let data = json!({
        "items": items
    });

    let html = exporter.export(&data).unwrap();

    // Should handle large datasets
    assert!(html.contains("item0"));
    assert!(html.contains("item50"));
    assert!(html.contains("item99"));
}

#[test]
fn test_markdown_exporter_large_dataset() {
    use serde_json::json;

    let exporter = export::MarkdownExporter::new("Large Dataset");
    let mut items = Vec::new();
    for i in 0..100 {
        items.push(format!("item{i}"));
    }

    let data = json!({
        "items": items
    });

    let markdown = exporter.export(&data).unwrap();

    // Should handle large datasets
    assert!(markdown.contains("item0"));
    assert!(markdown.contains("item50"));
    assert!(markdown.contains("item99"));
}
