//! Custom assertions for E2E CLI tests.
//!
//! **What**: Provides domain-specific assertions for testing CLI behavior.
//!
//! **How**: Implements assertion functions for files, git, JSON, and CLI output.
//!
//! **Why**: Makes tests more readable and provides better error messages.

use serde::de::DeserializeOwned;
use std::path::Path;

// =============================================================================
// File System Assertions
// =============================================================================

/// Asserts that a file exists at the given path.
///
/// # Panics
///
/// Panics if the file does not exist with a descriptive message.
///
/// # Examples
///
/// ```rust,ignore
/// assert_file_exists(&workspace.root().join("package.json"));
/// ```
pub fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "File does not exist: {}", path.display());
    assert!(path.is_file(), "Path exists but is not a file: {}", path.display());
}

/// Asserts that a directory exists at the given path.
///
/// # Panics
///
/// Panics if the directory does not exist.
///
/// # Examples
///
/// ```rust,ignore
/// assert_dir_exists(&workspace.root().join(".changesets"));
/// ```
pub fn assert_dir_exists(path: &Path) {
    assert!(path.exists(), "Directory does not exist: {}", path.display());
    assert!(path.is_dir(), "Path exists but is not a directory: {}", path.display());
}

/// Asserts that a file contains specific text.
///
/// # Panics
///
/// Panics if the file does not contain the expected text.
///
/// # Examples
///
/// ```rust,ignore
/// assert_file_contains(
///     &workspace.root().join("CHANGELOG.md"),
///     "## 1.1.0"
/// );
/// ```
#[allow(clippy::expect_used)]
pub fn assert_file_contains(path: &Path, expected_content: &str) {
    assert_file_exists(path);

    let content = std::fs::read_to_string(path).expect("Failed to read file");

    assert!(
        content.contains(expected_content),
        "File {} does not contain expected content.\nExpected to find: {expected_content}\nActual content:\n{content}",
        path.display()
    );
}

/// Asserts that a file does NOT contain specific text.
///
/// # Panics
///
/// Panics if the file contains the unexpected text.
///
/// # Examples
///
/// ```rust,ignore
/// assert_file_not_contains(
///     &workspace.root().join("CHANGELOG.md"),
///     "Breaking Change"
/// );
/// ```
#[allow(clippy::expect_used)]
pub fn assert_file_not_contains(path: &Path, unexpected_content: &str) {
    assert_file_exists(path);

    let content = std::fs::read_to_string(path).expect("Failed to read file");

    assert!(
        !content.contains(unexpected_content),
        "File {} contains unexpected content.\nShould not contain: {unexpected_content}\nActual content:\n{content}",
        path.display()
    );
}

// =============================================================================
// JSON Assertions
// =============================================================================

/// Asserts that a JSON file is valid and can be parsed.
///
/// Returns the parsed JSON value for further assertions.
///
/// # Panics
///
/// Panics if the file is not valid JSON.
///
/// # Examples
///
/// ```rust,ignore
/// let json = assert_json_file_valid::<serde_json::Value>(
///     &workspace.root().join("package.json")
/// );
/// assert_eq!(json["version"], "1.1.0");
/// ```
#[allow(clippy::expect_used)]
pub fn assert_json_file_valid<T: DeserializeOwned>(path: &Path) -> T {
    assert_file_exists(path);

    let content = std::fs::read_to_string(path).expect("Failed to read file");

    serde_json::from_str(&content).unwrap_or_else(|e| {
        panic!("File {} contains invalid JSON: {e}\nContent:\n{content}", path.display())
    })
}

/// Asserts that a JSON file contains a specific key-value pair.
///
/// # Panics
///
/// Panics if the JSON does not contain the expected key or value.
///
/// # Examples
///
/// ```rust,ignore
/// assert_json_contains(
///     &workspace.root().join("package.json"),
///     "version",
///     "1.1.0"
/// );
/// ```
#[allow(clippy::expect_used)]
pub fn assert_json_contains(path: &Path, key: &str, expected_value: &str) {
    let json: serde_json::Value = assert_json_file_valid(path);

    let actual_value = json
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("Key '{key}' not found in JSON file {}", path.display()));

    assert_eq!(
        actual_value,
        expected_value,
        "JSON file {} key '{key}' has unexpected value.\nExpected: {expected_value}\nActual: {actual_value}",
        path.display()
    );
}

// =============================================================================
// Git Assertions
// =============================================================================

/// Asserts that a git tag exists in the repository.
///
/// # Panics
///
/// Panics if the tag does not exist.
///
/// # Examples
///
/// ```rust,ignore
/// assert_git_tag_exists(workspace.root(), "v1.1.0");
/// ```
#[allow(clippy::expect_used)]
pub fn assert_git_tag_exists(repo_path: &Path, tag: &str) {
    let output = std::process::Command::new("git")
        .args(["tag", "-l", tag])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run git tag");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim() == tag,
        "Git tag '{tag}' does not exist in repository {}",
        repo_path.display()
    );
}

