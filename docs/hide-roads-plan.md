# Hide X-Plane Roads — Implementation Plan

## Overview

Add an option to hide X-Plane's default road rendering when using ortho scenery tiles, while keeping traffic (cars) visible. This is achieved by creating a custom "transparent roads" overlay that replaces road textures with transparent ones but preserves the CAR traffic commands.

This approach is proven by existing tools: **Transparency4Ortho** and **Xroads**.

## Current Status: NOT STARTED

This is a future enhancement plan. No implementation has been done yet.

## How It Works

X-Plane renders roads from `.net` (Vector Network Definition) files in `Resources/default scenery/1000 roads/`. Each road type has:
- **TEXTURE** commands that specify the road surface texture
- **CAR** commands that define traffic lanes (independent of road surface rendering)

To make roads invisible but keep traffic:
1. Copy `1000 roads/` from X-Plane installation
2. Replace road textures with transparent ones (or non-existent paths)
3. Keep all CAR commands intact
4. Install as custom scenery with `library.txt` scoping to ortho-covered regions

---

## Files to Modify/Create

| File | Change |
|------|--------|
| `src/config.rs` | Add `hide_roads`, `hide_roads_route_only` fields |
| `src/scenery/regions.rs` | **New** — Region bounds definitions |
| `src/scenery/roads.rs` | **New** — Road overlay generator |
| `src/xplane/simbrief.rs` | Add `get_covered_region_ids()` method |
| `src/ui/screens/settings.rs` | Add checkboxes |
| `src/ui/screens/dashboard.rs` | Add status indicator |

---

## Testing

1. **Unit tests**: Region bounds, lat/lon to region mapping
2. **Integration**: Install with hide_roads=true, verify roads invisible, CAR traffic still renders
3. **Cleanup**: Uninstall regions, verify road overlay properly removed
