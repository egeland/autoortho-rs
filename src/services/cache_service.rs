// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Cache service trait and implementations.
//!
//! This module provides the `CacheService` trait for abstracting tile cache
//! operations. It enables testing cache-dependent logic without disk I/O.

use crate::pipeline::cache::DdsCacheMetadata;
use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during cache operations.
#[derive(Debug, Error)]
pub enum CacheServiceError {
    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Metadata error: {0}")]
    Metadata(String),
}

/// Result type for cache service operations.
pub type CacheResult<T> = Result<T, CacheServiceError>;

/// Service trait for tile cache operations.
///
/// This trait abstracts over disk-backed and in-memory caches,
/// enabling integration tests without real I/O.
#[async_trait]
pub trait CacheService: Send + Sync {
    /// Get a cached DDS entry by key, returning data and metadata.
    /// Promotes the entry in the LRU so it won't be evicted soon.
    async fn get(&self, key: &str) -> CacheResult<(Vec<u8>, DdsCacheMetadata)>;

    /// Store a DDS entry in the cache.
    async fn put(&self, key: String, data: &[u8], metadata: &DdsCacheMetadata) -> CacheResult<()>;

    /// Check if a key exists in the cache.
    async fn has(&self, key: &str) -> bool;

    /// Remove a specific entry from the cache.
    async fn remove(&self, key: &str) -> CacheResult<()>;

    /// Clear all entries from the cache.
    async fn clear(&self) -> CacheResult<()>;

    /// Number of entries in the cache.
    async fn entry_count(&self) -> usize;

    /// Current cache size in bytes.
    async fn size_bytes(&self) -> u64;

    /// Maximum cache size in bytes.
    async fn max_size_bytes(&self) -> u64;

    /// Cache usage as a fraction (0.0–1.0).
    async fn usage_fraction(&self) -> f64;

    /// Promote a cache entry to most-recently-used without reading data.
    /// Returns true if the entry exists and was promoted.
    async fn promote(&self, key: &str) -> bool;

    /// Evict tiles NOT in the provided route_keys set.
    /// Returns number of tiles evicted.
    async fn evict_non_route_tiles(
        &self,
        route_keys: &std::collections::HashSet<String>,
        free_bytes_needed: u64,
    ) -> u32;
}

/// Production implementation backed by DdsCache.
///
/// Wraps a `parking_lot::Mutex<DdsCache>` to provide `&self` access
/// (interior mutability) matching the `CacheService` trait's `&self` methods.
#[cfg(feature = "fuse")]
pub struct DdsCacheService {
    cache: std::sync::Arc<parking_lot::Mutex<crate::pipeline::cache::DdsCache>>,
}

#[cfg(feature = "fuse")]
impl DdsCacheService {
    /// Create a new DdsCacheService wrapping an existing DdsCache.
    pub fn new(cache: crate::pipeline::cache::DdsCache) -> Self {
        Self {
            cache: std::sync::Arc::new(parking_lot::Mutex::new(cache)),
        }
    }

    /// Create from an existing Arc<Mutex<DdsCache>>.
    pub fn from_arc(
        cache: std::sync::Arc<parking_lot::Mutex<crate::pipeline::cache::DdsCache>>,
    ) -> Self {
        Self { cache }
    }
}

#[cfg(feature = "fuse")]
#[async_trait]
impl CacheService for DdsCacheService {
    async fn get(&self, key: &str) -> CacheResult<(Vec<u8>, DdsCacheMetadata)> {
        let mut cache = self.cache.lock();
        cache.get(key).map_err(|e| match e {
            crate::pipeline::cache::CacheError::KeyNotFound => {
                CacheServiceError::NotFound(key.to_string())
            }
            crate::pipeline::cache::CacheError::IoError(e) => CacheServiceError::Io(e),
            crate::pipeline::cache::CacheError::CompressionError(e) => {
                CacheServiceError::Compression(e)
            }
            crate::pipeline::cache::CacheError::MetadataError(e) => CacheServiceError::Metadata(e),
            _ => CacheServiceError::NotFound(key.to_string()),
        })
    }

    async fn put(&self, key: String, data: &[u8], metadata: &DdsCacheMetadata) -> CacheResult<()> {
        let mut cache = self.cache.lock();
        cache.put(key, data, metadata).map_err(|e| match e {
            crate::pipeline::cache::CacheError::IoError(e) => CacheServiceError::Io(e),
            crate::pipeline::cache::CacheError::CompressionError(e) => {
                CacheServiceError::Compression(e)
            }
            crate::pipeline::cache::CacheError::MetadataError(e) => CacheServiceError::Metadata(e),
            _ => CacheServiceError::NotFound("put failed".to_string()),
        })
    }

