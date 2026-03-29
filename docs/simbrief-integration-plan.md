# SimBrief Integration — Implementation Plan

## Overview

Integrate SimBrief flight plan data to enhance Dynamic Zoom and Prefetching.

## Current Status: ✅ COMPLETE

### What's Implemented

1. **Config** (`src/config.rs`)
   - `simbrief_user_id` field
   - `route_consideration_radius_nm`, `route_deviation_threshold_nm`, `route_prefetch_radius_nm`
   - `prefetch_route_percent`, `prefetch_airports`, `airport_radius_nm`, `near_airport_zoom`
   - **`use_simbrief_altitude`** - NEW: toggle to use SimBrief waypoints for dynamic zoom altitude

2. **SimBrief Client** (`src/xplane/simbrief.rs`)
   - Full OFP parsing with `FlightPlan`, `FlightFix`, `PrefetchPoint`
   - SID/STAR gap handling (interpolates altitude through procedural procedures)
   - On-route detection and deviation checking
   - Airport field elevation lookup

3. **DynamicZoom** (`src/dynamic_zoom.rs`)
   - **`zoom_for_position_with_simbrief()`** - NEW: method that uses SimBrief waypoints within consideration radius
   - Falls back to dataref altitude when no waypoints nearby or toggle disabled

4. **UI**
   - Settings: User ID input with tooltips
   - Dashboard: Fetch button, route preview, expandable waypoint list
   - **Dynamic Zoom section** - NEW: "Use SimBrief Altitude" toggle

5. **Prefetcher** (`src/tiles/prefetch.rs`)
   - `prefetch_route()` method using SimBrief PrefetchPoints
   - Route config: percent_ahead, waypoint_radius_nm, airport_radius_nm

6. **Main Integration** (`src/main.rs`)
   - Auto-fetch on startup if user_id configured
   - Background prefetch loop with SimBrief route
   - Uses `use_simbrief_altitude` config to toggle between SimBrief and dataref altitude for zoom

---

## Summary

SimBrief integration is now complete. When enabled:
- Dynamic zoom uses SimBrief planned altitudes from waypoints within the consideration radius
- Falls back to dataref altitude when off-route or toggle disabled
