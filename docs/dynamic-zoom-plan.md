# Dynamic Zoom Implementation Plan

## Overview

Implement altitude-based dynamic zoom level selection that:
1. Allows users to configure zoom rules by AGL altitude
2. Upserves higher-zoom tiles when available
3. Integrates with SimBrief for route-based prefetching

## Current State

- `DynamicZoom` struct exists in `src/dynamic_zoom.rs` but is NOT wired
- Config has `min_zoom`, `max_zoom`, `near_airport_zoom` but no rules
- Settings UI has min/max zoom sliders only - no dynamic zoom

## Implementation

### 1. Config Changes (`src/config.rs`)

Add:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomRule {
    /// Minimum AGL altitude in feet for this rule
    pub min_altitude_ft: f32,
    /// Zoom level to use at this altitude
    pub zoom_level: u32,
}

impl Default for ZoomRule {
    fn default() -> Self {
        Self { min_altitude_ft: 0.0, zoom_level: 19 }
    }
}
```

Add to `AutoOrthoConfig`:
```rust
pub enable_dynamic_zoom: bool,      // default true
pub zoom_rules: Vec<ZoomRule>,      // sorted by altitude ascending
```

**Default rules:**
- Rule 1: min_altitude=0, zoom=19 (airport + approach, 0-10000ft AGL)
- Rule 2: min_altitude=10000, zoom=16 (cruise, 10000ft+ AGL)

### 2. DynamicZoom Module (`src/dynamic_zoom.rs`)

Rewrite to use rule-based approach:
- `zoom_rules: Vec<ZoomRule>` - sorted by altitude
- `zoom_for_altitude_agl(altitude_agl_ft: f32) -> u32` - find matching rule
- `max_zoom() -> u32` - get from provider info
- `next_rule_min_altitude(current_rule: &ZoomRule) -> Option<f32>` - for UI bounds

### 3. Provider Max Zoom

- Use existing `PROVIDER_INFO` from `provider.rs`
- `PROVIDER_INFO.iter().find(|p| p.id == provider_id).map(|p| p.max_zoom)`
- UI sliders: max = provider's max_zoom

### 4. Upserving Higher-Zoom Tiles (`src/tiles/fetcher.rs`)

Add method:
```rust
/// Try to get chunk data at optimal zoom level.
/// Tries from max_zoom down to min_zoom, returns first found in cache.
pub async fn get_chunk_data_at_optimal_zoom(
    &self,
    row: u32,
    col: u32,
    maptype: &str,
    min_zoom: u32,
    max_zoom: u32,
) -> Result<Option<(Vec<u8>, u32)>, ChunkError>  // returns data + actual zoom
```

Update `src/fuse/filesystem.rs` to use upserving in `generate_tile()`.

### 5. SimBrief Prefetch (`src/main.rs`)

In the prefetch loop, for each prefetch point:
1. Get predicted altitude MSL from `AltitudePredictor`
2. Get ground height from SimBrief waypoint (`ground_height_ft`)
3. Calculate AGL = MSL - ground_height
4. Use `DynamicZoom::zoom_for_altitude_agl(agl)` to get zoom
5. Prefetch at determined zoom instead of fixed max_zoom

### 6. UI (`src/ui/screens/settings.rs`)

Add "Dynamic Zoom" section under Tiles:

```
Dynamic Zoom
─────────────────────────────────────────────
[✓] Enable Dynamic Zoom          Provider: ArcGIS (max zoom: 19)

Rules:
| Above (ft AGL) | Zoom |
|----------------|------|
| 0              | 19   |
| 10000          | 16   |

[Add Rule] [Delete Selected]
```

- Toggle checkbox for enable_dynamic_zoom
- Show provider name and max zoom
- Table showing rules (sorted)
- Add/Delete buttons
- Inline editing or modal for rule details

### 7. Validation

- Rule zoom_level must be ≤ provider's max_zoom
- Rules must be sorted without gaps
- Warn in UI if provider doesn't support requested zoom

## Files to Modify

1. `src/config.rs` - Add ZoomRule struct and config fields
2. `src/dynamic_zoom.rs` - Rewrite for rule-based approach
3. `src/tiles/fetcher.rs` - Add upserving method
4. `src/fuse/filesystem.rs` - Use upserving in tile generation
5. `src/main.rs` - Wire DynamicZoom, update SimBrief prefetch
6. `src/ui/state.rs` - Add Message variants for dynamic zoom
7. `src/ui/mod.rs` - Handle dynamic zoom messages
8. `src/ui/screens/settings.rs` - Add Dynamic Zoom UI section
9. `PLAN.md` - Mark dynamic_zoom as complete

## Testing

- Unit tests for DynamicZoom rules logic
- Integration test: verify upserving uses cached higher-zoom tiles
- Manual test: verify dynamic zoom responds to altitude changes
