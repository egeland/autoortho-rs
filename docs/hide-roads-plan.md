# Hide X-Plane Roads — Implementation Plan

## Overview

Add an option to hide X-Plane's default road rendering when using ortho scenery tiles, while keeping traffic (cars) visible. This is achieved by creating a custom "transparent roads" overlay that replaces road textures with transparent ones but preserves the CAR traffic commands.

This approach is proven by existing tools: **Transparency4Ortho** and **Xroads**.

---

## How It Works

X-Plane renders roads from `.net` (Vector Network Definition) files in `Resources/default scenery/1000 roads/`. Each road type has:

- **TEXTURE** commands that specify the road surface texture
- **ROAD_TYPE** commands that define width, repetition, texture index
- **SEGMENT** commands that define the 3D geometry
- **CAR** commands that define traffic lanes (independent of road surface rendering)

To make roads invisible but keep traffic:
1. Copy `1000 roads/` from X-Plane installation
2. Replace road textures with transparent ones (or non-existent paths)
3. Keep all CAR commands intact
4. Install as custom scenery with `library.txt` scoping to ortho-covered regions
5. Optionally restrict to SimBrief flight route regions

---

## Architecture

```
X-Plane/Custom Scenery/
└── z_autoortho_roads/
    ├── library.txt              # Scopes roads to installed regions
    └── 1000 roads/
        ├── roads.net           # Transparent roads (global)
        └── roads_EU.net        # Transparent roads (Europe)
```

---

## Region Mapping

Used for `library.txt` scoping and SimBrief route detection:

| Region ID | Name | Lat Range | Lon Range |
|-----------|------|-----------|-----------|
| `na` | North America | 15°N to 75°N | -170° to -50° |
| `sa` | South America | -55°S to 15°N | -85° to -30° |
| `eur` | Europe | 30°N to 75°N | -25° to 60° |
| `afr` | Africa | -35°S to 40°N | -20° to 55° |
| `asi` | Asia | -10°S to 55°N | 60° to 180° |
| `aus_pac` | Australia & Pacific | -50°S to 0°N | 110° to 180° |

---

## Implementation Steps

### Phase 1: Config & Core Infrastructure

#### 1.1 Add Config Option

**File**: `src/config.rs`

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoOrthoConfig {
    // ... existing fields ...
    
    #[serde(default)]
    pub hide_roads: bool,
    
    #[serde(default)]
    pub hide_roads_route_only: bool,  // Apply only to SimBrief route regions
}
```

Default: `hide_roads: false`, `hide_roads_route_only: true`

#### 1.2 Create Road Overlay Generator

**File**: `src/scenery/roads.rs` (new)

Functions:
- `copy_base_roads(xplane_path: &Path, target_dir: &Path)` — Copy 1000 roads from X-Plane install
- `make_roads_transparent(roads_dir: &Path)` — Modify .net files to use transparent/no texture
- `generate_library_txt(roads_dir: &Path, regions: &[RegionBounds])` — Create library.txt with region scoping
- `get_regions_for_flight_plan(flight_plan: &FlightPlan) -> Vec<String>` — Get region IDs from SimBrief route

#### 1.3 Modify Installer

**File**: `src/scenery/installer.rs`

Changes:
- When `hide_roads=true`, skip downloading overlay packages (`y_*.zip`) to save bandwidth
- After installing region, call road overlay generator if `hide_roads=true`
- When uninstalling region, clean up road overlay if no other regions need it
- Add `get_installed_region_ids()` helper

#### 1.4 Region Bounds Helper

**File**: `src/scenery/regions.rs` (new)

```rust
pub struct RegionBounds {
    pub id: &'static str,
    pub name: &'static str,
    pub lat_min: f64,
    pub lat_max: f64,
    pub lon_min: f64,
    pub lon_max: f64,
}

