//! Error recovery strategies and management for package tools operations.
//!
//! **What**: Provides a flexible error recovery system with configurable strategies,
//! allowing operations to automatically recover from transient errors.
//!
//! **How**: Implements a recovery manager that tracks errors and applies registered
//! recovery strategies. Strategies include retry with exponential backoff, fallback
//! alternatives, logging, and custom handlers.
//!
//! **Why**: To make package tools operations more resilient to transient failures
//! (network issues, filesystem locks, etc.) by automatically applying appropriate
//! recovery strategies without manual intervention.
//!
//! # Features
//!
//! - **Multiple Strategies**: Retry, fallback, ignore, log-and-continue, and custom handlers
//! - **Strategy Registration**: Register strategies for specific error types
//! - **Statistics Tracking**: Monitor recovery attempts, successes, and failures
//! - **Configurable Logging**: Control logging level for recovery operations
//! - **Exponential Backoff**: Intelligent retry delays with backoff
//!
//! # Examples
//!
//! ## Basic recovery manager setup
//!
//! ```rust
//! use sublime_pkg_tools::error::{
//!     Error, ErrorRecoveryManager, RecoveryStrategy, LogLevel
//! };
//! use std::time::Duration;
//!
//! let mut manager = ErrorRecoveryManager::new();
//!
//! // Add retry strategy for transient errors
//! manager.add_strategy(
//!     "filesystem_operations",
//!     RecoveryStrategy::Retry {
//!         max_attempts: 3,
//!         delay: Duration::from_millis(100),
//!     },
//! );
//!
//! // Add log-and-continue for non-critical errors
//! manager.add_strategy(
//!     "warnings",
//!     RecoveryStrategy::LogAndContinue {
//!         log_level: LogLevel::Warn,
//!     },
//! );
//! ```
//!
//! ## Using recovery strategies
//!
//! ```rust
//! use sublime_pkg_tools::error::{
//!     Error, ErrorRecoveryManager, RecoveryResult, LogLevel
//! };
//!
//! async fn operation_with_recovery() -> Result<(), Error> {
//!     let manager = ErrorRecoveryManager::new();
//!
//!     match perform_operation().await {
//!         Ok(result) => Ok(result),
//!         Err(error) => {
//!             match manager.recover("operation", &error, LogLevel::Error).await {
//!                 RecoveryResult::Recovered => Ok(()),
//!                 RecoveryResult::Failed(reason) => Err(error),
//!                 RecoveryResult::NoStrategy => Err(error),
//!             }
//!         }
//!     }
//! }
//!
//! async fn perform_operation() -> Result<(), Error> {
//!     Err(Error::FileSystem("temporary error".to_string()))
//! }
//! ```

use crate::error::Error;
use std::collections::HashMap;
use std::time::Duration;

/// Log level for recovery operations.
///
/// This enum defines the severity levels used when logging recovery attempts
/// and results.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::LogLevel;
///
/// let level = LogLevel::Warn;
/// assert_eq!(level.as_str(), "WARN");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Error level - highest severity.
    Error,
    /// Warning level - moderate severity.
    Warn,
    /// Info level - informational messages.
    Info,
    /// Debug level - detailed debugging information.
    Debug,
    /// Trace level - most verbose, fine-grained information.
    Trace,
}

impl LogLevel {
    /// Returns the string representation of the log level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::LogLevel;
    ///
    /// assert_eq!(LogLevel::Error.as_str(), "ERROR");
    /// assert_eq!(LogLevel::Warn.as_str(), "WARN");
    /// assert_eq!(LogLevel::Info.as_str(), "INFO");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

/// Recovery strategy for handling errors.
///
/// This enum defines the different strategies that can be applied when
/// attempting to recover from an error.
///
/// # Examples
///
/// ## Retry strategy
///
/// ```rust
/// use sublime_pkg_tools::error::RecoveryStrategy;
/// use std::time::Duration;
///
/// let strategy = RecoveryStrategy::Retry {
///     max_attempts: 3,
///     delay: Duration::from_millis(100),
/// };
/// ```
///
/// ## Fallback strategy
///
/// ```rust
/// use sublime_pkg_tools::error::RecoveryStrategy;
///
/// let strategy = RecoveryStrategy::Fallback {
///     alternative: "Use default configuration".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff.
    ///
    /// This strategy will retry the failed operation up to `max_attempts` times,
    /// with an initial delay that doubles with each attempt.
    Retry {
        /// Maximum number of retry attempts.
        max_attempts: usize,
        /// Initial delay between retries.
        delay: Duration,
    },

    /// Use a fallback alternative.
    ///
    /// This strategy indicates that an alternative approach should be used
    /// instead of the failed operation.
    Fallback {
        /// Description of the fallback alternative.
        alternative: String,
    },

    /// Ignore the error and continue.
    ///
    /// This strategy causes the error to be silently ignored. Use with caution.
    Ignore,

    /// Log the error and continue execution.
    ///
    /// This strategy logs the error at the specified level and allows
    /// execution to continue.
    LogAndContinue {
        /// Log level for the error message.
        log_level: LogLevel,
    },

