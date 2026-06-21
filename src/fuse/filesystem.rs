//! Platform-independent virtual DDS filesystem.
//!
//! This module provides the core filesystem logic used by both the fuser-based
//! FUSE mount and any other access method (e.g., direct API for testing).
//! It handles path parsing, DDS generation, caching, and directory structure.

use crate::fuse::tile_generator::TileGenerator;
use crate::fuse::{DdsPathParser, FuseError, MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
use crate::pipeline::cache::{DdsCache, DdsCacheMetadata};
use crate::stats::StatsStore;
use crate::tiles::fallback::FallbackConfig;
use crate::tiles::fallback::FallbackSystem;
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::tile_cache::TileCache;
use crate::ui::state::TileProgress;
use crate::webui::custommap::CustomMapStore;
use log::{debug, warn};
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Virtual DDS filesystem implementation.
///
/// Serves DDS texture files on-demand by fetching JPEG chunks from tile
/// providers, assembling them into 4096×4096 tiles, and compressing to DDS.
/// Generated DDS data is cached in memory for subsequent reads.
pub struct DdsFileSystem {
    parser: DdsPathParser,
    tile_generator: Arc<TileGenerator>,
    /// Two-layer tile cache: memory LRU + disk (zstd-compressed)
    tile_cache: Arc<TileCache>,
    /// Scenery root directory (for pass-through of real files)
    root: Option<std::path::PathBuf>,
    /// Night exclusion: when true, read_dds returns a solid-color fallback tile
    /// instead of fetching satellite imagery. Updated externally via the Arc.
    night_exclusion: Arc<AtomicBool>,
    /// Fallback system for missing tiles
    fallback: Option<Arc<FallbackSystem>>,
    /// Shared stats store for web UI
    stats: Option<Arc<StatsStore>>,
}

#[must_use]
pub struct DdsFileSystemBuilder {
    tile_generator: TileGenerator,
    cache_entries: usize,
    disk_cache: Option<Arc<parking_lot::Mutex<DdsCache>>>,
    root: Option<std::path::PathBuf>,
    night_exclusion: Arc<AtomicBool>,
    fallback: Option<Arc<FallbackSystem>>,
    stats: Option<Arc<StatsStore>>,
}

impl DdsFileSystemBuilder {
    fn new(fetcher: Arc<TileFetcher>, provider_id: &str) -> Self {
        Self {
            tile_generator: TileGenerator::new(fetcher, provider_id),
            cache_entries: 256,
            disk_cache: None,
            root: None,
            night_exclusion: Arc::new(AtomicBool::new(false)),
            fallback: None,
            stats: None,
        }
    }

    pub fn tile_progress(mut self, progress: Arc<TileProgress>) -> Self {
        self.tile_generator = self.tile_generator.tile_progress(progress);
        self
    }

    pub fn stats(mut self, stats: Arc<StatsStore>) -> Self {
        self.stats = Some(stats);
        self
    }

    pub fn cache_entries(mut self, entries: usize) -> Self {
        self.cache_entries = entries;
        self
    }

    pub fn disk_cache(mut self, cache: Arc<parking_lot::Mutex<DdsCache>>) -> Self {
        self.disk_cache = Some(cache);
        self
    }

    pub fn root(mut self, root: std::path::PathBuf) -> Self {
        self.root = Some(root);
        self
    }

    pub fn custom_map(mut self, custom_map: Arc<CustomMapStore>) -> Self {
        self.tile_generator = self.tile_generator.custom_map(custom_map);
        self
    }

    pub fn fallback_config(mut self, config: FallbackConfig) -> Self {
        if let Some(ref disk_cache) = self.disk_cache {
            self.fallback = Some(Arc::new(FallbackSystem::new(
                disk_cache.lock().cache_dir().clone(),
                config,
            )));
        }
        self
    }

    pub fn build(self) -> DdsFileSystem {
        let tile_cache = match self.disk_cache {
            Some(dc) => Arc::new(TileCache::from_disk_arc(self.cache_entries, dc)),
            None => Arc::new(TileCache::new(self.cache_entries)),
        };
        DdsFileSystem {
            parser: DdsPathParser::new(),
            tile_generator: Arc::new(self.tile_generator),
            tile_cache,
            root: self.root,
            night_exclusion: self.night_exclusion,
            fallback: self.fallback,
            stats: self.stats,
        }
    }
}

impl DdsFileSystem {
    pub fn builder(fetcher: Arc<TileFetcher>, provider_id: &str) -> DdsFileSystemBuilder {
        DdsFileSystemBuilder::new(fetcher, provider_id)
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

    /// Get a clone of the night exclusion flag Arc.
    /// External code (e.g., the dataref tracker loop) can set this to `true`
    /// to make `read_dds()` return fallback tiles instead of fetching imagery.
    pub fn night_exclusion_flag(&self) -> Arc<AtomicBool> {
        self.night_exclusion.clone()
    }

    /// Get the DDS format used by this filesystem.
    pub fn format(&self) -> crate::pipeline::dds::DdsFormat {
        self.tile_generator.dds_format()
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
    pub async fn read_dds(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>, FuseError> {
        let (row, col, maptype, zoom) = self.parser.parse(path)?;
        let tile_key = format!("{}_{}_{}_{}", row, col, maptype, zoom);

        // Check tile cache (memory → disk → upserving)
        if let Some(dds_arc) = self.tile_cache.get(&tile_key) {
            if let Some(ref stats) = self.stats {
                stats.record_cache_hit();
            }
            return Ok(slice_range(&dds_arc, offset, size).into_owned());
        }
        if let Some(ref stats) = self.stats {
            stats.record_cache_miss();
        }

        // Try fallback if configured and no cache hit
        if let Some(fb) = &self.fallback
            && let Some((fallback_data, fallback_zoom)) = fb.find_fallback(row, col, &maptype, zoom)
        {
            debug!(
                "DDS fallback from zoom {} to {}: {}_{}_{}_{}",
                fallback_zoom, zoom, row, col, maptype, zoom
            );
            self.tile_cache
                .put_memory_only(tile_key, fallback_data.clone());
            return Ok(slice_range(&fallback_data, offset, size).into_owned());
        }

        // Not cached — generate the DDS tile
        let result = self
            .tile_generator
            .generate_tile(row, col, &maptype, zoom)
            .await?;
        let dds_data = result.dds_data;

        // Record download in stats store
        if let Some(ref stats) = self.stats {
            stats.record_download(dds_data.len() as u64);
        }

        // Check if tile has missing chunks and fallback is configured
        if result.chunks_failed > 0
            && let Some(fb) = &self.fallback
        {
            debug!(
                "Tile {}_{}_{}_{} has {} missing chunks, using fallback",
                row, col, maptype, zoom, result.chunks_failed
            );
            let fallback_dds = fb.solid_fallback(4096, self.tile_generator.dds_format());
            self.tile_cache
                .put_memory_only(tile_key, fallback_dds.clone());
            return Ok(slice_range(&fallback_dds, offset, size).into_owned());
        }

        // Write to disk cache and store in memory
        let tile_size = 4096u32; // chunks_per_side(16) * chunk_size(256)
        let metadata = DdsCacheMetadata {
            v: 3,
            w: tile_size,
            h: tile_size,
            mm: result.mipmap_count,
            zl: zoom,
            max_zl: zoom,
            fmt: format!("{:?}", self.tile_generator.dds_format()),
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
        if let Err(e) = self.tile_cache.put(tile_key, dds_data.clone(), &metadata) {
            warn!("Failed to write DDS disk cache: {}", e);
        }

        Ok(slice_range(&dds_data, offset, size).into_owned())
    }

    /// Check if a DDS tile exists in memory or disk cache.
    pub fn has_dds(&self, path: &str) -> bool {
        let (row, col, maptype, zoom) = match self.parser.parse(path) {
            Ok(r) => r,
            Err(_) => return false,
        };
        let tile_key = format!("{}_{}_{}_{}", row, col, maptype, zoom);
        self.tile_cache.has(&tile_key)
    }

    /// Get the current size of the disk cache in bytes.
    pub fn disk_cache_size_bytes(&self) -> u64 {
        self.tile_cache.disk_size_bytes()
    }

    /// Check if an entry in a directory path exists as a real directory in the pass-through root.
    pub fn is_dir_in_root(&self, dir_path: &str, entry_name: &str) -> bool {
        if let Some(ref root) = self.root {
            let trimmed = dir_path.trim_start_matches('/');
            let full_path = if trimmed.is_empty() {
                root.join(entry_name)
            } else {
                root.join(trimmed).join(entry_name)
            };
            full_path.is_dir()
        } else {
            false
        }
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
        if let Err(e) = self.tile_cache.clear() {
            warn!("Failed to clear tile cache: {}", e);
        }
    }

    /// Number of DDS tiles currently cached in memory.
    pub fn cache_len(&self) -> usize {
        self.tile_cache.memory_len()
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> DdsCacheStats {
        let stats = self.tile_cache.stats();
        DdsCacheStats {
            hits: stats.hits,
            misses: stats.misses,
            evictions: stats.evictions,
            entries: stats.memory_entries,
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
/// Returns a borrowed slice when possible (zero-copy), otherwise allocates.
fn slice_range(data: &[u8], offset: u64, size: u32) -> Cow<'_, [u8]> {
    let offset = offset as usize;
    let size = size as usize;
    if offset >= data.len() {
        return Cow::Owned(Vec::new());
    }
    let end = (offset + size).min(data.len());
    if end - offset == size {
        Cow::Borrowed(&data[offset..end])
    } else {
        Cow::Owned(data[offset..end].to_vec())
    }
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
        DdsFileSystem::builder(Arc::new(fetcher), "ARC").build()
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
    fn test_list_dir_root_with_pass_through() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Create a real file in the pass-through root
        std::fs::write(tmp.path().join("sa_info.json"), b"test").unwrap();
        std::fs::create_dir_all(tmp.path().join("scenery")).unwrap();

        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let fs = DdsFileSystem::builder(Arc::new(fetcher), "ARC")
            .root(tmp.path().to_path_buf())
            .build();

        let entries = fs.list_dir("/").unwrap();
        assert!(
            entries.contains(&"textures".to_string()),
            "should contain virtual textures dir"
        );
        assert!(
            entries.contains(&"terrain".to_string()),
            "should contain virtual terrain dir"
        );
        assert!(
            entries.contains(&"sa_info.json".to_string()),
            "should contain pass-through file"
        );
        assert!(
            entries.contains(&"scenery".to_string()),
            "should contain pass-through dir"
        );
    }

    #[tokio::test]
    async fn test_list_dir_replicated_install_structure() {
        let tmp = tempfile::TempDir::new().unwrap();

        // Replicate a real installation structure in the pass-through root:
        // {cache_dir}/scenery/z_autoortho/
        //   na_info.json
        //   sa_info.json
        //   scenery/
        //     z_ao_na/
        //       Earth nav data/
        //         +40-070.dsf
        let root = tmp.path();
        std::fs::write(root.join("na_info.json"), b"na metadata").unwrap();
        std::fs::write(root.join("sa_info.json"), b"sa metadata").unwrap();
        std::fs::create_dir_all(root.join("scenery").join("z_ao_na").join("Earth nav data"))
            .unwrap();
        std::fs::write(
            root.join("scenery")
                .join("z_ao_na")
                .join("Earth nav data")
                .join("+40-070.dsf"),
            b"fake dsf",
        )
        .unwrap();

        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let fs = DdsFileSystem::builder(Arc::new(fetcher), "ARC")
            .root(root.to_path_buf())
            .build();

        // list_dir("/") should return virtual dirs + pass-through entries
        let root_entries = fs.list_dir("/").unwrap();
        assert!(
            root_entries.contains(&"textures".to_string()),
            "virtual textures dir"
        );
        assert!(
            root_entries.contains(&"terrain".to_string()),
            "virtual terrain dir"
        );
        assert!(
            root_entries.contains(&"scenery".to_string()),
            "pass-through scenery dir"
        );
        assert!(
            root_entries.contains(&"na_info.json".to_string()),
            "pass-through na_info"
        );
        assert!(
            root_entries.contains(&"sa_info.json".to_string()),
            "pass-through sa_info"
        );

        // list_dir("/textures") returns marker file
        let tex_entries = fs.list_dir("/textures").unwrap();
        assert!(tex_entries.contains(&MARKER_FILE.to_string()));

        // list_dir("/scenery") returns pack directories via pass-through
        let scenery_entries = fs.list_dir("/scenery").unwrap();
        assert!(
            scenery_entries.contains(&"z_ao_na".to_string()),
            "scenery should list pack dirs"
        );

        // list_dir("/scenery/z_ao_na/Earth nav data") returns DSF files
        let nav_entries = fs.list_dir("/scenery/z_ao_na/Earth nav data").unwrap();
        assert!(
            nav_entries.contains(&"+40-070.dsf".to_string()),
            "should see DSF files"
        );

        // get_attr for virtual directories
        assert!(fs.get_attr("/textures").await.unwrap().is_dir);
        assert!(fs.get_attr("/terrain").await.unwrap().is_dir);

        // get_attr for pass-through files
        let attr = fs.get_attr("/na_info.json").await.unwrap();
        assert!(!attr.is_dir);
        assert_eq!(attr.size, b"na metadata".len() as u64);

        // get_attr for pass-through directories
        assert!(fs.get_attr("/scenery").await.unwrap().is_dir);
        assert!(fs.get_attr("/scenery/z_ao_na").await.unwrap().is_dir);

        // get_attr for pass-through nested files
        let dsf_attr = fs
            .get_attr("/scenery/z_ao_na/Earth nav data/+40-070.dsf")
            .await
            .unwrap();
        assert!(!dsf_attr.is_dir);
        assert_eq!(dsf_attr.size, b"fake dsf".len() as u64);

        // get_attr for DDS tile (virtual, generated on demand)
        let dds_attr = fs.get_attr("/100_200_BI16.dds").await.unwrap();
        assert!(!dds_attr.is_dir);
        assert!(dds_attr.size > 1_000_000);
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
    fn test_slice_range_exact() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let result = slice_range(&data, 0, 10);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(&*result, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_slice_range_clamped() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let result = slice_range(&data, 8, 10);
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(&*result, &[8, 9]);
    }

    #[test]
    fn test_slice_range_past_end() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let result = slice_range(&data, 20, 5);
        assert!(matches!(result, Cow::Owned(_)));
        assert!(result.is_empty());
    }

    #[test]
    fn test_slice_range_empty() {
        let data: Vec<u8> = vec![];
        let result = slice_range(&data, 0, 5);
        assert!(matches!(result, Cow::Owned(_)));
        assert!(result.is_empty());
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
        let fs = DdsFileSystem::builder(Arc::new(fetcher), "ARC")
            .root(tmp.path().to_path_buf())
            .build();

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
        let fs = DdsFileSystem::builder(Arc::new(fetcher), "ARC")
            .root(tmp.path().to_path_buf())
            .build();

        let entries = fs.list_dir("/").unwrap();
        assert!(entries.contains(&"textures".to_string()));
        assert!(entries.contains(&"terrain".to_string()));
        assert!(entries.contains(&"test.dsf".to_string()));
        assert!(entries.contains(&"Earth nav data".to_string()));
    }
}
