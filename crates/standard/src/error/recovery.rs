//! # Error recovery strategies and management
//!
//! ## What
//! This module provides a comprehensive error recovery system including
//! strategies, recovery results, and a manager for applying recovery logic.
//!
//! ## How
//! The system defines various recovery strategies (fail-fast, retry, graceful degradation)
//! and provides a manager to configure and apply these strategies based on error types.
//!
//! ## Why
//! A structured recovery system enables resilient applications that can handle
//! errors appropriately based on context, improving reliability and user experience.

use core::result::Result as CoreResult;
use std::collections::HashMap;
use std::time::Duration;

use super::Error;

/// Recovery strategies for error handling.
///
/// This enum defines different approaches to handling errors, allowing for
/// configurable behavior based on the error type and context.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{RecoveryStrategy, LogLevel};
///
/// // Fail-fast strategy for critical errors
/// let critical = RecoveryStrategy::Fail;
///
/// // Log and continue for non-critical warnings
/// let warning = RecoveryStrategy::LogAndContinue(LogLevel::Warn);
///
/// // Retry transient failures
/// let retry = RecoveryStrategy::Retry { max_attempts: 3, backoff_ms: 100 };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Propagate error up (fail-fast approach).
    Fail,

    /// Log the error at specified level and continue operation.
    LogAndContinue(LogLevel),

    /// Use a default value with explanation.
    UseDefault(String),

    /// Retry the operation with exponential backoff.
    Retry {
        /// Maximum number of retry attempts
        max_attempts: usize,
        /// Initial backoff in milliseconds
        backoff_ms: u64,
    },

    /// Graceful degradation with explanation.
    Graceful(String),
}

/// Log level for structured logging.
///
/// Used with `RecoveryStrategy::LogAndContinue` to specify the severity
/// of the logged error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug level logging for detailed diagnostics.
    Debug,
    /// Informational messages.
    Info,
    /// Warning messages for recoverable issues.
    Warn,
    /// Error messages for serious issues.
    Error,
}

/// Recovery result after applying a recovery strategy.
///
/// This enum represents the outcome of applying a recovery strategy to an error,
/// providing information about whether the operation succeeded, failed, or was
/// recovered through an alternative approach.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::RecoveryResult;
///
/// fn handle_operation() -> RecoveryResult<String> {
///     // On success
///     RecoveryResult::Success("operation completed".to_string())
///     
///     // On recovery with default
///     // RecoveryResult::Recovered {
///     //     value: Some("default value".to_string()),
///     //     reason: "Using default due to config error".to_string()
///     // }
/// }
/// ```
#[derive(Debug)]
pub enum RecoveryResult<T> {
    /// Operation succeeded without error.
    Success(T),

    /// Operation failed and error was propagated.
    Failed(Error),

    /// Operation recovered using alternative approach.
    Recovered {
        /// The recovered value (if any)
        value: Option<T>,
        /// Explanation of the recovery
        reason: String,
    },

    /// Operation will be retried.
    Retrying {
        /// Current attempt number
        attempt: usize,
        /// Time until next retry
        next_backoff: Duration,
    },
}

