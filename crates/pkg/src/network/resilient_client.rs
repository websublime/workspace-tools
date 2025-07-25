//! Resilient HTTP client with caching, retry, and circuit breaker
//!
//! This module provides a resilient HTTP client that combines LRU caching,
//! retry policies with exponential backoff, and circuit breaker patterns
//! to create a robust network layer for package registry operations.
//!
//! # Examples
//!
//! ```
//! use sublime_package_tools::network::{ResilientClient, ResilientClientConfig};
//! use std::time::Duration;
//!
//! // Create a resilient client with custom configuration
//! let config = ResilientClientConfig {
//!     cache_size: 1000,
//!     cache_ttl: Duration::from_secs(300),
//!     retry_config: Default::default(),
//!     circuit_breaker_config: Default::default(),
//!     timeout: Duration::from_secs(30),
//! };
//! let client = ResilientClient::new("npm-registry", config);
//!
//! // Make a GET request with all resilience features
//! let response = client.get("https://registry.npmjs.org/lodash").await?;
//! ```

use super::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError,
    LruCache, RetryConfig, RetryPolicy, Retryable,
};
use reqwest::{Client, Request, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Configuration for the resilient HTTP client
#[derive(Debug, Clone)]
pub struct ResilientClientConfig {
    /// Size of the LRU cache
    pub cache_size: usize,
    /// Time-to-live for cache entries
    pub cache_ttl: Duration,
    /// Retry policy configuration
    pub retry_config: RetryConfig,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Request timeout
    pub timeout: Duration,
    /// User agent string
    pub user_agent: String,
    /// Enable caching for GET requests
    pub enable_cache: bool,
    /// Enable retry logic
    pub enable_retry: bool,
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
}

impl Default for ResilientClientConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            retry_config: RetryConfig::default(),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            timeout: Duration::from_secs(30),
            user_agent: "sublime-package-tools/0.1.0".to_string(),
            enable_cache: true,
            enable_retry: true,
            enable_circuit_breaker: true,
        }
    }
}

/// Cached response data
#[derive(Debug, Clone)]
struct CachedResponse {
    status: StatusCode,
    headers: reqwest::header::HeaderMap,
    body: Vec<u8>,
}

/// Resilient HTTP client with caching, retry, and circuit breaker
pub struct ResilientClient {
    /// Service name for logging and circuit breaker
    name: String,
    /// Configuration
    config: ResilientClientConfig,
    /// HTTP client
    client: Client,
    /// Response cache (URL -> Response data)
    cache: Arc<RwLock<LruCache<String, CachedResponse>>>,
    /// Retry policy
    retry_policy: RetryPolicy,
    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
}

