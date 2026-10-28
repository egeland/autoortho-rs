// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Tile healer: re-generates incomplete disk-cached tiles.
//!
//! Tiles generated while the tile provider is throttled can contain chunks
//! filled with the fallback color (see `AssemblyResult::failed_indices`).
//! Those tiles are cached as-is and would keep their dark-green patches
//! forever. The healer scans the disk cache for tiles whose metadata marks
//! them incomplete (`needs_healing()`) and re-generates a bounded number per
//! pass, persisting an attempt counter so perpetually failing tiles are
//! eventually left alone.

use crate::tiles::tile_resolution::TileResolution;
use tracing::{info, warn};

/// Maximum healing attempts per tile before it is left alone.
pub const MAX_ATTEMPTS: u32 = 3;

/// How long to wait between the first and subsequent healing passes.
pub const PASS_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15 * 60);

/// Delay before the first pass, letting startup (and X-Plane's initial tile
/// flood) settle first.
pub const INITIAL_DELAY: std::time::Duration = std::time::Duration::from_secs(60);

/// Pause between healing individual tiles, to stay gentle on the provider.
const BETWEEN_TILES: std::time::Duration = std::time::Duration::from_secs(1);

/// Outcome of one healing pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HealReport {
    /// Tiles found in the disk cache
    pub scanned: usize,
    /// Tiles re-generated (regardless of whether they fully healed)
    pub healed: usize,
    /// Incomplete tiles skipped (attempt cap reached)
    pub skipped: usize,
    /// Re-generation attempts that errored
    pub failed: usize,
}

/// Parse a disk cache key into `(row, col, maptype, zoom)`.
///
/// Two key formats exist on disk:
/// - `row_col_MAP_zoom` (written by `TileResolution`)
/// - `col_row_MAP_zzoom` (written by fallback up-serving)
fn parse_tile_key(key: &str) -> Option<(u32, u32, String, u32)> {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 4 {
        return None;
    }
    let (a, b, map, z) = (parts[0], parts[1], parts[2], parts[3]);
    if let Some(zoom) = z.strip_prefix('z') {
        // col_row_MAP_zzoom
        let col = a.parse().ok()?;
        let row = b.parse().ok()?;
        let zoom = zoom.parse().ok()?;
        Some((row, col, map.to_string(), zoom))
    } else {
        // row_col_MAP_zoom
        let row = a.parse().ok()?;
        let col = b.parse().ok()?;
        let zoom = z.parse().ok()?;
        Some((row, col, map.to_string(), zoom))
    }
}

/// Run one healing pass over the disk cache.
///
/// `max_tiles` bounds the work per pass; `max_attempts` caps how often a
/// tile may be re-generated before it is skipped.
pub async fn heal_pass(
    resolution: &TileResolution,
    max_tiles: usize,
    max_attempts: u32,
) -> HealReport {
    let mut report = HealReport::default();

    for key in resolution.tile_cache().disk_keys() {
        report.scanned += 1;

        let Some((row, col, maptype, zoom)) = parse_tile_key(&key) else {
            continue;
        };
        let Some(meta) = resolution.tile_cache().disk_metadata(&key) else {
            continue;
        };
        if !meta.needs_healing() {
            continue;
        }
        if meta.attempts >= max_attempts {
            report.skipped += 1;
            continue;
        }
        if report.healed + report.failed >= max_tiles {
            break;
        }

        match resolution
            .regenerate(row, col, &maptype, zoom, meta.attempts)
            .await
        {
            Ok(_tile) => {
                info!("Healed tile {}: attempt {}", key, meta.attempts + 1);
                report.healed += 1;
            }
            Err(e) => {
                warn!("Healing failed for {}: {}", key, e);
                report.failed += 1;
            }
        }

        tokio::time::sleep(BETWEEN_TILES).await;
    }

    report
}

/// Spawn the background healing loop. Runs one pass shortly after startup,
/// then periodically until shutdown.
pub fn spawn_healer(
    resolution: std::sync::Arc<TileResolution>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
    initial_delay: std::time::Duration,
    interval: std::time::Duration,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(initial_delay) => {}
                _ = shutdown.recv() => return,
            }

            let report = heal_pass(&resolution, usize::MAX, MAX_ATTEMPTS).await;
            if report.healed > 0 || report.skipped > 0 {
                info!(
                    "Healing pass done: scanned {}, healed {}, skipped {}, failed {}",
                    report.scanned, report.healed, report.skipped, report.failed
                );
            }

            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = shutdown.recv() => return,
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tile_key_resolution_format() {
        let parsed = parse_tile_key("100_200_BI_16");
        assert_eq!(parsed, Some((100, 200, "BI".to_string(), 16)));
    }

    #[test]
    fn test_parse_tile_key_fallback_format() {
        // col_row_MAP_zzoom (fallback up-serving format)
        let parsed = parse_tile_key("200_100_BI_z16");
        assert_eq!(parsed, Some((100, 200, "BI".to_string(), 16)));
    }

    #[test]
    fn test_parse_tile_key_rejects_garbage() {
        assert_eq!(parse_tile_key("not_a_key"), None);
        assert_eq!(parse_tile_key("100_200_x_y_z"), None);
        assert_eq!(parse_tile_key("100_200_XX_qq"), None);
    }

    /// End-to-end: an incomplete cached tile is found and re-generated; once
    /// the attempt cap is hit it is skipped instead of retried forever.
    #[tokio::test]
    async fn test_heal_pass_regenerates_incomplete_tiles() {
        use crate::pipeline::cache::DdsCache;
        use crate::test_utils::temp_dir;
        use crate::tiles::tile_cache::TileCache;
        use crate::tiles::tile_generator::TileGenerator;
        use crate::tiles::tile_resolution::TileResolution;

        let tmp = temp_dir();
        let provider = std::sync::Arc::new(crate::test_utils::MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider, "ARC");
        let generator =
            std::sync::Arc::new(TileGenerator::new(std::sync::Arc::new(fetcher), "ARC"));
        let dds_cache = DdsCache::new(tmp.path().join("dds"), 64 * 1024 * 1024);
        let cache = std::sync::Arc::new(TileCache::with_disk(64, dds_cache));
        let resolution = TileResolution::new(cache, generator);

        // Mock JPEGs never decode -> tile is incomplete, attempts = 0
        resolution.resolve(100, 200, "BI", 16).await.unwrap();
        assert!(
            resolution
                .tile_cache()
                .disk_metadata("100_200_BI_16")
                .unwrap()
                .needs_healing()
        );

        // Pass 1: heals the tile (attempt 1 of 3)
        let report = heal_pass(&resolution, 10, MAX_ATTEMPTS).await;
        assert_eq!(report.healed, 1);
        assert_eq!(report.skipped, 0);
        assert_eq!(
            resolution
                .tile_cache()
                .disk_metadata("100_200_BI_16")
                .unwrap()
                .attempts,
            1
        );

        // Pass 2 with cap 1: skipped, not retried forever
        let report = heal_pass(&resolution, 10, 1).await;
        assert_eq!(report.healed, 0);
        assert_eq!(report.skipped, 1);
    }
}
