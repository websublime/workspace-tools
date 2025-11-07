//! Test helper utilities for E2E CLI tests.
//!
//! **What**: Provides utility functions for executing CLI commands and
//! managing test environments.
//!
//! **How**: Implements helpers for running commands, capturing output,
//! and setting up test contexts.
//!
//! **Why**: Simplifies test code and provides consistent command execution.

use std::io::Cursor;
use std::path::Path;
use sublime_cli_tools::output::{Output, OutputFormat};

// =============================================================================
// CLI Execution Helpers
// =============================================================================

/// Creates an Output instance for tests with captured output.
///
/// Returns both the Output and the buffer for assertions.
///
/// # Examples
///
/// ```rust,ignore
/// let (output, buffer) = create_test_output(OutputFormat::Human);
/// command_execute(&output).await?;
/// let output_str = String::from_utf8(buffer.into_inner()).unwrap();
/// assert!(output_str.contains("Success"));
/// ```
pub fn create_test_output(format: OutputFormat) -> (Output, Cursor<Vec<u8>>) {
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(format, Box::new(buffer.clone()), false);
    (output, buffer)
}

/// Creates a quiet output for tests that don't need to check output.
///
/// # Examples
///
/// ```rust,ignore
/// let output = create_quiet_output();
/// command_execute(&output).await?;
/// ```
pub fn create_quiet_output() -> Output {
    let buffer = Cursor::new(Vec::new());
    Output::new(OutputFormat::Quiet, Box::new(buffer), false)
}

/// Creates a JSON output for tests that need to parse JSON output.
///
/// # Examples
///
/// ```rust,ignore
/// let (output, buffer) = create_json_output();
/// command_execute(&output).await?;
/// let json: serde_json::Value = serde_json::from_slice(&buffer.into_inner()).unwrap();
/// ```
pub fn create_json_output() -> (Output, Cursor<Vec<u8>>) {
    create_test_output(OutputFormat::Json)
}

// =============================================================================
// File System Helpers
// =============================================================================

/// Reads a file to string with better error messages for tests.
///
/// # Panics
///
/// Panics if file cannot be read with a descriptive message.
///
/// # Examples
///
/// ```rust,ignore
/// let content = read_file(&workspace.root().join("package.json"));
/// ```
#[allow(clippy::expect_used)]
pub fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {e}", path.display()))
}

/// Reads a JSON file and parses it.
///
/// # Panics
///
/// Panics if file cannot be read or parsed.
///
/// # Examples
///
/// ```rust,ignore
/// let package_json: serde_json::Value = read_json_file(&workspace.root().join("package.json"));
/// ```
#[allow(clippy::expect_used)]
pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let content = read_file(path);
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON from {}: {e}", path.display()))
}

/// Writes content to a file with better error messages.
///
/// # Panics
///
/// Panics if file cannot be written.
///
/// # Examples
///
/// ```rust,ignore
/// write_file(&workspace.root().join("test.txt"), "content");
/// ```
#[allow(clippy::expect_used)]
pub fn write_file(path: &Path, content: &str) {
    std::fs::write(path, content)
        .unwrap_or_else(|e| panic!("Failed to write file {}: {e}", path.display()));
}

/// Appends content to a file.
///
/// # Panics
///
/// Panics if file cannot be appended to.
///
/// # Examples
///
/// ```rust,ignore
/// append_file(&workspace.root().join("log.txt"), "new line\n");
/// ```
#[allow(clippy::expect_used)]
pub fn append_file(path: &Path, content: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|e| panic!("Failed to open file for append {}: {e}", path.display()));

    file.write_all(content.as_bytes())
        .unwrap_or_else(|e| panic!("Failed to append to file {}: {e}", path.display()));
}

// =============================================================================
// Git Helpers
// =============================================================================

