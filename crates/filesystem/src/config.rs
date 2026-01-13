//! # Configuration Module
//!
//! Provides configuration types for filesystem operations, including timeout settings.
//!
//! ## What
//!
//! This module defines [`FileSystemConfig`] and its builder, which control the behavior
//! of filesystem operations. The primary configuration options are timeout values for
//! different operation categories.
//!
//! ## How
//!
//! Configuration is created using the builder pattern:
//! 1. Call `FileSystemConfig::builder()` to get a `FileSystemConfigBuilder`
//! 2. Chain configuration methods (e.g., `with_read_timeout()`)
//! 3. Call `build()` to obtain the final `FileSystemConfig`
//!
//! Alternatively, use `FileSystemConfig::default()` for sensible defaults.
//!
//! The configuration is immutable once built, ensuring thread-safety and preventing
//! accidental modification during filesystem operations.
//!
//! ## Why
//!
//! Configurable timeouts are essential for:
//! - **Reliability**: Prevent operations from hanging on unresponsive filesystems
//! - **Flexibility**: Different use cases require different timeout values
//! - **Testability**: Mock filesystems can use shorter timeouts for faster tests
//! - **Predictability**: Known timeout behavior helps with error handling
//!
//! ## Example
//!
//! ```rust
//! use workspace_fs::FileSystemConfig;
//! use std::time::Duration;
//!
//! // Use default configuration (30s read/write, 60s operation)
//! let config = FileSystemConfig::default();
//! assert_eq!(config.read_timeout(), Duration::from_secs(30));
//! assert_eq!(config.write_timeout(), Duration::from_secs(30));
//! assert_eq!(config.operation_timeout(), Duration::from_secs(60));
//!
//! // Custom configuration with builder
//! let config = FileSystemConfig::builder()
//!     .with_read_timeout(Duration::from_secs(10))
//!     .with_write_timeout(Duration::from_secs(10))
//!     .with_operation_timeout(Duration::from_secs(30))
//!     .build();
//!
//! assert_eq!(config.read_timeout(), Duration::from_secs(10));
//! ```

use std::time::Duration;

// =============================================================================
// Constants
// =============================================================================

/// Default timeout for read operations (30 seconds).
///
/// This value is used when no custom read timeout is specified.
/// It provides a reasonable balance between allowing slow operations
/// to complete and failing fast on truly unresponsive systems.
const DEFAULT_READ_TIMEOUT_SECS: u64 = 30;

/// Default timeout for write operations (30 seconds).
///
/// This value is used when no custom write timeout is specified.
/// Write operations may take longer than reads due to disk I/O,
/// but 30 seconds is sufficient for most file operations.
const DEFAULT_WRITE_TIMEOUT_SECS: u64 = 30;

/// Default timeout for general operations (60 seconds).
///
/// This value is used for operations that don't fit into read/write
/// categories, such as directory traversal or metadata operations.
/// The longer timeout accounts for potentially complex operations.
const DEFAULT_OPERATION_TIMEOUT_SECS: u64 = 60;

// =============================================================================
// FileSystemConfig Struct
// =============================================================================

/// Configuration for filesystem operations.
///
/// This struct holds configuration values that control the behavior of filesystem
/// operations, primarily timeout durations for different operation types.
///
/// # Fields
///
/// | Field | Default | Description |
/// |-------|---------|-------------|
/// | `read_timeout` | 30s | Timeout for read operations |
/// | `write_timeout` | 30s | Timeout for write operations |
/// | `operation_timeout` | 60s | Timeout for general operations |
///
/// # Thread Safety
///
/// `FileSystemConfig` is `Send + Sync`, making it safe to share across threads.
/// The configuration is immutable once created, preventing race conditions.
///
/// # Example
///
/// ```rust
/// use workspace_fs::FileSystemConfig;
/// use std::time::Duration;
///
/// // Create with defaults
/// let config = FileSystemConfig::default();
///
/// // Access timeout values
/// let read_timeout = config.read_timeout();
/// let write_timeout = config.write_timeout();
/// let operation_timeout = config.operation_timeout();
///
/// // Create with custom values using builder
/// let custom_config = FileSystemConfig::builder()
///     .with_read_timeout(Duration::from_secs(5))
///     .build();
///
/// assert_eq!(custom_config.read_timeout(), Duration::from_secs(5));
/// // Other values remain at defaults
/// assert_eq!(custom_config.write_timeout(), Duration::from_secs(30));
/// ```
//
// Clippy exception: The `_timeout` suffix on field names is intentional and meaningful.
// These names (`read_timeout`, `write_timeout`, `operation_timeout`) follow common conventions
// in timeout configuration (see tokio::time, reqwest::ClientBuilder, etc.) and removing the
// suffix would make the fields ambiguous (`read`, `write`, `operation` could mean many things).
// The PLAN.md specification also explicitly requires these field names.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct FileSystemConfig {
    /// Timeout duration for read operations.
    ///
    /// This timeout applies to operations that read data from the filesystem,
    /// such as reading file contents or listing directory entries.
    read_timeout: Duration,

    /// Timeout duration for write operations.
    ///
    /// This timeout applies to operations that write data to the filesystem,
    /// such as writing file contents or creating directories.
    write_timeout: Duration,

    /// Timeout duration for general operations.
    ///
    /// This timeout applies to operations that don't fit into read/write
    /// categories, such as checking file existence, getting metadata,
    /// or copying files (which involves both read and write).
    operation_timeout: Duration,
}

