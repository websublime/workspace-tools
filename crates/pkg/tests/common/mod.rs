//! # Common Test Utilities
//!
//! This module provides reusable test utilities, mock implementations, and test fixtures
//! for the `sublime_pkg_tools` crate. It includes:
//!
//! - Mock implementations of external dependencies (filesystem, git, registry)
//! - Assertion helpers for common test scenarios
//! - Property-based testing generators
//! - Test fixture management
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::common::{MockFileSystem, create_test_monorepo};
//!
//! #[tokio::test]
//! async fn test_with_mock_fs() {
//!     let fs = MockFileSystem::new();
//!     // Use the mock filesystem in your tests
//! }
//! ```

pub mod assertions;
pub mod fixtures;
pub mod generators;
pub mod mocks;

// Re-export commonly used items
pub use assertions::*;
pub use fixtures::*;
pub use generators::*;
pub use mocks::*;

use std::path::PathBuf;

/// Helper function to get the fixtures directory path
///
/// # Returns
///
/// Returns the absolute path to the test fixtures directory
///
/// # Examples
///
/// ```rust,ignore
/// let fixtures_path = fixtures_dir();
/// let monorepo_path = fixtures_path.join("monorepo");
/// ```
#[must_use]
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

/// Helper function to get a specific fixture path
///
/// # Arguments
///
/// * `name` - The name of the fixture directory
///
/// # Returns
///
/// Returns the absolute path to the specified fixture
///
/// # Examples
///
/// ```rust,ignore
/// let monorepo = fixture_path("monorepo");
/// assert!(monorepo.exists());
/// ```
#[must_use]
pub fn fixture_path(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

/// Creates a temporary directory for tests
///
/// # Returns
///
/// Returns a `tempfile::TempDir` that will be cleaned up when dropped
///
/// # Errors
///
/// Returns an error if the temporary directory cannot be created
///
/// # Examples
///
/// ```rust,ignore
/// let temp = create_temp_dir().unwrap();
/// let temp_path = temp.path();
/// // Use temp_path for tests
/// ```
pub fn create_temp_dir() -> Result<tempfile::TempDir, std::io::Error> {
    tempfile::tempdir()
}

/// Creates a temporary directory with a specific prefix
///
/// # Arguments
///
/// * `prefix` - The prefix for the temporary directory name
///
/// # Returns
///
/// Returns a `tempfile::TempDir` that will be cleaned up when dropped
///
/// # Errors
///
/// Returns an error if the temporary directory cannot be created
///
/// # Examples
///
/// ```rust,ignore
/// let temp = create_temp_dir_with_prefix("test-pkg").unwrap();
/// ```
pub fn create_temp_dir_with_prefix(prefix: &str) -> Result<tempfile::TempDir, std::io::Error> {
    tempfile::Builder::new().prefix(prefix).tempdir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_dir_exists() {
        let dir = fixtures_dir();
        assert!(dir.is_absolute());
        assert!(dir.ends_with("tests/fixtures"));
    }

    #[test]
    fn test_fixture_path() {
        let path = fixture_path("test");
        assert!(path.ends_with("tests/fixtures/test"));
    }

    #[test]
    fn test_create_temp_dir() {
        let temp = create_temp_dir().unwrap();
        assert!(temp.path().exists());
        assert!(temp.path().is_dir());
    }

    #[test]
    fn test_create_temp_dir_with_prefix() {
        let temp = create_temp_dir_with_prefix("pkg-test").unwrap();
        assert!(temp.path().exists());
        assert!(temp.path().is_dir());
        let name = temp.path().file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("pkg-test"));
    }
}