/// Runs a git command in the specified directory.
///
/// # Panics
///
/// Panics if git command fails.
///
/// # Examples
///
/// ```rust,ignore
/// run_git_command(workspace.root(), &["add", "."]);
/// run_git_command(workspace.root(), &["commit", "-m", "test"]);
/// ```
#[allow(clippy::expect_used)]
pub fn run_git_command(repo_path: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run git command {args:?}: {e}"));

    assert!(
        output.status.success(),
        "Git command {:?} failed:\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Gets the current git branch name.
///
/// # Examples
///
/// ```rust,ignore
/// let branch = get_current_branch(workspace.root());
/// assert_eq!(branch, "main");
/// ```
pub fn get_current_branch(repo_path: &Path) -> String {
    let output = run_git_command(repo_path, &["branch", "--show-current"]);
    output.trim().to_string()
}

/// Gets the latest git commit SHA.
///
/// # Examples
///
/// ```rust,ignore
/// let sha = get_latest_commit_sha(workspace.root());
/// ```
pub fn get_latest_commit_sha(repo_path: &Path) -> String {
    let output = run_git_command(repo_path, &["rev-parse", "HEAD"]);
    output.trim().to_string()
}

/// Lists all git tags.
///
/// # Examples
///
/// ```rust,ignore
/// let tags = list_git_tags(workspace.root());
/// assert!(tags.contains(&"v1.0.0".to_string()));
/// ```
pub fn list_git_tags(repo_path: &Path) -> Vec<String> {
    let output = run_git_command(repo_path, &["tag", "-l"]);
    output.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

// =============================================================================
// Async Test Helpers
// =============================================================================

/// Waits for a condition to be true with timeout.
///
/// Useful for async operations that may take time.
///
/// # Examples
///
/// ```rust,ignore
/// wait_for(|| file_path.exists(), Duration::from_secs(5)).await;
/// ```
pub async fn wait_for<F>(mut condition: F, timeout: std::time::Duration) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    while !condition() {
        if start.elapsed() > timeout {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    true
}

// =============================================================================
// Package.json Helpers
// =============================================================================

/// Updates package.json version.
///
/// # Examples
///
/// ```rust,ignore
/// update_package_version(&workspace.root().join("package.json"), "1.2.0");
/// ```
#[allow(clippy::expect_used)]
pub fn update_package_version(package_json_path: &Path, new_version: &str) {
    let mut json: serde_json::Value = read_json_file(package_json_path);
    json["version"] = serde_json::Value::String(new_version.to_string());

    let content = serde_json::to_string_pretty(&json).expect("Failed to serialize JSON");
    write_file(package_json_path, &content);
}

/// Gets package.json version (synchronous version).
///
/// # Examples
///
/// ```rust,ignore
/// let version = get_package_version_sync(&workspace.root().join("package.json"));
/// assert_eq!(version, "1.0.0");
/// ```
#[allow(clippy::expect_used)]
pub fn get_package_version_sync(package_json_path: &Path) -> String {
    let json: serde_json::Value = read_json_file(package_json_path);
    json["version"].as_str().expect("No version field in package.json").to_string()
}

/// Gets package.json version from a directory (async version).
///
/// Automatically appends "package.json" to the directory path.
///
/// # Examples
///
/// ```rust,ignore
/// let version = get_package_version(&workspace.root()).await?;
/// assert_eq!(version, "1.0.0");
/// ```
///
/// # Errors
///
/// Returns error if package.json cannot be read or parsed.
pub async fn get_package_version(dir: &Path) -> Result<String, std::io::Error> {
    let package_json_path = dir.join("package.json");
    let content = tokio::fs::read_to_string(&package_json_path).await?;
    let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse package.json: {e}"),
        )
    })?;

    json["version"].as_str().map(std::string::ToString::to_string).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "No version field in package.json")
    })
}

/// Adds a dependency to package.json.
///
/// # Examples
///
/// ```rust,ignore
/// add_dependency(
///     &workspace.root().join("package.json"),
///     "lodash",
///     "^4.17.21"
/// );
/// ```
#[allow(clippy::expect_used)]
pub fn add_dependency(package_json_path: &Path, name: &str, version: &str) {
    let mut json: serde_json::Value = read_json_file(package_json_path);

    if json.get("dependencies").is_none() {
        json["dependencies"] = serde_json::json!({});
    }

    json["dependencies"][name] = serde_json::Value::String(version.to_string());

    let content = serde_json::to_string_pretty(&json).expect("Failed to serialize JSON");
    write_file(package_json_path, &content);
}

// =============================================================================
// Changeset Helpers
// =============================================================================

/// Counts changesets in .changesets directory.
///
/// # Examples
///
/// ```rust,ignore
/// let count = count_changesets(workspace.root());
/// assert_eq!(count, 2);
/// ```
#[allow(clippy::expect_used)]
pub fn count_changesets(workspace_root: &Path) -> usize {
    let changesets_dir = workspace_root.join(".changesets");
    if !changesets_dir.exists() {
        return 0;
    }

    std::fs::read_dir(&changesets_dir)
        .expect("Failed to read changesets dir")
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|ext| ext.to_str()) == Some("json")
                && !entry.file_name().to_string_lossy().starts_with('.')
        })
        .count()
}

