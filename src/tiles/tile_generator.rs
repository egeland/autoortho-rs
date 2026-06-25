//! Tile generation pipeline extracted from DdsFileSystem.
//!
//! Owns the fetch → decode → compose → compress → DDS pipeline.
//! Independently testable without FUSE or pass-through concerns.

use crate::pipeline::dds::DdsFormat;
use crate::tiles::assembler::{AssemblyConfig, AssemblyResult, assemble_tile};
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::zoom::ChunkGrid;
use crate::ui::state::TileProgress;
use crate::webui::custommap::CustomMapStore;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, warn};

/// Errors that can occur during tile generation.
#[derive(Debug, Error)]
pub enum TileGeneratorError {
    #[error("IO error: {0}")]
    Io(String),
}

/// Generates DDS tiles from coordinates, independent of filesystem concerns.
pub struct TileGenerator {
    fetcher: Arc<TileFetcher>,
    format: DdsFormat,
    custom_map: Option<Arc<CustomMapStore>>,
    default_provider: String,
    tile_progress: Option<Arc<TileProgress>>,
}

impl TileGenerator {
    pub fn new(fetcher: Arc<TileFetcher>, default_provider: &str) -> Self {
        Self {
            fetcher,
            format: DdsFormat::BC3,
            custom_map: None,
            default_provider: default_provider.to_string(),
            tile_progress: None,
        }
    }

    pub fn format(mut self, format: DdsFormat) -> Self {
        self.format = format;
        self
    }

    /// Get the current DDS format.
    pub fn dds_format(&self) -> DdsFormat {
        self.format
    }

    pub fn custom_map(mut self, custom_map: Arc<CustomMapStore>) -> Self {
        self.custom_map = Some(custom_map);
        self
    }

    pub fn tile_progress(mut self, progress: Arc<TileProgress>) -> Self {
        self.tile_progress = Some(progress);
        self
    }

    /// Get the provider for a tile based on custom map overrides.
    fn get_provider_for_tile(&self, row: u32, col: u32, zoom: u32) -> String {
        let (center_lat, center_lon) =
            match crate::tiles::coords::TileCoords::tile_to_latlng(col, row, zoom) {
                Ok(coords) => coords,
                Err(_) => return self.default_provider.clone(),
            };

        let cell_key = format!(
            "{},{}",
            center_lat.floor() as i32,
            center_lon.floor() as i32
        );

        if let Some(ref custom_map) = self.custom_map {
            let cells = custom_map.get_cells();
            if let Some(provider) = cells.get(&cell_key) {
                return provider.clone();
            }
        }

        self.default_provider.clone()
    }

    /// Generate a complete DDS tile by fetching and assembling chunks.
    pub async fn generate_tile(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        zoom: u32,
    ) -> Result<AssemblyResult, TileGeneratorError> {
        let start = Instant::now();
        let config = AssemblyConfig {
            chunks_per_side: 16,
            chunk_size: 256,
            format: self.format,
            missing_color: [66, 77, 55],
            seasonal_saturation: 1.0,
        };

        let grid = ChunkGrid {
            col,
            row,
            width: 16,
            height: 16,
            zoom,
            pixel_width: 4096,
            pixel_height: 4096,
        };

        if let Some(ref tp) = self.tile_progress {
            tp.start(row, col, zoom, maptype);
        }

        let mut jpeg_chunks: Vec<Option<Vec<u8>>> = Vec::with_capacity(256);
        let mut chunks_fetched = 0u32;
        for (chunk_col, chunk_row) in grid.iter_chunks() {
            let provider_id = self.get_provider_for_tile(chunk_row, chunk_col, zoom);

            let result = self
                .fetcher
                .get_chunk_data_with_provider(chunk_row, chunk_col, maptype, zoom, &provider_id)
                .await;
            match result {
                Ok(Some(data)) => {
                    jpeg_chunks.push(Some(data.to_vec()));
                }
                _ => jpeg_chunks.push(None),
            }
            chunks_fetched += 1;
            if let Some(ref tp) = self.tile_progress {
                tp.update_progress(chunks_fetched);
            }
        }

        let result = assemble_tile(&jpeg_chunks, &config)
            .map_err(|e| TileGeneratorError::Io(e.to_string()))?;

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

        if let Some(ref tp) = self.tile_progress {
            tp.finish();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::MockProvider;

    fn make_generator() -> TileGenerator {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "ARC");
        TileGenerator::new(Arc::new(fetcher), "ARC")
    }

    #[tokio::test]
    async fn test_generate_tile_returns_valid_dds() {
        let tile_gen = make_generator();
        let result = tile_gen.generate_tile(100, 200, "BI", 16).await.unwrap();
        // Mock returns minimal JPEGs that can't decode to 256x256,
        // so chunks use fallback color — but pipeline runs and DDS is produced
        assert!(!result.dds_data.is_empty());
        // DDS magic number
        assert_eq!(&result.dds_data[0..4], b"DDS ");
    }

    #[tokio::test]
    async fn test_generate_tile_all_chunks_fail_with_mock() {
        // Mock JPEGs are too small for the assembler, so all use fallback color
        let tile_gen = make_generator();
        let result = tile_gen.generate_tile(0, 0, "GO2", 12).await.unwrap();
        assert_eq!(result.chunks_failed, 256);
    }

    #[tokio::test]
    async fn test_get_provider_default() {
        let tile_gen = make_generator();
        let provider = tile_gen.get_provider_for_tile(100, 200, 16);
        assert_eq!(provider, "ARC");
    }

    #[test]
    fn test_tile_generator_builder() {
        let provider = Arc::new(MockProvider);
        let fetcher = Arc::new(TileFetcher::new(provider, "ARC"));
        let tile_gen = TileGenerator::new(fetcher.clone(), "GO2").format(DdsFormat::BC1);
        assert_eq!(tile_gen.default_provider, "GO2");
    }
}