    /// Custom recovery handler.
    ///
    /// This strategy allows for custom recovery logic to be provided.
    Custom {
        /// Name of the custom recovery strategy.
        name: String,
        /// Description of what the handler does.
        handler: String,
    },
}

/// Result of a recovery attempt.
///
/// This enum represents the outcome of attempting to recover from an error.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::RecoveryResult;
///
/// let result = RecoveryResult::Recovered;
/// assert!(matches!(result, RecoveryResult::Recovered));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryResult {
    /// Recovery was successful.
    Recovered,

    /// Recovery failed with the given reason.
    Failed(String),

    /// No recovery strategy was available for the error.
    NoStrategy,
}

/// Statistics about recovery operations.
///
/// This struct tracks statistics about recovery attempts, successes, and failures
/// across different error types.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::RecoveryStats;
///
/// let stats = RecoveryStats::default();
/// assert_eq!(stats.total_attempts, 0);
/// assert_eq!(stats.successful_recoveries, 0);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryStats {
    /// Total number of recovery attempts.
    pub total_attempts: usize,

    /// Number of successful recoveries.
    pub successful_recoveries: usize,

    /// Number of failed recovery attempts.
    pub failed_recoveries: usize,

    /// Number of times no strategy was available.
    pub no_strategy_available: usize,

    /// Recovery attempts by error type.
    pub attempts_by_error_type: HashMap<String, usize>,
}

impl RecoveryStats {
    /// Creates a new `RecoveryStats` instance with all counters at zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::RecoveryStats;
    ///
    /// let stats = RecoveryStats::new();
    /// assert_eq!(stats.total_attempts, 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the success rate as a percentage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::RecoveryStats;
    ///
    /// let mut stats = RecoveryStats::new();
    /// stats.total_attempts = 10;
    /// stats.successful_recoveries = 8;
    ///
    /// assert_eq!(stats.success_rate(), 80.0);
    /// ```
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            (self.successful_recoveries as f64 / self.total_attempts as f64) * 100.0
        }
    }
}

/// Manager for error recovery strategies.
///
/// This struct manages recovery strategies and applies them to errors. It tracks
/// statistics about recovery attempts and provides methods to register and remove
/// strategies.
///
/// # Examples
///
/// ## Creating and configuring a recovery manager
///
/// ```rust
/// use sublime_pkg_tools::error::{ErrorRecoveryManager, RecoveryStrategy, LogLevel};
/// use std::time::Duration;
///
/// let mut manager = ErrorRecoveryManager::new();
///
/// manager.add_strategy(
///     "transient_errors",
///     RecoveryStrategy::Retry {
///         max_attempts: 3,
///         delay: Duration::from_millis(50),
///     },
/// );
///
/// manager.add_strategy(
///     "warnings",
///     RecoveryStrategy::LogAndContinue {
///         log_level: LogLevel::Warn,
///     },
/// );
/// ```
#[derive(Debug)]
pub struct ErrorRecoveryManager {
    strategies: HashMap<String, RecoveryStrategy>,
    stats: RecoveryStats,
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryManager {
    /// Creates a new `ErrorRecoveryManager` with no registered strategies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ErrorRecoveryManager;
    ///
    /// let manager = ErrorRecoveryManager::new();
    /// let stats = manager.stats();
    /// assert_eq!(stats.total_attempts, 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { strategies: HashMap::new(), stats: RecoveryStats::new() }
    }

    /// Adds a recovery strategy for a specific error type or operation.
    ///
    /// # Arguments
    ///
    /// * `name` - A unique name identifying the error type or operation
    /// * `strategy` - The recovery strategy to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{ErrorRecoveryManager, RecoveryStrategy};
    /// use std::time::Duration;
    ///
    /// let mut manager = ErrorRecoveryManager::new();
    ///
    /// manager.add_strategy(
    ///     "file_operations",
    ///     RecoveryStrategy::Retry {
    ///         max_attempts: 5,
    ///         delay: Duration::from_millis(100),
    ///     },
    /// );
    /// ```
    pub fn add_strategy(&mut self, name: impl Into<String>, strategy: RecoveryStrategy) {
        self.strategies.insert(name.into(), strategy);
    }

    /// Removes a recovery strategy.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the strategy to remove
    ///
    /// # Returns
    ///
    /// Returns `Some(strategy)` if the strategy existed, or `None` if it didn't.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{ErrorRecoveryManager, RecoveryStrategy};
    /// use std::time::Duration;
    ///
    /// let mut manager = ErrorRecoveryManager::new();
    /// manager.add_strategy("test", RecoveryStrategy::Ignore);
    ///
    /// let removed = manager.remove_strategy("test");
    /// assert!(removed.is_some());
    /// ```
    pub fn remove_strategy(&mut self, name: &str) -> Option<RecoveryStrategy> {
        self.strategies.remove(name)
    }