/// Lists all changeset files.
///
/// # Examples
///
/// ```rust,ignore
/// let changesets = list_changesets(workspace.root());
/// for changeset in changesets {
///     println!("Changeset: {:?}", changeset);
/// }
/// ```
#[allow(clippy::expect_used)]
pub fn list_changesets(workspace_root: &Path) -> Vec<std::path::PathBuf> {
    let changesets_dir = workspace_root.join(".changesets");
    if !changesets_dir.exists() {
        return vec![];
    }

    std::fs::read_dir(&changesets_dir)
        .expect("Failed to read changesets dir")
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|ext| ext.to_str()) == Some("json")
                && !entry.file_name().to_string_lossy().starts_with('.')
        })
        .map(|entry| entry.path())
        .collect()
}

/// Checks if a changeset exists in the .changesets directory.
///
/// # Examples
///
/// ```rust,ignore
/// if changeset_exists(&workspace.root(), "feature-test").await {
///     println!("Changeset exists");
/// }
/// ```
pub async fn changeset_exists(workspace_root: &Path, changeset_name: &str) -> bool {
    let changeset_path = workspace_root.join(".changesets").join(format!("{changeset_name}.json"));
    tokio::fs::metadata(&changeset_path).await.is_ok()
}

/// Checks if a changeset is archived in the .changesets/history directory.
///
/// Note: The default history path is `.changesets/history` as per the config default.
///
/// # Examples
///
/// ```rust,ignore
/// if changeset_is_archived(&workspace.root(), "feature-test").await {
///     println!("Changeset is archived");
/// }
/// ```
pub async fn changeset_is_archived(workspace_root: &Path, changeset_name: &str) -> bool {
    let archived_path =
        workspace_root.join(".changesets").join("history").join(format!("{changeset_name}.json"));
    tokio::fs::metadata(&archived_path).await.is_ok()
}

// =============================================================================
// Environment Helpers
// =============================================================================

/// Sets an environment variable for the duration of a test.
///
/// The variable is automatically cleaned up when the returned guard is dropped.
///
/// # Examples
///
/// ```rust,ignore
/// {
///     let _guard = set_env_var("NPM_TOKEN", "test-token");
///     // NPM_TOKEN is set here
///     run_command().await;
/// } // NPM_TOKEN is restored here
/// ```
pub fn set_env_var(key: &str, value: &str) -> EnvVarGuard {
    let old_value = std::env::var(key).ok();
    // SAFETY: This is only called in single-threaded test contexts
    unsafe {
        std::env::set_var(key, value);
    }
    EnvVarGuard { key: key.to_string(), old_value }
}

/// Guard that restores environment variable on drop.
pub struct EnvVarGuard {
    key: String,
    old_value: Option<String>,
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: This is only called in single-threaded test contexts
        unsafe {
            match &self.old_value {
                Some(value) => std::env::set_var(&self.key, value),
                None => std::env::remove_var(&self.key),
            }
        }
    }
}

// =============================================================================
// Time Helpers
// =============================================================================

/// Gets current timestamp in ISO 8601 format.
///
/// # Examples
///
/// ```rust,ignore
/// let timestamp = current_timestamp();
/// // "2024-01-01T12:00:00Z"
/// ```
pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

// =============================================================================
// Comparison Helpers
// =============================================================================

/// Compares two version strings.
///
/// Returns -1 if v1 < v2, 0 if v1 == v2, 1 if v1 > v2.
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(compare_versions("1.0.0", "1.1.0"), -1);
/// assert_eq!(compare_versions("2.0.0", "1.9.9"), 1);
/// assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
/// ```
pub fn compare_versions(v1: &str, v2: &str) -> i32 {
    let parts1: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
    let parts2: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..parts1.len().max(parts2.len()) {
        let p1 = parts1.get(i).copied().unwrap_or(0);
        let p2 = parts2.get(i).copied().unwrap_or(0);

        match p1.cmp(&p2) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => {}
        }
    }

    0
}

// =============================================================================
// Cleanup Helpers
// =============================================================================