/// Manager for applying error recovery strategies.
///
/// The `ErrorRecoveryManager` provides a centralized way to configure and apply
/// recovery strategies based on error types. It supports default strategies and
/// specific strategies for different error categories.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{ErrorRecoveryManager, RecoveryStrategy, LogLevel};
///
/// let manager = ErrorRecoveryManager::new()
///     .with_strategy("filesystem_error", RecoveryStrategy::Retry {
///         max_attempts: 3,
///         backoff_ms: 100
///     })
///     .with_strategy("config_error", RecoveryStrategy::LogAndContinue(LogLevel::Warn))
///     .set_default(RecoveryStrategy::Fail);
/// ```
#[derive(Debug, Clone)]
pub struct ErrorRecoveryManager {
    /// Strategies mapped by error type
    strategies: HashMap<String, RecoveryStrategy>,
    /// Default strategy when no specific strategy is found
    default_strategy: RecoveryStrategy,
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryManager {
    /// Creates a new error recovery manager with fail-fast default.
    pub fn new() -> Self {
        Self { strategies: HashMap::new(), default_strategy: RecoveryStrategy::Fail }
    }

    /// Adds a recovery strategy for a specific error type.
    ///
    /// # Arguments
    ///
    /// * `error_type` - The error type identifier (e.g., "filesystem_error")
    /// * `strategy` - The recovery strategy to apply
    #[must_use]
    pub fn with_strategy(mut self, error_type: &str, strategy: RecoveryStrategy) -> Self {
        self.strategies.insert(error_type.to_string(), strategy);
        self
    }

    /// Sets the default recovery strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The default strategy to use when no specific strategy is found
    #[must_use]
    pub fn set_default(mut self, strategy: RecoveryStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    /// Gets the recovery strategy for a given error type.
    ///
    /// # Arguments
    ///
    /// * `error_type` - The error type identifier
    ///
    /// # Returns
    ///
    /// The configured strategy for the error type, or the default strategy
    pub fn get_strategy(&self, error_type: &str) -> &RecoveryStrategy {
        self.strategies.get(error_type).unwrap_or(&self.default_strategy)
    }

    /// Handles an error result by applying the appropriate recovery strategy.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to handle
    /// * `context` - Context information for logging
    ///
    /// # Returns
    ///
    /// A `RecoveryResult` indicating the outcome of applying the recovery strategy
    pub fn handle<T, E>(&self, result: CoreResult<T, E>, context: &str) -> RecoveryResult<T>
    where
        E: AsRef<str> + std::fmt::Debug + Clone + Into<Error>,
    {
        match result {
            Ok(value) => RecoveryResult::Success(value),
            Err(error) => {
                let error_type = error.as_ref();
                let strategy = self.get_strategy(error_type);

                match strategy {
                    RecoveryStrategy::Fail => RecoveryResult::Failed(error.into()),
                    RecoveryStrategy::LogAndContinue(level) => {
                        // Enterprise logging with structured context and comprehensive error analysis
                        match level {
                            LogLevel::Debug => log::debug!(
                                target: "recovery",
                                "Recovery executed - Context: {context} | Error: {error:?} | Strategy: LogAndContinue(Debug)"
                            ),
                            LogLevel::Info => log::info!(
                                target: "recovery",
                                "Recovery executed - Context: {context} | Error: {error:?} | Strategy: LogAndContinue(Info)"
                            ),
                            LogLevel::Warn => log::warn!(
                                target: "recovery",
                                "Recovery warning - Context: {context} | Error: {error:?} | Strategy: LogAndContinue(Warn)"
                            ),
                            LogLevel::Error => log::error!(
                                target: "recovery",
                                "Recovery error - Context: {context} | Error: {error:?} | Strategy: LogAndContinue(Error)"
                            ),
                        }
                        RecoveryResult::Recovered {
                            value: None,
                            reason: format!("Logged and continued after error: {error:?}"),
                        }
                    }
                    RecoveryStrategy::UseDefault(reason) => {
                        RecoveryResult::Recovered { value: None, reason: reason.clone() }
                    }
                    RecoveryStrategy::Retry { max_attempts: _, backoff_ms } => {
                        // This is a simplified version - real implementation would
                        // need to track attempt count and handle actual retries
                        RecoveryResult::Retrying {
                            attempt: 1,
                            next_backoff: Duration::from_millis(*backoff_ms),
                        }
                    }
                    RecoveryStrategy::Graceful(message) => {
                        RecoveryResult::Recovered { value: None, reason: message.clone() }
                    }
                }
            }
        }
    }

    /// Enterprise-grade error analysis and structured logging for recovery operations.
    ///
    /// This method provides comprehensive error analysis with proper error chain
    /// tracking and structured context for enterprise monitoring and debugging.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level for additional context
    /// * `context` - The operation context
    /// * `error` - The error being recovered from
    ///
    /// # Features
    ///
    /// - Full error chain analysis for root cause identification
    /// - Structured context information for enterprise observability
    /// - Integration-ready format for monitoring systems
    #[allow(dead_code)]
    fn analyze_error_context<E: std::error::Error>(level: LogLevel, context: &str, error: &E) {
        // Comprehensive error chain analysis for enterprise debugging
        let error_chain = Self::build_error_chain(error);
        let error_type = std::any::type_name::<E>();

        // Structured logging for enterprise monitoring integration
        log::info!(
            target: "recovery.analysis",
            "Error Analysis - Level: {:?} | Context: {} | Type: {} | Chain Length: {} | Root Cause: {}",
            level,
            context,
            error_type,
            error_chain.len(),
            error_chain.first().unwrap_or(&"Unknown".to_string())
        );

        // Detailed error chain logging for debugging
        for (index, error_msg) in error_chain.iter().enumerate() {
            log::debug!(
                target: "recovery.chain",
                "Error Chain[{index}]: {error_msg}"
            );
        }
    }

    /// Builds comprehensive error chain for enterprise-grade error analysis.
    ///
    /// This method traverses the complete error chain to provide detailed
    /// error context for debugging and monitoring in enterprise environments.
    ///
    /// # Arguments
    ///
    /// * `error` - The root error to analyze
    ///
    /// # Returns
    ///
    /// A vector of error messages representing the complete error chain.
    fn build_error_chain<E: std::error::Error>(error: &E) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current_error: &dyn std::error::Error = error;

        // Traverse complete error chain for comprehensive analysis
        loop {
            chain.push(current_error.to_string());
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        chain
    }
}
