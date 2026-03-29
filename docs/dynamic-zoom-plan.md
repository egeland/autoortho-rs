# Dynamic Zoom Implementation Plan

## Overview

Implement altitude-based dynamic zoom level selection that:
1. Allows users to configure zoom rules by AGL altitude
2. Upserves higher-zoom tiles when available
3. Integrates with SimBrief for route-based prefetching

## Current State ✅ MOSTLY DONE

- `DynamicZoom` struct exists in `src/dynamic_zoom.rs` ✅
- Config has `enable_dynamic_zoom` and `zoom_rules` ✅
- Settings UI has dynamic zoom toggle and rule display ✅
- Wired into SimBrief prefetch in main.rs ✅
- Upserving implemented in filesystem.rs ✅

## Implementation

### Implementation Status

All items below are DONE ✅:
- Config: `ZoomRule` struct, `enable_dynamic_zoom`, `zoom_rules` fields
- DynamicZoom module with rule-based zoom selection
- Provider max zoom integration
- Upserving in both fetcher and filesystem
- SimBrief prefetch with dynamic zoom
- UI with toggle and rule display

### Remaining Work

- [ ] Add/Edit/Delete rules UI - currently display only
- [ ] Rule validation UI warnings

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
