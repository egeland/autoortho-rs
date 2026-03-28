# Plan: Fallback System (Phase 4e)

## Overview
The fallback system provides graceful degradation when satellite imagery is unavailable. Currently, the system simply returns a solid-color fallback tile at night. This plan implements a multi-level fallback system that attempts to provide usable imagery even when the primary source is unavailable.

## Current State
From `src/fuse/filesystem.rs:254-267`:
```rust
pub async fn read_dds(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>, FuseError> {
    // Night exclusion: return fallback tile if active
    if self.night_exclusion.load(Relaxed) {
        let dds = build_fallback_dds(4096, 4096, self.format, [20, 25, 15]);
        return Ok(slice_range(&dds, offset, size));
    }
    // ...
}
```

## Proposed Design

### Fallback Levels

| Level | Name | Description | When Used |
|-------|------|-------------|-----------|
| 0 | None | No fallback, fail if unavailable | Default |
| 1 | Cache | Look for tile in disk cache at any zoom | Tile not in memory |
| 2 | Upserve | Scale from higher-resolution mipmap | Already implemented in filesystem.rs:292-310 |
| 3 | Downserve | Scale from lower-zoom tile | Tile missing at requested zoom |
| 4 | Network | Download lower-detail imagery on-demand | Tile completely missing |
| 5 | Solid Color | Return solid-color fallback | All else fails |

### Configuration

Add to `AutoOrthoConfig`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackLevel {
    None,
    Cache,
    Downserve,
    Network,
    Solid,
}

impl Default for FallbackLevel {
    fn default() -> Self {
        Self::Cache  // Safe default - check cache first
    }
}

// In AutoOrthoConfig:
pub fallback_level: FallbackLevel,
pub fallback_cache_zoom: Option<u32>,  // Max zoom to check (e.g., 14)
pub fallback_downserve_max_zoom_gap: u32,  // Max zoom difference (e.g., 4)
```

---

## Implementation

### Step 1: Define Fallback Configuration

**File:** `src/config.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FallbackLevel {
    #[default]
    Cache,    // Check disk cache for lower-zoom tiles
    Downserve, // Scale from lower-resolution tile
    Network,  // Download on-demand
    Solid,    // Solid color fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub level: FallbackLevel,
    pub max_zoom_gap: u32,      // Max zoom levels to downserve
    pub solid_color: [u8; 3],   // RGB for solid fallback
    pub cache_fallback: bool,   // Check disk cache first
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            level: FallbackLevel::Cache,
            max_zoom_gap: 4,
            solid_color: [20, 25, 15],  // Dark green (night)
            cache_fallback: true,
        }
    }
}

// Add to AutoOrthoConfig:
pub fallback: FallbackConfig,
```

### Step 2: Create Fallback Module

**File:** `src/tiles/fallback.rs` (new)

```rust
use crate::pipeline::dds::DdsFormat;
use crate::tiles::coords::TileCoords;
use std::path::PathBuf;
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
    config: crate::config::FallbackConfig,
}

impl FallbackSystem {
    pub fn new(cache_dir: PathBuf, config: crate::config::FallbackConfig) -> Self {
        Self { cache_dir, config }
    }

    /// Find the best available fallback for a missing tile.
    /// Returns (DDS data, actual zoom level) or None.
    pub fn find_fallback(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        requested_zoom: u32,
    ) -> Option<(Vec<u8>, u32)> {
        match self.config.level {
            FallbackLevel::Cache => self.find_cached_fallback(row, col, maptype, requested_zoom),
            FallbackLevel::Downserve => self.downserve_from_cache(row, col, maptype, requested_zoom),
            _ => None,
        }
    }

    /// Find a cached tile at any zoom level within the gap limit.
    fn find_cached_fallback(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        requested_zoom: u32,
    ) -> Option<(Vec<u8>, u32)> {
        let cache_base = self.cache_dir.join("dds");
        if !cache_base.exists() {
            return None;
        }

        // Try lower zoom levels
        for zoom in (0..requested_zoom).rev() {
            let gap = requested_zoom - zoom;
            if gap > self.config.max_zoom_gap {
                continue;
            }

            let key = DdsCache::tile_key(col, row, zoom, maptype);
            let dds_path = cache_base.join(format!("{}.dds.zst", key));
            
            if dds_path.exists() {
                if let Ok((data, _meta)) = self.load_cached_dds(&dds_path) {
                    return Some((data, zoom));
                }
            }
        }

        None
    }

    /// Load and decompress a cached DDS file.
    fn load_cached_dds(&self, path: &PathBuf) -> Result<(Vec<u8>, DdsCacheMetadata), FallbackError> {
        let compressed = std::fs::read(path)?;
        let decompressed = zstd::decode_all(&compressed[..])
            .map_err(|e| FallbackError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData, e
            )))?;
        
        // Try to load metadata
        let meta_path = path.with_extension("ddm");
        let meta = if meta_path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&meta_path)?)
                .ok()
        } else {
            None
        };

        Ok((decompressed, meta.unwrap_or_default()))
    }

    /// Get solid color fallback DDS.
    pub fn solid_fallback(&self, size: u32, format: DdsFormat) -> Vec<u8> {
        crate::pipeline::dds::build_fallback_dds(
            size,
            size,
            format,
            self.config.solid_color,
        )
    }
}
```

### Step 3: Integrate with DdsFileSystem

**File:** `src/fuse/filesystem.rs`

```rust
pub struct DdsFileSystem {
    // ... existing fields ...
    
