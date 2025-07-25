//! Network resilience utilities
//!
//! This module provides components for building resilient network operations:
//! - LRU caching with TTL support
//! - Retry policies with exponential backoff
//! - Circuit breaker pattern implementation
//! - Resilient HTTP client combining all features
//!
//! # Examples
//!
//! ## Using the resilient client
//!
//! ```
//! use sublime_package_tools::network::{ResilientClient, ResilientClientConfig};
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ResilientClientConfig {
//!     cache_size: 1000,
//!     cache_ttl: Duration::from_secs(300),
//!     ..Default::default()
//! };
//! 
//! let client = ResilientClient::new("npm-registry", config);
//! let response = client.get("https://registry.npmjs.org/lodash/latest").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using individual components
//!
//! ```
//! use sublime_package_tools::network::{LruCache, RetryPolicy, CircuitBreaker};
//! use std::time::Duration;
//!
//! // LRU Cache
//! let mut cache = LruCache::<String, String>::new(100, Duration::from_secs(300));
//! cache.insert("key".to_string(), "value".to_string());
//!
//! // Retry Policy
//! let retry = RetryPolicy::default();
//! # #[tokio::main]
//! # async fn main() -> Result<(), std::io::Error> {
//! let result = retry.execute(|| async {
//!     // Your fallible operation
//!     Ok::<_, std::io::Error>("success")
//! }).await;
//! # Ok(())
//! # }
//!
//! // Circuit Breaker
//! let circuit_breaker = CircuitBreaker::new("service", Default::default());
//! ```

mod circuit_breaker;
mod lru_cache;
mod resilient_client;
mod retry_policy;

pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError,
    CircuitBreakerManager, CircuitState,
};
pub use lru_cache::LruCache;
pub use resilient_client::{
    ResilientClient, ResilientClientBuilder, ResilientClientConfig,
    ResilientClientError,
};
pub use retry_policy::{
    RetryConfig, RetryPolicy, Retryable, RetryableError,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_module_exports() {
        // Test that all major types are accessible
        let _cache = LruCache::<String, String>::new(10, Duration::from_secs(60));
        let _retry = RetryPolicy::default();
        let _circuit = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        let _client = ResilientClient::new("test", ResilientClientConfig::default());
    }

    #[tokio::test]
    async fn test_integration() {
        // Test that components work together
        let config = ResilientClientConfig {
            cache_size: 10,
            cache_ttl: Duration::from_secs(60),
            retry_config: RetryConfig {
                max_retries: 2,
                initial_delay: Duration::from_millis(10),
                ..Default::default()
            },
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 3,
                ..Default::default()
            },
            ..Default::default()
        };

        let client = ResilientClient::new("integration-test", config);
        
        // The client should be usable
        assert_eq!(client.cache_stats().await.0, 0); // No hits yet
    }
}