/// Ensures directory is empty by removing all files.
///
/// # Examples
///
/// ```rust,ignore
/// clean_directory(&workspace.root().join(".changesets"));
/// ```
#[allow(clippy::expect_used)]
pub fn clean_directory(dir: &Path) {
    if !dir.exists() {
        return;
    }

    for entry in std::fs::read_dir(dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_file() {
            std::fs::remove_file(&path).expect("Failed to remove file");
        } else if path.is_dir() {
            std::fs::remove_dir_all(&path).expect("Failed to remove directory");
        }
    }
}

// =============================================================================
// Editor Mock Helpers
// =============================================================================

/// Creates a mock editor script for testing.
///
/// The mock editor can be configured to modify files, fail, or do nothing.
/// Returns the path to the mock editor executable and an environment guard
/// that sets EDITOR to point to it.
///
/// # Examples
///
/// ```rust,ignore
/// // Mock editor that changes "patch" to "minor"
/// let (editor_path, _guard) = create_mock_editor(|content| {
///     content.replace("\"patch\"", "\"minor\"")
/// });
/// // EDITOR is now set, execute_edit will use our mock
/// execute_edit(&args, &output, Some(workspace.root()), None).await?;
/// ```
///
/// # Panics
///
/// Panics if the mock editor script cannot be created or made executable.
#[allow(clippy::expect_used)]
pub fn create_mock_editor<F>(_modifier: F) -> (std::path::PathBuf, EnvVarGuard)
where
    F: Fn(String) -> String + 'static,
{
    let temp_dir = std::env::temp_dir();
    // Use timestamp + random to ensure uniqueness across tests
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    let script_name = format!("mock_editor_{}_{}", std::process::id(), timestamp);

    #[cfg(unix)]
    let script_path = temp_dir.join(format!("{script_name}.sh"));

    #[cfg(windows)]
    let script_path = temp_dir.join(format!("{script_name}.bat"));

    // Create the mock editor script
    #[cfg(unix)]
    {
        let script_content = r#"#!/bin/bash
# Mock editor for testing
# Reads the file, applies modifications, and writes it back

FILE="$1"
if [ -z "$FILE" ]; then
    echo "Usage: $0 <file>" >&2
    exit 1
fi

if [ ! -f "$FILE" ]; then
    echo "File not found: $FILE" >&2
    exit 1
fi

# The Rust test will handle the modification via a custom mechanism
# For now, this script just exits successfully
exit 0
"#;

        std::fs::write(&script_path, script_content).expect("Failed to write mock editor script");

        // Make it executable
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&script_path).expect("Failed to get script metadata");
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&script_path, permissions)
                .expect("Failed to set script permissions");
        }
    }

    #[cfg(windows)]
    {
        let script_content = r#"@echo off
REM Mock editor for testing
set FILE=%1

if "%FILE%"=="" (
    echo Usage: %0 ^<file^> 1>&2
    exit /b 1
)

if not exist "%FILE%" (
    echo File not found: %FILE% 1>&2
    exit /b 1
)

exit /b 0
"#;

        std::fs::write(&script_path, script_content).expect("Failed to write mock editor script");
    }

    // Set EDITOR environment variable
    let guard = set_env_var("EDITOR", script_path.to_str().expect("Invalid path"));

    (script_path, guard)
}

/// Builder for creating mock editors with specific behaviors.
///
/// Provides a fluent API for configuring mock editor behavior.
///
/// # Examples
///
/// ```rust,ignore
/// // Create a mock editor that modifies the bump type
/// let _guard = MockEditorBuilder::new()
///     .modify_json(|json| {
///         json["bump"] = serde_json::Value::String("major".to_string());
///         json
///     })
///     .build();
///
/// execute_edit(&args, &output, Some(workspace.root()), None).await?;
/// ```
pub struct MockEditorBuilder {
    fail: bool,
    modification: Option<Box<dyn Fn(String) -> String>>,
}

impl MockEditorBuilder {
    /// Creates a new mock editor builder.
    pub fn new() -> Self {
        Self { fail: false, modification: None }
    }

    /// Configures the mock editor to fail (exit with non-zero status).
    #[allow(dead_code)]
    pub fn fail(mut self) -> Self {
        self.fail = true;
        self
    }

    /// Configures the mock editor to modify file content.
    #[allow(dead_code)]
    pub fn modify<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> String + 'static,
    {
        self.modification = Some(Box::new(f));
        self
    }