impl ResilientClient {
    /// Create a new resilient client with the given name and configuration
    #[must_use]
    pub fn new(name: impl Into<String>, config: ResilientClientConfig) -> Self {
        let name = name.into();
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))
            .unwrap_or_else(|_| Client::new());

        let cache = Arc::new(RwLock::new(LruCache::new(
            config.cache_size,
            config.cache_ttl,
        )));

        let retry_policy = RetryPolicy::new(config.retry_config.clone());
        let circuit_breaker = CircuitBreaker::new(&name, config.circuit_breaker_config.clone());

        Self {
            name,
            config,
            client,
            cache,
            retry_policy,
            circuit_breaker,
        }
    }

    /// Execute a GET request with caching
    ///
    /// Cached responses are returned for subsequent requests to the same URL
    /// within the cache TTL period.
    pub async fn get(&self, url: &str) -> Result<Response, ResilientClientError> {
        // Check cache first if enabled
        if self.config.enable_cache {
            if let Some(cached) = self.get_cached_response(url).await {
                debug!("Cache hit for {}", url);
                return Ok(cached);
            }
        }

        // Build request
        let request = self.client.get(url).build()?;
        
        // Execute with resilience features
        let response = self.execute_with_resilience(request).await?;
        
        // Cache successful GET responses
        if self.config.enable_cache && response.status().is_success() {
            let cached_response = self.cache_response(url, response).await?;
            Ok(cached_response)
        } else {
            Ok(response)
        }
    }

    /// Execute a GET request and deserialize JSON response
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, ResilientClientError> {
        let response = self.get(url).await?;
        
        if !response.status().is_success() {
            return Err(ResilientClientError::HttpError {
                status: response.status(),
                url: url.to_string(),
            });
        }

        response
            .json()
            .await
            .map_err(ResilientClientError::JsonError)
    }

    /// Execute a POST request
    pub async fn post(&self, url: &str) -> RequestBuilder {
        self.client.post(url)
    }

    /// Execute a PUT request
    pub async fn put(&self, url: &str) -> RequestBuilder {
        self.client.put(url)
    }

    /// Execute a DELETE request
    pub async fn delete(&self, url: &str) -> RequestBuilder {
        self.client.delete(url)
    }

    /// Execute a request with a custom request builder
    ///
    /// This method applies retry and circuit breaker logic but not caching.
    pub async fn execute(&self, request_builder: RequestBuilder) -> Result<Response, ResilientClientError> {
        let request = request_builder.build()?;
        self.execute_with_resilience(request).await
    }

    /// Execute a request with all resilience features
    async fn execute_with_resilience(&self, request: Request) -> Result<Response, ResilientClientError> {
        let url = request.url().to_string();
        
        // Circuit breaker wrapper
        let execute = || async {
            // Retry wrapper
            if self.config.enable_retry {
                self.retry_policy
                    .execute(|| async {
                        let request_clone = request.try_clone()
                            .ok_or_else(|| ResilientClientError::RequestCloneError)?;
                        
                        debug!("Executing request to {}", url);
                        self.client
                            .execute(request_clone)
                            .await
                            .map_err(ResilientClientError::from)
                    })
                    .await
            } else {
                // Single execution without retry
                debug!("Executing request to {} (no retry)", url);
                self.client
                    .execute(request)
                    .await
                    .map_err(ResilientClientError::from)
            }
        };

        // Apply circuit breaker if enabled
        if self.config.enable_circuit_breaker {
            self.circuit_breaker
                .call(execute)
                .await
                .map_err(|e| match e {
                    CircuitBreakerError::CircuitOpen { name } => {
                        ResilientClientError::CircuitBreakerOpen { service: name }
                    }
                    CircuitBreakerError::OperationFailed(e) => e,
                })
        } else {
            execute().await
        }
    }

    /// Get a cached response if available
    async fn get_cached_response(&self, url: &str) -> Option<Response> {
        let mut cache = self.cache.write().await;
        
        if let Some(cached) = cache.get(&url.to_string()) {
            // Reconstruct response from cached data
            let http_response = http::Response::builder()
                .status(cached.status)
                .body(cached.body.clone())
                .unwrap_or_else(|_| {
                    // Create a minimal fallback response if building fails
                    let mut response = http::Response::new(Vec::new());
                    *response.status_mut() = cached.status;
                    response
                });
            let response = Response::from(http_response);
            
            Some(response)
        } else {
            None
        }
    }

    /// Cache a response
    async fn cache_response(&self, url: &str, response: Response) -> Result<Response, ResilientClientError> {
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response.bytes().await?;
        
        let cached = CachedResponse {
            status,
            headers: headers.clone(),
            body: body_bytes.to_vec(),
        };

        let mut cache = self.cache.write().await;
        cache.insert(url.to_string(), cached);
        debug!("Cached response for {}", url);
        
        // Reconstruct response from cached data for return
        let http_response = http::Response::builder()
            .status(status)
            .body(body_bytes.to_vec())
            .unwrap_or_else(|_| {
                // Create a minimal fallback response if building fails
                let mut response = http::Response::new(body_bytes.to_vec());
                *response.status_mut() = status;
                response
            });
        let reconstructed_response = Response::from(http_response);
        
        Ok(reconstructed_response)
    }

    /// Clear the response cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Cleared cache for {}", self.name);
    }

    /// Get cache statistics
    ///
    /// Returns (hits, misses, evictions, expirations)
    pub async fn cache_stats(&self) -> (u64, u64, u64, u64) {
        let cache = self.cache.read().await;
        cache.stats()
    }

    /// Get circuit breaker statistics
    ///
    /// Returns (total_calls, total_successes, total_failures, total_rejections)
    pub async fn circuit_breaker_stats(&self) -> (u64, u64, u64, u64) {
        self.circuit_breaker.stats().await
    }

    /// Reset the circuit breaker
    pub async fn reset_circuit_breaker(&self) {
        self.circuit_breaker.reset().await;
    }

    /// Get the underlying HTTP client
    ///
    /// This bypasses all resilience features and should be used with caution.
    pub fn raw_client(&self) -> &Client {
        &self.client
    }
}

