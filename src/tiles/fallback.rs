// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use crate::pipeline::cache::{DdsCache, DdsCacheMetadata};
use std::path::PathBuf;

// === FallbackLevel — moved from config.rs ===

/// Fallback level for missing tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum FallbackLevel {
    #[default]
    Cache, // Check disk cache for lower-zoom tiles
    Blur,    // Blur scaled from lower-zoom tile
    Network, // Download on-demand
    Solid,   // Solid color fallback
}

// === FallbackConfig — moved from config.rs ===

/// Fallback configuration for missing tiles
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FallbackConfig {
    pub level: FallbackLevel,
    pub max_zoom_gap: u32,
    pub solid_color: [u8; 3],
    pub cache_fallback: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            level: FallbackLevel::Cache,
            max_zoom_gap: 4,
            solid_color: [20, 25, 15],
            cache_fallback: true,
        }
    }
}

impl FallbackConfig {
    /// Validate fallback configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_zoom_gap < 1 || self.max_zoom_gap > 10 {
            return Err(format!(
                "fallback.max_zoom_gap out of range (1-10), got {}",
                self.max_zoom_gap
            ));
        }
        Ok(())
    }
}
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FallbackError {
    #[error("No fallback available")]
    NoFallback,
    #[error("Zoom gap too large: {0} > {1}")]
    ZoomGapTooLarge(u32, u32),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct FallbackSystem {
    cache_dir: PathBuf,
    config: FallbackConfig,
}

impl FallbackSystem {
    pub fn new(cache_dir: PathBuf, config: FallbackConfig) -> Self {
        Self { cache_dir, config }
    }

    pub fn from_dds_cache(disk_cache: &DdsCache, config: FallbackConfig) -> Self {
        Self {
            cache_dir: disk_cache.cache_dir().to_path_buf(),
            config,
        }
    }

    pub fn config(&self) -> &FallbackConfig {
        &self.config
    }

    pub fn set_level(&mut self, level: FallbackLevel) {
        self.config.level = level;
    }

    pub fn set_solid_color(&mut self, color: [u8; 3]) {
        self.config.solid_color = color;
    }

    pub fn set_max_zoom_gap(&mut self, gap: u32) {
        self.config.max_zoom_gap = gap;
    }

    pub fn set_cache_fallback(&mut self, enabled: bool) {
        self.config.cache_fallback = enabled;
    }

    pub fn find_fallback(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        requested_zoom: u32,
    ) -> Option<(Vec<u8>, u32)> {
        match self.config.level {
            FallbackLevel::Cache | FallbackLevel::Blur => {
                self.find_cached_fallback(row, col, maptype, requested_zoom)
            }
            FallbackLevel::Solid => None,
            FallbackLevel::Network => {
                if self.config.cache_fallback {
                    self.find_cached_fallback(row, col, maptype, requested_zoom)
                } else {
                    None
                }
            }
        }
    }

    fn find_cached_fallback(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        requested_zoom: u32,
    ) -> Option<(Vec<u8>, u32)> {
        if !self.config.cache_fallback {
            return None;
        }

        let cache_base = self.cache_dir.join("dds");
        if !cache_base.exists() {
            return None;
        }

        for zoom in (0..requested_zoom).rev() {
            let gap = requested_zoom.saturating_sub(zoom);
            if gap > self.config.max_zoom_gap {
                continue;
            }

            let key = DdsCache::tile_key(col, row, zoom, maptype);
            let dds_path = cache_base.join(format!("{}.dds.zst", key));

            if dds_path.exists()
                && let Ok((data, _meta)) = self.load_cached_dds(&dds_path)
            {
                return Some((data, zoom));
            }
        }

        None
    }

    fn load_cached_dds(
        &self,
        path: &PathBuf,
    ) -> Result<(Vec<u8>, DdsCacheMetadata), FallbackError> {
        let compressed = std::fs::read(path)?;
        let decompressed = zstd::decode_all(&compressed[..]).map_err(|e| {
            FallbackError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;

        let meta_path = path.with_extension("ddm");
        let meta = if meta_path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&meta_path)?).ok()
        } else {
            None
        };

        Ok((decompressed, meta.unwrap_or_default()))
    }

    pub fn solid_fallback(&self, size: u32, format: crate::pipeline::dds::DdsFormat) -> Vec<u8> {
        crate::pipeline::dds::build_fallback_dds(size, size, format, self.config.solid_color)
    }

    pub fn needs_fallback(&self) -> bool {
        matches!(
            self.config.level,
            FallbackLevel::Cache | FallbackLevel::Blur | FallbackLevel::Network
        ) && self.config.cache_fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::dds::DdsFormat;
    use tempfile::TempDir;

    fn test_fallback_system() -> FallbackSystem {
        let tmp = TempDir::new().unwrap();
        FallbackSystem::new(tmp.path().to_path_buf(), FallbackConfig::default())
    }

    #[test]
    fn test_fallback_system_creation() {
        let fb = test_fallback_system();
        assert_eq!(fb.config.level, FallbackLevel::Cache);
        assert_eq!(fb.config.max_zoom_gap, 4);
        assert!(fb.config.cache_fallback);
    }

    #[test]
    fn test_fallback_no_cache_dir() {
        let fb = test_fallback_system();
        let result = fb.find_fallback(200, 100, "ARC", 16);
        assert!(result.is_none());
    }

    #[test]
    fn test_fallback_with_zoom_gap_limit() {
        let mut fb = test_fallback_system();
        fb.config.max_zoom_gap = 2;
        fb.config.cache_fallback = true;

        let cache_dir = fb.cache_dir.join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let key = DdsCache::tile_key(100, 200, 10, "ARC");
        let dds_path = cache_dir.join(format!("{}.dds.zst", key));
        std::fs::write(&dds_path, vec![0x41, 0x42, 0x43]).unwrap();

        let result = fb.find_fallback(200, 100, "ARC", 16);
        assert!(result.is_none());
    }

    #[test]
    fn test_fallback_within_zoom_gap() {
        let fb = test_fallback_system();
        assert!(fb.config.cache_fallback);

        let cache_dir = fb.cache_dir.join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let key = DdsCache::tile_key(100, 200, 14, "ARC");
        let dds_path = cache_dir.join(format!("{}.dds.zst", key));

        let original_data = vec![0x41, 0x42, 0x43];
        let compressed = zstd::encode_all(original_data.as_slice(), 0).unwrap();
        std::fs::write(&dds_path, compressed).unwrap();

        let result = fb.find_fallback(200, 100, "ARC", 16);
        assert!(result.is_some());
        let (data, zoom) = result.unwrap();
        assert_eq!(data, original_data);
        assert_eq!(zoom, 14);
    }

    #[test]
    fn test_fallback_cache_disabled() {
        let mut fb = test_fallback_system();
        fb.config.cache_fallback = false;

        let cache_dir = fb.cache_dir.join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let key = DdsCache::tile_key(100, 200, 14, "ARC");
        let dds_path = cache_dir.join(format!("{}.dds.zst", key));
        std::fs::write(&dds_path, vec![0x41, 0x42, 0x43]).unwrap();

        let result = fb.find_fallback(200, 100, "ARC", 16);
        assert!(result.is_none());
    }

    #[test]
    fn test_fallback_solid_level_disabled() {
        let mut fb = test_fallback_system();
        fb.config.level = FallbackLevel::Solid;

        let cache_dir = fb.cache_dir.join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let key = DdsCache::tile_key(100, 200, 14, "ARC");
        let dds_path = cache_dir.join(format!("{}.dds.zst", key));
        std::fs::write(&dds_path, vec![0x41, 0x42, 0x43]).unwrap();

        let result = fb.find_fallback(200, 100, "ARC", 16);
        assert!(result.is_none());
    }

    #[test]
    fn test_solid_fallback() {
        let fb = test_fallback_system();
        let dds = fb.solid_fallback(4096, DdsFormat::BC3);
        assert!(!dds.is_empty());
    }

    #[test]
    fn test_needs_fallback() {
        let mut fb = test_fallback_system();
        assert!(fb.needs_fallback());

        fb.set_level(FallbackLevel::Solid);
        assert!(!fb.needs_fallback());

        fb.set_level(FallbackLevel::Cache);
        fb.set_cache_fallback(false);
        assert!(!fb.needs_fallback());
    }

    #[test]
    fn test_config_setters() {
        let mut fb = test_fallback_system();

        fb.set_level(FallbackLevel::Blur);
        assert_eq!(fb.config.level, FallbackLevel::Blur);

        fb.set_solid_color([100, 150, 200]);
        assert_eq!(fb.config.solid_color, [100, 150, 200]);

        fb.set_max_zoom_gap(6);
        assert_eq!(fb.config.max_zoom_gap, 6);

        fb.set_cache_fallback(false);
        assert!(!fb.config.cache_fallback);
    }
}
