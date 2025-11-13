//! Unified caching system for VTCode
//!
//! Provides generic cache implementations with:
//! - TTL (Time-To-Live) expiration
//! - Size-based eviction
//! - Hit/miss statistics
//! - Thread-safe operations
//!
//! # Example
//!
//! ```rust,ignore
//! use vtcode_core::core::cache::{Cache, TtlCache};
//! use std::time::Duration;
//!
//! let mut cache = TtlCache::new(Duration::from_secs(300));
//! cache.insert("key", "value");
//! assert_eq!(cache.get("key"), Some(&"value"));
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Generic cache trait for all cache implementations
pub trait Cache<K, V> {
    /// Retrieve a value from the cache
    fn get(&self, key: &K) -> Option<&V>;

    /// Insert a value into the cache
    fn insert(&mut self, key: K, value: V);

    /// Remove a value from the cache
    fn remove(&mut self, key: &K) -> Option<V>;

    /// Clear all entries from the cache
    fn clear(&mut self);

    /// Get the number of entries in the cache
    fn len(&self) -> usize;

    /// Check if the cache is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Cache entry with expiration tracking
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    timestamp: Instant,
    size_bytes: usize,
}

impl<V> CacheEntry<V> {
    fn new(value: V, size_bytes: usize) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
            size_bytes,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() >= ttl
    }
}

/// Cache with Time-To-Live (TTL) expiration
///
/// Entries are automatically evicted after the TTL expires.
pub struct TtlCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    ttl: Duration,
    stats: CacheStats,
}

impl<K: Hash + Eq + Clone, V: Clone> TtlCache<K, V> {
    /// Create a new TTL cache with the specified expiration duration
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            stats: CacheStats::default(),
        }
    }

    /// Create a cache with custom capacity and TTL
    pub fn with_capacity(capacity: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            ttl,
            stats: CacheStats::default(),
        }
    }

    /// Get a value from the cache, returning None if expired
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired(self.ttl) {
                // Entry expired, remove it
                self.entries.remove(key);
                self.stats.expired_evictions += 1;
                self.stats.misses += 1;
                None
            } else {
                self.stats.hits += 1;
                Some(entry.value.clone())
            }
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert(&mut self, key: K, value: V) {
        self.insert_with_size(key, value, 0)
    }

    /// Insert a value with size tracking
    pub fn insert_with_size(&mut self, key: K, value: V, size_bytes: usize) {
        let entry = CacheEntry::new(value, size_bytes);
        self.entries.insert(key, entry);
        self.stats.entries = self.entries.len();
        self.stats.total_size_bytes += size_bytes;
    }

    /// Remove a value from the cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|entry| {
            self.stats.total_size_bytes = self
                .stats
                .total_size_bytes
                .saturating_sub(entry.size_bytes);
            self.stats.entries = self.entries.len();
            entry.value
        })
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats = CacheStats::default();
    }

    /// Remove expired entries
    pub fn cleanup_expired(&mut self) {
        let ttl = self.ttl;
        let initial_count = self.entries.len();

        self.entries.retain(|_, entry| !entry.is_expired(ttl));

        let removed = initial_count - self.entries.len();
        self.stats.expired_evictions += removed;
        self.stats.entries = self.entries.len();
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the TTL duration
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

/// Cache with size-based eviction (LRU-style using quick_cache)
///
/// This cache uses the quick_cache library for high-performance caching
/// with automatic LRU eviction.
pub struct SizedCache<K, V> {
    cache: quick_cache::sync::Cache<K, CacheEntry<V>>,
    max_size_bytes: usize,
    stats: std::sync::Arc<std::sync::Mutex<CacheStats>>,
}

impl<K: Hash + Eq + Clone, V: Clone> SizedCache<K, V> {
    /// Create a new sized cache
    pub fn new(capacity: usize, max_size_bytes: usize) -> Self {
        Self {
            cache: quick_cache::sync::Cache::new(capacity),
            max_size_bytes,
            stats: std::sync::Arc::new(std::sync::Mutex::new(CacheStats::default())),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        let mut stats = self.stats.lock().unwrap();

        if let Some(entry) = self.cache.get(key) {
            stats.hits += 1;
            Some(entry.value.clone())
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V, size_bytes: usize) {
        let entry = CacheEntry::new(value, size_bytes);

        let mut stats = self.stats.lock().unwrap();

        // Check memory limits
        if stats.total_size_bytes + size_bytes > self.max_size_bytes {
            stats.memory_evictions += 1;
        }

        self.cache.insert(key, entry);
        stats.entries = self.cache.len();
        stats.total_size_bytes += size_bytes;
    }

    /// Remove a value from the cache
    pub fn remove(&self, key: &K) {
        self.cache.remove(key);
        let mut stats = self.stats.lock().unwrap();
        stats.entries = self.cache.len();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.len() == 0
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub entries: usize,
    pub total_size_bytes: usize,
    pub expired_evictions: usize,
    pub memory_evictions: usize,
}

impl CacheStats {
    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Total number of accesses (hits + misses)
    pub fn total_accesses(&self) -> usize {
        self.hits + self.misses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_cache_basic_operations() {
        let mut cache = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1".to_string()));
        assert_eq!(cache.len(), 1);

        cache.remove(&"key1");
        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_ttl_cache_expiration() {
        let mut cache = TtlCache::new(Duration::from_millis(100));

        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1".to_string()));

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.stats().expired_evictions, 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1", "value1");
        cache.get(&"key1");
        cache.get(&"key1");
        cache.get(&"nonexistent");

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 200.0 / 3.0);
    }

    #[test]
    fn test_cleanup_expired() {
        let mut cache = TtlCache::new(Duration::from_millis(50));

        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        std::thread::sleep(Duration::from_millis(100));

        cache.cleanup_expired();
        assert_eq!(cache.len(), 0);
    }
}