pub fn get_region_for_latlon(lat: f64, lon: f64) -> Option<&'static RegionBounds>;
pub fn get_all_regions() -> Vec<&'static RegionBounds>;
```

---

### Phase 2: SimBrief Integration

#### 2.1 Add Route Region Detection to SimBrief Module

**File**: `src/xplane/simbrief.rs`

Add to `FlightPlan`:
```rust
impl FlightPlan {
    /// Get region IDs covered by this flight plan (origin, destination, route waypoints)
    pub fn get_covered_region_ids(&self) -> Vec<String> {
        // Collect all fixes, find their regions
        // Return unique region IDs
    }
    
    /// Get bounding box of entire route
    pub fn get_route_bounds(&self) -> RouteBounds {
        // min/max lat/lon across all fixes
    }
}
```

---

### Phase 3: UI

#### 3.1 Add Settings Toggle

**File**: `src/ui/screens/settings.rs`

Add checkbox:
- "Hide X-Plane Roads" — toggle `hide_roads`
- "Apply to Flight Route Only" (indented, visible when hide_roads=true) — toggle `hide_roads_route_only`

#### 3.2 Dashboard Status

**File**: `src/ui/screens/dashboard.rs`

Add status indicator:
- "Roads: Hidden (Global)" or "Roads: Hidden (Route Only)" or "Roads: Visible"

---

## Key Design Decisions

### 1. Transparent Texture Approach

Options:
- **A**: Embed 1x1 transparent PNG in binary (~100 bytes)
- **B**: Use non-existent texture path (X-Plane logs warnings but renders nothing)
- **C**: Use fully-transparent UV coordinates in SEGMENT commands

**Decision**: Option B — simplest, no extra files needed. Modify `TEXTURE` commands to reference non-existent files like `transparent.png`.

### 2. .net File Modification Strategy

For each `.net` file (roads.net, roads_EU.net):
1. Find all TEXTURE commands
2. Replace texture filenames with transparent version
3. Keep all ROAD_TYPE, SEGMENT, CAR commands intact

Example transformation:
```
# Before
TEXTURE 3 roads_legacy.png
TEXTURE 0 roadbridges_legacy.png

# After  
TEXTURE 3 transparent.png
TEXTURE 0 transparent.png
```

### 3. library.txt Format

```ini
SCENERY_PACK
{
    scenery_path = "1000 roads"
    required = true
}

# Latitude/Longitude scoping (X-Plane 11+)
REGION
{
    name = "North America Ortho"
    lat  = 15, 75
    lon  = -170, -50
}
```

### 4. SimBrief Route Mode Behavior

When `hide_roads_route_only=true`:
1. Fetch SimBrief flight plan
2. Get region IDs from origin, destination, and significant waypoints
3. Only install transparent roads for those regions
4. If no flight plan: fall back to global or prompt user

---

## Files to Modify/Create

| File | Change |
|------|--------|
| `src/config.rs` | Add `hide_roads`, `hide_roads_route_only` fields |
| `src/scenery/regions.rs` | **New** — Region bounds definitions |
| `src/scenery/roads.rs` | **New** — Road overlay generator |
| `src/scenery/installer.rs` | Skip overlay downloads, call generator |
| `src/xplane/simbrief.rs` | Add `get_covered_region_ids()` method |
| `src/ui/screens/settings.rs` | Add checkboxes |
| `src/ui/screens/dashboard.rs` | Add status indicator |
| `src/ui/state.rs` | Add road overlay state |

---

## Testing

1. **Unit tests**: Region bounds, lat/lon to region mapping
2. **Integration**: 
   - Install with hide_roads=true, verify roads invisible in X-Plane
   - Install with hide_roads_route_only=true + SimBrief route spanning NA→EUR, verify only those regions affected
   - Verify CAR traffic still renders on roads
3. **Cleanup**: Uninstall regions, verify road overlay properly removed

---

## Open Questions

1. **Default behavior**: Should `hide_roads` default to true or false?
2. **X-Plane not installed**: How to handle first-time setup when X-Plane path not configured yet? (Prompt to configure X-Plane path first)
3. **Road file locations**: Are roads always in `Resources/default scenery/1000 roads/`? Need to verify for X-Plane 11 vs 12.
