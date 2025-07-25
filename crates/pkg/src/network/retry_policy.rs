//! Retry policy implementation with exponential backoff
//!
//! This module provides configurable retry policies for network operations,
//! including exponential backoff with jitter to prevent thundering herd problems.
//!
//! # Examples
//!
//! ```
//! use sublime_package_tools::network::{RetryPolicy, RetryConfig};
//! use std::time::Duration;
//!
//! // Create a retry policy with exponential backoff
//! let config = RetryConfig {
//!     max_retries: 3,
//!     initial_delay: Duration::from_millis(100),
//!     max_delay: Duration::from_secs(10),
//!     exponential_base: 2.0,
//!     jitter: true,
//! };
//! let policy = RetryPolicy::new(config);
//!
//! // Use with an async operation
//! let result = policy.execute(|| async {
//!     // Your network operation here
//!     Ok::<_, std::io::Error>("success")
//! }).await;
//! ```

use rand::Rng;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Configuration for retry policy
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 means no retries)
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Base for exponential backoff (typically 2.0)
    pub exponential_base: f64,
    /// Whether to add random jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            exponential_base: 2.0,
            jitter: true,
        }
    }
}

/// Retry policy with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    config: RetryConfig,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(RetryConfig::default())
    }
}

impl RetryPolicy {
    /// Create a new retry policy with the given configuration
    #[must_use]
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an operation with retry logic
    ///
    /// The operation will be retried according to the policy configuration
    /// if it returns an error. The error type must implement the `Retryable`
    /// trait to determine if a retry should be attempted.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The async operation closure
    /// * `Fut` - The future returned by the operation
    /// * `T` - The success type
    /// * `E` - The error type (must implement `Retryable`)
    ///
    /// # Returns
    ///
    /// The result of the operation, or the last error if all retries failed
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Retryable + std::fmt::Display,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if !error.is_retryable() {
                        debug!("Non-retryable error: {}", error);
                        return Err(error);
                    }

                    if attempt >= self.config.max_retries {
                        warn!(
                            "Operation failed after {} retries: {}",
                            attempt, error
                        );
                        return Err(error);
                    }

                    let delay = self.calculate_delay(attempt);
                    warn!(
                        "Attempt {} failed: {}. Retrying in {:?}",
                        attempt + 1,
                        error,
                        delay
                    );

                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Execute an operation with retry logic and a custom retry predicate
    ///
    /// This variant allows you to provide a custom function to determine
    /// whether an error should trigger a retry.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The async operation closure
    /// * `Fut` - The future returned by the operation
    /// * `P` - The retry predicate closure
    /// * `T` - The success type
    /// * `E` - The error type
    ///
    /// # Returns
    ///
    /// The result of the operation, or the last error if all retries failed
    pub async fn execute_with_predicate<F, Fut, P, T, E>(
        &self,
        mut operation: F,
        mut should_retry: P,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        P: FnMut(&E) -> bool,
        E: std::fmt::Display,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if !should_retry(&error) {
                        debug!("Retry predicate returned false: {}", error);
                        return Err(error);
                    }

                    if attempt >= self.config.max_retries {
                        warn!(
                            "Operation failed after {} retries: {}",
                            attempt, error
                        );
                        return Err(error);
                    }

                    let delay = self.calculate_delay(attempt);
                    warn!(
                        "Attempt {} failed: {}. Retrying in {:?}",
                        attempt + 1,
                        error,
                        delay
                    );

                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Calculate the delay for a given attempt number
    ///
    /// Uses exponential backoff with optional jitter
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let exponential_delay = self.config.initial_delay.as_millis() as f64
            * self.config.exponential_base.powi(attempt as i32);

        let capped_delay = exponential_delay.min(self.config.max_delay.as_millis() as f64);

        let final_delay = if self.config.jitter {
            // Add jitter between 0% and 50% of the delay
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.0..0.5);
            capped_delay * (1.0 + jitter_factor)
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay as u64)
    }

    /// Get the maximum number of retries
    #[must_use]
    pub fn max_retries(&self) -> u32 {
        self.config.max_retries
    }

    /// Create a retry policy with no retries (execute once only)
    #[must_use]
    pub fn no_retry() -> Self {
        Self::new(RetryConfig {
            max_retries: 0,
            ..Default::default()
        })
    }

    /// Create a retry policy with fixed delay between retries
    #[must_use]
    pub fn fixed_delay(max_retries: u32, delay: Duration) -> Self {
        Self::new(RetryConfig {
            max_retries,
            initial_delay: delay,
            max_delay: delay,
            exponential_base: 1.0, // No exponential growth
            jitter: false,
        })
    }
}

