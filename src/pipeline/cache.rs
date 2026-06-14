use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Compression error: {0}")]
    CompressionError(String),
    #[error("Key not found")]
    KeyNotFound,
    #[error("Stale cache entry")]
    Stale,
    #[error("Metadata error: {0}")]
    MetadataError(String),
}

/// DDM metadata format (v3), matching the Python DynamicDDSCache.
/// Stored as a JSON sidecar file alongside each cached DDS.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DdsCacheMetadata {
    /// Schema version (always 3)
    pub v: u32,
    /// DDS dimensions
    pub w: u32,
    pub h: u32,
    /// Mipmap count
    pub mm: u32,
    /// Tile zoom level (from .ter filename)
    pub zl: u32,
    /// Max zoom (zoom at which chunks were fetched)
    pub max_zl: u32,
    /// DDS format ("BC1" or "BC3")
    pub fmt: String,
    /// Map type ("BI", "GO2", etc.)
    pub map: String,
    /// Build timestamp (seconds since epoch)
    pub built: f64,
    /// Tile position
    pub tile_row: u32,
    pub tile_col: u32,
    /// Which mipmap levels have data
    pub populated_mipmaps: Vec<u32>,
    /// Chunk indices that were missing at build time
    pub missing_indices: Vec<u32>,
    /// Chunk indices that used fallback color
    pub fallback_indices: Vec<u32>,
    /// Disk compression method ("zstd" or "none")
    pub disk_compression: String,
}

impl DdsCacheMetadata {
    /// Check if this cached DDS is stale relative to the current config.
    pub fn is_stale(&self, expected_fmt: &str, expected_max_zl: u32) -> bool {
        // Format changed
        if self.fmt != expected_fmt {
            return true;
        }
        // Zoom level changed
        if self.max_zl != expected_max_zl {
            return true;
        }
        false
    }

    /// Check if this DDS needs healing (had missing chunks).
    pub fn needs_healing(&self) -> bool {
        !self.missing_indices.is_empty() || !self.fallback_indices.is_empty()
    }
}

/// Maximum number of entries the LRU index can track.
const MAX_CACHE_ENTRIES: usize = 50_000;

/// DDS disk cache with zstd compression, DDM metadata, and LRU eviction.
pub struct DdsCache {
    cache_dir: PathBuf,
    max_size_bytes: u64,
    current_size_bytes: u64,
    index: LruCache<String, u64>, // key → compressed size on disk (LRU ordered)
}

impl std::fmt::Debug for DdsCache {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DdsCache")
            .field("cache_dir", &self.cache_dir)
            .field("max_size_bytes", &self.max_size_bytes)
            .field("current_size_bytes", &self.current_size_bytes)
            .field("index_len", &self.index.len())
            .finish()
    }
}

impl DdsCache {
    pub fn new(cache_dir: PathBuf, max_size_bytes: u64) -> Self {
        Self {
            cache_dir,
            max_size_bytes,
            current_size_bytes: 0,
            index: LruCache::new(NonZeroUsize::new(MAX_CACHE_ENTRIES).unwrap()),
        }
    }

    /// Open an existing cache directory, rebuilding the index from files on disk.
    pub fn open(cache_dir: PathBuf, max_size_bytes: u64) -> Result<Self, CacheError> {
        std::fs::create_dir_all(&cache_dir)?;

        let mut index = LruCache::new(NonZeroUsize::new(MAX_CACHE_ENTRIES).unwrap());
        let mut current_size_bytes: u64 = 0;

        for entry in std::fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && let Some(key) = name.strip_suffix(".dds.zst")
            {
                let size = entry.metadata()?.len();
                index.put(key.to_string(), size);
                current_size_bytes += size;
            }
        }

