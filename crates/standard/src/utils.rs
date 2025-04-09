//! # Utility Functions Module
//!
//! This module provides general utility functions used throughout the crate.
//!
//! Currently, it includes string manipulation utilities for handling command outputs.

/// Strips the trailing newline from a string.
///
/// This function removes any trailing newline characters from the input string.
/// It handles both Windows-style (`\r\n`) and Unix-style (`\n`) line endings.
///
/// # Arguments
///
/// * `input` - A reference to a String from which trailing newlines will be removed
///
/// # Returns
///
/// A new String with any trailing newline characters removed and whitespace trimmed.
///
/// # Examples
///
/// ```
/// let input = String::from("Hello, world!\n");
/// let result = strip_trailing_newline(&input);
/// assert_eq!(result, "Hello, world!");
///
/// let input = String::from("Hello, world!\r\n");
/// let result = strip_trailing_newline(&input);
/// assert_eq!(result, "Hello, world!");
/// ```
pub fn strip_trailing_newline(input: &String) -> String {
    input.strip_suffix("\r\n").or(input.strip_suffix("\n")).unwrap_or(input).trim().to_string()
}
