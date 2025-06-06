//! # Cache Entry Module
//!
//! This module provides caching functionality for package-related data.
//!
//! When working with package registries or other remote data sources,
//! it's important to cache results to avoid repeated network calls.
//! The `CacheEntry<T>` struct provides generic time-based caching for any data type.

use std::time::{Duration, Instant};

/// Generic cache entry with time-based expiration.
///
/// This structure wraps any type of data with a timestamp, allowing for
/// time-based cache invalidation. It's useful for caching registry data,
/// package information, or any other data that should expire after a certain time.
///
/// # Type Parameters
///
/// * `T` - The type of data being cached
///
/// # Examples
///
/// ```
/// use sublime_package_tools::CacheEntry;
/// use std::time::Duration;
///
/// // Create a cache entry with a string
/// let entry = CacheEntry::new(String::from("cached data"));
///
/// // Check if the entry is still valid (not expired)
/// let ttl = Duration::from_secs(60); // 1 minute cache TTL
/// if entry.is_valid(ttl) {
///     // Use the cached data
///     let data = entry.get();
///     println!("Cached data: {}", data);
/// } else {
///     // Data is expired, need to refresh
///     println!("Cache expired, fetching fresh data");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached data
    data: T,
    /// Timestamp when the entry was created
    timestamp: Instant,
}

impl<T: Clone> CacheEntry<T> {
    /// Creates a new cache entry with the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to cache
    ///
    /// # Returns
    ///
    /// A new cache entry containing the data and current timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::CacheEntry;
    ///
    /// // Cache some package versions
    /// let versions = vec!["1.0.0", "1.1.0", "2.0.0"];
    /// let cached_versions = CacheEntry::new(versions);
    /// ```
    pub fn new(data: T) -> Self {
        Self { data, timestamp: Instant::now() }
    }

    /// Checks if the cache entry is still valid (not expired).
    ///
    /// # Arguments
    ///
    /// * `ttl` - Time-to-live duration; the cache is valid if less time has elapsed
    ///
    /// # Returns
    ///
    /// `true` if the cache entry is still valid, `false` if it has expired.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::CacheEntry;
    /// use std::time::Duration;
    ///
    /// // Create a cache entry
    /// let entry = CacheEntry::new("data");
    ///
    /// // Check if it's valid with a 5 minute TTL
    /// let five_minutes = Duration::from_secs(5 * 60);
    /// if entry.is_valid(five_minutes) {
    ///     println!("Cache is still valid");
    /// }
    /// ```
    #[must_use]
    pub fn is_valid(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() < ttl
    }

    /// Gets a clone of the cached data.
    ///
    /// # Returns
    ///
    /// A clone of the cached data.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::CacheEntry;
    ///
    /// // Create a cache entry
    /// let entry = CacheEntry::new(vec![1, 2, 3]);
    ///
    /// // Get the cached data
    /// let data = entry.get();
    /// assert_eq!(data, vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn get(&self) -> T {
        self.data.clone()
    }
}
