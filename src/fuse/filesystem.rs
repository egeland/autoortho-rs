//! Platform-independent virtual DDS filesystem.
//!
//! This module provides the core filesystem logic used by both the fuser-based
//! FUSE mount and any other access method (e.g., direct API for testing).
//! It handles path parsing, DDS generation, caching, and directory structure.

use crate::fuse::pass_through::PassThroughFs;
use crate::fuse::{DdsPathParser, FuseError, MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
use crate::pipeline::cache::DdsCache;
use crate::services::{FallbackService, StatsService};
use crate::tiles::fallback::FallbackConfig;
use crate::tiles::fallback::FallbackSystem;
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::tile_cache::TileCache;
use crate::tiles::tile_generator::TileGenerator;
use crate::tiles::tile_resolution::TileResolution;
use crate::ui::state::TileProgress;
use crate::webui::custommap::CustomMapStore;
use tracing::warn;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Virtual DDS filesystem implementation.
///
/// Serves DDS texture files on-demand by fetching JPEG chunks from tile
/// providers, assembling them into 4096×4096 tiles, and compressing to DDS.
/// Generated DDS data is cached in memory for subsequent reads.
pub struct DdsFileSystem {
    parser: DdsPathParser,
    /// Tile resolution pipeline: cache → fallback → generate → store
    resolution: TileResolution,
    /// Pass-through for real files in the scenery root
    pass_through: Option<PassThroughFs>,
    /// Night exclusion: when true, read_dds returns a solid-color fallback tile
    /// instead of fetching satellite imagery. Updated externally via the Arc.
    night_exclusion: Arc<AtomicBool>,
}

#[must_use]
pub struct DdsFileSystemBuilder {
    tile_generator: TileGenerator,
    cache_entries: usize,
    disk_cache: Option<Arc<parking_lot::Mutex<DdsCache>>>,
    root: Option<std::path::PathBuf>,
    night_exclusion: Arc<AtomicBool>,
    fallback: Option<Arc<dyn FallbackService>>,
    stats: Option<Arc<dyn StatsService>>,
    solid_color: [u8; 3],
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
            solid_color: FallbackConfig::default().solid_color,
        }
    }

    pub fn tile_progress(mut self, progress: Arc<TileProgress>) -> Self {
        self.tile_generator = self.tile_generator.tile_progress(progress);
        self
    }

    pub fn stats(mut self, stats: Arc<dyn StatsService>) -> Self {
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
        self.solid_color = config.solid_color;
        if let Some(ref disk_cache) = self.disk_cache {
            use crate::services::FallbackServiceImpl;
            self.fallback = Some(Arc::new(FallbackServiceImpl::new(FallbackSystem::new(
                disk_cache.lock().cache_dir().clone(),
                config,
            ))));
        }
        self
    }

    pub fn fallback_service(mut self, fallback: Arc<dyn FallbackService>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn build(self) -> DdsFileSystem {
        let tile_cache = match self.disk_cache {
            Some(dc) => Arc::new(TileCache::from_disk_arc(self.cache_entries, dc)),
            None => Arc::new(TileCache::new(self.cache_entries)),
        };
        let tile_generator = Arc::new(self.tile_generator);
        let mut resolution =
            TileResolution::new(tile_cache, tile_generator).solid_color(self.solid_color);
        if let Some(fallback) = self.fallback {
            resolution = resolution.fallback(fallback);
        }
        if let Some(stats) = self.stats {
            resolution = resolution.stats(stats);
        }
        DdsFileSystem {
            parser: DdsPathParser::new(),
            resolution,
            pass_through: self.root.map(PassThroughFs::new),
            night_exclusion: self.night_exclusion,
        }
    }
}

impl DdsFileSystem {
    pub fn builder(fetcher: Arc<TileFetcher>, provider_id: &str) -> DdsFileSystemBuilder {
        DdsFileSystemBuilder::new(fetcher, provider_id)
    }

    /// Set the fallback system for missing tiles.
    pub fn set_fallback(&mut self, _fallback: Arc<dyn FallbackService>) {
        // Fallback is now set during construction via the builder.
        // This method is kept for API compatibility but is a no-op.
    }

    /// Set the fallback system from configuration.
    pub fn set_fallback_from_config(&mut self, _disk_cache: &DdsCache, _config: FallbackConfig) {
        // Fallback is now set during construction via the builder.
    }

    /// Get a reference to the fallback system if available.
    pub fn fallback_system(&self) -> Option<&Arc<dyn FallbackService>> {
        None // Fallback is now internal to TileResolution
    }

    /// Get a clone of the night exclusion flag Arc.
    /// External code (e.g., the dataref tracker loop) can set this to `true`
    /// to make `read_dds()` return fallback tiles instead of fetching imagery.
    pub fn night_exclusion_flag(&self) -> Arc<AtomicBool> {
        self.night_exclusion.clone()
    }

