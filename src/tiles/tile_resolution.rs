// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Tile resolution pipeline.
//!
//! Orchestrates the full tile resolution flow:
//! cache check → fallback → generate → cache store.
//! Extracted from `DdsFileSystem` for testability and locality.

use crate::pipeline::cache::DdsCacheMetadata;
use crate::services::{FallbackService, StatsService};
use crate::tiles::tile_cache::TileCache;
use crate::tiles::tile_generator::{TileGenerator, TileGeneratorError};
use std::borrow::Cow;
use std::sync::Arc;
use tracing::{debug, warn};

/// Result of resolving a tile.
pub struct ResolvedTile {
    /// The DDS data bytes.
    pub data: Vec<u8>,
    /// Whether this was a cache hit.
    pub cache_hit: bool,
    /// Whether fallback was used.
    pub used_fallback: bool,
}

/// Orchestrates tile resolution: cache → fallback → generate → store.
///
/// Owns the resolution pipeline previously inlined in `DdsFileSystem::read_dds`.
/// Composes `TileCache`, `TileGenerator`, `FallbackService`, and `StatsService`.
pub struct TileResolution {
    tile_cache: Arc<TileCache>,
    tile_generator: Arc<TileGenerator>,
    fallback: Option<Arc<dyn FallbackService>>,
    stats: Option<Arc<dyn StatsService>>,
    solid_color: [u8; 3],
}

impl TileResolution {
    pub fn new(tile_cache: Arc<TileCache>, tile_generator: Arc<TileGenerator>) -> Self {
        Self {
            tile_cache,
            tile_generator,
            fallback: None,
            stats: None,
            solid_color: [66, 77, 55],
        }
    }

    pub fn fallback(mut self, fallback: Arc<dyn FallbackService>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn stats(mut self, stats: Arc<dyn StatsService>) -> Self {
        self.stats = Some(stats);
        self
    }

    pub fn solid_color(mut self, color: [u8; 3]) -> Self {
        self.solid_color = color;
        self
    }

    /// Get a reference to the tile generator.
    pub fn tile_generator(&self) -> &TileGenerator {
        &self.tile_generator
    }

    /// Get a reference to the tile cache.
    pub fn tile_cache(&self) -> &TileCache {
        &self.tile_cache
    }

    /// Resolve a tile: check cache, try fallback, generate if needed, store result.
    pub async fn resolve(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        zoom: u32,
    ) -> Result<ResolvedTile, TileGeneratorError> {
        let tile_key = format!("{}_{}_{}_{}", row, col, maptype, zoom);

        // Check tile cache (memory → disk → upserving)
        if let Some(dds_arc) = self.tile_cache.get(&tile_key) {
            if let Some(ref stats) = self.stats {
                stats.record_cache_hit().await;
            }
            return Ok(ResolvedTile {
                data: dds_arc.as_ref().clone(),
                cache_hit: true,
                used_fallback: false,
            });
        }
        if let Some(ref stats) = self.stats {
            stats.record_cache_miss().await;
        }

        // Try fallback if configured and no cache hit
        if let Some(fb) = &self.fallback
            && let Some((fallback_data, fallback_zoom)) =
                fb.find_fallback(row, col, maptype, zoom).await
        {
            debug!(
                "DDS fallback from zoom {} to {}: {}_{}_{}_{}",
                fallback_zoom, zoom, row, col, maptype, zoom
            );
            self.tile_cache
                .put_memory_only(tile_key, fallback_data.clone());
            return Ok(ResolvedTile {
                data: fallback_data,
                cache_hit: false,
                used_fallback: true,
            });
        }

        // Not cached — generate the DDS tile
        let result = self
            .tile_generator
            .generate_tile(row, col, maptype, zoom)
            .await?;
        let dds_data = result.dds_data;

        // Record download in stats store
        if let Some(ref stats) = self.stats {
            stats.record_download(dds_data.len() as u64).await;
        }

        // Check if tile has missing chunks and fallback is configured
        if result.chunks_failed > 0
            && let Some(fb) = &self.fallback
        {
            debug!(
                "Tile {}_{}_{}_{} has {} missing chunks, using fallback",
                row, col, maptype, zoom, result.chunks_failed
            );
            let fallback_dds = fb
                .solid_fallback(4096, self.tile_generator.dds_format(), self.solid_color)
                .await;
            self.tile_cache
                .put_memory_only(tile_key, fallback_dds.clone());
            return Ok(ResolvedTile {
                data: fallback_dds,
                cache_hit: false,
                used_fallback: true,
            });
        }

        // Write to disk cache and store in memory
        let tile_size = 4096u32;
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

        Ok(ResolvedTile {
            data: dds_data,
            cache_hit: false,
            used_fallback: false,
        })
    }

    /// Extract a byte range from data, handling bounds correctly.
    pub fn slice_range<'a>(data: &'a [u8], offset: u64, size: u32) -> Cow<'a, [u8]> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::fallback_service::tests::FakeFallbackService;
    use crate::services::stats_service::tests::FakeStatsService;
    use crate::test_utils::MockProvider;
    use crate::tiles::tile_cache::TileCache;
    use std::sync::Arc;