impl FileSystemConfig {
    /// Returns a builder for creating a custom configuration.
    ///
    /// The builder starts with default values and allows customizing
    /// individual timeout settings before building the final configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::builder()
    ///     .with_read_timeout(Duration::from_secs(10))
    ///     .with_write_timeout(Duration::from_secs(15))
    ///     .with_operation_timeout(Duration::from_secs(45))
    ///     .build();
    ///
    /// assert_eq!(config.read_timeout(), Duration::from_secs(10));
    /// assert_eq!(config.write_timeout(), Duration::from_secs(15));
    /// assert_eq!(config.operation_timeout(), Duration::from_secs(45));
    /// ```
    #[must_use]
    pub fn builder() -> FileSystemConfigBuilder {
        FileSystemConfigBuilder::default()
    }

    /// Returns the read operation timeout.
    ///
    /// This timeout is used for operations that read data from the filesystem,
    /// such as:
    /// - Reading file contents (`read_to_string`, `read`)
    /// - Listing directory entries (`read_dir`)
    /// - Reading symbolic link targets
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::default();
    /// assert_eq!(config.read_timeout(), Duration::from_secs(30));
    ///
    /// let custom = FileSystemConfig::builder()
    ///     .with_read_timeout(Duration::from_millis(500))
    ///     .build();
    /// assert_eq!(custom.read_timeout(), Duration::from_millis(500));
    /// ```
    #[must_use]
    pub fn read_timeout(&self) -> Duration {
        self.read_timeout
    }

    /// Returns the write operation timeout.
    ///
    /// This timeout is used for operations that write data to the filesystem,
    /// such as:
    /// - Writing file contents (`write`)
    /// - Creating files or directories (`create_dir`, `create_dir_all`)
    /// - Removing files or directories (`remove_file`, `remove_dir`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::default();
    /// assert_eq!(config.write_timeout(), Duration::from_secs(30));
    ///
    /// let custom = FileSystemConfig::builder()
    ///     .with_write_timeout(Duration::from_secs(120))
    ///     .build();
    /// assert_eq!(custom.write_timeout(), Duration::from_secs(120));
    /// ```
    #[must_use]
    pub fn write_timeout(&self) -> Duration {
        self.write_timeout
    }

    /// Returns the general operation timeout.
    ///
    /// This timeout is used for operations that don't fit into read/write
    /// categories, such as:
    /// - Getting file metadata (`metadata`)
    /// - Checking file existence (`exists`)
    /// - Copying files (`copy`) - which involves both read and write
    /// - Renaming files (`rename`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::default();
    /// assert_eq!(config.operation_timeout(), Duration::from_secs(60));
    ///
    /// let custom = FileSystemConfig::builder()
    ///     .with_operation_timeout(Duration::from_secs(180))
    ///     .build();
    /// assert_eq!(custom.operation_timeout(), Duration::from_secs(180));
    /// ```
    #[must_use]
    pub fn operation_timeout(&self) -> Duration {
        self.operation_timeout
    }
}

impl Default for FileSystemConfig {
    /// Creates a configuration with default timeout values.
    ///
    /// # Default Values
    ///
    /// | Setting | Default Value |
    /// |---------|---------------|
    /// | `read_timeout` | 30 seconds |
    /// | `write_timeout` | 30 seconds |
    /// | `operation_timeout` | 60 seconds |
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::default();
    ///
    /// assert_eq!(config.read_timeout(), Duration::from_secs(30));
    /// assert_eq!(config.write_timeout(), Duration::from_secs(30));
    /// assert_eq!(config.operation_timeout(), Duration::from_secs(60));
    /// ```
    fn default() -> Self {
        Self {
            read_timeout: Duration::from_secs(DEFAULT_READ_TIMEOUT_SECS),
            write_timeout: Duration::from_secs(DEFAULT_WRITE_TIMEOUT_SECS),
            operation_timeout: Duration::from_secs(DEFAULT_OPERATION_TIMEOUT_SECS),
        }
    }
}

