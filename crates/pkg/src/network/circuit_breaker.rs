//! Circuit breaker pattern implementation
//!
//! This module provides a circuit breaker implementation to prevent cascading
//! failures when external services are experiencing issues. The circuit breaker
//! has three states: Closed (normal operation), Open (failing fast), and
//! Half-Open (testing if service has recovered).
//!
//! # Examples
//!
//! ```
//! use sublime_package_tools::network::{CircuitBreaker, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! // Create a circuit breaker
//! let config = CircuitBreakerConfig {
//!     failure_threshold: 5,
//!     success_threshold: 2,
//!     timeout: Duration::from_secs(30),
//!     half_open_max_calls: 3,
//! };
//! let circuit_breaker = CircuitBreaker::new("npm-registry", config);
//!
//! // Use with an async operation
//! let result = circuit_breaker.call(|| async {
//!     // Your network operation here
//!     Ok::<_, std::io::Error>("success")
//! }).await;
//! ```

use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - normal operation
    Closed,
    /// Circuit is open - failing fast
    Open,
    /// Circuit is half-open - testing recovery
    HalfOpen,
}

/// Configuration for circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to open the circuit
    pub failure_threshold: u32,
    /// Number of consecutive successes to close the circuit from half-open
    pub success_threshold: u32,
    /// Duration to wait before transitioning from open to half-open
    pub timeout: Duration,
    /// Maximum concurrent calls allowed in half-open state
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}

/// Circuit breaker error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open and rejecting calls
    #[error("Circuit breaker is open for {name}")]
    CircuitOpen { name: String },
    /// Operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(E),
}

/// Internal state tracking for circuit breaker
#[derive(Debug)]
struct CircuitBreakerState {
    /// Current state of the circuit
    state: CircuitState,
    /// Number of consecutive failures
    consecutive_failures: u32,
    /// Number of consecutive successes (in half-open state)
    consecutive_successes: u32,
    /// When the circuit was opened
    opened_at: Option<Instant>,
    /// Number of concurrent calls in half-open state
    half_open_calls: u32,
    /// Total call statistics
    total_calls: u64,
    total_failures: u64,
    total_successes: u64,
    total_rejections: u64,
}