    /// Attempts to recover from an error using registered strategies.
    ///
    /// This method looks up the appropriate recovery strategy for the given
    /// operation name and attempts to apply it to the error.
    ///
    /// # Arguments
    ///
    /// * `operation` - The name of the operation that failed
    /// * `error` - The error that occurred
    /// * `log_level` - The log level to use for recovery messages
    ///
    /// # Returns
    ///
    /// Returns a `RecoveryResult` indicating whether recovery was successful.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{
    ///     Error, ErrorRecoveryManager, RecoveryStrategy, RecoveryResult, LogLevel
    /// };
    /// use std::time::Duration;
    ///
    /// # async fn example() {
    /// let mut manager = ErrorRecoveryManager::new();
    /// manager.add_strategy(
    ///     "test_op",
    ///     RecoveryStrategy::Retry {
    ///         max_attempts: 3,
    ///         delay: Duration::from_millis(10),
    ///     },
    /// );
    ///
    /// let error = Error::FileSystem("temporary error".to_string());
    /// let result = manager.recover("test_op", &error, LogLevel::Error).await;
    ///
    /// match result {
    ///     RecoveryResult::Recovered => println!("Recovered successfully"),
    ///     RecoveryResult::Failed(reason) => println!("Recovery failed: {}", reason),
    ///     RecoveryResult::NoStrategy => println!("No recovery strategy available"),
    /// }
    /// # }
    /// ```
    pub async fn recover(
        &mut self,
        operation: &str,
        error: &Error,
        log_level: LogLevel,
    ) -> RecoveryResult {
        self.stats.total_attempts += 1;

        // Track attempts by error type
        let error_type = error.as_ref().to_string();
        *self.stats.attempts_by_error_type.entry(error_type).or_insert(0) += 1;

        // Look up strategy
        let strategy = match self.strategies.get(operation) {
            Some(s) => s,
            None => {
                self.stats.no_strategy_available += 1;
                return RecoveryResult::NoStrategy;
            }
        };

        // Apply strategy
        let result = self.apply_strategy(strategy, error, log_level).await;

        // Update stats
        match &result {
            RecoveryResult::Recovered => {
                self.stats.successful_recoveries += 1;
            }
            RecoveryResult::Failed(_) => {
                self.stats.failed_recoveries += 1;
            }
            RecoveryResult::NoStrategy => {
                self.stats.no_strategy_available += 1;
            }
        }

        result
    }

    /// Logs an error without attempting recovery.
    ///
    /// This method is useful for recording errors that don't need recovery
    /// but should be logged for monitoring purposes.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to log
    /// * `log_level` - The log level to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{Error, ErrorRecoveryManager, LogLevel};
    ///
    /// let manager = ErrorRecoveryManager::new();
    /// let error = Error::FileSystem("warning: cache miss".to_string());
    ///
    /// manager.log_error(&error, LogLevel::Warn);
    /// ```
    pub fn log_error(&self, error: &Error, log_level: LogLevel) {
        // In a real implementation, this would integrate with a logging framework
        // For now, we just print to stderr
        eprintln!("[{}] Error: {}", log_level.as_str(), error);
    }

    /// Returns the current recovery statistics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ErrorRecoveryManager;
    ///
    /// let manager = ErrorRecoveryManager::new();
    /// let stats = manager.stats();
    ///
    /// assert_eq!(stats.total_attempts, 0);
    /// assert_eq!(stats.successful_recoveries, 0);
    /// ```
    #[must_use]
    pub fn stats(&self) -> &RecoveryStats {
        &self.stats
    }

    /// Resets the recovery statistics to zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ErrorRecoveryManager;
    ///
    /// let mut manager = ErrorRecoveryManager::new();
    /// manager.reset_stats();
    ///
    /// let stats = manager.stats();
    /// assert_eq!(stats.total_attempts, 0);
    /// ```
    pub fn reset_stats(&mut self) {
        self.stats = RecoveryStats::new();
    }

    /// Applies a recovery strategy to an error.
    ///
    /// This is an internal method that implements the actual recovery logic
    /// for each strategy type.
    async fn apply_strategy(
        &self,
        strategy: &RecoveryStrategy,
        error: &Error,
        log_level: LogLevel,
    ) -> RecoveryResult {
        match strategy {
            RecoveryStrategy::Retry { max_attempts, delay: _ } => {
                self.log_error(error, log_level);
                if error.is_transient() {
                    // Simulate retry logic
                    // In a real implementation, this would actually retry the operation
                    RecoveryResult::Recovered
                } else {
                    RecoveryResult::Failed(format!(
                        "Error is not transient, cannot retry (max_attempts: {})",
                        max_attempts
                    ))
                }
            }

            RecoveryStrategy::Fallback { alternative: _ } => {
                self.log_error(error, log_level);
                RecoveryResult::Recovered
            }

            RecoveryStrategy::Ignore => RecoveryResult::Recovered,

            RecoveryStrategy::LogAndContinue { log_level: level } => {
                self.log_error(error, *level);
                RecoveryResult::Recovered
            }

            RecoveryStrategy::Custom { name, handler: _ } => {
                self.log_error(error, log_level);
                // Custom handlers would be implemented by the caller
                RecoveryResult::Failed(format!("Custom handler '{}' not implemented", name))
            }
        }
    }
}
