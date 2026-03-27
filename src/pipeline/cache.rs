use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// DDS disk cache with zstd compression and DDM metadata.
pub struct DdsCache {
    cache_dir: PathBuf,
    max_size_bytes: u64,
    current_size_bytes: u64,
    index: HashMap<String, u64>, // key → compressed size on disk
}

impl DdsCache {
    pub fn new(cache_dir: PathBuf, max_size_bytes: u64) -> Self {
        Self {
            cache_dir,
            max_size_bytes,
            current_size_bytes: 0,
            index: HashMap::new(),
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
    pub fn get(&self, key: &str) -> Result<(Vec<u8>, DdsCacheMetadata), CacheError> {
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

        // Update index
        if let Some(old_size) = self.index.insert(key, compressed_size) {
            self.current_size_bytes -= old_size;
        }
        self.current_size_bytes += compressed_size;

        Ok(())
    }

    /// Check if key exists in cache
    pub fn contains(&self, key: &str) -> bool {
        self.dds_path(key).exists()
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
        self.index.remove(key);
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
        let cache = DdsCache::new(tmp_dir.path().to_path_buf(), 1024 * 1024);
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
}