/// Trait for errors that can be retried
///
/// Implement this trait for error types to indicate whether
/// a failed operation should be retried.
pub trait Retryable {
    /// Determine if this error indicates the operation should be retried
    ///
    /// # Returns
    ///
    /// `true` if the operation should be retried, `false` otherwise
    fn is_retryable(&self) -> bool;
}

/// Common retryable conditions for network errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryableError {
    /// Temporary network failure
    NetworkError,
    /// Server returned 5xx status
    ServerError,
    /// Request timed out
    Timeout,
    /// Rate limit exceeded (429)
    RateLimited,
    /// Connection reset or refused
    ConnectionError,
    /// DNS resolution failure
    DnsError,
}

impl Retryable for RetryableError {
    fn is_retryable(&self) -> bool {
        match self {
            Self::NetworkError
            | Self::ServerError
            | Self::Timeout
            | Self::RateLimited
            | Self::ConnectionError
            | Self::DnsError => true,
        }
    }
}

/// Implementation of Retryable for reqwest errors
impl Retryable for reqwest::Error {
    fn is_retryable(&self) -> bool {
        // Network errors are retryable
        if self.is_timeout() || self.is_connect() {
            return true;
        }

        // Check status code for server errors
        if let Some(status) = self.status() {
            // 5xx errors are retryable
            if status.is_server_error() {
                return true;
            }
            
            // 429 Too Many Requests is retryable
            if status.as_u16() == 429 {
                return true;
            }
            
            // 408 Request Timeout is retryable
            if status.as_u16() == 408 {
                return true;
            }
        }

        false
    }
}

/// Implementation of Retryable for std::io::Error
impl Retryable for std::io::Error {
    fn is_retryable(&self) -> bool {
        use std::io::ErrorKind;

        matches!(self.kind(), 
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::TimedOut
            | ErrorKind::Interrupted
            | ErrorKind::UnexpectedEof
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_successful_operation() {
        let policy = RetryPolicy::default();
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = policy
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, std::io::Error>("success")
                }
            })
            .await;

        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, "success");
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_on_failure() {
        let policy = RetryPolicy::new(RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        });

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = policy
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::ConnectionRefused,
                            "connection refused",
                        ))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, "success");
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let policy = RetryPolicy::new(RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        });

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = policy
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>(std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        "connection refused",
                    ))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let policy = RetryPolicy::default();
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = policy
            .execute(|| {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "permission denied",
                    ))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1); // No retries
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let policy = RetryPolicy::new(RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            exponential_base: 2.0,
            jitter: false,
        });

        // Test delay calculation
        assert_eq!(policy.calculate_delay(0), Duration::from_millis(10));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(20));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(40));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(80));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(100)); // Capped at max
    }

    #[tokio::test]
    async fn test_custom_predicate() {
        let policy = RetryPolicy::default();
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = policy
            .execute_with_predicate(
                || {
                    let attempts = Arc::clone(&attempts_clone);
                    async move {
                        let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                        if attempt < 2 {
                            Err("temporary error")
                        } else {
                            Ok("success")
                        }
                    }
                },
                |error| error.contains("temporary"),
            )
            .await;

        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, "success");
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_no_retry_policy() {
        let policy = RetryPolicy::no_retry();
        assert_eq!(policy.max_retries(), 0);
    }

    #[test]
    fn test_fixed_delay_policy() {
        let policy = RetryPolicy::fixed_delay(3, Duration::from_millis(50));
        assert_eq!(policy.calculate_delay(0), Duration::from_millis(50));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(50));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(50));
    }
}