    fn make_resolution() -> TileResolution {
        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let generator = Arc::new(TileGenerator::new(Arc::new(fetcher), "ARC"));
        let cache = Arc::new(TileCache::new(64));
        TileResolution::new(cache, generator)
    }

    #[tokio::test]
    async fn test_resolve_cache_miss_generates_tile() {
        let res = make_resolution();
        let result = res.resolve(100, 200, "BI", 16).await.unwrap();

        assert!(!result.data.is_empty());
        assert_eq!(&result.data[0..4], b"DDS ");
        assert!(!result.cache_hit);
        assert!(!result.used_fallback);
    }

    #[tokio::test]
    async fn test_resolve_cache_hit_on_second_call() {
        let res = make_resolution();

        // First call — cache miss, generates tile
        let r1 = res.resolve(100, 200, "BI", 16).await.unwrap();
        assert!(!r1.cache_hit);

        // Second call — cache hit
        let r2 = res.resolve(100, 200, "BI", 16).await.unwrap();
        assert!(r2.cache_hit);
        assert_eq!(r1.data, r2.data);
    }

    #[tokio::test]
    async fn test_resolve_records_stats() {
        let stats = Arc::new(FakeStatsService::new());
        let res = make_resolution().stats(stats.clone());

        // Cache miss
        res.resolve(100, 200, "BI", 16).await.unwrap();
        let snap = stats.snapshot().await;
        assert_eq!(snap.cache_misses, 1);
        assert_eq!(snap.tiles_downloaded, 1);

        // Cache hit
        res.resolve(100, 200, "BI", 16).await.unwrap();
        let snap = stats.snapshot().await;
        assert_eq!(snap.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_resolve_uses_solid_fallback_on_chunk_failure() {
        let fallback = Arc::new(FakeFallbackService::new(true));
        let res = make_resolution().fallback(fallback);

        // Mock JPEGs are too small to decode → all chunks fail → solid fallback
        let result = res.resolve(100, 200, "BI", 16).await.unwrap();
        assert!(result.used_fallback);
        assert!(!result.data.is_empty());
    }

    #[tokio::test]
    async fn test_resolve_no_fallback_when_not_configured() {
        let res = make_resolution();

        // No fallback service configured — generates tile with fallback color chunks
        let result = res.resolve(100, 200, "BI", 16).await.unwrap();
        assert!(!result.used_fallback);
    }

    #[tokio::test]
    async fn test_slice_range_within_bounds() {
        let data = vec![1, 2, 3, 4, 5];
        let sliced = TileResolution::slice_range(&data, 1, 3);
        assert_eq!(&*sliced, &[2, 3, 4]);
    }

    #[tokio::test]
    async fn test_slice_range_out_of_bounds() {
        let data = vec![1, 2, 3];
        let sliced = TileResolution::slice_range(&data, 10, 5);
        assert!(sliced.is_empty());
    }
}
