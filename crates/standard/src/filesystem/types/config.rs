//! # Async Filesystem Configuration
//!
//! ## What
//! This module provides configuration options for async filesystem operations,
//! including timeout settings and concurrency limits.
//!
//! ## How
//! The `AsyncFileSystemConfig` struct uses a builder pattern for ergonomic
//! configuration with sensible defaults.
//!
//! ## Why
//! Configurable timeouts and concurrency limits are essential for handling
//! different repository sizes and performance requirements.

use std::time::Duration;

/// Configuration for async filesystem operations.
///
/// This struct provides configuration options for async filesystem operations,
/// including timeout settings and concurrency limits.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::filesystem::AsyncFileSystemConfig;
/// use std::time::Duration;
///
/// let config = AsyncFileSystemConfig::new()
///     .with_operation_timeout(Duration::from_secs(30))
///     .with_read_timeout(Duration::from_secs(10))
///     .with_write_timeout(Duration::from_secs(20));
/// ```
#[derive(Debug, Clone)]
pub struct AsyncFileSystemConfig {
    /// Timeout for general operations
    pub operation_timeout: Duration,
    /// Timeout for read operations
    pub read_timeout: Duration,
    /// Timeout for write operations
    pub write_timeout: Duration,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
}

impl Default for AsyncFileSystemConfig {
    fn default() -> Self {
        let fs_config = crate::config::FilesystemConfig::default();
        Self {
            operation_timeout: fs_config.async_io.operation_timeout,
            read_timeout: fs_config.async_io.operation_timeout, // Use operation_timeout for read by default
            write_timeout: fs_config.async_io.operation_timeout, // Use operation_timeout for write by default
            max_concurrent_operations: fs_config.async_io.max_concurrent_operations,
        }
    }
}

impl From<&crate::config::FilesystemConfig> for AsyncFileSystemConfig {
    /// Creates an AsyncFileSystemConfig from a FilesystemConfig.
    ///
    /// This allows the filesystem manager to use configuration settings
    /// from the global configuration.
    fn from(config: &crate::config::FilesystemConfig) -> Self {
        Self {
            operation_timeout: config.async_io.operation_timeout,
            read_timeout: config.async_io.operation_timeout, // Use operation_timeout for read
            write_timeout: config.async_io.operation_timeout, // Use operation_timeout for write
            max_concurrent_operations: config.async_io.max_concurrent_operations,
        }
    }
}

impl From<&crate::config::standard::AsyncIoConfig> for AsyncFileSystemConfig {
    /// Creates an AsyncFileSystemConfig from an AsyncIoConfig.
    ///
    /// This provides a direct conversion from the async I/O configuration
    /// section of the global configuration.
    fn from(config: &crate::config::standard::AsyncIoConfig) -> Self {
        Self {
            operation_timeout: config.operation_timeout,
            read_timeout: config.operation_timeout, // Use operation_timeout for read
            write_timeout: config.operation_timeout, // Use operation_timeout for write
            max_concurrent_operations: config.max_concurrent_operations,
        }
    }
}

impl AsyncFileSystemConfig {
    /// Creates a new configuration with default values.
    ///
    /// # Returns
    ///
    /// A new `AsyncFileSystemConfig` with default timeout and concurrency settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::AsyncFileSystemConfig;
    ///
    /// let config = AsyncFileSystemConfig::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the timeout for general operations.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration for operations
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::AsyncFileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = AsyncFileSystemConfig::new()
    ///     .with_operation_timeout(Duration::from_secs(60));
    /// ```
    #[must_use]
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = timeout;
        self
    }

    /// Sets the timeout for read operations.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration for read operations
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::AsyncFileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = AsyncFileSystemConfig::new()
    ///     .with_read_timeout(Duration::from_secs(5));
    /// ```
    #[must_use]
    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Sets the timeout for write operations.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration for write operations
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::AsyncFileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = AsyncFileSystemConfig::new()
    ///     .with_write_timeout(Duration::from_secs(15));
    /// ```
    #[must_use]
    pub fn with_write_timeout(mut self, timeout: Duration) -> Self {
        self.write_timeout = timeout;
        self
    }

    /// Sets the maximum number of concurrent operations.
    ///
    /// # Arguments
    ///
    /// * `max_concurrent` - The maximum number of concurrent operations
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::AsyncFileSystemConfig;
    ///
    /// let config = AsyncFileSystemConfig::new()
    ///     .with_max_concurrent_operations(20);
    /// ```
    #[must_use]
    pub fn with_max_concurrent_operations(mut self, max_concurrent: usize) -> Self {
        self.max_concurrent_operations = max_concurrent;
        self
    }
}
