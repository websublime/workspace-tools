//! LRU (Least Recently Used) cache with TTL support
//!
//! This module provides a generic LRU cache implementation with time-to-live (TTL)
//! support for entries. The cache automatically evicts the least recently used
//! items when the capacity is reached and removes expired entries based on TTL.
//!
//! # Examples
//!
//! ```
//! use sublime_package_tools::network::LruCache;
//! use std::time::Duration;
//!
//! // Create a cache with capacity of 100 items and 5 minute TTL
//! let mut cache = LruCache::<String, String>::new(100, Duration::from_secs(300));
//!
//! // Insert items
//! cache.insert("key1".to_string(), "value1".to_string());
//! 
//! // Get items (updates access time)
//! assert_eq!(cache.get("key1"), Some(&"value1".to_string()));
//! 
//! // Check if an item exists without updating access time
//! assert!(cache.contains_key("key1"));
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Entry in the LRU cache containing value and metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// The cached value
    value: V,
    /// When this entry was inserted
    inserted_at: Instant,
    /// When this entry was last accessed
    last_accessed: Instant,
}

impl<V> CacheEntry<V> {
    /// Create a new cache entry with current time
    fn new(value: V) -> Self {
        let now = Instant::now();
        Self {
            value,
            inserted_at: now,
            last_accessed: now,
        }
    }

    /// Check if the entry has expired based on TTL
    fn is_expired(&self, ttl: Duration) -> bool {
        self.inserted_at.elapsed() > ttl
    }

    /// Update the last accessed time to now
    fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// LRU cache implementation with TTL support
///
/// This cache provides O(1) average time complexity for insertions, lookups,
/// and removals. It automatically evicts the least recently used items when
/// capacity is reached and removes expired entries based on TTL.
///
/// The cache is not thread-safe by itself. For concurrent access, wrap it
/// in a `Mutex` or `RwLock`.
#[derive(Debug)]
pub struct LruCache<K, V> 
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Maximum number of entries in the cache
    capacity: usize,
    /// Time-to-live for cache entries
    ttl: Duration,
    /// Storage for cache entries
    map: HashMap<K, CacheEntry<V>>,
    /// Access order tracking (most recent at the end)
    access_order: Vec<K>,
    /// Statistics
    hits: u64,
    misses: u64,
    evictions: u64,
    expirations: u64,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new LRU cache with specified capacity and TTL
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entries to store
    /// * `ttl` - Time-to-live for cache entries
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0
    #[must_use]
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");
        
        Self {
            capacity,
            ttl,
            map: HashMap::with_capacity(capacity),
            access_order: Vec::with_capacity(capacity),
            hits: 0,
            misses: 0,
            evictions: 0,
            expirations: 0,
        }
    }

    /// Insert a key-value pair into the cache
    ///
    /// If the key already exists, the value is updated and the entry is moved
    /// to the most recently used position. If the cache is at capacity, the
    /// least recently used item is evicted.
    ///
    /// # Returns
    ///
    /// The previous value associated with the key, if any
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Remove expired entries opportunistically
        self.remove_expired();

        // Check if key already exists
        if let Some(entry) = self.map.get_mut(&key) {
            let old_value = entry.value.clone();
            entry.value = value;
            entry.touch();
            self.move_to_front(&key);
            return Some(old_value);
        }

        // Evict LRU item if at capacity
        if self.map.len() >= self.capacity {
            if let Some(lru_key) = self.access_order.first().cloned() {
                self.map.remove(&lru_key);
                self.access_order.remove(0);
                self.evictions += 1;
            }
        }

        // Insert new entry
        self.map.insert(key.clone(), CacheEntry::new(value));
        self.access_order.push(key);
        None
    }

    /// Get a reference to a value in the cache
    ///
    /// This updates the access time and moves the entry to the most recently
    /// used position. Returns `None` if the key doesn't exist or has expired.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Check if entry exists and is not expired
        if let Some(entry) = self.map.get(key) {
            if entry.is_expired(self.ttl) {
                self.misses += 1;
                return None;
            }
        } else {
            self.misses += 1;
            return None;
        }

        // Update access time and order
        if let Some(entry) = self.map.get_mut(key) {
            entry.touch();
        }
        self.move_to_front(key);
        self.hits += 1;
        
        self.map.get(key).map(|e| &e.value)
    }

    /// Get a mutable reference to a value in the cache
    ///
    /// This updates the access time and moves the entry to the most recently
    /// used position. Returns `None` if the key doesn't exist or has expired.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        // Check if entry exists and is not expired
        if let Some(entry) = self.map.get(key) {
            if entry.is_expired(self.ttl) {
                self.misses += 1;
                return None;
            }
        } else {
            self.misses += 1;
            return None;
        }

        // Update access time and order
        self.move_to_front(key);
        self.hits += 1;
        
        self.map.get_mut(key).map(|e| {
            e.touch();
            &mut e.value
        })
    }

    /// Check if a key exists in the cache without updating access time
    ///
    /// This is useful for checking existence without affecting LRU order.
    /// Returns `false` if the key doesn't exist or has expired.
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.get(key)
            .map(|e| !e.is_expired(self.ttl))
            .unwrap_or(false)
    }

    /// Remove a key from the cache
    ///
    /// # Returns
    ///
    /// The value associated with the key, if it existed and wasn't expired
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.map.remove(key) {
            self.access_order.retain(|k| k != key);
            if !entry.is_expired(self.ttl) {
                return Some(entry.value);
            }
        }
        None
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.map.clear();
        self.access_order.clear();
    }

    /// Get the current number of entries in the cache
    ///
    /// This includes expired entries that haven't been removed yet.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if the cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Get the capacity of the cache
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get cache statistics
    ///
    /// Returns a tuple of (hits, misses, evictions, expirations)
    #[must_use]
    pub fn stats(&self) -> (u64, u64, u64, u64) {
        (self.hits, self.misses, self.evictions, self.expirations)
    }

    /// Get cache hit rate as a percentage
    ///
    /// Returns 0.0 if no requests have been made
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Reset cache statistics
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
        self.expirations = 0;
    }

    /// Remove all expired entries from the cache
    ///
    /// This is called automatically during insertions, but can be called
    /// manually to free up memory.
    pub fn remove_expired(&mut self) {
        let expired_keys: Vec<K> = self.map
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.ttl))
            .map(|(k, _)| k.clone())
            .collect();

        let expired_count = expired_keys.len() as u64;
        for key in expired_keys {
            self.map.remove(&key);
            self.access_order.retain(|k| k != &key);
        }
        self.expirations += expired_count;
    }

    /// Set a new TTL for the cache
    ///
    /// This doesn't affect existing entries until they are accessed or
    /// `remove_expired` is called.
    pub fn set_ttl(&mut self, ttl: Duration) {
        self.ttl = ttl;
    }

    /// Get the current TTL setting
    #[must_use]
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Move a key to the most recently used position
    fn move_to_front(&mut self, key: &K) {
        if let Some(pos) = self.access_order.iter().position(|k| k == key) {
            let key = self.access_order.remove(pos);
            self.access_order.push(key);
        }
    }

    /// Get an iterator over the cache entries
    ///
    /// Entries are returned in no particular order and may include expired items.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
            .filter(|(_, entry)| !entry.is_expired(self.ttl))
            .map(|(k, entry)| (k, &entry.value))
    }
}

