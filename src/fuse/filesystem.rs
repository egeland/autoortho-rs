//! Platform-independent virtual DDS filesystem.
//!
//! This module provides the core filesystem logic used by both the fuser-based
//! FUSE mount and any other access method (e.g., direct API for testing).
//! It handles path parsing, DDS generation, caching, and directory structure.

use crate::config::FallbackConfig;
use crate::fuse::{DdsPathParser, FuseError, MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
use crate::pipeline::cache::{DdsCache, DdsCacheMetadata};
use crate::pipeline::dds::DdsFormat;
use crate::tiles::assembler::{AssemblyConfig, AssemblyResult, assemble_tile};
use crate::tiles::coords::TileCoords;
use crate::tiles::fallback::FallbackSystem;
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::zoom::ChunkGrid;
use crate::webui::custommap::CustomMapStore;
use log::{debug, warn};
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Instant;

/// Virtual DDS filesystem implementation.
///
/// Serves DDS texture files on-demand by fetching JPEG chunks from tile
/// providers, assembling them into 4096×4096 tiles, and compressing to DDS.
/// Generated DDS data is cached in memory for subsequent reads.
pub struct DdsFileSystem {
    parser: DdsPathParser,
    fetcher: Arc<TileFetcher>,
    format: DdsFormat,
    /// In-memory LRU cache of generated DDS tiles (tile_key → DDS bytes)
    /// Bounded by max_entries to prevent memory exhaustion
    /// Uses RwLock for concurrent reads with exclusive writes
    dds_cache: RwLock<LruCache<String, Arc<Vec<u8>>>>,
    /// Persistent disk cache for DDS tiles (compressed with zstd)
    disk_cache: Option<Arc<std::sync::Mutex<DdsCache>>>,
    /// Scenery root directory (for pass-through of real files)
    root: Option<std::path::PathBuf>,
    /// Night exclusion: when true, read_dds returns a solid-color fallback tile
    /// instead of fetching satellite imagery. Updated externally via the Arc.
    night_exclusion: Arc<AtomicBool>,
    /// Custom map store for per-cell provider overrides
    custom_map: Option<Arc<CustomMapStore>>,
    /// Default provider from config (fallback when no custom map override)
    default_provider: String,
    /// Fallback system for missing tiles
    fallback: Option<Arc<FallbackSystem>>,
    /// Cache statistics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_evictions: AtomicU64,
}

pub struct DdsFileSystemBuilder {
    parser: DdsPathParser,
    fetcher: Arc<TileFetcher>,
    format: DdsFormat,
    cache_entries: usize,
    disk_cache: Option<Arc<std::sync::Mutex<DdsCache>>>,
    root: Option<std::path::PathBuf>,
    night_exclusion: Arc<AtomicBool>,
    custom_map: Option<Arc<CustomMapStore>>,
    default_provider: String,
    fallback: Option<Arc<FallbackSystem>>,
}

impl DdsFileSystemBuilder {
    fn new(fetcher: Arc<TileFetcher>, provider_id: &str) -> Self {
        Self {
            parser: DdsPathParser::new(),
            fetcher,
            format: DdsFormat::BC3,
            cache_entries: 256,
            disk_cache: None,
            root: None,
            night_exclusion: Arc::new(AtomicBool::new(false)),
            custom_map: None,
            default_provider: provider_id.to_string(),
            fallback: None,
        }
    }

    pub fn cache_entries(mut self, entries: usize) -> Self {
        self.cache_entries = entries;
        self
    }

    pub fn disk_cache(mut self, cache: Arc<std::sync::Mutex<DdsCache>>) -> Self {
        self.disk_cache = Some(cache);
        self
    }

    pub fn root(mut self, root: std::path::PathBuf) -> Self {
        self.root = Some(root);
        self
    }

    pub fn custom_map(mut self, custom_map: Arc<CustomMapStore>) -> Self {
        self.custom_map = Some(custom_map);
        self
    }

    pub fn fallback_config(mut self, config: FallbackConfig) -> Self {
        if let Some(ref disk_cache) = self.disk_cache {
            self.fallback = Some(Arc::new(FallbackSystem::new(
                disk_cache.lock().unwrap().cache_dir().clone(),
                config,
            )));
        }
        self
    }

    pub fn build(self) -> DdsFileSystem {
        DdsFileSystem {
            parser: self.parser,
            fetcher: self.fetcher,
            format: self.format,
            dds_cache: RwLock::new(LruCache::new(
                NonZero::new(self.cache_entries.max(1)).unwrap(),
            )),
            disk_cache: self.disk_cache,
            root: self.root,
            night_exclusion: self.night_exclusion,
            custom_map: self.custom_map,
            default_provider: self.default_provider,
            fallback: self.fallback,
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_evictions: AtomicU64::new(0),
        }
    }
}

impl DdsFileSystem {
    pub fn new(fetcher: Arc<TileFetcher>, provider_id: &str) -> Self {
        Self::builder(fetcher, provider_id).build()
    }

    pub fn builder(fetcher: Arc<TileFetcher>, provider_id: &str) -> DdsFileSystemBuilder {
        DdsFileSystemBuilder::new(fetcher, provider_id)
    }

    /// Create with a specific in-memory cache size (number of tiles).
    pub fn with_cache_size(
        fetcher: Arc<TileFetcher>,
        provider_id: &str,
        cache_entries: usize,
    ) -> Self {
        Self::builder(fetcher, provider_id)
            .cache_entries(cache_entries)
            .build()
    }

    /// Create with a scenery root for real file pass-through.
    pub fn with_root(
        fetcher: Arc<TileFetcher>,
        root: std::path::PathBuf,
        provider_id: &str,
    ) -> Self {
        Self::builder(fetcher, provider_id).root(root).build()
    }

    /// Create with a persistent disk cache for DDS tiles.
    pub fn with_disk_cache(
        fetcher: Arc<TileFetcher>,
        disk_cache: Arc<std::sync::Mutex<DdsCache>>,
        provider_id: &str,
    ) -> Self {
        Self::builder(fetcher, provider_id)
            .disk_cache(disk_cache)
            .build()
    }

    /// Set the custom map store for per-cell provider overrides.
    pub fn set_custom_map(&mut self, custom_map: Arc<CustomMapStore>) {
        self.custom_map = Some(custom_map);
    }

    /// Set the fallback system for missing tiles.
    pub fn set_fallback(&mut self, fallback: Arc<FallbackSystem>) {
        self.fallback = Some(fallback);
    }

    /// Set the fallback system from configuration.
    pub fn set_fallback_from_config(&mut self, disk_cache: &DdsCache, config: FallbackConfig) {
        self.fallback = Some(Arc::new(FallbackSystem::from_dds_cache(disk_cache, config)));
    }

    /// Get a reference to the fallback system if available.
    pub fn fallback_system(&self) -> Option<&Arc<FallbackSystem>> {
        self.fallback.as_ref()
    }

    /// Create a new filesystem with custom map support.
    pub fn new_with_custom_map(
        fetcher: Arc<TileFetcher>,
        custom_map: Arc<CustomMapStore>,
        provider_id: &str,
    ) -> Self {
        Self::builder(fetcher, provider_id)
            .custom_map(custom_map)
            .build()
    }

    /// Create with a scenery root and custom map support.
    pub fn with_root_and_custom_map(
        fetcher: Arc<TileFetcher>,
        root: std::path::PathBuf,
        custom_map: Arc<CustomMapStore>,
        provider_id: &str,
    ) -> Self {
        Self::builder(fetcher, provider_id)
            .root(root)
            .custom_map(custom_map)
            .build()
    }

    /// Create with a persistent disk cache and custom map support.
    pub fn with_disk_cache_and_custom_map(
        fetcher: Arc<TileFetcher>,
        disk_cache: Arc<std::sync::Mutex<DdsCache>>,
        custom_map: Arc<CustomMapStore>,
        provider_id: &str,
    ) -> Self {
        Self::builder(fetcher, provider_id)
            .disk_cache(disk_cache)
            .custom_map(custom_map)
            .build()
    }

    /// Create with a fallback system for missing tiles.
    pub fn with_fallback(
        fetcher: Arc<TileFetcher>,
        disk_cache: Arc<std::sync::Mutex<DdsCache>>,
        provider_id: &str,
        fallback_config: FallbackConfig,
    ) -> Self {
        Self::builder(fetcher, provider_id)
            .disk_cache(disk_cache)
            .fallback_config(fallback_config)
            .build()
    }

    /// Get the provider for a given tile based on custom map overrides.
    /// Returns the provider ID to use (custom map override or default).
    fn get_provider_for_tile(&self, row: u32, col: u32, zoom: u32) -> String {
        // Get tile center coordinates
        let (center_lat, center_lon) = match TileCoords::tile_to_latlng(col, row, zoom) {
            Ok(coords) => coords,
            Err(_) => return self.default_provider.clone(),
        };

        // Compute cell key (floor coordinates)
        let cell_key = format!(
            "{},{}",
            center_lat.floor() as i32,
            center_lon.floor() as i32
        );

        // Check for custom map override
        if let Some(ref custom_map) = self.custom_map {
            let cells = custom_map.get_cells();
            if let Some(provider) = cells.get(&cell_key) {
                return provider.clone();
            }
        }

        self.default_provider.clone()
    }

    /// Get a clone of the night exclusion flag Arc.
    /// External code (e.g., the dataref tracker loop) can set this to `true`
    /// to make `read_dds()` return fallback tiles instead of fetching imagery.
    pub fn night_exclusion_flag(&self) -> Arc<AtomicBool> {
        self.night_exclusion.clone()
    }

    /// Set the DDS compression format.
    pub fn set_format(&mut self, format: DdsFormat) {
        self.format = format;
    }

    /// Get attributes for a path.
    pub async fn get_attr(&self, path: &str) -> Result<FileAttr, FuseError> {
        // Root directory
        if path == "/" {
            return Ok(FileAttr::directory());
        }

        // Poison pill
        if is_poison_path(path) {
            return Err(FuseError::ShutdownRequested);
        }

        // Virtual directories
        let trimmed = path.trim_start_matches('/');
        if VIRTUAL_DIRS.contains(&trimmed) {
            return Ok(FileAttr::directory());
        }

        // Marker file
        if trimmed.ends_with(MARKER_FILE) {
            return Ok(FileAttr::file(0));
        }

        // DDS file
        if let Ok((_row, _col, _maptype, zoom)) = self.parser.parse(path) {
            let size = crate::fuse::calculate_dds_size(zoom);
            return Ok(FileAttr::file(size as u64));
        }

        // Real file pass-through
        if let Some(ref root) = self.root {
            let full_path = root.join(trimmed);
            if full_path.exists() {
                let meta =
                    std::fs::metadata(&full_path).map_err(|e| FuseError::IoError(e.to_string()))?;
                if meta.is_dir() {
                    return Ok(FileAttr::directory());
                } else {
                    return Ok(FileAttr::file(meta.len()));
                }
            }
        }

        Err(FuseError::InvalidPath)
    }

    /// Read a byte range from a DDS file.
    ///
    /// If the DDS is not yet generated, triggers the full pipeline:
    /// fetch chunks → decode JPEGs → compose tile → compress DDS.
    ///
    /// When night exclusion is active, returns a solid-color fallback tile
    /// without fetching any satellite imagery.
    pub async fn read_dds(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>, FuseError> {
        // Night exclusion: return fallback tile if active
        if self
            .night_exclusion
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let dds = if let Some(fb) = &self.fallback {
                fb.solid_fallback(4096, self.format)
            } else {
                crate::pipeline::dds::build_fallback_dds(
                    4096,
                    4096,
                    self.format,
                    [20, 25, 15], // dark green for night
                )
            };
            return Ok(slice_range(&dds, offset, size));
        }

        let (row, col, maptype, zoom) = self.parser.parse(path)?;
        let tile_key = format!("{}_{}_{}_{}", row, col, maptype, zoom);

        // Check in-memory cache first
        {
            let mut cache = self.dds_cache.write();
            if let Some(dds) = cache.get(&tile_key) {
                self.cache_hits
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Ok(slice_range(dds, offset, size));
            }
            self.cache_misses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        // Check disk cache for requested zoom
        if let Some(ref dc) = self.disk_cache
            && let Ok(cache) = dc.lock()
            && let Ok((dds_data, _meta)) = cache.get(&tile_key)
        {
            debug!("DDS disk cache hit: {}", tile_key);
            let arc = Arc::new(dds_data);
            let mut mem_cache = self.dds_cache.write();
            let was_full = mem_cache.len() >= mem_cache.cap().get();
            mem_cache.push(tile_key, arc.clone());
            if was_full {
                self.cache_evictions
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            return Ok(slice_range(&arc, offset, size));
        }

        // Try upserving: check if higher-zoom DDS is cached
        if let Some(ref dc) = self.disk_cache
            && let Ok(cache) = dc.lock()
        {
            for higher_zoom in (zoom + 1)..=22 {
                let upserve_key = format!("{}_{}_{}_{}", row, col, maptype, higher_zoom);
                if let Ok((dds_data, _meta)) = cache.get(&upserve_key) {
                    debug!(
                        "DDS upserving from zoom {} to {}: {}",
                        higher_zoom, zoom, upserve_key
                    );
                    let arc = Arc::new(dds_data);
                    let mut mem_cache = self.dds_cache.write();
                    let was_full = mem_cache.len() >= mem_cache.cap().get();
                    // Store at the requested zoom key so future requests work
                    mem_cache.push(tile_key, arc.clone());
                    if was_full {
                        self.cache_evictions
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                    return Ok(slice_range(&arc, offset, size));
                }
            }
        }

        // Try fallback (downserve) if configured and no cache hit
        if let Some(fb) = &self.fallback
            && let Some((fallback_data, fallback_zoom)) = fb.find_fallback(row, col, &maptype, zoom)
        {
            debug!(
                "DDS fallback from zoom {} to {}: {}_{}_{}_{}",
                fallback_zoom, zoom, row, col, maptype, zoom
            );
            let dds = fallback_data;
            let arc = Arc::new(dds);
            let mut mem_cache = self.dds_cache.write();
            let was_full = mem_cache.len() >= mem_cache.cap().get();
            mem_cache.push(tile_key, arc.clone());
            if was_full {
                self.cache_evictions
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            return Ok(slice_range(&arc, offset, size));
        }

        // Not cached — generate the DDS tile
        let result = self.generate_tile(row, col, &maptype, zoom).await?;
        let dds_data = result.dds_data;

        // Check if tile has missing chunks and fallback is configured
        if result.chunks_failed > 0
            && let Some(fb) = &self.fallback
        {
            debug!(
                "Tile {}_{}_{}_{} has {} missing chunks, using fallback",
                row, col, maptype, zoom, result.chunks_failed
            );
            let fallback_dds = fb.solid_fallback(4096, self.format);
            let arc = Arc::new(fallback_dds);
            let mut mem_cache = self.dds_cache.write();
            let was_full = mem_cache.len() >= mem_cache.cap().get();
            mem_cache.push(tile_key, arc.clone());
            if was_full {
                self.cache_evictions
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            return Ok(slice_range(&arc, offset, size));
        }

        // Write to disk cache
        if let Some(ref dc) = self.disk_cache
            && let Ok(mut cache) = dc.lock()
        {
            let config = AssemblyConfig {
                chunks_per_side: 16,
                chunk_size: 256,
                format: self.format,
                missing_color: [66, 77, 55],
                seasonal_saturation: 1.0,
            };
            let metadata = DdsCacheMetadata {
                v: 3,
                w: config.tile_size(),
                h: config.tile_size(),
                mm: result.mipmap_count,
                zl: zoom,
                max_zl: zoom,
                fmt: format!("{:?}", self.format),
                map: maptype.to_string(),
                built: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64(),
                tile_row: row,
                tile_col: col,
                populated_mipmaps: (0..result.mipmap_count).collect(),
                missing_indices: vec![],
                fallback_indices: vec![],
                disk_compression: "zstd".to_string(),
            };
            if let Err(e) = cache.put(tile_key.clone(), &dds_data, &metadata) {
                warn!("Failed to write DDS disk cache: {}", e);
            } else {
                debug!("DDS disk cache write: {}", tile_key);
            }
        }

        // Cache it in memory (LRU will evict oldest entries when full)
        let dds_arc = {
            let mut cache = self.dds_cache.write();
            let was_full = cache.len() >= cache.cap().get();
            let arc = Arc::new(dds_data);
            cache.push(tile_key, arc.clone());
            if was_full {
                // An entry was evicted to make room
                self.cache_evictions
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            arc
        };

        Ok(slice_range(&dds_arc, offset, size))
    }

    /// Generate a complete DDS tile by fetching and assembling chunks.
    async fn generate_tile(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        zoom: u32,
    ) -> Result<AssemblyResult, FuseError> {
        let start = Instant::now();
        let config = AssemblyConfig {
            chunks_per_side: 16,
            chunk_size: 256,
            format: self.format,
            missing_color: [66, 77, 55],
            seasonal_saturation: 1.0,
        };

        // Calculate chunk grid for this tile
        let grid = ChunkGrid {
            col,
            row,
            width: 16,
            height: 16,
            zoom,
            pixel_width: 4096,
            pixel_height: 4096,
        };

        // Fetch all 256 chunks with per-chunk provider resolution
        let mut jpeg_chunks: Vec<Option<Vec<u8>>> = Vec::with_capacity(256);
        for (chunk_col, chunk_row) in grid.iter_chunks() {
            // Get the provider for this specific chunk based on custom map overrides
            let provider_id = self.get_provider_for_tile(chunk_row, chunk_col, zoom);

            let result = self
                .fetcher
                .get_chunk_data_with_provider(chunk_row, chunk_col, maptype, zoom, &provider_id)
                .await;
            match result {
                Ok(Some(data)) => jpeg_chunks.push(Some(data)),
                _ => jpeg_chunks.push(None),
            }
        }

        // Assemble into DDS
        let result =
            assemble_tile(&jpeg_chunks, &config).map_err(|e| FuseError::IoError(e.to_string()))?;

        let elapsed = start.elapsed();
        debug!(
            "Generated DDS tile {}_{}_{}_{}: {}×{}, {}/{} chunks decoded, {:?}",
            row,
            col,
            maptype,
            zoom,
            config.tile_size(),
            config.tile_size(),
            result.chunks_decoded,
            config.total_chunks(),
            elapsed,
        );

        if result.chunks_failed > 0 {
            warn!(
                "Tile {}_{}_{}_{}: {} chunks used fallback color",
                row, col, maptype, zoom, result.chunks_failed
            );
        }

        Ok(result)
    }

    /// Get the current size of the disk cache in bytes.
    pub fn disk_cache_size_bytes(&self) -> u64 {
        self.disk_cache
            .as_ref()
            .and_then(|dc| dc.lock().ok())
            .map(|c| c.size_bytes())
            .unwrap_or(0)
    }

    /// List directory contents.
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, FuseError> {
        let trimmed = path.trim_start_matches('/');

        // Root directory
        if path == "/" || trimmed.is_empty() {
            let mut entries = vec![".".to_string(), "..".to_string()];
            for dir in VIRTUAL_DIRS {
                entries.push(dir.to_string());
            }
            // Add real directories from scenery root
            if let Some(ref root) = self.root
                && let Ok(read_dir) = std::fs::read_dir(root)
            {
                for entry in read_dir.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !VIRTUAL_DIRS.contains(&name.as_str()) {
                        entries.push(name);
                    }
                }
            }
            return Ok(entries);
        }

        // Virtual directories show only the marker file
        if VIRTUAL_DIRS.contains(&trimmed) {
            return Ok(vec![
                ".".to_string(),
                "..".to_string(),
                MARKER_FILE.to_string(),
            ]);
        }

        // Real directory pass-through
        if let Some(ref root) = self.root {
            let full_path = root.join(trimmed);
            if full_path.is_dir() {
                let mut entries = vec![".".to_string(), "..".to_string()];
                if let Ok(read_dir) = std::fs::read_dir(&full_path) {
                    for entry in read_dir.flatten() {
                        entries.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
                return Ok(entries);
            }
        }

        Err(FuseError::InvalidPath)
    }

    /// Clear the in-memory DDS cache and the disk cache.
    pub fn clear_cache(&self) {
        self.dds_cache.write().clear();
        if let Some(ref dc) = self.disk_cache
            && let Ok(mut cache) = dc.lock()
            && let Err(e) = cache.clear()
        {
            warn!("Failed to clear DDS disk cache: {}", e);
        }
    }

    /// Number of DDS tiles currently cached in memory.
    pub fn cache_len(&self) -> usize {
        self.dds_cache.read().len()
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> DdsCacheStats {
        DdsCacheStats {
            hits: self.cache_hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.cache_misses.load(std::sync::atomic::Ordering::Relaxed),
            evictions: self
                .cache_evictions
                .load(std::sync::atomic::Ordering::Relaxed),
            entries: self.cache_len(),
        }
    }
}

/// DDS cache statistics.
#[derive(Debug, Clone, Default)]
pub struct DdsCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries: usize,
}

/// Extract a byte range from a buffer, handling bounds correctly.
fn slice_range(data: &[u8], offset: u64, size: u32) -> Vec<u8> {
    let offset = offset as usize;
    let size = size as usize;
    if offset >= data.len() {
        return Vec::new();
    }
    let end = (offset + size).min(data.len());
    data[offset..end].to_vec()
}

/// File attributes returned by the virtual filesystem.
#[derive(Debug, Clone)]
pub struct FileAttr {
    pub size: u64,
    pub is_dir: bool,
}

impl FileAttr {
    pub fn directory() -> Self {
        Self {
            size: 4096,
            is_dir: true,
        }
    }

    pub fn file(size: u64) -> Self {
        Self {
            size,
            is_dir: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;

    struct MockProvider;

    impl crate::tiles::provider::TileProvider for MockProvider {
        fn fetch(
            &self,
            _row: u32,
            _col: u32,
            _zoom: u32,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Vec<u8>, crate::tiles::provider::TileProviderError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                // Return a valid minimal JPEG (1x1 pixel)
                Ok(vec![
                    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01,
                    0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
                ])
            })
        }

        fn name(&self) -> &str {
            "Mock"
        }
    }

    fn make_fs() -> DdsFileSystem {
        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        DdsFileSystem::new(Arc::new(fetcher), "ARC")
    }

    #[tokio::test]
    async fn test_get_attr_root() {
        let fs = make_fs();
        let attr = fs.get_attr("/").await.unwrap();
        assert!(attr.is_dir);
    }

    #[tokio::test]
    async fn test_get_attr_virtual_dir() {
        let fs = make_fs();
        let attr = fs.get_attr("/textures").await.unwrap();
        assert!(attr.is_dir);
    }

    #[tokio::test]
    async fn test_get_attr_dds_file() {
        let fs = make_fs();
        let attr = fs.get_attr("/textures/100_200_BI16.dds").await.unwrap();
        assert!(!attr.is_dir);
        assert!(attr.size > 1_000_000); // DDS file should be several MB
    }

    #[tokio::test]
    async fn test_get_attr_poison() {
        let fs = make_fs();
        let result = fs.get_attr("/.poison").await;
        assert!(matches!(result, Err(FuseError::ShutdownRequested)));
    }

    #[tokio::test]
    async fn test_get_attr_marker() {
        let fs = make_fs();
        let attr = fs.get_attr("/textures/AOISWORKING").await.unwrap();
        assert!(!attr.is_dir);
        assert_eq!(attr.size, 0);
    }

    #[tokio::test]
    async fn test_get_attr_invalid() {
        let fs = make_fs();
        let result = fs.get_attr("/nonexistent").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_list_dir_root() {
        let fs = make_fs();
        let entries = fs.list_dir("/").unwrap();
        assert!(entries.contains(&"textures".to_string()));
        assert!(entries.contains(&"terrain".to_string()));
        assert!(entries.contains(&".".to_string()));
    }

    #[test]
    fn test_list_dir_textures() {
        let fs = make_fs();
        let entries = fs.list_dir("/textures").unwrap();
        assert!(entries.contains(&MARKER_FILE.to_string()));
    }

    #[test]
    fn test_list_dir_invalid() {
        let fs = make_fs();
        assert!(fs.list_dir("/nonexistent").is_err());
    }

    #[test]
    fn test_slice_range() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(slice_range(&data, 0, 5), vec![0, 1, 2, 3, 4]);
        assert_eq!(slice_range(&data, 5, 5), vec![5, 6, 7, 8, 9]);
        assert_eq!(slice_range(&data, 8, 10), vec![8, 9]); // Clamped
        assert_eq!(slice_range(&data, 20, 5), Vec::<u8>::new()); // Past end
    }

    #[test]
    fn test_cache_operations() {
        let fs = make_fs();
        assert_eq!(fs.cache_len(), 0);
        fs.clear_cache();
        assert_eq!(fs.cache_len(), 0);
    }

    #[tokio::test]
    async fn test_get_attr_with_root() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Create a real file in the scenery root
        std::fs::write(tmp.path().join("test.dsf"), b"fake dsf").unwrap();
        std::fs::create_dir(tmp.path().join("Earth nav data")).unwrap();

        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let fs = DdsFileSystem::with_root(Arc::new(fetcher), tmp.path().to_path_buf(), "ARC");

        // Real file should be accessible
        let attr = fs.get_attr("/test.dsf").await.unwrap();
        assert!(!attr.is_dir);
        assert_eq!(attr.size, 8); // "fake dsf" = 8 bytes

        // Real directory
        let attr = fs.get_attr("/Earth nav data").await.unwrap();
        assert!(attr.is_dir);
    }

    #[test]
    fn test_list_dir_root_with_real_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("test.dsf"), b"data").unwrap();
        std::fs::create_dir(tmp.path().join("Earth nav data")).unwrap();

        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let fs = DdsFileSystem::with_root(Arc::new(fetcher), tmp.path().to_path_buf(), "ARC");

        let entries = fs.list_dir("/").unwrap();
        assert!(entries.contains(&"textures".to_string()));
        assert!(entries.contains(&"terrain".to_string()));
        assert!(entries.contains(&"test.dsf".to_string()));
        assert!(entries.contains(&"Earth nav data".to_string()));
    }
}
