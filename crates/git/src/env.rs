//! Environment variable abstraction for testability
//!
//! This module provides traits and implementations for accessing environment variables,
//! enabling dependency injection and facilitating testing of code that depends on
//! environment state.
//!
//! # What
//!
//! Provides the `EnvProvider` trait to abstract environment variable access, along with
//! concrete implementations for production and testing scenarios.
//!
//! # How
//!
//! - `EnvProvider` trait: Defines the interface for environment variable access
//! - `SystemEnvProvider`: Production implementation using `std::env::var()`
//! - `MockEnvProvider`: Test implementation using a configurable HashMap
//!
//! # Why
//!
//! Environment variables are global mutable state that cannot be reliably controlled
//! in tests, especially when:
//! - Tests run in parallel
//! - Running in CI/CD environments with pre-set environment variables
//! - Multiple tests need different environment configurations
//!
//! This abstraction enables clean, isolated testing without unsafe global state modification.
//!
//! # Examples
//!
//! ## Production Usage
//!
//! ```rust
//! use sublime_git_tools::{EnvProvider, SystemEnvProvider};
//! use std::sync::Arc;
//!
//! let env = Arc::new(SystemEnvProvider);
//! match env.var("HOME") {
//!     Ok(home) => println!("Home directory: {}", home),
//!     Err(_) => println!("HOME not set"),
//! }
//! ```
//!
//! ## Testing Usage
//!
//! ```ignore
//! // MockEnvProvider is only available in test builds (#[cfg(test)])
//! use sublime_git_tools::{EnvProvider, MockEnvProvider};
//! use std::sync::Arc;
//!
//! let env = Arc::new(
//!     MockEnvProvider::new()
//!         .with_var("GITHUB_REF_NAME", "feature/test-branch")
//!         .with_var("CI", "true")
//! );
//!
//! assert_eq!(env.var("GITHUB_REF_NAME").unwrap(), "feature/test-branch");
//! assert_eq!(env.var("CI").unwrap(), "true");
//! assert!(env.var("NOT_SET").is_err());
//! ```

use std::env::VarError;

#[cfg(test)]
use std::collections::HashMap;

/// Trait for accessing environment variables
///
/// This trait abstracts environment variable access to enable testing of code
/// that depends on environment state without modifying global process state.
///
/// Implementations must be thread-safe (`Send + Sync`) to allow use across
/// multiple threads and in concurrent testing scenarios.
///
/// # Examples
///
/// ```rust
/// use sublime_git_tools::{EnvProvider, SystemEnvProvider};
///
/// let env = SystemEnvProvider;
/// match env.var("PATH") {
///     Ok(path) => println!("PATH: {}", path),
///     Err(e) => println!("Error: {:?}", e),
/// }
/// ```
pub trait EnvProvider: Send + Sync {
    /// Gets an environment variable by name
    ///
    /// # Arguments
    ///
    /// * `key` - The environment variable name to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The value of the environment variable
    /// * `Err(VarError::NotPresent)` - Variable is not set
    /// * `Err(VarError::NotUnicode(_))` - Variable contains invalid Unicode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_git_tools::{EnvProvider, SystemEnvProvider};
    ///
    /// let env = SystemEnvProvider;
    /// if let Ok(user) = env.var("USER") {
    ///     println!("Current user: {}", user);
    /// }
    /// ```
    fn var(&self, key: &str) -> Result<String, VarError>;
}

/// System environment provider using `std::env::var`
///
/// This is the production implementation that delegates to the standard library's
/// environment variable access. It reads from the actual process environment.
///
/// # Thread Safety
///
/// This implementation is thread-safe and can be safely shared across threads.
/// Reading environment variables in Rust is thread-safe, though writing to them
/// requires unsafe code.
///
/// # Examples
///
/// ```rust
/// use sublime_git_tools::{EnvProvider, SystemEnvProvider};
/// use std::sync::Arc;
///
/// let env = Arc::new(SystemEnvProvider);
/// let home = env.var("HOME").expect("HOME should be set");
/// println!("Home directory: {}", home);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemEnvProvider;

impl EnvProvider for SystemEnvProvider {
    fn var(&self, key: &str) -> Result<String, VarError> {
        std::env::var(key)
    }
}

/// Mock environment provider for testing
///
/// This implementation uses an internal HashMap to store environment variable
/// values, allowing tests to create controlled, isolated environment configurations
/// without modifying the global process state.
///
/// # Thread Safety
///
/// This implementation is thread-safe. Each instance has its own independent
/// HashMap of variables, preventing interference between tests.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use sublime_git_tools::{EnvProvider, MockEnvProvider};
///
/// let env = MockEnvProvider::new()
///     .with_var("GITHUB_REF_NAME", "main")
///     .with_var("CI", "true");
///
/// assert_eq!(env.var("GITHUB_REF_NAME").unwrap(), "main");
/// assert_eq!(env.var("CI").unwrap(), "true");
/// assert!(env.var("NOT_SET").is_err());
/// ```
///
/// ## Testing CI Environments
///
/// ```rust
/// use sublime_git_tools::{EnvProvider, MockEnvProvider};
///
/// // Simulate GitHub Actions environment
/// let github_env = MockEnvProvider::new()
///     .with_var("GITHUB_ACTIONS", "true")
///     .with_var("GITHUB_REF_NAME", "feature/new-feature")
///     .with_var("GITHUB_HEAD_REF", "");
///
/// // Simulate GitLab CI environment
/// let gitlab_env = MockEnvProvider::new()
///     .with_var("CI", "true")
///     .with_var("CI_COMMIT_REF_NAME", "develop");
///
/// assert_eq!(github_env.var("GITHUB_REF_NAME").unwrap(), "feature/new-feature");
/// assert_eq!(gitlab_env.var("CI_COMMIT_REF_NAME").unwrap(), "develop");
/// ```
#[cfg(test)]
#[derive(Debug, Clone, Default)]
pub struct MockEnvProvider {
    /// Internal storage for environment variable key-value pairs
    vars: HashMap<String, String>,
}