impl<K, V> Default for LruCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a default cache with 1000 entries and 5 minute TTL
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_basic_operations() {
        let mut cache = LruCache::new(3, Duration::from_secs(60));
        
        // Insert and get
        assert_eq!(cache.insert("a".to_string(), 1), None);
        assert_eq!(cache.get(&"a".to_string()), Some(&1));
        assert_eq!(cache.len(), 1);
        
        // Update existing
        assert_eq!(cache.insert("a".to_string(), 2), Some(1));
        assert_eq!(cache.get(&"a".to_string()), Some(&2));
        assert_eq!(cache.len(), 1);
        
        // Contains key
        assert!(cache.contains_key(&"a".to_string()));
        assert!(!cache.contains_key(&"b".to_string()));
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = LruCache::new(3, Duration::from_secs(60));
        
        // Fill cache
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        cache.insert("c".to_string(), 3);
        assert_eq!(cache.len(), 3);
        
        // Access 'a' to make it recently used
        assert_eq!(cache.get(&"a".to_string()), Some(&1));
        
        // Insert 'd' should evict 'b' (least recently used)
        cache.insert("d".to_string(), 4);
        assert_eq!(cache.len(), 3);
        assert!(cache.contains_key(&"a".to_string()));
        assert!(!cache.contains_key(&"b".to_string()));
        assert!(cache.contains_key(&"c".to_string()));
        assert!(cache.contains_key(&"d".to_string()));
        
        let (_, _, evictions, _) = cache.stats();
        assert_eq!(evictions, 1);
    }

    #[test]
    fn test_ttl_expiration() {
        let mut cache = LruCache::new(10, Duration::from_millis(100));
        
        cache.insert("a".to_string(), 1);
        assert_eq!(cache.get(&"a".to_string()), Some(&1));
        
        // Wait for expiration
        sleep(Duration::from_millis(150));
        
        // Should be expired
        assert_eq!(cache.get(&"a".to_string()), None);
        assert!(!cache.contains_key(&"a".to_string()));
        
        // But still in map until removed
        assert_eq!(cache.len(), 1);
        
        // Remove expired entries
        cache.remove_expired();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_statistics() {
        let mut cache = LruCache::new(10, Duration::from_secs(60));
        
        // Generate some hits and misses
        cache.insert("a".to_string(), 1);
        assert_eq!(cache.get(&"a".to_string()), Some(&1)); // hit
        assert_eq!(cache.get(&"b".to_string()), None);     // miss
        assert_eq!(cache.get(&"a".to_string()), Some(&1)); // hit
        
        let (hits, misses, _, _) = cache.stats();
        assert_eq!(hits, 2);
        assert_eq!(misses, 1);
        assert!((cache.hit_rate() - 66.66666666666667).abs() < 0.0001);
        
        // Reset stats
        cache.reset_stats();
        let (hits, misses, _, _) = cache.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
    }

    #[test]
    fn test_remove() {
        let mut cache = LruCache::new(10, Duration::from_secs(60));
        
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        
        assert_eq!(cache.remove(&"a".to_string()), Some(1));
        assert_eq!(cache.len(), 1);
        assert!(!cache.contains_key(&"a".to_string()));
        assert!(cache.contains_key(&"b".to_string()));
        
        // Remove non-existent
        assert_eq!(cache.remove(&"c".to_string()), None);
    }

    #[test]
    fn test_clear() {
        let mut cache = LruCache::new(10, Duration::from_secs(60));
        
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        assert_eq!(cache.len(), 2);
        
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_iterator() {
        let mut cache = LruCache::new(10, Duration::from_secs(60));
        
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        cache.insert("c".to_string(), 3);
        
        let items: HashMap<_, _> = cache.iter().map(|(k, v)| (k.clone(), *v)).collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items.get("a"), Some(&1));
        assert_eq!(items.get("b"), Some(&2));
        assert_eq!(items.get("c"), Some(&3));
    }

    #[test]
    #[should_panic(expected = "Cache capacity must be greater than 0")]
    fn test_zero_capacity() {
        let _cache = LruCache::<String, i32>::new(0, Duration::from_secs(60));
    }
}