// =============================================================================
// FileSystemConfigBuilder Struct
// =============================================================================

/// Builder for creating a custom [`FileSystemConfig`].
///
/// This builder implements the builder pattern, allowing fluent construction
/// of configuration with customized values. All values start at their defaults
/// and can be selectively overridden.
///
/// # Example
///
/// ```rust
/// use workspace_fs::FileSystemConfig;
/// use std::time::Duration;
///
/// // Create a builder and customize values
/// let config = FileSystemConfig::builder()
///     .with_read_timeout(Duration::from_secs(10))
///     .with_write_timeout(Duration::from_secs(10))
///     .with_operation_timeout(Duration::from_secs(30))
///     .build();
///
/// // Values not set remain at defaults
/// let partial = FileSystemConfig::builder()
///     .with_read_timeout(Duration::from_secs(5))
///     .build();
///
/// assert_eq!(partial.read_timeout(), Duration::from_secs(5));
/// assert_eq!(partial.write_timeout(), Duration::from_secs(30)); // default
/// ```
#[derive(Debug, Clone)]
pub struct FileSystemConfigBuilder {
    /// The configuration being built.
    ///
    /// Starts with default values and is modified by builder methods.
    config: FileSystemConfig,
}

impl Default for FileSystemConfigBuilder {
    /// Creates a new builder with default configuration values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfigBuilder;
    ///
    /// let builder = FileSystemConfigBuilder::default();
    /// let config = builder.build();
    ///
    /// // All values are at defaults
    /// use std::time::Duration;
    /// assert_eq!(config.read_timeout(), Duration::from_secs(30));
    /// ```
    fn default() -> Self {
        Self { config: FileSystemConfig::default() }
    }
}

impl FileSystemConfigBuilder {
    /// Sets the read operation timeout.
    ///
    /// This timeout will be used for filesystem operations that read data,
    /// such as reading file contents or listing directories.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The duration to use for read operation timeouts
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::builder()
    ///     .with_read_timeout(Duration::from_secs(10))
    ///     .build();
    ///
    /// assert_eq!(config.read_timeout(), Duration::from_secs(10));
    /// ```
    #[must_use]
    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.config.read_timeout = timeout;
        self
    }

    /// Sets the write operation timeout.
    ///
    /// This timeout will be used for filesystem operations that write data,
    /// such as writing file contents or creating directories.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The duration to use for write operation timeouts
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::builder()
    ///     .with_write_timeout(Duration::from_secs(15))
    ///     .build();
    ///
    /// assert_eq!(config.write_timeout(), Duration::from_secs(15));
    /// ```
    #[must_use]
    pub fn with_write_timeout(mut self, timeout: Duration) -> Self {
        self.config.write_timeout = timeout;
        self
    }

    /// Sets the general operation timeout.
    ///
    /// This timeout will be used for filesystem operations that don't fit
    /// into read/write categories, such as metadata queries or file copying.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The duration to use for general operation timeouts
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::builder()
    ///     .with_operation_timeout(Duration::from_secs(90))
    ///     .build();
    ///
    /// assert_eq!(config.operation_timeout(), Duration::from_secs(90));
    /// ```
    #[must_use]
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.config.operation_timeout = timeout;
        self
    }

    /// Builds the [`FileSystemConfig`] with the configured values.
    ///
    /// Consumes the builder and returns the final, immutable configuration.
    /// Any values not explicitly set will use their defaults.
    ///
    /// # Example
    ///
    /// ```rust
    /// use workspace_fs::FileSystemConfig;
    /// use std::time::Duration;
    ///
    /// let config = FileSystemConfig::builder()
    ///     .with_read_timeout(Duration::from_secs(5))
    ///     .build();
    ///
    /// assert_eq!(config.read_timeout(), Duration::from_secs(5));
    /// assert_eq!(config.write_timeout(), Duration::from_secs(30)); // default
    /// assert_eq!(config.operation_timeout(), Duration::from_secs(60)); // default
    /// ```
    #[must_use]
    pub fn build(self) -> FileSystemConfig {
        self.config
    }
}

// =============================================================================
// Static Assertions
// =============================================================================

// Static assertions to ensure FileSystemConfig is Send + Sync
// This is important for use in async contexts and multi-threaded applications
const _: () = {
    const fn assert_send<T: Send>() {}
    const fn assert_sync<T: Sync>() {}
    assert_send::<FileSystemConfig>();
    assert_sync::<FileSystemConfig>();
    assert_send::<FileSystemConfigBuilder>();
    assert_sync::<FileSystemConfigBuilder>();
};