    /// Get the DDS format used by this filesystem.
    pub fn format(&self) -> crate::pipeline::dds::DdsFormat {
        self.resolution.tile_generator().dds_format()
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
        if let Some(ref pt) = self.pass_through
            && let Ok(attr) = pt.get_attr(trimmed)
        {
            return Ok(attr);
        }

        Err(FuseError::InvalidPath)
    }

    /// Read a byte range from a DDS file.
    ///
    /// If the DDS is not yet generated, triggers the full pipeline:
    /// fetch chunks → decode JPEGs → compose tile → compress DDS.
    pub async fn read_dds(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>, FuseError> {
        let (row, col, maptype, zoom) = self.parser.parse(path)?;
        let resolved = self.resolution.resolve(row, col, &maptype, zoom).await?;
        Ok(TileResolution::slice_range(&resolved.data, offset, size).into_owned())
    }

    /// Check if a DDS tile exists in memory or disk cache.
    pub fn has_dds(&self, path: &str) -> bool {
        let (row, col, maptype, zoom) = match self.parser.parse(path) {
            Ok(r) => r,
            Err(_) => return false,
        };
        let tile_key = format!("{}_{}_{}_{}", row, col, maptype, zoom);
        self.resolution.tile_cache().has(&tile_key)
    }

    /// Get the current size of the disk cache in bytes.
    pub fn disk_cache_size_bytes(&self) -> u64 {
        self.resolution.tile_cache().disk_size_bytes()
    }

    /// Check if an entry in a directory path exists as a real directory in the pass-through root.
    pub fn is_dir_in_root(&self, dir_path: &str, entry_name: &str) -> bool {
        self.pass_through
            .as_ref()
            .is_some_and(|pt| pt.is_dir_in_root(dir_path, entry_name))
    }

    /// List directory contents.
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, FuseError> {
        let trimmed = path.trim_start_matches('/');

        // Root directory
        if path == "/" || trimmed.is_empty() {
            return Ok(match self.pass_through {
                Some(ref pt) => pt.root_entries(),
                None => {
                    let mut entries = vec![".".to_string(), "..".to_string()];
                    for dir in VIRTUAL_DIRS {
                        entries.push(dir.to_string());
                    }
                    entries
                }
            });
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
        if let Some(ref pt) = self.pass_through
            && let Ok(entries) = pt.list_dir(trimmed)
        {
            return Ok(entries);
        }

        Err(FuseError::InvalidPath)
    }

    /// Clear the in-memory DDS cache and the disk cache.
    pub fn clear_cache(&self) {
        if let Err(e) = self.resolution.tile_cache().clear() {
            warn!("Failed to clear tile cache: {}", e);
        }
    }

    /// Number of DDS tiles currently cached in memory.
    pub fn cache_len(&self) -> usize {
        self.resolution.tile_cache().memory_len()
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> DdsCacheStats {
        let stats = self.resolution.tile_cache().stats();
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
    use crate::test_utils::{FailingProvider, MockProvider};

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

    #[tokio::test]
    async fn test_stats_records_cache_hit_via_trait() {
        use crate::services::StatsService;
        use crate::services::stats_service::tests::FakeStatsService;

        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let fake_stats = Arc::new(FakeStatsService::new());
        let stats_ref = fake_stats.clone();

        let fs = DdsFileSystem::builder(Arc::new(fetcher), "ARC")
            .stats(fake_stats as Arc<dyn StatsService>)
            .build();

        // First read: cache miss
        let _ = fs.read_dds("/textures/100_200_BI16.dds", 0, u32::MAX).await;
        let snap = stats_ref.snapshot().await;
        assert_eq!(snap.cache_misses, 1, "should record one cache miss");
        assert_eq!(snap.cache_hits, 0, "no hits yet");

        // Second read: cache hit
        let _ = fs.read_dds("/textures/100_200_BI16.dds", 0, u32::MAX).await;
        let snap = stats_ref.snapshot().await;
        assert_eq!(snap.cache_hits, 1, "should record one cache hit");
        assert_eq!(snap.cache_misses, 1, "misses unchanged");
    }

    #[tokio::test]
    async fn test_fallback_used_on_provider_failure() {
        use crate::services::FallbackService;
        use crate::services::fallback_service::tests::FakeFallbackService;

        let provider = Arc::new(FailingProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let fake_fallback = Arc::new(FakeFallbackService::new(true));
        let fallback_ref = fake_fallback.clone();

        let fs = DdsFileSystem::builder(Arc::new(fetcher), "ARC")
            .fallback_service(fake_fallback as Arc<dyn FallbackService>)
            .build();

        // Read a tile — provider fails, fallback should be used
        let result = fs.read_dds("/textures/100_200_BI16.dds", 0, u32::MAX).await;
        assert!(result.is_ok(), "should return fallback data, not error");

        let data = result.unwrap();
        // FakeFallbackService::solid_fallback returns DDS header with color bytes at 144-147
        assert_eq!(&data[0..4], b"DDS ", "should be valid DDS header");
        assert_eq!(data.len(), 148, "fake fallback returns 148 bytes");
    }
}