/// Asserts that the git repository is clean (no uncommitted changes).
///
/// # Panics
///
/// Panics if there are uncommitted changes.
///
/// # Examples
///
/// ```rust,ignore
/// assert_git_clean(workspace.root());
/// ```
#[allow(clippy::expect_used)]
pub fn assert_git_clean(repo_path: &Path) {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run git status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "Git repository {} is not clean. Uncommitted changes:\n{stdout}",
        repo_path.display()
    );
}

/// Asserts that the git repository has uncommitted changes.
///
/// # Panics
///
/// Panics if the repository is clean.
///
/// # Examples
///
/// ```rust,ignore
/// assert_git_dirty(workspace.root());
/// ```
#[allow(clippy::expect_used)]
pub fn assert_git_dirty(repo_path: &Path) {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run git status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Git repository {} is clean, expected uncommitted changes",
        repo_path.display()
    );
}

/// Asserts that a git commit with specific message exists.
///
/// # Panics
///
/// Panics if the commit message is not found in recent commits.
///
/// # Examples
///
/// ```rust,ignore
/// assert_git_commit_exists(workspace.root(), "chore: bump version");
/// ```
#[allow(clippy::expect_used)]
pub fn assert_git_commit_exists(repo_path: &Path, message_contains: &str) {
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-n", "10", "--format=%s"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run git log");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(message_contains),
        "No recent commit found containing '{message_contains}' in repository {}\nRecent commits:\n{stdout}",
        repo_path.display()
    );
}

// =============================================================================
// Count Assertions
// =============================================================================

/// Asserts that a directory contains an expected number of files matching a pattern.
///
/// # Panics
///
/// Panics if the count does not match.
///
/// # Examples
///
/// ```rust,ignore
/// assert_file_count(
///     &workspace.root().join(".changesets"),
///     "*.json",
///     2
/// );
/// ```
#[allow(clippy::expect_used)]
pub fn assert_file_count(dir: &Path, extension: &str, expected_count: usize) {
    assert_dir_exists(dir);

    let count =
        std::fs::read_dir(dir)
            .expect("Failed to read directory")
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                entry.path().extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
                    ext == extension.trim_start_matches('*').trim_start_matches('.')
                })
            })
            .count();

    assert_eq!(
        count,
        expected_count,
        "Directory {} contains {count} files with extension '{extension}', expected {expected_count}",
        dir.display()
    );
}

// =============================================================================
// CLI Output Assertions
// =============================================================================

/// Test result from CLI execution
#[derive(Debug)]
pub struct TestResult {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}

impl TestResult {
    /// Asserts that the command succeeded (exit code 0).
    ///
    /// # Panics
    ///
    /// Panics if exit code is not 0.
    pub fn assert_success(&self) {
        assert_eq!(
            self.exit_code, 0,
            "Command failed with exit code {}.\nstdout:\n{}\nstderr:\n{}",
            self.exit_code, self.stdout, self.stderr
        );
    }

    /// Asserts that the command failed (non-zero exit code).
    ///
    /// # Panics
    ///
    /// Panics if exit code is 0.
    pub fn assert_failure(&self) {
        assert_ne!(
            self.exit_code, 0,
            "Command succeeded but failure was expected.\nstdout:\n{}\nstderr:\n{}",
            self.stdout, self.stderr
        );
    }

    /// Asserts that stdout contains specific text.
    ///
    /// # Panics
    ///
    /// Panics if stdout does not contain the text.
    pub fn assert_stdout_contains(&self, expected: &str) {
        assert!(
            self.stdout.contains(expected),
            "stdout does not contain expected text.\nExpected to find: {}\nActual stdout:\n{}",
            expected,
            self.stdout
        );
    }

    /// Asserts that stderr contains specific text.
    ///
    /// # Panics
    ///
    /// Panics if stderr does not contain the text.
    pub fn assert_stderr_contains(&self, expected: &str) {
        assert!(
            self.stderr.contains(expected),
            "stderr does not contain expected text.\nExpected to find: {}\nActual stderr:\n{}",
            expected,
            self.stderr
        );
    }

    /// Parses stdout as JSON and returns it.
    ///
    /// # Panics
    ///
    /// Panics if stdout is not valid JSON.
    #[allow(clippy::expect_used)]
    pub fn json_output(&self) -> serde_json::Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|e| {
            panic!("stdout is not valid JSON: {}\nActual stdout:\n{}", e, self.stdout)
        })
    }
}
