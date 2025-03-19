//! JavaScript bindings for string utilities

use napi_derive::napi;
use ws_std::utils::strip_trailing_newline as ws_strip_newline;

/// Strips the trailing newline from a string
///
/// @param {string} input - The input string
/// @returns {string} String with trailing newline removed
#[napi]
pub fn strip_trailing_newline(input: String) -> String {
    ws_strip_newline(&input)
}

#[cfg(test)]
mod utils_binding_tests {
    use super::*;

    #[test]
    fn test_strip_trailing_newline() {
        // Test with Unix newline
        let input_unix = "Hello, world!\n".to_string();
        assert_eq!(strip_trailing_newline(input_unix), "Hello, world!");

        // Test with Windows newline
        let input_windows = "Hello, world!\r\n".to_string();
        assert_eq!(strip_trailing_newline(input_windows), "Hello, world!");

        // Test with no newline
        let input_no_newline = "Hello, world!".to_string();
        assert_eq!(strip_trailing_newline(input_no_newline), "Hello, world!");

        // Test with multiple newlines - only the last should be stripped
        let input_multiple = "Hello,\nworld!\n".to_string();
        assert_eq!(strip_trailing_newline(input_multiple), "Hello,\nworld!");

        // Test with empty string
        let input_empty = "".to_string();
        assert_eq!(strip_trailing_newline(input_empty), "");
    }
}
