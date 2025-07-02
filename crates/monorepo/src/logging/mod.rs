//! Standardized logging utilities for monorepo operations
//!
//! This module provides consistent logging patterns, macros, and utilities
//! to ensure uniform logging across the entire monorepo tooling system.
//!
//! # What
//! Centralized logging standards and utilities providing structured logging,
//! consistent formatting, and contextual information for all operations.
//!
//! # How
//! Uses the `log` crate with custom macros that enforce consistent patterns,
//! include contextual metadata, and provide operation-specific logging helpers.
//!
//! # Why
//! Essential for debugging, monitoring, and understanding the flow of operations
//! in complex monorepo workflows. Standardization ensures logs are useful and
//! consistent across all modules.
//!
//! # Examples
//!
//! ```rust
//! use sublime_monorepo_tools::logging::{log_operation, log_package_operation};
//!
//! // Log a standard operation
//! log_operation("task_execution", "Starting task build", Some("my-package"));
//!
//! // Log with package context
//! log_package_operation("version_bump", "my-package", "Bumping version to 1.2.0");
//! ```

use std::fmt::Display;

/// Log level guidelines for consistent usage across the codebase
///
/// - **ERROR**: Unrecoverable errors that prevent operation completion
/// - **WARN**: Recoverable issues or potential problems that don't block execution
/// - **INFO**: High-level operation status and important state changes
/// - **DEBUG**: Detailed operation flow and intermediate results
/// - **TRACE**: Very detailed debugging information (rarely used)
pub struct LoggingStandards;

/// Standard operation logging with consistent formatting
///
/// # Arguments
///
/// * `operation` - The operation being performed (e.g., "task_execution", "version_bump")
/// * `message` - The log message
/// * `context` - Optional context identifier (e.g., package name, task name)
#[inline]
pub fn log_operation(operation: &str, message: impl Display, context: Option<&str>) {
    match context {
        Some(ctx) => log::info!("[{}] {}: {}", operation, ctx, message),
        None => log::info!("[{}] {}", operation, message),
    }
}

/// Log operation start with consistent formatting
#[inline]
pub fn log_operation_start(operation: &str, context: Option<&str>) {
    log_operation(operation, "Starting", context);
}

/// Log operation completion with consistent formatting
#[inline]
pub fn log_operation_complete(operation: &str, context: Option<&str>) {
    log_operation(operation, "Completed successfully", context);
}

/// Log operation failure with consistent formatting
#[inline]
pub fn log_operation_error(operation: &str, error: impl Display, context: Option<&str>) {
    match context {
        Some(ctx) => log::error!("[{}] {} failed: {}", operation, ctx, error),
        None => log::error!("[{}] Failed: {}", operation, error),
    }
}

/// Package-specific operation logging
#[inline]
pub fn log_package_operation(operation: &str, package: &str, message: impl Display) {
    log::info!("[{}] Package '{}': {}", operation, package, message);
}

/// Task execution logging
#[inline]
pub fn log_task_execution(task_name: &str, status: &str, details: Option<&str>) {
    match details {
        Some(d) => log::info!("[task] {}: {} - {}", task_name, status, d),
        None => log::info!("[task] {}: {}", task_name, status),
    }
}

/// Dependency operation logging
#[inline]
pub fn log_dependency_operation(operation: &str, from_package: &str, to_package: &str) {
    log::debug!("[dependency] {}: {} -> {}", operation, from_package, to_package);
}

/// File operation logging
#[inline]
pub fn log_file_operation(operation: &str, file_path: impl Display, success: bool) {
    if success {
        log::debug!("[file] {} {}: Success", operation, file_path);
    } else {
        log::warn!("[file] {} {}: Failed", operation, file_path);
    }
}

/// Performance logging for operation timing
#[inline]
#[allow(clippy::cast_precision_loss)]
pub fn log_performance(operation: &str, duration_ms: u64, item_count: Option<usize>) {
    match item_count {
        Some(count) => log::info!(
            "[performance] {}: {}ms for {} items ({:.2}ms/item)",
            operation,
            duration_ms,
            count,
            duration_ms as f64 / count as f64
        ),
        None => log::info!("[performance] {}: {}ms", operation, duration_ms),
    }
}

/// Warning for deprecated functionality
#[inline]
pub fn log_deprecation_warning(feature: &str, alternative: &str) {
    log::warn!(
        "[deprecation] '{}' is deprecated and will be removed in a future version. Use '{}' instead.",
        feature,
        alternative
    );
}

/// Structured error context logging
pub struct ErrorContext {
    operation: String,
    package: Option<String>,
    file: Option<String>,
    details: Vec<(String, String)>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self { operation: operation.into(), package: None, file: None, details: Vec::new() }
    }

    /// Add package context
    #[must_use]
    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    /// Add file context
    #[must_use]
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Add custom detail
    #[must_use]
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.push((key.into(), value.into()));
        self
    }

    /// Log the error with context
    pub fn log_error(self, error: impl Display) {
        let mut message =
            format!("[error] Operation '{operation}' failed: {error}", operation = self.operation);

        if let Some(pkg) = self.package {
            message.push_str(&format!("\n  Package: {pkg}"));
        }

        if let Some(file) = self.file {
            message.push_str(&format!("\n  File: {file}"));
        }

        for (key, value) in self.details {
            message.push_str(&format!("\n  {key}: {value}"));
        }

        log::error!("{}", message);
    }
}

/// Macro for consistent debug logging with function context
#[macro_export]
macro_rules! debug_context {
    ($msg:expr) => {
        log::debug!("[{}:{}] {}", module_path!(), line!(), $msg)
    };
    ($msg:expr, $($arg:tt)*) => {
        log::debug!("[{}:{}] {}", module_path!(), line!(), format!($msg, $($arg)*))
    };
}

/// Macro for operation timing
#[macro_export]
macro_rules! time_operation {
    ($operation:expr, $block:block) => {{
        let _start = std::time::Instant::now();
        let _result = $block;
        let _duration = _start.elapsed();
        $crate::logging::log_performance($operation, _duration.as_millis() as u64, None);
        _result
    }};
}

pub mod patterns {
    //! # Logging Patterns and Standards
    //!
    //! ## Log Levels
    //!
    //! - **ERROR**: Operation failures that prevent completion
    //!   - Format: `[operation] Context failed: Error details`
    //!   - Example: `[version_bump] Package 'my-app' failed: Invalid version format`
    //!
    //! - **WARN**: Issues that don't block execution
    //!   - Format: `[category] Warning: Details`
    //!   - Example: `[dependency] Warning: Circular dependency detected`
    //!
    //! - **INFO**: Major operation status
    //!   - Format: `[operation] Status: Details`
    //!   - Example: `[release] Starting: Version 1.2.0 for 5 packages`
    //!
    //! - **DEBUG**: Detailed flow information
    //!   - Format: `[module] Action: Details`
    //!   - Example: `[task] Checking condition: file_exists("package.json")`
    //!
    //! ## Context Tags
    //!
    //! Always use context tags in square brackets:
    //! - `[task]` - Task execution
    //! - `[package]` - Package operations
    //! - `[version]` - Version management
    //! - `[dependency]` - Dependency operations
    //! - `[file]` - File system operations
    //! - `[git]` - Git operations
    //! - `[hook]` - Git hook operations
    //! - `[plugin]` - Plugin operations
    //! - `[performance]` - Performance metrics
    //! - `[error]` - Error details
    //! - `[deprecation]` - Deprecation warnings
}
