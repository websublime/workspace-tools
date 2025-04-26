//! Caching system for improved performance.
//!
//! What:
//! This module provides a flexible caching system for storing and retrieving
//! frequently accessed data with minimal overhead. It supports various
//! caching strategies and expiration policies.
//!
//! Who:
//! Used by developers who need to:
//! - Improve performance by avoiding redundant operations
//! - Cache results of expensive operations
//! - Implement time-limited data storage
//! - Reduce load on external systems
//!
//! Why:
//! Caching is essential for:
//! - Improving response times
//! - Reducing resource consumption
//! - Enhancing system scalability
//! - Minimizing external service calls

mod store;

pub use store::{Cache, CacheConfig, CacheStrategy};