/// Errors that can occur in the resilient client
#[derive(Debug, thiserror::Error)]
pub enum ResilientClientError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    
    /// HTTP error response
    #[error("HTTP error {status} for URL: {url}")]
    HttpError {
        status: StatusCode,
        url: String,
    },
    
    /// JSON deserialization failed
    #[error("JSON deserialization failed: {0}")]
    JsonError(reqwest::Error),
    
    /// Circuit breaker is open
    #[error("Circuit breaker is open for service: {service}")]
    CircuitBreakerOpen {
        service: String,
    },
    
    /// Failed to clone request
    #[error("Failed to clone request for retry")]
    RequestCloneError,
}

impl Retryable for ResilientClientError {
    fn is_retryable(&self) -> bool {
        match self {
            Self::RequestError(e) => e.is_retryable(),
            Self::HttpError { status, .. } => {
                // Retry on server errors and rate limiting
                status.is_server_error() || *status == StatusCode::TOO_MANY_REQUESTS
            }
            Self::JsonError(_) => false,
            Self::CircuitBreakerOpen { .. } => false,
            Self::RequestCloneError => false,
        }
    }
}

/// Builder for creating resilient clients with custom configuration
pub struct ResilientClientBuilder {
    name: String,
    config: ResilientClientConfig,
}

impl ResilientClientBuilder {
    /// Create a new builder with the given service name
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: ResilientClientConfig::default(),
        }
    }

    /// Set the cache size
    #[must_use]
    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.cache_size = size;
        self
    }

    /// Set the cache TTL
    #[must_use]
    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.cache_ttl = ttl;
        self
    }

    /// Set the retry configuration
    #[must_use]
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.config.retry_config = config;
        self
    }

    /// Set the circuit breaker configuration
    #[must_use]
    pub fn circuit_breaker_config(mut self, config: CircuitBreakerConfig) -> Self {
        self.config.circuit_breaker_config = config;
        self
    }

    /// Set the request timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the user agent
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Enable or disable caching
    #[must_use]
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.config.enable_cache = enable;
        self
    }

    /// Enable or disable retry
    #[must_use]
    pub fn enable_retry(mut self, enable: bool) -> Self {
        self.config.enable_retry = enable;
        self
    }

    /// Enable or disable circuit breaker
    #[must_use]
    pub fn enable_circuit_breaker(mut self, enable: bool) -> Self {
        self.config.enable_circuit_breaker = enable;
        self
    }

    /// Build the resilient client
    #[must_use]
    pub fn build(self) -> ResilientClient {
        ResilientClient::new(self.name, self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder() {
        let client = ResilientClientBuilder::new("test")
            .cache_size(500)
            .cache_ttl(Duration::from_secs(600))
            .timeout(Duration::from_secs(60))
            .user_agent("test-agent/1.0")
            .enable_cache(false)
            .build();
        
        assert_eq!(client.config.cache_size, 500);
        assert_eq!(client.config.cache_ttl, Duration::from_secs(600));
        assert_eq!(client.config.timeout, Duration::from_secs(60));
        assert_eq!(client.config.user_agent, "test-agent/1.0");
        assert!(!client.config.enable_cache);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let client = ResilientClient::new("test", ResilientClientConfig::default());
        
        // Check initial stats
        let (hits, misses, evictions, expirations) = client.cache_stats().await;
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(evictions, 0);
        assert_eq!(expirations, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let client = ResilientClient::new("test", ResilientClientConfig::default());
        
        // Check initial stats
        let (calls, successes, failures, rejections) = client.circuit_breaker_stats().await;
        assert_eq!(calls, 0);
        assert_eq!(successes, 0);
        assert_eq!(failures, 0);
        assert_eq!(rejections, 0);
    }
}