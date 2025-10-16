//! # Test Assertion Helpers
//!
//! This module provides custom assertion helpers for common test scenarios
//! in the `sublime_pkg_tools` crate.
//!
//! ## What
//!
//! Provides convenient assertion macros and functions for:
//! - Version comparison assertions
//! - File content assertions
//! - Package structure assertions
//! - Changeset validation assertions
//!
//! ## How
//!
//! Each assertion function takes the values to compare and provides detailed
//! error messages when assertions fail, making test failures easier to debug.
//!
//! ## Why
//!
//! Custom assertions provide:
//! - More descriptive error messages
//! - Reusable test logic
//! - Consistent test patterns across the codebase

use semver::Version;
use std::path::Path;

/// Asserts that a version string matches the expected version
///
/// # Arguments
///
/// * `actual` - The actual version string
/// * `expected` - The expected version string
///
/// # Panics
///
/// Panics if the versions don't match or if either string is not a valid version
///
/// # Examples
///
/// ```rust,ignore
/// assert_version_eq("1.2.3", "1.2.3"); // OK
/// assert_version_eq("1.2.3", "1.2.4"); // Panics
/// ```
pub fn assert_version_eq(actual: &str, expected: &str) {
    let actual_ver =
        Version::parse(actual).unwrap_or_else(|_| panic!("Invalid actual version: {}", actual));
    let expected_ver = Version::parse(expected)
        .unwrap_or_else(|_| panic!("Invalid expected version: {}", expected));

    assert_eq!(actual_ver, expected_ver, "Version mismatch: expected {}, got {}", expected, actual);
}

/// Asserts that a version is greater than another version
///
/// # Arguments
///
/// * `actual` - The actual version string
/// * `expected_min` - The minimum expected version string (exclusive)
///
/// # Panics
///
/// Panics if actual is not greater than expected_min
///
/// # Examples
///
/// ```rust,ignore
/// assert_version_gt("1.2.3", "1.2.2"); // OK
/// assert_version_gt("1.2.3", "1.2.3"); // Panics
/// ```
pub fn assert_version_gt(actual: &str, expected_min: &str) {
    let actual_ver =
        Version::parse(actual).unwrap_or_else(|_| panic!("Invalid actual version: {}", actual));
    let min_ver = Version::parse(expected_min)
        .unwrap_or_else(|_| panic!("Invalid expected version: {}", expected_min));

    assert!(actual_ver > min_ver, "Version {} is not greater than {}", actual, expected_min);
}

/// Asserts that a version is greater than or equal to another version
///
/// # Arguments
///
/// * `actual` - The actual version string
/// * `expected_min` - The minimum expected version string (inclusive)
///
/// # Panics
///
/// Panics if actual is less than expected_min
///
/// # Examples
///
/// ```rust,ignore
/// assert_version_gte("1.2.3", "1.2.3"); // OK
/// assert_version_gte("1.2.3", "1.2.4"); // Panics
/// ```
pub fn assert_version_gte(actual: &str, expected_min: &str) {
    let actual_ver =
        Version::parse(actual).unwrap_or_else(|_| panic!("Invalid actual version: {}", actual));
    let min_ver = Version::parse(expected_min)
        .unwrap_or_else(|_| panic!("Invalid expected version: {}", expected_min));

    assert!(
        actual_ver >= min_ver,
        "Version {} is not greater than or equal to {}",
        actual,
        expected_min
    );
}

/// Asserts that a path exists
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Panics
///
/// Panics if the path does not exist
///
/// # Examples
///
/// ```rust,ignore
/// assert_path_exists(Path::new("/tmp")); // OK on Unix
/// assert_path_exists(Path::new("/nonexistent")); // Panics
/// ```
pub fn assert_path_exists(path: &Path) {
    assert!(path.exists(), "Path does not exist: {}", path.display());
}

