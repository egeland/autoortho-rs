//! Platform-independent virtual DDS filesystem.
//!
//! This module provides the core filesystem logic used by both the fuser-based
//! FUSE mount and any other access method (e.g., direct API for testing).
//! It handles path parsing, DDS generation, caching, and directory structure.

use crate::fuse::{DdsPathParser, FuseError, MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
use crate::pipeline::dds::DdsFormat;
use crate::tiles::assembler::{AssemblyConfig, assemble_tile};
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::zoom::ChunkGrid;
use log::{debug, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    /// In-memory cache of generated DDS tiles (tile_key → DDS bytes)
    dds_cache: Mutex<HashMap<String, Arc<Vec<u8>>>>,
    /// Scenery root directory (for pass-through of real files)
    root: Option<std::path::PathBuf>,
}

impl DdsFileSystem {
    pub fn new(fetcher: Arc<TileFetcher>) -> Self {
        Self {
            parser: DdsPathParser::new(),
            fetcher,
            format: DdsFormat::BC3,
            dds_cache: Mutex::new(HashMap::new()),
            root: None,
        }
    }

    /// Create with a scenery root for real file pass-through.
    pub fn with_root(fetcher: Arc<TileFetcher>, root: std::path::PathBuf) -> Self {
        Self {
            parser: DdsPathParser::new(),
            fetcher,
            format: DdsFormat::BC3,
            dds_cache: Mutex::new(HashMap::new()),
            root: Some(root),
        }
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
    pub async fn read_dds(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>, FuseError> {
        let (row, col, maptype, zoom) = self.parser.parse(path)?;
        let tile_key = format!("{}_{}_{}_{}", row, col, maptype, zoom);

        // Check in-memory cache first
        {
            let cache = self.dds_cache.lock().expect("dds cache mutex poisoned");
            if let Some(dds) = cache.get(&tile_key) {
                return Ok(slice_range(dds, offset, size));
            }
        }

        // Not cached — generate the DDS tile
        let dds_data = self.generate_tile(row, col, &maptype, zoom).await?;

        // Cache it
        let dds_arc = {
            let mut cache = self.dds_cache.lock().expect("dds cache mutex poisoned");
            let arc = Arc::new(dds_data);
            cache.insert(tile_key, arc.clone());
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
    ) -> Result<Vec<u8>, FuseError> {
        let start = Instant::now();
        let config = AssemblyConfig {
            chunks_per_side: 16,
            chunk_size: 256,
            format: self.format,
            missing_color: [66, 77, 55],
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

        // Fetch all 256 chunks
        let mut jpeg_chunks: Vec<Option<Vec<u8>>> = Vec::with_capacity(256);
        for (chunk_col, chunk_row) in grid.iter_chunks() {
            let result = self
                .fetcher
                .get_chunk_data(chunk_row, chunk_col, maptype, zoom)
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

        Ok(result.dds_data)
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

    /// Clear the in-memory DDS cache.
    pub fn clear_cache(&self) {
        self.dds_cache
            .lock()
            .expect("dds cache mutex poisoned")
            .clear();
    }

    /// Number of DDS tiles currently cached in memory.
    pub fn cache_len(&self) -> usize {
        self.dds_cache
            .lock()
            .expect("dds cache mutex poisoned")
            .len()
    }
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
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider);
        DdsFileSystem::new(Arc::new(fetcher))
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
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider);
        let fs = DdsFileSystem::with_root(Arc::new(fetcher), tmp.path().to_path_buf());

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
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider);
        let fs = DdsFileSystem::with_root(Arc::new(fetcher), tmp.path().to_path_buf());

        let entries = fs.list_dir("/").unwrap();
        assert!(entries.contains(&"textures".to_string()));
        assert!(entries.contains(&"terrain".to_string()));
        assert!(entries.contains(&"test.dsf".to_string()));
        assert!(entries.contains(&"Earth nav data".to_string()));
    }
}