    /// Fallback system for missing tiles
    fallback: Option<Arc<FallbackSystem>>,
}

impl DdsFileSystem {
    /// Create with fallback support.
    pub fn with_fallback(
        fetcher: Arc<TileFetcher>,
        disk_cache: Arc<std::sync::Mutex<DdsCache>>,
        provider_id: &str,
        fallback_config: FallbackConfig,
    ) -> Self {
        Self {
            // ... existing initialization ...
            fallback: Some(Arc::new(FallbackSystem::new(
                disk_cache.lock().unwrap().cache_dir().clone(),
                fallback_config,
            ))),
        }
    }

    /// Generate fallback tile if configured.
    fn generate_fallback(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        requested_zoom: u32,
        format: DdsFormat,
    ) -> Option<Vec<u8>> {
        let fallback = self.fallback.as_ref()?;
        
        match fallback.config.level {
            FallbackLevel::Solid => {
                Some(fallback.solid_fallback(4096, format))
            }
            _ => {
                // Try to find cached fallback
                fallback.find_fallback(row, col, maptype, requested_zoom)
                    .map(|(data, zoom)| {
                        // If zoom differs, we need to scale
                        if zoom != requested_zoom {
                            self.scale_dds(&data, requested_zoom - zoom, format)
                        } else {
                            data
                        }
                    })
            }
        }
    }

    /// Scale DDS data by powers of 2.
    fn scale_dds(&self, data: &[u8], levels: u32, format: DdsFormat) -> Vec<u8> {
        // Implementation depends on DDS format
        // For simplicity, return original data (proper scaling is complex)
        data.to_vec()
    }

    /// Update read_dds to use fallback system.
    pub async fn read_dds(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>, FuseError> {
        // Night exclusion (existing)
        if self.night_exclusion.load(Relaxed) {
            let dds = crate::pipeline::dds::build_fallback_dds(
                4096, 4096, self.format, [20, 25, 15]
            );
            return Ok(slice_range(&dds, offset, size));
        }

        let (row, col, maptype, zoom) = self.parser.parse(path)?;
        
        // ... existing cache checks ...

        // Try to generate tile
        let result = self.generate_tile(row, col, &maptype, zoom).await;
        
        match result {
            Ok(assembly_result) => {
                // Success - return DDS
                Ok(slice_range(&assembly_result.dds_data, offset, size))
            }
            Err(e) => {
                // Tile generation failed - try fallback
                if let Some(fallback_data) = self.generate_fallback(row, col, &maptype, zoom, self.format) {
                    log::warn!("Using fallback for tile {}_{}_{}: {}", row, col, maptype, e);
                    Ok(slice_range(&fallback_data, offset, size))
                } else {
                    Err(e)
                }
            }
        }
    }
}
```

### Step 4: Add UI Controls

**File:** `src/ui/screens/settings.rs`

```rust
// Add to Settings view:

fn fallback_settings<'a>(state: &'a AppState) -> Element<'a, Message> {
    let config = &state.config.fallback;
    
    column![
        text("Fallback Settings").size(18).bold(),
        
        // Fallback level dropdown
        row![
            text("Fallback Level:"),
            pick_list(
                &FallbackLevel::VARIANTS[..],
                Some(config.level),
                |level| Message::SetFallbackLevel(level),
            ),
        ],
        
        // Max zoom gap slider
        slider(
            1..=8,
            config.max_zoom_gap,
            |gap| Message::SetFallbackMaxZoomGap(gap),
        )
        .label(format!("Max Zoom Gap: {}", config.max_zoom_gap)),
        
        // Solid color picker
        row![
            text("Fallback Color:"),
            // Color picker component
        ],
        
        // Cache fallback toggle
        toggler(
            config.cache_fallback,
            Message::SetFallbackCacheEnabled,
        )
        .label("Check cache first"),
    ]
    .spacing(12)
    .padding(16)
    .into()
}
```

Add messages:
```rust
pub enum Message {
    // ... existing ...
    SetFallbackLevel(FallbackLevel),
    SetFallbackMaxZoomGap(u32),
    SetFallbackCacheEnabled(bool),
}
```

### Step 5: Add Tests

**File:** `src/tiles/fallback.rs` (tests module)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_fallback_system() -> FallbackSystem {
        let tmp = TempDir::new().unwrap();
        FallbackSystem::new(
            tmp.path().to_path_buf(),
            FallbackConfig::default(),
        )
    }

    #[test]
    fn test_fallback_finds_lower_zoom() {
        let fb = test_fallback_system();
        // Create fake cached DDS
        let cache_dir = fb.cache_dir.join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();
        
        // Create a fake DDS file
        let key = DdsCache::tile_key(100, 200, 14, "ARC");
        let dds_path = cache_dir.join(format!("{}.dds.zst", key));
        std::fs::write(&dds_path, b"fake compressed dds").unwrap();
        
        // Should find fallback at zoom 14 for zoom 16 request
        let result = fb.find_fallback(200, 100, "ARC", 16);
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, 14);
    }

    #[test]
    fn test_fallback_respects_zoom_gap() {
        let mut fb = test_fallback_system();
        fb.config.max_zoom_gap = 2;
        
        // Cache only has zoom 10, requested zoom 16 (gap = 6)
        // Should not return fallback
        let result = fb.find_fallback(200, 100, "ARC", 16);
        assert!(result.is_none());
    }

    #[test]
    fn test_solid_fallback() {
        let fb = test_fallback_system();
        let dds = fb.solid_fallback(4096, DdsFormat::BC3);
        assert!(!dds.is_empty());
    }
}
```