/// Asserts that a path does not exist
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Panics
///
/// Panics if the path exists
///
/// # Examples
///
/// ```rust,ignore
/// assert_path_not_exists(Path::new("/nonexistent")); // OK
/// ```
pub fn assert_path_not_exists(path: &Path) {
    assert!(!path.exists(), "Path exists but should not: {}", path.display());
}

/// Asserts that a path is a file
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Panics
///
/// Panics if the path is not a file or does not exist
///
/// # Examples
///
/// ```rust,ignore
/// assert_is_file(Path::new("/etc/hosts")); // OK on Unix
/// ```
pub fn assert_is_file(path: &Path) {
    assert!(path.exists(), "Path does not exist: {}", path.display());
    assert!(path.is_file(), "Path is not a file: {}", path.display());
}

/// Asserts that a path is a directory
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Panics
///
/// Panics if the path is not a directory or does not exist
///
/// # Examples
///
/// ```rust,ignore
/// assert_is_dir(Path::new("/tmp")); // OK on Unix
/// ```
pub fn assert_is_dir(path: &Path) {
    assert!(path.exists(), "Path does not exist: {}", path.display());
    assert!(path.is_dir(), "Path is not a directory: {}", path.display());
}

/// Asserts that a string contains a substring
///
/// # Arguments
///
/// * `haystack` - The string to search in
/// * `needle` - The substring to search for
///
/// # Panics
///
/// Panics if the substring is not found
///
/// # Examples
///
/// ```rust,ignore
/// assert_contains("hello world", "world"); // OK
/// assert_contains("hello world", "goodbye"); // Panics
/// ```
pub fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "String does not contain expected substring.\nHaystack: {}\nNeedle: {}",
        haystack,
        needle
    );
}

/// Asserts that a string does not contain a substring
///
/// # Arguments
///
/// * `haystack` - The string to search in
/// * `needle` - The substring that should not be present
///
/// # Panics
///
/// Panics if the substring is found
///
/// # Examples
///
/// ```rust,ignore
/// assert_not_contains("hello world", "goodbye"); // OK
/// assert_not_contains("hello world", "world"); // Panics
/// ```
pub fn assert_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "String contains unexpected substring.\nHaystack: {}\nNeedle: {}",
        haystack,
        needle
    );
}

/// Asserts that a JSON string contains a specific field with an expected value
///
/// # Arguments
///
/// * `json_str` - The JSON string to parse
/// * `field` - The field path (e.g., "name" or "nested.field")
/// * `expected` - The expected value as a string
///
/// # Panics
///
/// Panics if the JSON is invalid, field doesn't exist, or value doesn't match
///
/// # Examples
///
/// ```rust,ignore
/// let json = r#"{"name": "test", "version": "1.0.0"}"#;
/// assert_json_field(json, "name", "test"); // OK
/// ```
pub fn assert_json_field(json_str: &str, field: &str, expected: &str) {
    let value: serde_json::Value =
        serde_json::from_str(json_str).unwrap_or_else(|e| panic!("Invalid JSON: {}", e));

    let actual = field.split('.').fold(Some(&value), |acc, key| acc.and_then(|v| v.get(key)));

    match actual {
        Some(actual_value) => {
            let actual_str = match actual_value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string().trim_matches('"').to_string(),
            };
            assert_eq!(
                actual_str, expected,
                "JSON field '{}' has unexpected value: expected '{}', got '{}'",
                field, expected, actual_str
            );
        }
        None => panic!("JSON field '{}' not found in: {}", field, json_str),
    }
}

/// Asserts that a collection has the expected length
///
/// # Arguments
///
/// * `collection` - The collection to check
/// * `expected_len` - The expected length
///
/// # Panics
///
/// Panics if the lengths don't match
///
/// # Examples
///
/// ```rust,ignore
/// let vec = vec![1, 2, 3];
/// assert_len(&vec, 3); // OK
/// assert_len(&vec, 4); // Panics
/// ```
pub fn assert_len<T>(collection: &[T], expected_len: usize) {
    assert_eq!(
        collection.len(),
        expected_len,
        "Collection length mismatch: expected {}, got {}",
        expected_len,
        collection.len()
    );
}