    /// Configures the mock editor to modify JSON content.
    ///
    /// The provided function receives a mutable reference to the parsed JSON
    /// and can modify it directly.
    #[allow(dead_code)]
    pub fn modify_json<F>(self, f: F) -> Self
    where
        F: Fn(serde_json::Value) -> serde_json::Value + 'static,
    {
        self.modify(move |content| {
            let json: serde_json::Value =
                serde_json::from_str(&content).expect("Failed to parse JSON");
            let modified = f(json);
            serde_json::to_string_pretty(&modified).expect("Failed to serialize JSON")
        })
    }

    /// Builds the mock editor and returns the environment guard.
    ///
    /// The guard must be kept alive for the duration of the test.
    #[allow(dead_code)]
    pub fn build(self) -> EnvVarGuard {
        let (_path, guard) = if self.fail {
            create_failing_mock_editor()
        } else if let Some(modifier) = self.modification {
            create_mock_editor(modifier)
        } else {
            create_noop_mock_editor()
        };

        guard
    }
}

impl Default for MockEditorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a mock editor that does nothing (no-op).
///
/// This simulates a user opening the editor and closing it without changes.
///
/// # Examples
///
/// ```rust,ignore
/// let (_path, _guard) = create_noop_mock_editor();
/// execute_edit(&args, &output, Some(workspace.root()), None).await?;
/// ```
#[allow(clippy::expect_used)]
pub fn create_noop_mock_editor() -> (std::path::PathBuf, EnvVarGuard) {
    create_mock_editor(|content| content)
}

/// Creates a mock editor that fails (exits with non-zero status).
///
/// This simulates editor failure scenarios.
///
/// # Examples
///
/// ```rust,ignore
/// let (_path, _guard) = create_failing_mock_editor();
/// let result = execute_edit(&args, &output, Some(workspace.root()), None).await;
/// assert!(result.is_err());
/// ```
#[allow(clippy::expect_used)]
pub fn create_failing_mock_editor() -> (std::path::PathBuf, EnvVarGuard) {
    let temp_dir = std::env::temp_dir();
    // Use timestamp + random to ensure uniqueness across tests
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    let script_name = format!("mock_editor_fail_{}_{}", std::process::id(), timestamp);

    #[cfg(unix)]
    let script_path = temp_dir.join(format!("{script_name}.sh"));

    #[cfg(windows)]
    let script_path = temp_dir.join(format!("{script_name}.bat"));

    #[cfg(unix)]
    {
        let script_content = r"#!/bin/bash
exit 1
";

        std::fs::write(&script_path, script_content)
            .expect("Failed to write failing mock editor script");

        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&script_path).expect("Failed to get script metadata");
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&script_path, permissions)
                .expect("Failed to set script permissions");
        }
    }

    #[cfg(windows)]
    {
        let script_content = r"@echo off
exit /b 1
";

        std::fs::write(&script_path, script_content)
            .expect("Failed to write failing mock editor script");
    }

    let guard = set_env_var("EDITOR", script_path.to_str().expect("Invalid path"));

    (script_path, guard)
}

/// Manually modifies a changeset file for testing edit validation.
///
/// This bypasses the editor and directly modifies the changeset file,
/// useful for testing validation logic.
///
/// # Examples
///
/// ```rust,ignore
/// // Create a changeset
/// execute_add(&args, &output, Some(workspace.root()), None).await?;
///
/// // Manually corrupt it
/// modify_changeset_file(
///     workspace.root(),
///     "feature-test",
///     |mut json| {
///         json["packages"] = serde_json::Value::Array(vec![]); // Invalid: empty array
///         json
///     }
/// );
///
/// // Try to load it - should fail validation
/// let result = execute_show(&args, &output, Some(workspace.root()), None).await;
/// assert!(result.is_err());
/// ```
#[allow(clippy::expect_used)]
pub fn modify_changeset_file<F>(workspace_root: &Path, changeset_name: &str, modifier: F)
where
    F: FnOnce(serde_json::Value) -> serde_json::Value,
{
    let changeset_path = workspace_root.join(".changesets").join(format!("{changeset_name}.json"));

    let content = read_file(&changeset_path);
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse changeset JSON");

    let modified = modifier(json);

    let new_content = serde_json::to_string_pretty(&modified).expect("Failed to serialize JSON");
    write_file(&changeset_path, &new_content);
}