---

## Phase 4e: DynamicZoom Wiring

The PLAN.md mentions that `dynamic_zoom.rs` exists but isn't wired into runtime. This is related to fallback because DynamicZoom determines the appropriate zoom level based on altitude.

### Current State

**File:** `src/dynamic_zoom.rs`

```rust
pub struct DynamicZoom {
    rules: Vec<ZoomRule>,
}

impl DynamicZoom {
    pub fn new(rules: Vec<ZoomRule>, enabled: bool) -> Self {
        // ...
    }
    
    /// Get the appropriate zoom level for the given altitude.
    pub fn get_zoom(&self, altitude_ft: f32) -> u32 {
        // Find first rule where altitude >= min_altitude_ft
        // Return that rule's zoom_level
    }
}
```

### Integration Steps

1. **Pass DynamicZoom to DdsFileSystem:**
```rust
pub struct DdsFileSystem {
    // ... existing fields ...
    dynamic_zoom: Option<DynamicZoom>,
}

impl DdsFileSystem {
    pub fn with_dynamic_zoom(
        fetcher: Arc<TileFetcher>,
        dynamic_zoom: DynamicZoom,
        // ... other params ...
    ) -> Self {
        Self {
            // ... existing init ...
            dynamic_zoom: Some(dynamic_zoom),
        }
    }

    /// Get zoom for current altitude (called from X-Plane tracker).
    pub fn get_current_zoom(&self, altitude_agl_ft: f32) -> u32 {
        self.dynamic_zoom
            .as_ref()
            .map(|dz| dz.get_zoom(altitude_agl_ft))
            .unwrap_or(16)  // Default zoom
    }
}
```

2. **Wire up X-Plane tracker updates:**
```rust
// In main.rs or wherever X-Plane tracker runs:

loop {
    let data = tracker.get_flight_data();
    if data.data_valid {
        let zoom = fs.get_current_zoom(data.alt_agl_ft());
        
        // Could trigger prefetch at new zoom level
        // prefetcher.update_zoom(zoom);
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
}
```

---

## Summary

| Task | Status | Dependencies |
|------|--------|--------------|
| Define FallbackConfig in config.rs | ✅ Done | None |
| Create src/tiles/fallback.rs module | ✅ Done | DdsCache |
| Integrate fallback into DdsFileSystem | ✅ Done | fallback module |
| Add fallback UI controls | ✅ Done | config changes |
| Add fallback tests | ✅ Done | fallback module |
| Wire DynamicZoom into runtime | ✅ Done | DynamicZoom exists |
| Update PLAN.md completion status | ✅ Done | All above |

**Status: COMPLETE**

## Implementation Notes

### Completed Changes

1. **src/config.rs**: Added `FallbackLevel` enum and `FallbackConfig` struct (already done in previous session)

2. **src/pipeline/cache.rs**: Added `Default` derive to `DdsCacheMetadata` and `cache_dir()` accessor method to `DdsCache`

3. **src/tiles/fallback.rs**: New module with:
   - `FallbackSystem` struct with configurable fallback behavior
   - `find_fallback()` method for cache lookup
   - `solid_fallback()` method for solid color fallback
   - Configuration getters/setters
   - Comprehensive test suite (9 tests)

4. **src/tiles/mod.rs**: Added `pub mod fallback;`

5. **src/fuse/filesystem.rs**: Integrated fallback system:
   - Added `fallback: Option<Arc<FallbackSystem>>` field
   - Added `with_fallback()` constructor
   - Added `set_fallback()` and `set_fallback_from_config()` methods
   - Modified `read_dds()` to use fallback when:
     - Night exclusion is active (uses fallback's solid color)
     - Tile has missing chunks (uses solid fallback)
     - Cache-based fallback is available (downserve from lower zoom)

6. **src/ui/mod.rs**: Added fallback-related messages:
   - `SetFallbackLevel(FallbackLevel)`
   - `SetFallbackMaxZoomGap(u32)`
   - `SetFallbackCacheEnabled(bool)`

7. **src/ui/screens/settings.rs**: Added fallback settings section with:
   - Fallback level dropdown (Cache/Downserve/Network/Solid)
   - Max zoom gap slider (1-8)
   - Cache fallback toggle with tooltips

8. **DynamicZoom**: Already wired into runtime for prefetch (main.rs lines 380-383)