/// Asserts that a collection is empty
///
/// # Arguments
///
/// * `collection` - The collection to check
///
/// # Panics
///
/// Panics if the collection is not empty
///
/// # Examples
///
/// ```rust,ignore
/// let vec: Vec<i32> = vec![];
/// assert_empty(&vec); // OK
/// ```
pub fn assert_empty<T>(collection: &[T]) {
    assert!(collection.is_empty(), "Collection is not empty: contains {} items", collection.len());
}

/// Asserts that a collection is not empty
///
/// # Arguments
///
/// * `collection` - The collection to check
///
/// # Panics
///
/// Panics if the collection is empty
///
/// # Examples
///
/// ```rust,ignore
/// let vec = vec![1, 2, 3];
/// assert_not_empty(&vec); // OK
/// ```
pub fn assert_not_empty<T>(collection: &[T]) {
    assert!(!collection.is_empty(), "Collection is empty but should not be");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_assert_version_eq_success() {
        assert_version_eq("1.2.3", "1.2.3");
    }

    #[test]
    #[should_panic(expected = "Version mismatch")]
    fn test_assert_version_eq_failure() {
        assert_version_eq("1.2.3", "1.2.4");
    }

    #[test]
    fn test_assert_version_gt_success() {
        assert_version_gt("1.2.3", "1.2.2");
        assert_version_gt("2.0.0", "1.9.9");
    }

    #[test]
    #[should_panic(expected = "is not greater than")]
    fn test_assert_version_gt_failure() {
        assert_version_gt("1.2.3", "1.2.3");
    }

    #[test]
    fn test_assert_version_gte_success() {
        assert_version_gte("1.2.3", "1.2.3");
        assert_version_gte("1.2.3", "1.2.2");
    }

    #[test]
    #[should_panic(expected = "is not greater than or equal to")]
    fn test_assert_version_gte_failure() {
        assert_version_gte("1.2.2", "1.2.3");
    }

    #[test]
    fn test_assert_contains_success() {
        assert_contains("hello world", "world");
        assert_contains("hello world", "hello");
    }

    #[test]
    #[should_panic(expected = "does not contain")]
    fn test_assert_contains_failure() {
        assert_contains("hello world", "goodbye");
    }

    #[test]
    fn test_assert_not_contains_success() {
        assert_not_contains("hello world", "goodbye");
    }

    #[test]
    #[should_panic(expected = "contains unexpected")]
    fn test_assert_not_contains_failure() {
        assert_not_contains("hello world", "world");
    }

    #[test]
    fn test_assert_json_field_success() {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        assert_json_field(json, "name", "test");
        assert_json_field(json, "version", "1.0.0");
    }

    #[test]
    #[should_panic(expected = "has unexpected value")]
    fn test_assert_json_field_wrong_value() {
        let json = r#"{"name": "test"}"#;
        assert_json_field(json, "name", "other");
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn test_assert_json_field_missing() {
        let json = r#"{"name": "test"}"#;
        assert_json_field(json, "missing", "value");
    }

    #[test]
    fn test_assert_len_success() {
        let vec = vec![1, 2, 3];
        assert_len(&vec, 3);
    }

    #[test]
    #[should_panic(expected = "length mismatch")]
    fn test_assert_len_failure() {
        let vec = vec![1, 2, 3];
        assert_len(&vec, 4);
    }

    #[test]
    fn test_assert_empty_success() {
        let vec: Vec<i32> = vec![];
        assert_empty(&vec);
    }

    #[test]
    #[should_panic(expected = "not empty")]
    fn test_assert_empty_failure() {
        let vec = vec![1, 2, 3];
        assert_empty(&vec);
    }

    #[test]
    fn test_assert_not_empty_success() {
        let vec = vec![1, 2, 3];
        assert_not_empty(&vec);
    }

    #[test]
    #[should_panic(expected = "is empty")]
    fn test_assert_not_empty_failure() {
        let vec: Vec<i32> = vec![];
        assert_not_empty(&vec);
    }
}
