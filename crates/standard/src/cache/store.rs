use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Entry in the cache with expiration handling.
///
/// Represents a value stored in the cache with metadata for expiration tracking.
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// The cached value
    value: T,
    /// When this entry was created or last accessed
    timestamp: Instant,
    /// Time-to-live for this entry
    ttl: Duration,
}

impl<T: Clone> CacheEntry<T> {
    /// Creates a new cache entry.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to cache
    /// * `ttl` - Time-to-live for this entry
    ///
    /// # Returns
    ///
    /// A new cache entry with the current timestamp
    fn new(value: T, ttl: Duration) -> Self {
        Self { value, timestamp: Instant::now(), ttl }
    }

    /// Returns true if this entry has expired.
    ///
    /// # Returns
    ///
    /// `true` if the entry has expired, `false` otherwise
    fn is_expired(&self) -> bool {
        let elapsed = Instant::now().duration_since(self.timestamp);
        elapsed >= self.ttl
    }

    /// Updates the timestamp for this entry.
    ///
    /// Resets the expiration time by updating the timestamp to the current time.
    fn touch(&mut self) {
        self.timestamp = Instant::now();
    }
}

/// Cache strategy for cache invalidation.
///
/// Defines how entries are removed from the cache when it reaches capacity.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::cache::CacheStrategy;
///
/// // Expiry strategy removes entries when they expire
/// let expiry_strategy = CacheStrategy::Expiry;
///
/// // LRU strategy removes the least recently used entries
/// let lru_strategy = CacheStrategy::LRU;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStrategy {
    /// Invalidate entries when they expire
    Expiry,
    /// Invalidate entries when capacity is reached (least recently used)
    LRU,
}

/// Configuration for a cache.
///
/// Defines the behavior and constraints for a cache instance.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::cache::{CacheConfig, CacheStrategy};
/// use std::time::Duration;
///
/// let config = CacheConfig {
///     default_ttl: Duration::from_secs(60),  // 1 minute
///     capacity: 100,                         // Store up to 100 items
///     strategy: CacheStrategy::LRU,          // Remove least recently used items when full
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default time-to-live for entries
    pub default_ttl: Duration,
    /// Maximum number of entries
    pub capacity: usize,
    /// Cache invalidation strategy
    pub strategy: CacheStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(300), // 5 minutes
            capacity: 1000,
            strategy: CacheStrategy::LRU,
        }
    }
}