    async fn has(&self, key: &str) -> bool {
        let cache = self.cache.lock();
        cache.contains(key)
    }

    async fn remove(&self, key: &str) -> CacheResult<()> {
        let mut cache = self.cache.lock();
        cache.remove(key).map_err(|e| match e {
            crate::pipeline::cache::CacheError::IoError(e) => CacheServiceError::Io(e),
            _ => CacheServiceError::NotFound(key.to_string()),
        })
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut cache = self.cache.lock();
        cache.clear().map_err(|e| match e {
            crate::pipeline::cache::CacheError::IoError(e) => CacheServiceError::Io(e),
            _ => CacheServiceError::NotFound("clear failed".to_string()),
        })
    }

    async fn entry_count(&self) -> usize {
        let cache = self.cache.lock();
        cache.entry_count()
    }

    async fn size_bytes(&self) -> u64 {
        let cache = self.cache.lock();
        cache.size_bytes()
    }

    async fn max_size_bytes(&self) -> u64 {
        let cache = self.cache.lock();
        cache.max_size_bytes()
    }

    async fn usage_fraction(&self) -> f64 {
        let cache = self.cache.lock();
        cache.usage_fraction()
    }

    async fn promote(&self, key: &str) -> bool {
        let mut cache = self.cache.lock();
        cache.promote(key)
    }

