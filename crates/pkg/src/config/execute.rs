//! Execute configuration for command execution timeouts and parallelism.
//!
//! **What**: Defines configuration for command execution operations, including global timeouts,
//! per-package timeouts, and maximum parallel execution limits.
//!
//! **How**: This module provides the `ExecuteConfig` structure that controls how commands are
//! executed across packages in a workspace, with configurable timeout and concurrency settings.
//!
//! **Why**: To enable controlled, predictable command execution in monorepo environments where
//! running scripts across many packages requires proper timeout handling and resource management
//! to prevent runaway processes and ensure consistent behavior.

use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Default timeout for execute commands in seconds.
///
/// A value of 300 seconds (5 minutes) provides a reasonable default for most
/// command execution scenarios while preventing indefinite hangs.
const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Default timeout per individual package in seconds.
///
/// A value of 60 seconds (1 minute) allows for typical package scripts while
/// ensuring that a single slow package doesn't block the entire execution.
const DEFAULT_PER_PACKAGE_TIMEOUT_SECS: u64 = 60;

/// Default maximum number of parallel executions.
///
/// A value of 8 provides good parallelism for most development machines while
/// avoiding excessive resource consumption.
const DEFAULT_MAX_PARALLEL: usize = 8;

/// Returns the default timeout value for execute commands.
///
/// Used by serde for default deserialization.
#[inline]
#[must_use]
fn default_timeout_secs() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

/// Returns the default per-package timeout value.
///
/// Used by serde for default deserialization.
#[inline]
#[must_use]
fn default_per_package_timeout_secs() -> u64 {
    DEFAULT_PER_PACKAGE_TIMEOUT_SECS
}

/// Returns the default maximum parallel executions.
///
/// Used by serde for default deserialization.
#[inline]
#[must_use]
fn default_max_parallel() -> usize {
    DEFAULT_MAX_PARALLEL
}

/// Configuration for command execution operations.
///
/// This structure controls how commands are executed across packages in a workspace,
/// including timeout handling and concurrency limits. It enables safe, predictable
/// execution of scripts like build, test, and lint commands.
///
/// # Timeout Behavior
///
/// - `timeout_secs`: Controls the overall execution timeout for all packages combined.
///   A value of `0` means no global timeout.
/// - `per_package_timeout_secs`: Controls the timeout for each individual package execution.
///   A value of `0` means no per-package timeout.
///
/// # Parallelism
///
/// The `max_parallel` setting controls how many packages can execute concurrently.
/// Higher values speed up execution but consume more system resources.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::ExecuteConfig;
///
/// let config = ExecuteConfig::default();
/// assert_eq!(config.timeout_secs, 300);
/// assert_eq!(config.per_package_timeout_secs, 60);
/// assert_eq!(config.max_parallel, 8);
/// ```
///
/// # TOML Representation
///
/// ```toml
/// [package_tools.execute]
/// timeout_secs = 300
/// per_package_timeout_secs = 60
/// max_parallel = 8
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ExecuteConfig {
    /// Default timeout for execute commands in seconds.
    ///
    /// This is the overall timeout for executing commands across all packages.
    /// A value of `0` disables the global timeout, allowing execution to run
    /// indefinitely (not recommended for CI environments).
    ///
    /// # Default
    ///
    /// `300` (5 minutes)
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ExecuteConfig;
    ///
    /// let config = ExecuteConfig {
    ///     timeout_secs: 600, // 10 minutes
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.timeout_secs, 600);
    /// ```
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Timeout per individual package in seconds.
    ///
    /// This timeout applies to each package individually. If a single package's
    /// execution exceeds this time, it will be terminated even if the global
    /// timeout hasn't been reached.
    ///
    /// A value of `0` disables per-package timeout, allowing individual packages
    /// to run until the global timeout (if set) or indefinitely.
    ///
    /// # Default
    ///
    /// `60` (1 minute)
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ExecuteConfig;
    ///
    /// let config = ExecuteConfig {
    ///     per_package_timeout_secs: 120, // 2 minutes per package
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.per_package_timeout_secs, 120);
    /// ```
    #[serde(default = "default_per_package_timeout_secs")]
    pub per_package_timeout_secs: u64,

    /// Maximum number of parallel executions.
    ///
    /// Controls how many packages can execute commands concurrently. Higher values
    /// can speed up execution but consume more system resources (CPU, memory, I/O).
    ///
    /// # Constraints
    ///
    /// Must be at least `1`. There's no upper limit, but values greater than the
    /// number of available CPU cores may not provide additional benefit and could
    /// increase resource contention.
    ///
    /// # Default
    ///
    /// `8`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ExecuteConfig;
    ///
    /// let config = ExecuteConfig {
    ///     max_parallel: 4, // More conservative parallelism
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.max_parallel, 4);
    /// ```
    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
}

impl Default for ExecuteConfig {
    /// Creates a new `ExecuteConfig` with default values.
    ///
    /// The default configuration provides reasonable values for most use cases:
    /// - Global timeout of 5 minutes
    /// - Per-package timeout of 1 minute
    /// - Up to 8 parallel executions
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ExecuteConfig;
    ///
    /// let config = ExecuteConfig::default();
    /// assert_eq!(config.timeout_secs, 300);
    /// assert_eq!(config.per_package_timeout_secs, 60);
    /// assert_eq!(config.max_parallel, 8);
    /// ```
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout_secs(),
            per_package_timeout_secs: default_per_package_timeout_secs(),
            max_parallel: default_max_parallel(),
        }
    }
}

impl Configurable for ExecuteConfig {
    /// Validates the execute configuration.
    ///
    /// This method ensures that:
    /// - `max_parallel` is at least 1 (need at least one execution slot)
    ///
    /// Note that `timeout_secs` and `per_package_timeout_secs` can be `0`,
    /// which means "no timeout" - this is a valid configuration choice.
    ///
    /// # Errors
    ///
    /// Returns an error if `max_parallel` is 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ExecuteConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let config = ExecuteConfig::default();
    /// assert!(config.validate().is_ok());
    ///
    /// let invalid_config = ExecuteConfig {
    ///     max_parallel: 0,
    ///     ..Default::default()
    /// };
    /// assert!(invalid_config.validate().is_err());
    /// ```
    fn validate(&self) -> ConfigResult<()> {
        // max_parallel must be at least 1 - you need at least one execution slot
        if self.max_parallel == 0 {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "execute.max_parallel: Must be at least 1. Cannot have zero parallel executions.".to_string(),
            });
        }

        Ok(())
    }

    /// Merges this configuration with another configuration.
    ///
    /// Values from `other` take precedence over values in `self`. This enables
    /// layered configuration where base settings can be overridden by more specific
    /// configurations.
    ///
    /// # Arguments
    ///
    /// * `other` - The configuration to merge into this one
    ///
    /// # Errors
    ///
    /// This method currently cannot fail, but returns `ConfigResult` for
    /// consistency with other configuration types.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ExecuteConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let mut base = ExecuteConfig::default();
    /// let override_config = ExecuteConfig {
    ///     timeout_secs: 600,
    ///     per_package_timeout_secs: 120,
    ///     max_parallel: 4,
    /// };
    ///
    /// base.merge_with(override_config).expect("Merge should succeed");
    /// assert_eq!(base.timeout_secs, 600);
    /// assert_eq!(base.per_package_timeout_secs, 120);
    /// assert_eq!(base.max_parallel, 4);
    /// ```
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.timeout_secs = other.timeout_secs;
        self.per_package_timeout_secs = other.per_package_timeout_secs;
        self.max_parallel = other.max_parallel;
        Ok(())
    }
}
