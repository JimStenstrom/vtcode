//! Command permission cache
//! Caches policy evaluation results with TTL to improve performance

use crate::core::cache::TtlCache;
use std::time::Duration;
use tracing::debug;

/// Cache for command permission decisions
///
/// Migrated to use unified TtlCache for automatic expiration handling
pub struct PermissionCache {
    cache: TtlCache<String, (bool, String)>,
}

impl PermissionCache {
    /// Create cache with 5-minute default TTL
    pub fn new() -> Self {
        Self {
            cache: TtlCache::new(Duration::from_secs(300)),
        }
    }

    /// Create cache with custom TTL
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            cache: TtlCache::new(ttl),
        }
    }

    /// Check if a command is cached and not expired
    pub fn get(&mut self, command: &str) -> Option<bool> {
        self.cache.get(&command.to_string()).map(|(allowed, reason)| {
            debug!(
                command = command,
                reason = &reason,
                "Permission cache hit"
            );
            allowed
        })
    }

    /// Store a permission decision in cache
    pub fn put(&mut self, command: &str, allowed: bool, reason: &str) {
        self.cache.insert(command.to_string(), (allowed, reason.to_string()));
        debug!(
            command = command,
            allowed = allowed,
            reason = reason,
            "Cached permission decision"
        );
    }

    /// Clear expired entries
    pub fn cleanup_expired(&mut self) {
        self.cache.cleanup_expired();
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        let stats = self.cache.stats();
        // Return (total entries, expired evictions)
        (stats.entries, stats.expired_evictions)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.cache.clear();
        debug!("Permission cache cleared");
    }
}

impl Default for PermissionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_stores_decision() {
        let mut cache = PermissionCache::new();
        cache.put("cargo fmt", true, "allow_glob match");
        assert_eq!(cache.get("cargo fmt"), Some(true));
    }

    #[test]
    fn test_cache_expires() {
        let mut cache = PermissionCache::with_ttl(Duration::from_millis(100));
        cache.put("cargo fmt", true, "test");

        // Immediately available
        assert_eq!(cache.get("cargo fmt"), Some(true));

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get("cargo fmt"), None);
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = PermissionCache::with_ttl(Duration::from_millis(100));
        cache.put("cmd1", true, "test");
        cache.put("cmd2", false, "test");

        thread::sleep(Duration::from_millis(150));
        let (total, _) = cache.stats();
        assert_eq!(total, 2);

        cache.cleanup_expired();
        let (total, _) = cache.stats();
        assert_eq!(total, 0);
    }
}