    async fn evict_non_route_tiles(
        &self,
        route_keys: &std::collections::HashSet<String>,
        free_bytes_needed: u64,
    ) -> u32 {
        let mut cache = self.cache.lock();
        cache.evict_non_route_tiles(route_keys, free_bytes_needed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Fake cache service for testing without disk I/O.
    #[derive(Debug, Clone)]
    pub struct FakeCacheService {
        entries: Arc<Mutex<HashMap<String, (Vec<u8>, DdsCacheMetadata)>>>,
        max_size: u64,
    }

    impl FakeCacheService {
        pub fn new(max_size: u64) -> Self {
            Self {
                entries: Arc::new(Mutex::new(HashMap::new())),
                max_size,
            }
        }
    }

    #[async_trait]
    impl CacheService for FakeCacheService {
        async fn get(&self, key: &str) -> CacheResult<(Vec<u8>, DdsCacheMetadata)> {
            let entries = self.entries.lock().await;
            entries
                .get(key)
                .cloned()
                .ok_or_else(|| CacheServiceError::NotFound(key.to_string()))
        }

        async fn put(
            &self,
            key: String,
            data: &[u8],
            metadata: &DdsCacheMetadata,
        ) -> CacheResult<()> {
            let mut entries = self.entries.lock().await;
            entries.insert(key, (data.to_vec(), metadata.clone()));
            Ok(())
        }

        async fn has(&self, key: &str) -> bool {
            let entries = self.entries.lock().await;
            entries.contains_key(key)
        }

        async fn remove(&self, key: &str) -> CacheResult<()> {
            let mut entries = self.entries.lock().await;
            entries.remove(key);
            Ok(())
        }

        async fn clear(&self) -> CacheResult<()> {
            let mut entries = self.entries.lock().await;
            entries.clear();
            Ok(())
        }

        async fn entry_count(&self) -> usize {
            let entries = self.entries.lock().await;
            entries.len()
        }

        async fn size_bytes(&self) -> u64 {
            // Approximate: sum of data sizes
            let entries = self.entries.lock().await;
            entries.values().map(|(data, _)| data.len() as u64).sum()
        }

        async fn max_size_bytes(&self) -> u64 {
            self.max_size
        }

        async fn usage_fraction(&self) -> f64 {
            let size = self.size_bytes().await;
            if self.max_size == 0 {
                1.0
            } else {
                size as f64 / self.max_size as f64
            }
        }

        async fn promote(&self, _key: &str) -> bool {
            // Fake doesn't track LRU, just return true if exists
            let entries = self.entries.lock().await;
            entries.contains_key(_key)
        }

        async fn evict_non_route_tiles(
            &self,
            route_keys: &std::collections::HashSet<String>,
            _free_bytes_needed: u64,
        ) -> u32 {
            let mut entries = self.entries.lock().await;
            let keys_to_evict: Vec<String> = entries
                .keys()
                .filter(|k| !route_keys.contains(*k))
                .cloned()
                .collect();
            let count = keys_to_evict.len() as u32;
            for key in keys_to_evict {
                entries.remove(&key);
            }
            count
        }
    }

    fn sample_metadata() -> DdsCacheMetadata {
        DdsCacheMetadata {
            v: 3,
            w: 4096,
            h: 4096,
            mm: 13,
            zl: 16,
            max_zl: 16,
            fmt: "BC1".to_string(),
            map: "BI".to_string(),
            built: 1700000000.0,
            tile_row: 12345,
            tile_col: 54321,
            populated_mipmaps: vec![0, 1, 2, 3, 4],
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        }
    }

    #[tokio::test]
    async fn test_fake_cache_put_get_roundtrip() {
        let cache = FakeCacheService::new(1024 * 1024);
        let meta = sample_metadata();
        let data = b"test DDS data";

        cache.put("key1".to_string(), data, &meta).await.unwrap();

        let (retrieved, retrieved_meta) = cache.get("key1").await.unwrap();
        assert_eq!(retrieved, data);
        assert_eq!(retrieved_meta.fmt, "BC1");
    }

    #[tokio::test]
    async fn test_fake_cache_has() {
        let cache = FakeCacheService::new(1024 * 1024);
        let meta = sample_metadata();

        assert!(!cache.has("key1").await);

        cache.put("key1".to_string(), b"data", &meta).await.unwrap();

        assert!(cache.has("key1").await);
        assert!(!cache.has("key2").await);
    }

    #[tokio::test]
    async fn test_fake_cache_remove() {
        let cache = FakeCacheService::new(1024 * 1024);
        let meta = sample_metadata();

        cache.put("key1".to_string(), b"data", &meta).await.unwrap();
        assert!(cache.has("key1").await);

        cache.remove("key1").await.unwrap();
        assert!(!cache.has("key1").await);
    }

    #[tokio::test]
    async fn test_fake_cache_clear() {
        let cache = FakeCacheService::new(1024 * 1024);
        let meta = sample_metadata();

        cache
            .put("key1".to_string(), b"data1", &meta)
            .await
            .unwrap();
        cache
            .put("key2".to_string(), b"data2", &meta)
            .await
            .unwrap();
        assert_eq!(cache.entry_count().await, 2);

        cache.clear().await.unwrap();
        assert_eq!(cache.entry_count().await, 0);
    }

    #[tokio::test]
    async fn test_fake_cache_entry_count_and_size() {
        let cache = FakeCacheService::new(1024 * 1024);
        let meta = sample_metadata();

        assert_eq!(cache.entry_count().await, 0);
        assert_eq!(cache.size_bytes().await, 0);

        cache
            .put("key1".to_string(), b"data1", &meta)
            .await
            .unwrap();
        cache
            .put("key2".to_string(), b"data2", &meta)
            .await
            .unwrap();

        assert_eq!(cache.entry_count().await, 2);
        assert!(cache.size_bytes().await > 0);
    }

    #[tokio::test]
    async fn test_fake_cache_usage_fraction() {
        let cache = FakeCacheService::new(100);
        let meta = sample_metadata();

        assert!((cache.usage_fraction().await - 0.0).abs() < f64::EPSILON);

        cache.put("key1".to_string(), b"data", &meta).await.unwrap();

        let frac = cache.usage_fraction().await;
        assert!(frac > 0.0);
    }

    #[tokio::test]
    async fn test_fake_cache_evict_non_route_tiles() {
        let cache = FakeCacheService::new(1024 * 1024);
        let meta = sample_metadata();

        cache
            .put("route_1".to_string(), b"data", &meta)
            .await
            .unwrap();
        cache
            .put("route_2".to_string(), b"data", &meta)
            .await
            .unwrap();
        cache
            .put("other_1".to_string(), b"data", &meta)
            .await
            .unwrap();

        let mut route_keys = std::collections::HashSet::new();
        route_keys.insert("route_1".to_string());
        route_keys.insert("route_2".to_string());

        let evicted = cache.evict_non_route_tiles(&route_keys, 0).await;
        assert_eq!(evicted, 1);
        assert!(cache.has("route_1").await);
        assert!(cache.has("route_2").await);
        assert!(!cache.has("other_1").await);
    }

    /// Test that CacheService works as a trait object.
    #[tokio::test]
    async fn test_cache_service_trait_object() {
        let fake = FakeCacheService::new(1024 * 1024);
        let service: Box<dyn CacheService> = Box::new(fake);
        let meta = sample_metadata();

        service
            .put("key1".to_string(), b"data", &meta)
            .await
            .unwrap();
        assert!(service.has("key1").await);
    }

    /// Test that we can swap implementations without changing client code.
    #[tokio::test]
    async fn test_cache_service_impl_swap() {
        async fn count_entries<C: CacheService>(cache: &C) -> usize {
            cache.entry_count().await
        }

        let fake = FakeCacheService::new(1024 * 1024);
        assert_eq!(count_entries(&fake).await, 0);
    }
}
