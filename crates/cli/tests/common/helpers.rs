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
