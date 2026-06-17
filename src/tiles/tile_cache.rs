// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Tile cache: memory LRU + disk cache with upserving.
//!
//! `TileCache` encapsulates the two-layer cache used by `DdsFileSystem`:
//! - **Memory cache**: LRU of generated DDS tiles (fast, bounded)
//! - **Disk cache**: zstd-compressed DDS files (persistent, larger)
//!
//! The cache also handles *upserving*: if a tile isn't cached at the
//! requested zoom, it checks higher-zoom entries and returns those.

use crate::pipeline::cache::{DdsCache, DdsCacheMetadata};
use log::debug;
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// In-memory tile cache with disk backing and upserving.
///
/// This struct owns the two-layer cache previously inlined in `DdsFileSystem`.
/// It is fully testable without FUSE or network access.
pub struct TileCache {
    /// In-memory LRU cache of generated DDS tiles (tile_key → DDS bytes)
    memory: RwLock<LruCache<String, Arc<Vec<u8>>>>,
    /// Persistent disk cache for DDS tiles (compressed with zstd)
    disk: Option<Arc<parking_lot::Mutex<DdsCache>>>,
    /// Cache statistics
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

/// Snapshot of cache statistics.
#[derive(Debug, Clone, Default)]
pub struct TileCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_entries: usize,
}

