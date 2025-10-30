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