/// A thread-safe cache implementation with expiration.
///
/// Provides a generic key-value store with configurable expiration policies
/// and thread-safe access.
///
/// # Type Parameters
///
/// * `K` - Key type that must implement `Eq + Hash + Clone`
/// * `V` - Value type that must implement `Clone`
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::cache::{Cache, CacheConfig, CacheStrategy};
/// use std::time::Duration;
///
/// // Create a cache with default settings
/// let cache = Cache::<String, String>::new();
///
/// // Store a value
/// cache.put("key1".to_string(), "value1".to_string());
///
/// // Retrieve a value
/// match cache.get(&"key1".to_string()) {
///     Some(value) => println!("Found: {}", value),
///     None => println!("Value not found or expired"),
/// }
///
/// // Create a cache with custom configuration
/// let config = CacheConfig {
///     default_ttl: Duration::from_secs(30),
///     capacity: 100,
///     strategy: CacheStrategy::Expiry,
/// };
/// let custom_cache = Cache::<u32, String>::with_config(config);
/// ```
#[derive(Debug)]
pub struct Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Map of cache entries
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Cache configuration
    config: CacheConfig,
    /// Counter for cache hits
    hits: Arc<Mutex<usize>>,
    /// Counter for cache misses
    misses: Arc<Mutex<usize>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Creates a new cache with default configuration.
    ///
    /// Uses the default configuration with 5-minute TTL, 1000 capacity,
    /// and LRU eviction strategy.
    ///
    /// # Returns
    ///
    /// A new cache instance with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<String, String>::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config: CacheConfig::default(),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a new cache with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom cache configuration
    ///
    /// # Returns
    ///
    /// A new cache instance with the specified configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::{Cache, CacheConfig, CacheStrategy};
    /// use std::time::Duration;
    ///
    /// let config = CacheConfig {
    ///     default_ttl: Duration::from_secs(60),
    ///     capacity: 500,
    ///     strategy: CacheStrategy::LRU,
    /// };
    ///
    /// let cache = Cache::<String, Vec<u8>>::with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Gets a value from the cache, returning None if not found or expired.
    ///
    /// Automatically updates the hit/miss statistics and removes expired entries.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// The cached value if found and not expired, or `None` otherwise
    pub fn get(&self, key: &K) -> Option<V> {
        let entries_lock_result = self.entries.write();

        if let Err(e) = &entries_lock_result {
            log::error!("Failed to acquire write lock on cache: {}", e);
            return None;
        }

        // Use match to avoid unwrap/expect - we've already checked for errors
        let Ok(mut entries) = entries_lock_result else { return None };

        // First check if the key exists
        if let Some(entry) = entries.get_mut(key) {
            // Then check if it's expired
            if entry.is_expired() {
                // Remove expired entry
                entries.remove(key);

                if let Err(e) = self.increment_misses() {
                    log::error!("Failed to increment cache misses counter: {}", e);
                }

                None
            } else {
                // Update timestamp for LRU and return value
                entry.touch();

                if let Err(e) = self.increment_hits() {
                    log::error!("Failed to increment cache hits counter: {}", e);
                }

                Some(entry.value.clone())
            }
        } else {
            if let Err(e) = self.increment_misses() {
                log::error!("Failed to increment cache misses counter: {}", e);
            }

            None
        }
    }

    /// Helper method to increment the hits counter.
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error message if the lock couldn't be acquired
    fn increment_hits(&self) -> Result<(), String> {
        let mut hits = self.hits.lock().map_err(|e| format!("Failed to lock hits counter: {e}"))?;
        *hits += 1;
        Ok(())
    }

    /// Helper method to increment the misses counter.
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error message if the lock couldn't be acquired
    fn increment_misses(&self) -> Result<(), String> {
        let mut misses =
            self.misses.lock().map_err(|e| format!("Failed to lock misses counter: {e}"))?;
        *misses += 1;
        Ok(())
    }

    /// Puts a value in the cache with the default TTL.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to store the value under
    /// * `value` - The value to store
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<String, u32>::new();
    /// cache.put("counter".to_string(), 42);
    /// ```
    pub fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, self.config.default_ttl);
    }

    /// Puts a value in the cache with a custom TTL.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to store the value under
    /// * `value` - The value to store
    /// * `ttl` - Time-to-live for this entry
    pub fn put_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let entries_lock_result = self.entries.write();

        if let Err(e) = &entries_lock_result {
            log::error!("Failed to acquire write lock on cache: {}", e);
            return;
        }

        // Use match to avoid unwrap/expect - we've already checked for errors
        let Ok(mut entries) = entries_lock_result else { return };

        // Check capacity before insertion
        if entries.len() >= self.config.capacity && !entries.contains_key(&key) {
            // If we're at capacity and using LRU, remove least recently used entry
            if self.config.strategy == CacheStrategy::LRU {
                // Find the oldest entry
                let oldest_key =
                    entries.iter().min_by_key(|(_, entry)| entry.timestamp).map(|(k, _)| k.clone());

                if let Some(oldest_key) = oldest_key {
                    entries.remove(&oldest_key);
                }
            }
        }

        // Insert or update the entry
        entries.insert(key, CacheEntry::new(value, ttl));
    }

    /// Removes a value from the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove
    ///
    /// # Returns
    ///
    /// `true` if the key was found and removed, `false` otherwise
    pub fn remove(&self, key: &K) -> bool {
        let entries_lock_result = self.entries.write();

        if let Err(e) = &entries_lock_result {
            log::error!("Failed to acquire write lock on cache: {}", e);
            return false;
        }

        // Use match to avoid unwrap/expect - we've already checked for errors
        let Ok(mut entries) = entries_lock_result else { return false };

        entries.remove(key).is_some()
    }

    /// Clears all entries from the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<String, String>::new();
    /// cache.put("key1".to_string(), "value1".to_string());
    /// cache.put("key2".to_string(), "value2".to_string());
    ///
    /// assert_eq!(cache.len(), 2);
    ///
    /// cache.clear();
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn clear(&self) {
        let entries_lock_result = self.entries.write();

        if let Err(e) = &entries_lock_result {
            log::error!("Failed to acquire write lock on cache: {}", e);
            return;
        }

        // Use match to avoid unwrap/expect - we've already checked for errors
        let Ok(mut entries) = entries_lock_result else { return };

        entries.clear();
    }

    /// Gets the number of entries in the cache.
    ///
    /// # Returns
    ///
    /// The number of entries currently in the cache
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<String, String>::new();
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.put("key1".to_string(), "value1".to_string());
    /// assert_eq!(cache.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        match self.entries.read() {
            Ok(entries) => entries.len(),
            Err(e) => {
                log::error!("Failed to acquire read lock on cache: {}", e);
                0 // Return 0 as fallback
            }
        }
    }

    /// Returns true if the cache is empty.
    ///
    /// # Returns
    ///
    /// `true` if the cache contains no entries, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<u32, String>::new();
    /// assert!(cache.is_empty());
    ///
    /// cache.put(1, "value".to_string());
    /// assert!(!cache.is_empty());
    ///
    /// cache.clear();
    /// assert!(cache.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the number of cache hits.
    ///
    /// # Returns
    ///
    /// The number of successful cache retrievals
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<String, String>::new();
    /// cache.put("key".to_string(), "value".to_string());
    ///
    /// // Initial hits count is 0
    /// assert_eq!(cache.hits(), 0);
    ///
    /// // This should increment the hits counter
    /// cache.get(&"key".to_string());
    ///
    /// // Hits count should now be 1
    /// assert_eq!(cache.hits(), 1);
    /// ```
    pub fn hits(&self) -> usize {
        match self.hits.lock() {
            Ok(hits) => *hits,
            Err(e) => {
                log::error!("Failed to lock hits counter: {}", e);
                0 // Return 0 as fallback
            }
        }
    }

    /// Gets the number of cache misses.
    ///
    /// # Returns
    ///
    /// The number of unsuccessful cache retrievals
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<String, String>::new();
    ///
    /// // Initial misses count is 0
    /// assert_eq!(cache.misses(), 0);
    ///
    /// // This should increment the misses counter
    /// cache.get(&"nonexistent".to_string());
    ///
    /// // Misses count should now be 1
    /// assert_eq!(cache.misses(), 1);
    /// ```
    pub fn misses(&self) -> usize {
        match self.misses.lock() {
            Ok(misses) => *misses,
            Err(e) => {
                log::error!("Failed to lock misses counter: {}", e);
                0 // Return 0 as fallback
            }
        }
    }

    /// Gets the hit rate (hits / (hits + misses)).
    ///
    /// # Returns
    ///
    /// The cache hit rate as a value between 0.0 and 1.0
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    ///
    /// let cache = Cache::<String, String>::new();
    /// cache.put("key".to_string(), "value".to_string());
    ///
    /// // Hit the cache
    /// cache.get(&"key".to_string());
    ///
    /// // Miss the cache
    /// cache.get(&"nonexistent".to_string());
    ///
    /// // Hit rate should be 0.5 (1 hit out of 2 total)
    /// assert_eq!(cache.hit_rate(), 0.5);
    /// ```
    #[allow(clippy::cast_precision_loss)]
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits();
        let misses = self.misses();
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Removes expired entries from the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::cache::Cache;
    /// use std::time::Duration;
    ///
    /// let cache = Cache::<String, String>::new();
    ///
    /// // Add an entry with a short TTL
    /// cache.put_with_ttl(
    ///     "expires-soon".to_string(),
    ///     "temporary".to_string(),
    ///     Duration::from_millis(1)
    /// );
    ///
    /// // Wait for the entry to expire
    /// std::thread::sleep(Duration::from_millis(10));
    ///
    /// // Clean should remove the expired entry
    /// cache.clean();
    /// assert!(cache.get(&"expires-soon".to_string()).is_none());
    /// ```
    pub fn clean(&self) {
        let entries_lock_result = self.entries.write();

        if let Err(e) = &entries_lock_result {
            log::error!("Failed to acquire write lock on cache: {}", e);
            return;
        }

        // Use match to avoid unwrap/expect - we've already checked for errors
        let Ok(mut entries) = entries_lock_result else { return };

        // Use a copy of the keys to avoid borrowing issues
        let expired_keys: Vec<K> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        // Remove expired entries
        for key in expired_keys {
            entries.remove(&key);
        }
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic_operations() {
        let cache = Cache::<String, String>::new();

        // Put and get
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Non-existent key
        assert_eq!(cache.get(&"key2".to_string()), None);

        // Remove
        assert!(cache.remove(&"key1".to_string()));
        assert!(!cache.remove(&"key1".to_string())); // Already removed
        assert_eq!(cache.get(&"key1".to_string()), None);

        // Clear
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_cache_expiry_behavior() {
        // Use a deterministic test that doesn't rely on actual thread::sleep
        let config = CacheConfig {
            default_ttl: Duration::from_secs(10),
            capacity: 10,
            strategy: CacheStrategy::Expiry,
        };

        let cache = Cache::<String, String>::with_config(config);

        // Put some values in the cache
        cache.put_with_ttl("key1".to_string(), "value1".to_string(), Duration::from_secs(20));
        cache.put("key2".to_string(), "value2".to_string()); // Default TTL (10s)

        // Both should be available immediately
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));

        // Force clean to ensure correct cleanup behavior
        cache.clean();

        // Items should still be available after cleaning
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));

        // Manually simulate behavior for key2 expiration
        // We'll manually remove and verify the behavior
        let mut entries = cache.entries.write().expect("Failed to get write lock");
        entries.remove(&"key2".to_string());
        drop(entries); // Release the lock

        assert_eq!(cache.get(&"key2".to_string()), None);
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_cache_manual_expiry() {
        let cache = Cache::<String, String>::new();

        // Add an entry
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Manually expire the entry by removing it
        let mut entries = cache.entries.write().expect("Failed to get write lock");
        entries.remove(&"key1".to_string());
        drop(entries);

        // The entry should be gone
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_ttl_expiry() {
        let config = CacheConfig {
            default_ttl: Duration::from_millis(50),
            capacity: 10,
            strategy: CacheStrategy::Expiry,
        };

        let cache = Cache::<String, String>::with_config(config);

        // Add entry
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Wait for expiry
        thread::sleep(Duration::from_millis(100));

        // Entry should be gone on next access (cleaned during get)
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[allow(clippy::unchecked_duration_subtraction)]
    #[allow(clippy::expect_used)]
    #[test]
    fn test_cache_ttl_without_timing() {
        // Create a cache with a default TTL
        let config = CacheConfig {
            default_ttl: Duration::from_secs(60),
            capacity: 10,
            strategy: CacheStrategy::Expiry,
        };

        let cache = Cache::<String, String>::with_config(config);

        // Add entries with different TTLs
        cache.put("default_ttl".to_string(), "value1".to_string());
        cache.put_with_ttl(
            "custom_ttl".to_string(),
            "value2".to_string(),
            Duration::from_secs(120),
        );

        // Verify both entries exist
        assert_eq!(cache.get(&"default_ttl".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"custom_ttl".to_string()), Some("value2".to_string()));

        // Now directly manipulate the cache entries to simulate expiration
        {
            let mut entries = cache.entries.write().expect("Failed to get write lock");

            // Get the default TTL entry and manually set its timestamp to the past
            if let Some(entry) = entries.get_mut(&"default_ttl".to_string()) {
                // Set timestamp to way in the past to ensure it's expired
                entry.timestamp = Instant::now() - Duration::from_secs(3600);
            }

            // The custom TTL entry should still be valid
        }

        // After manually expiring the default TTL entry, it should be removed on next access
        assert_eq!(cache.get(&"default_ttl".to_string()), None);
        assert_eq!(cache.get(&"custom_ttl".to_string()), Some("value2".to_string()));

        // Now expire the custom TTL entry
        {
            let mut entries = cache.entries.write().expect("Failed to get write lock");

            if let Some(entry) = entries.get_mut(&"custom_ttl".to_string()) {
                // Set timestamp to way in the past
                entry.timestamp = Instant::now() - Duration::from_secs(3600);
            }
        }

        // Both entries should now be expired
        assert_eq!(cache.get(&"default_ttl".to_string()), None);
        assert_eq!(cache.get(&"custom_ttl".to_string()), None);
    }

    #[test]
    fn test_cache_custom_ttl() {
        // Use even longer TTLs to avoid timing issues
        let config = CacheConfig {
            default_ttl: Duration::from_millis(100),
            capacity: 10,
            strategy: CacheStrategy::Expiry,
        };

        let cache = Cache::<String, String>::with_config(config);

        // Add entry with much longer TTL
        cache.put_with_ttl(
            "long_ttl".to_string(),
            "value".to_string(),
            Duration::from_millis(1000), // 10x the default TTL
        );

        // Add entry with default (short) TTL
        cache.put("short_ttl".to_string(), "value".to_string());

        // Ensure entries are initially present
        assert_eq!(cache.get(&"short_ttl".to_string()), Some("value".to_string()));
        assert_eq!(cache.get(&"long_ttl".to_string()), Some("value".to_string()));

        // Wait for default TTL to expire with some buffer
        thread::sleep(Duration::from_millis(200)); // 2x the default TTL

        // Instead of checking with get(), which updates the timestamp,
        // we'll use clean() to remove expired entries and then check len()
        cache.clean();

        // Verify the short TTL entry is gone (via len check)
        // We should have 1 entry left (the long TTL one)
        assert_eq!(cache.len(), 1, "Short TTL entry should have been removed");

        // Verify the remaining entry is the long TTL one
        assert_eq!(cache.get(&"long_ttl".to_string()), Some("value".to_string()));

        // Wait for long TTL to expire with buffer
        thread::sleep(Duration::from_millis(1000)); // Full long TTL duration

        // Clean the cache again and verify all entries are gone
        cache.clean();
        assert_eq!(cache.len(), 0, "All entries should have expired");
    }

    #[test]
    fn test_cache_lru_strategy() {
        let config = CacheConfig {
            default_ttl: Duration::from_secs(10), // Long TTL
            capacity: 2,                          // Very small capacity
            strategy: CacheStrategy::LRU,
        };

        let cache = Cache::<String, String>::with_config(config);

        // Fill the cache
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());

        // Access key1 to make it more recently used
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Add a third item, should evict key2 (least recently used)
        cache.put("key3".to_string(), "value3".to_string());

        // Key1 and Key3 should be present, Key2 evicted
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
        assert_eq!(cache.get(&"key3".to_string()), Some("value3".to_string()));
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn test_cache_hit_rate() {
        let cache = Cache::<String, String>::new();

        // No hits or misses yet
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
        assert_eq!(cache.hit_rate(), 0.0);

        // Add an entry
        cache.put("key1".to_string(), "value1".to_string());

        // Get existing (hit)
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Get non-existent (miss)
        assert_eq!(cache.get(&"key2".to_string()), None);

        // Check counters
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);
        assert_eq!(cache.hit_rate(), 0.5); // 1 hit, 1 miss = 50%
    }
}