        Ok(Self {
            cache_dir,
            max_size_bytes,
            current_size_bytes,
            index,
        })
    }

    /// Number of entries in the cache index.
    pub fn entry_count(&self) -> usize {
        self.index.len()
    }

    /// Cache usage as a fraction (0.0–1.0).
    pub fn usage_fraction(&self) -> f64 {
        if self.max_size_bytes == 0 {
            return 1.0;
        }
        self.current_size_bytes as f64 / self.max_size_bytes as f64
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Remove all cached `.dds.zst` and `.ddm` files and reset the index.
    pub fn clear(&mut self) -> Result<(), CacheError> {
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && (name.ends_with(".dds.zst") || name.ends_with(".ddm"))
            {
                std::fs::remove_file(&path)?;
            }
        }
        self.index.clear();
        self.current_size_bytes = 0;
        Ok(())
    }

    /// Evict least-recently-used entries until there's space for `needed_bytes`.
    fn evict_until_fits(&mut self, needed_bytes: u64) {
        while self.current_size_bytes + needed_bytes > self.max_size_bytes && !self.index.is_empty()
        {
            if let Some((key, size)) = self.index.pop_lru() {
                let dds_path = self.dds_path(&key);
                let ddm_path = self.ddm_path(&key);
                if let Err(e) = std::fs::remove_file(&dds_path) {
                    log::warn!("Failed to evict {}: {}", dds_path.display(), e);
                }
                let _ = std::fs::remove_file(&ddm_path);
                self.current_size_bytes = self.current_size_bytes.saturating_sub(size);
                log::debug!("Cache evicted {} ({} bytes)", key, size);
            } else {
                break;
            }
        }
    }

    /// Generate the cache key for a tile.
    pub fn tile_key(tile_col: u32, tile_row: u32, zoom: u32, maptype: &str) -> String {
        format!("{}_{}_{}_z{}", tile_col, tile_row, maptype, zoom)
    }

    fn dds_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.dds.zst", key))
    }

    fn ddm_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.ddm", key))
    }

    /// Get a DDS from cache (decompressed), along with its metadata.
    /// Promotes the entry in the LRU so it won't be evicted soon.
    pub fn get(&mut self, key: &str) -> Result<(Vec<u8>, DdsCacheMetadata), CacheError> {
        // Touch the LRU entry to mark as recently used
        self.index.promote(key);
        let dds_path = self.dds_path(key);
        let ddm_path = self.ddm_path(key);

        if !dds_path.exists() {
            return Err(CacheError::KeyNotFound);
        }

        // Read metadata
        let meta = if ddm_path.exists() {
            let meta_str = std::fs::read_to_string(&ddm_path)?;
            serde_json::from_str(&meta_str).map_err(|e| CacheError::MetadataError(e.to_string()))?
        } else {
            return Err(CacheError::MetadataError("Missing DDM file".to_string()));
        };

        // Read and decompress DDS
        let compressed = std::fs::read(&dds_path)?;
        let decompressed = zstd::decode_all(&compressed[..])
            .map_err(|e| CacheError::CompressionError(e.to_string()))?;

        Ok((decompressed, meta))
    }

    /// Get only metadata without decompressing the DDS data.
    pub fn get_metadata(&self, key: &str) -> Result<DdsCacheMetadata, CacheError> {
        let ddm_path = self.ddm_path(key);
        if !ddm_path.exists() {
            return Err(CacheError::KeyNotFound);
        }
        let meta_str = std::fs::read_to_string(&ddm_path)?;
        serde_json::from_str(&meta_str).map_err(|e| CacheError::MetadataError(e.to_string()))
    }

    /// Put a DDS into cache (compressed), with metadata.
    /// Uses atomic write (temp file + rename) for crash safety.
    /// Evicts LRU entries if the cache would exceed its size budget.
    pub fn put(
        &mut self,
        key: String,
        data: &[u8],
        metadata: &DdsCacheMetadata,
    ) -> Result<(), CacheError> {
        std::fs::create_dir_all(&self.cache_dir)?;

        let dds_path = self.dds_path(&key);
        let ddm_path = self.ddm_path(&key);

        // Compress DDS data
        let compressed =
            zstd::encode_all(data, 3).map_err(|e| CacheError::CompressionError(e.to_string()))?;

        let compressed_size = compressed.len() as u64;

        // Evict LRU entries if needed to stay within budget
        if self.current_size_bytes + compressed_size > self.max_size_bytes {
            self.evict_until_fits(compressed_size);
        }

        // Atomic write: temp file → rename
        let tmp_dds = self.cache_dir.join(format!("{}.dds.zst.tmp", key));
        std::fs::write(&tmp_dds, &compressed)?;
        std::fs::rename(&tmp_dds, &dds_path)?;

        // Write metadata
        let meta_json = serde_json::to_string_pretty(metadata)
            .map_err(|e| CacheError::MetadataError(e.to_string()))?;
        let tmp_ddm = self.cache_dir.join(format!("{}.ddm.tmp", key));
        std::fs::write(&tmp_ddm, &meta_json)?;
        std::fs::rename(&tmp_ddm, &ddm_path)?;

        // Update index — use push to also update LRU position
        if let Some((_, old_size)) = self.index.push(key, compressed_size) {
            self.current_size_bytes -= old_size;
        }
        self.current_size_bytes += compressed_size;

        Ok(())
    }

    /// Check if key exists in cache
    pub fn contains(&self, key: &str) -> bool {
        self.dds_path(key).exists()
    }

    /// Promote a cache entry to most-recently-used without reading data.
    /// Returns true if the entry exists and was promoted.
    pub fn promote(&mut self, key: &str) -> bool {
        if self.dds_path(key).exists() {
            self.index.promote(key);
            true
        } else {
            false
        }
    }

    /// Evict tiles NOT in the provided route_keys set.
    /// Evicts from LRU (oldest first) until free_bytes_needed is available.
    /// Returns number of tiles evicted.
    pub fn evict_non_route_tiles(
        &mut self,
        route_keys: &std::collections::HashSet<String>,
        free_bytes_needed: u64,
    ) -> u32 {
        let mut evicted = 0u32;

        // Collect non-route keys from LRU (oldest first)
        let keys_to_evict: Vec<String> = self
            .index
            .iter()
            .rev() // LRU order (oldest first)
            .filter(|(k, _)| !route_keys.contains(*k))
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_evict {
            if self.current_size_bytes + free_bytes_needed <= self.max_size_bytes {
                break; // Enough space
            }
            if let Err(e) = self.remove(&key) {
                log::warn!("Failed to evict non-route tile {}: {}", key, e);
            } else {
                evicted += 1;
                log::debug!("Evicted non-route tile {}", key);
            }
        }

        evicted
    }

    /// Remove a cache entry
    pub fn remove(&mut self, key: &str) -> Result<(), CacheError> {
        let dds_path = self.dds_path(key);
        let ddm_path = self.ddm_path(key);

        if dds_path.exists() {
            let size = std::fs::metadata(&dds_path)?.len();
            std::fs::remove_file(&dds_path)?;
            self.current_size_bytes = self.current_size_bytes.saturating_sub(size);
        }
        if ddm_path.exists() {
            std::fs::remove_file(&ddm_path)?;
        }
        self.index.pop(key);
        Ok(())
    }

    /// Get current cache size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.current_size_bytes
    }

    /// Get max cache size
    pub fn max_size_bytes(&self) -> u64 {
        self.max_size_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_cache_put_get_roundtrip() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);

        let original = b"test DDS data for roundtrip";
        let meta = sample_metadata();
        cache.put("key1".to_string(), original, &meta).unwrap();

        let (retrieved, retrieved_meta) = cache.get("key1").unwrap();
        assert_eq!(retrieved, original);
        assert_eq!(retrieved_meta.v, 3);
        assert_eq!(retrieved_meta.fmt, "BC1");
        assert_eq!(retrieved_meta.tile_row, 12345);
    }

    #[test]
    fn test_cache_get_missing_key() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
        assert!(cache.get("nonexistent").is_err());
    }

    #[test]
    fn test_cache_contains() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
        let meta = sample_metadata();

        cache.put("key1".to_string(), b"data", &meta).unwrap();

        assert!(cache.contains("key1"));
        assert!(!cache.contains("key2"));
    }

    #[test]
    fn test_cache_compression() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
        let meta = sample_metadata();

        // Highly compressible data
        let data = vec![0u8; 10000];
        cache.put("key1".to_string(), &data, &meta).unwrap();

        // Compressed should be much smaller
        assert!(cache.size_bytes() < 1000);

        let (retrieved, _) = cache.get("key1").unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_cache_size_tracking() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
        let meta = sample_metadata();

        assert_eq!(cache.size_bytes(), 0);

        cache.put("key1".to_string(), b"test", &meta).unwrap();
        assert!(cache.size_bytes() > 0);
    }

    #[test]
    fn test_cache_remove() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
        let meta = sample_metadata();

        cache.put("key1".to_string(), b"data", &meta).unwrap();
        assert!(cache.contains("key1"));

        cache.remove("key1").unwrap();
        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_metadata_staleness() {
        let meta = sample_metadata();
        assert!(!meta.is_stale("BC1", 16));
        assert!(meta.is_stale("BC3", 16)); // Format changed
        assert!(meta.is_stale("BC1", 17)); // Zoom changed
    }

    #[test]
    fn test_metadata_needs_healing() {
        let mut meta = sample_metadata();
        assert!(!meta.needs_healing());

        meta.missing_indices = vec![5, 10];
        assert!(meta.needs_healing());
    }

    #[test]
    fn test_tile_key_format() {
        let key = DdsCache::tile_key(100, 200, 16, "BI");
        assert_eq!(key, "100_200_BI_z16");
    }

    #[test]
    fn test_get_metadata_only() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
        let meta = sample_metadata();

        cache
            .put("key1".to_string(), b"big dds data here", &meta)
            .unwrap();

        // Should be able to read metadata without decompressing DDS
        let retrieved_meta = cache.get_metadata("key1").unwrap();
        assert_eq!(retrieved_meta.tile_col, 54321);
    }

    #[test]
    fn test_atomic_write_creates_no_tmp_files() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
        let meta = sample_metadata();

        cache.put("key1".to_string(), b"data", &meta).unwrap();

        // Should have .dds.zst and .ddm, no .tmp files
        let entries: Vec<_> = std::fs::read_dir(tmp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        for entry in &entries {
            let name = entry.file_name().to_string_lossy().to_string();
            assert!(!name.ends_with(".tmp"), "Found tmp file: {}", name);
        }
    }

    #[test]
    fn test_cache_eviction_on_budget() {
        let tmp_dir = TempDir::new().unwrap();
        // Small budget: 200 bytes (compressed data is ~20 bytes per entry)
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 200);
        let meta = sample_metadata();

        // Fill the cache with entries
        cache.put("a".to_string(), b"data_a", &meta).unwrap();
        cache.put("b".to_string(), b"data_b", &meta).unwrap();
        cache.put("c".to_string(), b"data_c", &meta).unwrap();

        let size_before = cache.entry_count();
        assert!(size_before > 0);

        // Keep adding — eviction should keep cache within budget
        for i in 0..20 {
            cache
                .put(format!("key_{}", i), b"more data here", &meta)
                .unwrap();
        }

        // Cache should stay within budget
        assert!(
            cache.size_bytes() <= cache.max_size_bytes(),
            "Cache size {} exceeds budget {}",
            cache.size_bytes(),
            cache.max_size_bytes()
        );
    }

    #[test]
    fn test_cache_eviction_removes_files() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 100);
        let meta = sample_metadata();

        cache.put("first".to_string(), b"data1", &meta).unwrap();
        assert!(cache.contains("first"));

        // Add enough to force eviction of "first"
        for i in 0..10 {
            cache
                .put(format!("fill_{}", i), b"more data", &meta)
                .unwrap();
        }

        // "first" should have been evicted (LRU)
        assert!(!cache.contains("first"));
    }

    #[test]
    fn test_cache_lru_order_respected() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 150);
        let meta = sample_metadata();

        cache.put("old".to_string(), b"data_old", &meta).unwrap();
        cache.put("mid".to_string(), b"data_mid", &meta).unwrap();

        // Access "old" to make it recently used
        let _ = cache.get("old");

        // Add entries to force eviction — "mid" should be evicted first (least recently used)
        for i in 0..10 {
            cache.put(format!("new_{}", i), b"new_data", &meta).unwrap();
        }

        // "mid" was LRU, should be evicted
        assert!(!cache.contains("mid"));
    }

    #[test]
    fn test_cache_usage_fraction() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1000);
        let meta = sample_metadata();

        // Empty cache
        assert!((cache.usage_fraction() - 0.0).abs() < f64::EPSILON);

        // Add some data
        cache.put("a".to_string(), b"data_a", &meta).unwrap();
        let frac = cache.usage_fraction();
        assert!(frac > 0.0 && frac < 1.0, "Expected 0 < {} < 1", frac);
    }

    #[test]
    fn test_cache_promote_keeps_tile_on_eviction() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 200);
        let meta = sample_metadata();

        // Add two entries
        cache
            .put("keep_me".to_string(), b"data_keep", &meta)
            .unwrap();
        cache
            .put("evict_me".to_string(), b"data_evict", &meta)
            .unwrap();

        // Promote "keep_me" to make it recently used
        assert!(cache.promote("keep_me"));

        // Add entries to force eviction — "evict_me" should be evicted first
        for i in 0..5 {
            cache.put(format!("new_{}", i), b"new", &meta).unwrap();
        }

        // "keep_me" should survive because it was promoted
        assert!(
            cache.contains("keep_me"),
            "Promoted tile should not be evicted"
        );
    }

    #[test]
    fn test_cache_reverse_promote_keeps_origin_over_destination() {
        let tmp_dir = TempDir::new().unwrap();
        // Use small cache to force eviction
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 80);
        let meta = sample_metadata();

        // Add tiles simulating a route: origin -> mid -> destination
        cache
            .put("origin".to_string(), b"data_origin", &meta)
            .unwrap();
        cache.put("mid".to_string(), b"data_mid", &meta).unwrap();
        cache
            .put("destination".to_string(), b"data_dest", &meta)
            .unwrap();

        // Promote in REVERSE order (destination -> origin)
        // This makes origin the most recently used (top of LRU)
        assert!(cache.promote("destination"));
        assert!(cache.promote("mid"));
        assert!(cache.promote("origin"));

        // Add entries to force eviction
        // destination should be evicted first (least recently used)
        // origin should survive (most recently used)
        for i in 0..5 {
            cache.put(format!("new_{}", i), b"new", &meta).unwrap();
        }

        assert!(
            cache.contains("origin"),
            "Origin tile should survive (promoted last = MRU)"
        );
        assert!(
            !cache.contains("destination"),
            "Destination should be evicted first (promoted first = LRU)"
        );
    }

    #[test]
    fn test_cache_promote_returns_false_for_missing() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);

        assert!(!cache.promote("nonexistent"));
    }

    #[test]
    fn test_evict_non_route_tiles_frees_space() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 100);
        let meta = sample_metadata();

        // Fill cache with tiles: 3 from old route, 1 from new route
        cache.put("old_a".to_string(), b"data", &meta).unwrap();
        cache.put("old_b".to_string(), b"data", &meta).unwrap();
        cache.put("old_c".to_string(), b"data", &meta).unwrap();
        cache.put("new_origin".to_string(), b"data", &meta).unwrap();

        // Route only needs new_origin + 2 more tiles
        let mut route_keys = std::collections::HashSet::new();
        route_keys.insert("new_origin".to_string());
        route_keys.insert("new_waypoint1".to_string());
        route_keys.insert("new_waypoint2".to_string());

        // Ask to free 50 bytes (enough for ~2 new tiles)
        let evicted = cache.evict_non_route_tiles(&route_keys, 50);

        // Should have evicted old route tiles (not new_origin)
        assert!(evicted > 0, "Should evict non-route tiles");
        assert!(
            cache.contains("new_origin"),
            "Route tile should not be evicted"
        );
        assert!(
            !cache.contains("old_a") || !cache.contains("old_b"),
            "Old tiles should be evicted"
        );
    }

    #[test]
    fn test_evict_non_route_tiles_preserves_route_tiles() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = DdsCache::new(tmp_dir.path().to_path_buf(), 80);
        let meta = sample_metadata();

        // Fill cache with route tiles and non-route tiles
        cache.put("route_1".to_string(), b"data", &meta).unwrap();
        cache.put("route_2".to_string(), b"data", &meta).unwrap();
        cache.put("other_1".to_string(), b"data", &meta).unwrap();
        cache.put("other_2".to_string(), b"data", &meta).unwrap();

        let mut route_keys = std::collections::HashSet::new();
        route_keys.insert("route_1".to_string());
        route_keys.insert("route_2".to_string());

        // Evict enough for 3 more tiles
        cache.evict_non_route_tiles(&route_keys, 60);

        // Route tiles should survive
        assert!(cache.contains("route_1"), "Route tile 1 should survive");
        assert!(cache.contains("route_2"), "Route tile 2 should survive");
        // Non-route tiles should be evicted
        assert!(
            !cache.contains("other_1") || !cache.contains("other_2"),
            "Non-route tiles should be evicted"
        );
    }
}
