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
//! ```rust,ignore
//! use workspace_fs::FileSystemConfig;
//! use std::time::Duration;
//!
//! // Use default configuration (30s read/write, 60s operation)
//! let config = FileSystemConfig::default();
//!
//! // Custom configuration with builder
//! let config = FileSystemConfig::builder()
//!     .with_read_timeout(Duration::from_secs(10))
//!     .with_write_timeout(Duration::from_secs(10))
//!     .with_operation_timeout(Duration::from_secs(30))
//!     .build();
//! ```

// TODO: will be implemented on epic workspace-node-tools-g2t (Configuration Module)
#![allow(clippy::todo)]