impl CircuitBreakerState {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            consecutive_successes: 0,
            opened_at: None,
            half_open_calls: 0,
            total_calls: 0,
            total_failures: 0,
            total_successes: 0,
            total_rejections: 0,
        }
    }

    fn record_success(&mut self, config: &CircuitBreakerConfig) {
        self.total_successes += 1;
        self.consecutive_failures = 0;

        match self.state {
            CircuitState::Closed => {
                // Already closed, nothing to do
            }
            CircuitState::HalfOpen => {
                self.consecutive_successes += 1;
                if self.consecutive_successes >= config.success_threshold {
                    info!("Circuit breaker transitioning from half-open to closed");
                    self.state = CircuitState::Closed;
                    self.consecutive_successes = 0;
                    self.opened_at = None;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                warn!("Success recorded while circuit is open");
            }
        }
    }

    fn record_failure(&mut self, config: &CircuitBreakerConfig) {
        self.total_failures += 1;
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;

        match self.state {
            CircuitState::Closed => {
                if self.consecutive_failures >= config.failure_threshold {
                    warn!(
                        "Circuit breaker opening after {} consecutive failures",
                        self.consecutive_failures
                    );
                    self.state = CircuitState::Open;
                    self.opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker reopening from half-open state after failure");
                self.state = CircuitState::Open;
                self.opened_at = Some(Instant::now());
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    fn should_attempt_reset(&self, config: &CircuitBreakerConfig) -> bool {
        if let Some(opened_at) = self.opened_at {
            opened_at.elapsed() >= config.timeout
        } else {
            false
        }
    }

    fn can_make_call(&self, config: &CircuitBreakerConfig) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => self.half_open_calls < config.half_open_max_calls,
        }
    }
}

/// Circuit breaker implementation
#[derive(Clone)]
pub struct CircuitBreaker {
    /// Name of the circuit breaker (for logging)
    name: String,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Internal state (wrapped for thread safety)
    state: Arc<RwLock<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given name and configuration
    #[must_use]
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            name: name.into(),
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::new())),
        }
    }

    /// Execute an operation through the circuit breaker
    ///
    /// The operation will be executed if the circuit is closed or half-open
    /// (with restrictions). If the circuit is open, the operation will be
    /// rejected immediately.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The async operation closure
    /// * `Fut` - The future returned by the operation
    /// * `T` - The success type
    /// * `E` - The error type
    ///
    /// # Returns
    ///
    /// The result of the operation wrapped in `CircuitBreakerError`
    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if we should transition from open to half-open
        {
            let mut state = self.state.write().await;
            if state.state == CircuitState::Open && state.should_attempt_reset(&self.config) {
                info!("Circuit breaker {} transitioning to half-open", self.name);
                state.state = CircuitState::HalfOpen;
                state.consecutive_failures = 0;
                state.consecutive_successes = 0;
                state.half_open_calls = 0;
            }
        }

        // Check if we can make the call
        {
            let mut state = self.state.write().await;
            state.total_calls += 1;

            if !state.can_make_call(&self.config) {
                state.total_rejections += 1;
                debug!(
                    "Circuit breaker {} rejecting call (state: {:?})",
                    self.name, state.state
                );
                return Err(CircuitBreakerError::CircuitOpen {
                    name: self.name.clone(),
                });
            }

            // Increment half-open calls if in half-open state
            if state.state == CircuitState::HalfOpen {
                state.half_open_calls += 1;
            }
        }

        // Execute the operation
        let result = operation().await;

        // Update state based on result
        {
            let mut state = self.state.write().await;
            
            // Decrement half-open calls if in half-open state
            if state.state == CircuitState::HalfOpen && state.half_open_calls > 0 {
                state.half_open_calls -= 1;
            }

            match &result {
                Ok(_) => {
                    debug!("Circuit breaker {} call succeeded", self.name);
                    state.record_success(&self.config);
                }
                Err(error) => {
                    debug!("Circuit breaker {} call failed: {}", self.name, error);
                    state.record_failure(&self.config);
                }
            }
        }

        result.map_err(CircuitBreakerError::OperationFailed)
    }

    /// Get the current state of the circuit breaker
    pub async fn state(&self) -> CircuitState {
        self.state.read().await.state
    }

    /// Get statistics about the circuit breaker
    ///
    /// Returns (total_calls, total_successes, total_failures, total_rejections)
    pub async fn stats(&self) -> (u64, u64, u64, u64) {
        let state = self.state.read().await;
        (
            state.total_calls,
            state.total_successes,
            state.total_failures,
            state.total_rejections,
        )
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        info!("Circuit breaker {} manually reset", self.name);
        state.state = CircuitState::Closed;
        state.consecutive_failures = 0;
        state.consecutive_successes = 0;
        state.opened_at = None;
        state.half_open_calls = 0;
    }

    /// Force the circuit breaker to open state
    pub async fn trip(&self) {
        let mut state = self.state.write().await;
        warn!("Circuit breaker {} manually tripped", self.name);
        state.state = CircuitState::Open;
        state.opened_at = Some(Instant::now());
    }

    /// Get the name of the circuit breaker
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Multiple circuit breakers manager
///
/// Useful for managing circuit breakers for different services
#[derive(Clone)]
pub struct CircuitBreakerManager {
    breakers: Arc<Mutex<std::collections::HashMap<String, CircuitBreaker>>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// Create a new circuit breaker manager with default configuration
    #[must_use]
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            default_config,
        }
    }

    /// Get or create a circuit breaker for the given service
    pub async fn get_or_create(&self, service_name: &str) -> CircuitBreaker {
        let mut breakers = self.breakers.lock().await;
        
        if let Some(breaker) = breakers.get(service_name) {
            breaker.clone()
        } else {
            let breaker = CircuitBreaker::new(service_name, self.default_config.clone());
            breakers.insert(service_name.to_string(), breaker.clone());
            breaker
        }
    }

    /// Get all circuit breakers
    pub async fn all(&self) -> Vec<CircuitBreaker> {
        self.breakers.lock().await.values().cloned().collect()
    }

    /// Reset all circuit breakers
    pub async fn reset_all(&self) {
        let breakers = self.breakers.lock().await;
        for breaker in breakers.values() {
            breaker.reset().await;
        }
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_closed_success() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        
        let result = cb.call(|| async { Ok::<_, &str>("success") }).await;
        
        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, "success");
        }
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::new("test", config);
        
        // Cause failures
        for _ in 0..3 {
            let _ = cb.call(|| async { Err::<(), _>("error") }).await;
        }
        
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Next call should be rejected
        let result = cb.call(|| async { Ok::<_, &str>("success") }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen { .. })));
    }

    #[tokio::test]
    async fn test_circuit_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let cb = CircuitBreaker::new("test", config);
        
        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async { Err::<(), _>("error") }).await;
        }
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;
        
        // Should transition to half-open on next call
        let _ = cb.call(|| async { Ok::<_, &str>("success") }).await;
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
        
        // One more success should close the circuit
        let _ = cb.call(|| async { Ok::<_, &str>("success") }).await;
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_concurrent_calls_limit() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(50),
            half_open_max_calls: 2,
            ..Default::default()
        };
        let cb = Arc::new(CircuitBreaker::new("test", config));
        
        // Open the circuit
        let _ = cb.call(|| async { Err::<(), _>("error") }).await;
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;
        
        // Start multiple concurrent calls
        let call_count = Arc::new(AtomicU32::new(0));
        let mut handles = vec![];
        
        for _ in 0..5 {
            let cb_clone = Arc::clone(&cb);
            let count_clone = Arc::clone(&call_count);
            let handle = tokio::spawn(async move {
                cb_clone.call(|| async {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok::<_, &str>("success")
                }).await
            });
            handles.push(handle);
        }
        
        // Wait a bit for calls to start
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Should only allow 2 concurrent calls in half-open state
        assert!(call_count.load(Ordering::SeqCst) <= 2);
        
        // Wait for all to complete
        for handle in handles {
            let _ = handle.await;
        }
    }

    #[tokio::test]
    async fn test_manual_reset_and_trip() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        
        // Manually trip the circuit
        cb.trip().await;
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Calls should be rejected
        let result = cb.call(|| async { Ok::<_, &str>("success") }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen { .. })));
        
        // Manually reset
        cb.reset().await;
        assert_eq!(cb.state().await, CircuitState::Closed);
        
        // Calls should work again
        let result = cb.call(|| async { Ok::<_, &str>("success") }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_statistics() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new("test", config);
        
        // Success
        let _ = cb.call(|| async { Ok::<_, &str>("success") }).await;
        
        // Failures
        let _ = cb.call(|| async { Err::<(), _>("error") }).await;
        let _ = cb.call(|| async { Err::<(), _>("error") }).await;
        
        // Rejection
        let _ = cb.call(|| async { Ok::<_, &str>("success") }).await;
        
        let (total, successes, failures, rejections) = cb.stats().await;
        assert_eq!(total, 4);
        assert_eq!(successes, 1);
        assert_eq!(failures, 2);
        assert_eq!(rejections, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::default();
        
        // Get or create breakers
        let cb1 = manager.get_or_create("service1").await;
        let cb2 = manager.get_or_create("service2").await;
        let cb1_again = manager.get_or_create("service1").await;
        
        // Should get the same instance
        assert_eq!(cb1.name(), cb1_again.name());
        assert_ne!(cb1.name(), cb2.name());
        
        // Check all breakers
        let all = manager.all().await;
        assert_eq!(all.len(), 2);
    }
}