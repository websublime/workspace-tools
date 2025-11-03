//! # Common Test Utilities for CLI Integration Tests
//!
//! This module provides reusable test utilities and fixtures for CLI command
//! integration tests, including workspace setup helpers and assertion utilities.
//!
//! ## What
//!
//! Provides common test infrastructure:
//! - Temporary workspace creation with realistic structures
//! - Mock configuration generation
//! - Test fixture helpers
//! - Assertion utilities
//!
//! ## How
//!
//! Uses `tempfile` for temporary directories that are automatically cleaned up,
//! creates realistic package.json files and workspace structures, and provides
//! helpers for verifying command execution results.
//!
//! ## Why
//!
//! Centralizing test utilities ensures consistency across integration tests,
//! reduces code duplication, and makes tests easier to write and maintain.

use std::path::Path;

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
/// let temp = create_temp_dir_with_prefix("test-cli").unwrap();
/// ```
pub fn create_temp_dir_with_prefix(prefix: &str) -> Result<tempfile::TempDir, std::io::Error> {
    tempfile::Builder::new().prefix(prefix).tempdir()
}

/// Reads and parses a package.json file
///
/// # Arguments
///
/// * `package_dir` - Directory containing the package.json file
///
/// # Returns
///
/// Returns the parsed JSON value
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed
pub async fn read_package_json(
    package_dir: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let path = package_dir.join("package.json");
    let content = tokio::fs::read_to_string(&path).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    Ok(json)
}

/// Extracts the version from a package.json file
///
/// # Arguments
///
/// * `package_dir` - Directory containing the package.json file
///
/// # Returns
///
/// Returns the version string
///
/// # Errors
///
/// Returns an error if the file cannot be read or the version is missing
pub async fn get_package_version(package_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let json = read_package_json(package_dir).await?;
    json.get("version")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| "Missing version field".into())
}

/// Checks if a changeset file exists
///
/// # Arguments
///
/// * `workspace_root` - Workspace root directory
/// * `changeset_id` - Changeset identifier (usually branch name)
///
/// # Returns
///
/// Returns true if the changeset file exists
pub async fn changeset_exists(workspace_root: &Path, changeset_id: &str) -> bool {
    let changeset_path = workspace_root.join(".changesets").join(format!("{changeset_id}.json"));
    tokio::fs::try_exists(changeset_path).await.unwrap_or(false)
}

/// Checks if a changeset has been archived
///
/// # Arguments
///
/// * `workspace_root` - Workspace root directory
/// * `changeset_id` - Changeset identifier (usually branch name)
///
/// # Returns
///
/// Returns true if the changeset has been archived
pub async fn changeset_is_archived(workspace_root: &Path, changeset_id: &str) -> bool {
    let archive_dir = workspace_root.join(".changesets").join("history");
    if !tokio::fs::try_exists(&archive_dir).await.unwrap_or(false) {
        return false;
    }

    // Check if any archived changeset matches the ID
    let Ok(mut entries) = tokio::fs::read_dir(&archive_dir).await else {
        return false;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(file_name) = entry.file_name().to_str()
            && file_name.starts_with(changeset_id)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_dir() {
        let temp = create_temp_dir().unwrap();
        assert!(temp.path().exists());
        assert!(temp.path().is_dir());
    }

    #[test]
    fn test_create_temp_dir_with_prefix() {
        let temp = create_temp_dir_with_prefix("cli-test").unwrap();
        assert!(temp.path().exists());
        assert!(temp.path().is_dir());
        let name = temp.path().file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("cli-test"));
    }
}