impl TileCache {
    /// Create a new TileCache with the given memory capacity.
    pub fn new(memory_entries: usize) -> Self {
        Self {
            memory: RwLock::new(LruCache::new(
                NonZero::new(memory_entries).unwrap_or(NonZero::new(1).unwrap()),
            )),
            disk: None,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Create a new TileCache with disk backing.
    pub fn with_disk(memory_entries: usize, disk_cache: DdsCache) -> Self {
        Self {
            memory: RwLock::new(LruCache::new(
                NonZero::new(memory_entries).unwrap_or(NonZero::new(1).unwrap()),
            )),
            disk: Some(Arc::new(parking_lot::Mutex::new(disk_cache))),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Create from an existing Arc<Mutex<DdsCache>>.
    pub fn from_disk_arc(
        memory_entries: usize,
        disk_cache: Arc<parking_lot::Mutex<DdsCache>>,
    ) -> Self {
        Self {
            memory: RwLock::new(LruCache::new(
                NonZero::new(memory_entries).unwrap_or(NonZero::new(1).unwrap()),
            )),
            disk: Some(disk_cache),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Get a DDS tile from cache, trying memory then disk then upserving.
    ///
    /// Returns `Some((dds_data, from_disk))` if found, `None` otherwise.
    /// - `from_disk` indicates whether the data came from disk cache (for callers
    ///   that need to promote it into memory).
    pub fn get(&self, tile_key: &str) -> Option<Arc<Vec<u8>>> {
        // 1. Check memory cache
        {
            let mut mem = self.memory.write();
            if let Some(dds) = mem.get(tile_key) {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(dds.clone());
            }
        }

        // 2. Check disk cache
        if let Some(ref dc) = self.disk
            && let Ok((dds_data, _meta)) = dc.lock().get(tile_key)
        {
            debug!("TileCache disk hit: {}", tile_key);
            self.hits.fetch_add(1, Ordering::Relaxed);
            let arc = Arc::new(dds_data);
            self.promote_to_memory(tile_key, arc.clone());
            return Some(arc);
        }

        // 3. Try upserving: check if higher-zoom DDS is cached on disk
        if let Some(ref dc) = self.disk {
            // Parse tile_key format: "{row}_{col}_{maptype}_{zoom}"
            if let Some((_prefix, zoom_str)) = tile_key.rsplit_once('_')
                && let Ok(zoom) = zoom_str.parse::<u32>()
            {
                let mut cache = dc.lock();
                for higher_zoom in (zoom + 1)..=22 {
                    // Reconstruct key properly: row_col_maptype_zoom
                    let parts: Vec<&str> = tile_key.split('_').collect();
                    if parts.len() >= 4 {
                        let upserve_key =
                            format!("{}_{}_{}_{}", parts[0], parts[1], parts[2], higher_zoom);
                        if let Ok((dds_data, _meta)) = cache.get(&upserve_key) {
                            debug!(
                                "TileCache upserving from zoom {} to {}: {}",
                                higher_zoom, zoom, upserve_key
                            );
                            self.hits.fetch_add(1, Ordering::Relaxed);
                            let arc = Arc::new(dds_data);
                            self.promote_to_memory(tile_key, arc.clone());
                            return Some(arc);
                        }
                    }
                }
            }
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Check if a tile exists in memory or disk cache (without promoting).
    pub fn has(&self, tile_key: &str) -> bool {
        // Check memory
        if self.memory.read().peek(tile_key).is_some() {
            return true;
        }
        // Check disk
        if let Some(ref dc) = self.disk
            && dc.lock().contains(tile_key)
        {
            return true;
        }
        false
    }

    /// Store a DDS tile in both memory and disk cache.
    pub fn put(
        &self,
        tile_key: String,
        dds_data: Vec<u8>,
        metadata: &DdsCacheMetadata,
    ) -> Result<(), String> {
        // Write to disk cache
        if let Some(ref dc) = self.disk {
            dc.lock()
                .put(tile_key.clone(), &dds_data, metadata)
                .map_err(|e| format!("Disk cache write failed: {}", e))?;
        }

        // Store in memory cache
        self.promote_to_memory(&tile_key, Arc::new(dds_data));
        Ok(())
    }

    /// Promote a tile to memory cache without disk interaction.
    pub fn put_memory_only(&self, tile_key: String, dds_data: Vec<u8>) {
        self.promote_to_memory(&tile_key, Arc::new(dds_data));
    }

    /// Promote a key to most-recently-used in the disk cache.
    pub fn promote_disk(&self, tile_key: &str) -> bool {
        if let Some(ref dc) = self.disk {
            dc.lock().promote(tile_key)
        } else {
            false
        }
    }

    /// Clear both memory and disk caches.
    pub fn clear(&self) -> Result<(), String> {
        self.memory.write().clear();
        if let Some(ref dc) = self.disk {
            dc.lock()
                .clear()
                .map_err(|e| format!("Disk cache clear failed: {}", e))?;
        }
        Ok(())
    }

    /// Number of entries in the memory cache.
    pub fn memory_len(&self) -> usize {
        self.memory.read().len()
    }

    /// Get cache statistics.
    pub fn stats(&self) -> TileCacheStats {
        TileCacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            memory_entries: self.memory_len(),
        }
    }

    /// Disk cache size in bytes (0 if no disk cache).
    pub fn disk_size_bytes(&self) -> u64 {
        self.disk
            .as_ref()
            .map(|dc| dc.lock().size_bytes())
            .unwrap_or(0)
    }

    /// Promote a tile into the memory cache, evicting LRU if full.
    fn promote_to_memory(&self, tile_key: &str, dds: Arc<Vec<u8>>) {
        let mut mem = self.memory.write();
        let was_full = mem.len() >= mem.cap().get();
        mem.push(tile_key.to_string(), dds);
        if was_full {
            self.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::cache::DdsCacheMetadata;
    use tempfile::TempDir;

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

    #[test]
    fn test_memory_only_cache_put_get() {
        let cache = TileCache::new(10);
        let meta = sample_metadata();

        cache
            .put("key1".to_string(), vec![0x41, 0x42, 0x43], &meta)
            .unwrap();

        let result = cache.get("key1");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), vec![0x41, 0x42, 0x43]);
    }

    #[test]
    fn test_memory_only_cache_miss() {
        let cache = TileCache::new(10);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_memory_only_cache_has() {
        let cache = TileCache::new(10);
        let meta = sample_metadata();

        assert!(!cache.has("key1"));
        cache.put("key1".to_string(), vec![0x41], &meta).unwrap();
        assert!(cache.has("key1"));
        assert!(!cache.has("key2"));
    }

    #[test]
    fn test_memory_only_cache_eviction() {
        let cache = TileCache::new(2); // Only 2 entries
        let meta = sample_metadata();

        cache.put("a".to_string(), vec![1], &meta).unwrap();
        cache.put("b".to_string(), vec![2], &meta).unwrap();
        cache.put("c".to_string(), vec![3], &meta).unwrap(); // Evicts "a"

        assert!(!cache.has("a"));
        assert!(cache.has("b"));
        assert!(cache.has("c"));

        let stats = cache.stats();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_memory_only_cache_stats() {
        let cache = TileCache::new(10);
        let meta = sample_metadata();

        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);

        cache.get("miss1");
        cache.get("miss2");
        assert_eq!(cache.stats().misses, 2);

        cache.put("hit1".to_string(), vec![1], &meta).unwrap();
        cache.get("hit1");
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_memory_only_cache_clear() {
        let cache = TileCache::new(10);
        let meta = sample_metadata();

        cache.put("a".to_string(), vec![1], &meta).unwrap();
        cache.put("b".to_string(), vec![2], &meta).unwrap();
        assert_eq!(cache.memory_len(), 2);

        cache.clear().unwrap();
        assert_eq!(cache.memory_len(), 0);
        assert!(!cache.has("a"));
    }

    #[test]
    fn test_disk_cache_put_get() {
        let tmp = TempDir::new().unwrap();
        let dds_cache = DdsCache::new(tmp.path().to_path_buf(), 1024 * 1024);
        let cache = TileCache::with_disk(10, dds_cache);
        let meta = sample_metadata();

        cache
            .put("key1".to_string(), vec![0x41, 0x42, 0x43], &meta)
            .unwrap();

        // Should be in memory
        let result = cache.get("key1");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), vec![0x41, 0x42, 0x43]);
    }

    #[test]
    fn test_disk_cache_hit_after_memory_miss() {
        let tmp = TempDir::new().unwrap();
        let dds_cache = DdsCache::new(tmp.path().to_path_buf(), 1024 * 1024);
        let cache = TileCache::with_disk(2, dds_cache);
        let meta = sample_metadata();

        // Put 3 entries, filling memory (capacity=2)
        cache.put("a".to_string(), vec![1], &meta).unwrap();
        cache.put("b".to_string(), vec![2], &meta).unwrap();
        cache.put("c".to_string(), vec![3], &meta).unwrap(); // "a" evicted from memory

        // "a" should still be on disk
        assert!(cache.has("a"));
        let result = cache.get("a");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), vec![1]);
    }

    #[test]
    fn test_upserving_from_disk() {
        let tmp = TempDir::new().unwrap();
        let dds_cache = DdsCache::new(tmp.path().to_path_buf(), 1024 * 1024);
        let cache = TileCache::with_disk(10, dds_cache);
        let meta = sample_metadata();

        // Store at zoom 18
        cache
            .put("100_200_BI_18".to_string(), vec![0x18], &meta)
            .unwrap();

        // Request at zoom 16 — should upserve from zoom 18
        let result = cache.get("100_200_BI_16");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), vec![0x18]);
    }

    #[test]
    fn test_disk_cache_stats() {
        let tmp = TempDir::new().unwrap();
        let dds_cache = DdsCache::new(tmp.path().to_path_buf(), 1024 * 1024);
        let cache = TileCache::with_disk(10, dds_cache);
        let meta = sample_metadata();

        cache.put("key1".to_string(), vec![0x41], &meta).unwrap();
        assert!(cache.disk_size_bytes() > 0);
    }

    #[test]
    fn test_put_memory_only() {
        let cache = TileCache::new(10);

        cache.put_memory_only("key1".to_string(), vec![0x41, 0x42]);

        let result = cache.get("key1");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), vec![0x41, 0x42]);
    }

    #[test]
    fn test_tile_cache_trait_object_compatible() {
        // Verify TileCache can be wrapped in Arc for sharing
        let cache = Arc::new(TileCache::new(10));
        let cache2 = cache.clone();

        let meta = sample_metadata();
        cache.put("key1".to_string(), vec![1], &meta).unwrap();

        let result = cache2.get("key1");
        assert!(result.is_some());
    }
}