#[cfg(test)]
impl MockEnvProvider {
    /// Creates a new empty mock environment provider
    ///
    /// # Returns
    ///
    /// A new `MockEnvProvider` with no environment variables set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_git_tools::MockEnvProvider;
    ///
    /// let env = MockEnvProvider::new();
    /// assert!(env.var("ANY_VAR").is_err());
    /// ```
    pub fn new() -> Self {
        Self { vars: HashMap::new() }
    }

    /// Adds an environment variable to the mock provider
    ///
    /// This method uses a builder pattern, allowing method chaining to configure
    /// multiple variables in a fluent style.
    ///
    /// # Arguments
    ///
    /// * `key` - The environment variable name
    /// * `value` - The environment variable value
    ///
    /// # Returns
    ///
    /// Self with the variable added, enabling method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_git_tools::{EnvProvider, MockEnvProvider};
    ///
    /// let env = MockEnvProvider::new()
    ///     .with_var("VAR1", "value1")
    ///     .with_var("VAR2", "value2")
    ///     .with_var("VAR3", "value3");
    ///
    /// assert_eq!(env.var("VAR1").unwrap(), "value1");
    /// assert_eq!(env.var("VAR2").unwrap(), "value2");
    /// assert_eq!(env.var("VAR3").unwrap(), "value3");
    /// ```
    #[must_use]
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
impl EnvProvider for MockEnvProvider {
    fn var(&self, key: &str) -> Result<String, VarError> {
        self.vars.get(key).cloned().ok_or(VarError::NotPresent)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_system_env_provider_reads_real_variables() {
        let env = SystemEnvProvider;

        // PATH should always exist
        let result = env.var("PATH");
        assert!(result.is_ok(), "PATH should be set in environment");
        assert!(!result.unwrap().is_empty(), "PATH should not be empty");
    }

    #[test]
    fn test_system_env_provider_returns_error_for_missing_variables() {
        let env = SystemEnvProvider;

        let result = env.var("THIS_VARIABLE_SHOULD_NOT_EXIST_12345");
        assert!(result.is_err(), "Non-existent variable should return error");
        assert!(
            matches!(result.unwrap_err(), VarError::NotPresent),
            "Error should be VarError::NotPresent"
        );
    }

    #[test]
    fn test_mock_env_provider_new_is_empty() {
        let env = MockEnvProvider::new();

        assert!(env.var("ANY_VAR").is_err(), "New mock provider should have no variables");
    }

    #[test]
    fn test_mock_env_provider_with_var_single() {
        let env = MockEnvProvider::new().with_var("TEST_VAR", "test_value");

        assert_eq!(env.var("TEST_VAR").unwrap(), "test_value", "Should return configured value");
        assert!(env.var("OTHER_VAR").is_err(), "Should return error for unconfigured variable");
    }

    #[test]
    fn test_mock_env_provider_with_var_multiple() {
        let env = MockEnvProvider::new()
            .with_var("VAR1", "value1")
            .with_var("VAR2", "value2")
            .with_var("VAR3", "value3");

        assert_eq!(env.var("VAR1").unwrap(), "value1");
        assert_eq!(env.var("VAR2").unwrap(), "value2");
        assert_eq!(env.var("VAR3").unwrap(), "value3");
        assert!(env.var("VAR4").is_err());
    }

    #[test]
    fn test_mock_env_provider_with_var_empty_value() {
        let env = MockEnvProvider::new().with_var("EMPTY_VAR", "");

        assert_eq!(env.var("EMPTY_VAR").unwrap(), "", "Should support empty string values");
    }

    #[test]
    fn test_mock_env_provider_with_var_overwrite() {
        let env = MockEnvProvider::new()
            .with_var("TEST_VAR", "first_value")
            .with_var("TEST_VAR", "second_value");

        assert_eq!(
            env.var("TEST_VAR").unwrap(),
            "second_value",
            "Later value should overwrite earlier value"
        );
    }

    #[test]
    fn test_mock_env_provider_ci_simulation() {
        // Simulate GitHub Actions environment
        let github_env = MockEnvProvider::new()
            .with_var("GITHUB_ACTIONS", "true")
            .with_var("GITHUB_REF_NAME", "feature/test-branch")
            .with_var("GITHUB_HEAD_REF", "");

        assert_eq!(github_env.var("GITHUB_ACTIONS").unwrap(), "true");
        assert_eq!(github_env.var("GITHUB_REF_NAME").unwrap(), "feature/test-branch");
        assert_eq!(github_env.var("GITHUB_HEAD_REF").unwrap(), "");

        // Simulate GitLab CI environment
        let gitlab_env =
            MockEnvProvider::new().with_var("CI", "true").with_var("CI_COMMIT_REF_NAME", "develop");

        assert_eq!(gitlab_env.var("CI").unwrap(), "true");
        assert_eq!(gitlab_env.var("CI_COMMIT_REF_NAME").unwrap(), "develop");
    }

    #[test]
    fn test_system_env_provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SystemEnvProvider>();
    }

    #[test]
    fn test_mock_env_provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockEnvProvider>();
    }
}
